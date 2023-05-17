defmodule Editor.GUI do
  use GenServer
  require Logger

  def start_link(name) do
    GenServer.start_link(__MODULE__, :ok, name: name)
  end

  def init(:ok) do
    Logger.debug("Starting GUI on #{inspect(self())}")
    dir = "."

    port =
      if Mix.env() != :test do
        port = Port.open({:spawn, "priv/native/editor"}, [:binary])

        # {busy_limits_msgq, {Low, High} | disabled}
        Port.monitor(port)
        Process.link(port)
        list_files(dir)
        port
      end

    {:ok, %{port: port, serial: 0, file: nil, dir: dir}}
  end

  defp list_files(dir) do
    Path.join([dir, "*"])
    |> Path.wildcard()
    |> Enum.map(&Path.basename/1)
    |> Editor.Glue.set_available_files_json(0)
    |> send_command()
  end

  defp send_message(fun, arg, state) do
    message = fun.(arg, state.serial)
    send_command(message)
    state.serial
  end

  defp send_command(message) do
    GenServer.cast(self(), {:send_command, message})
  end

  def handle_cast({:send_command, message}, %{serial: serial} = state) do
    send(state.port, {self(), {:command, "#{message}\n"}})
    {:noreply, %{state | serial: serial + 1}}
  end

  def handle_info({:DOWN, _, :port, _, _}, state) do
    Logger.debug("GUI port died")
    {:stop, :shutdown, state}
  end

  def handle_info({_port, {:data, data}}, state) do
    Logger.debug("GUI data: #{inspect(data)}")

    result =
      data
      |> String.split("\n")
      |> Enum.reject(&(&1 == ""))
      |> Enum.flat_map(fn line ->
        result = Editor.Glue.decode_event(line)
        Logger.debug("Decoded GUI event: #{inspect(result)}")

        Logger.debug("GUI state: #{inspect(state)}")

        case result do
          %{typ: :exit} ->
            Logger.debug("GUI exited")
            [{:stop, :shutdown, state}]

          %{typ: :click_file_event, data: [file]} ->
            path = Path.join([state.dir, file])

            if File.dir?(path) do
              list_files(path)
              [{:noreply, %{state | dir: path}}]
            else
              Logger.debug("GUI clicked file: #{inspect(file)}")
              send_message(&Editor.Glue.open_file_json/2, [state.dir, file], state)
              [{:noreply, %{state | file: file}}]
            end

          %{typ: :buffer_changed, data: [contents]} ->
            Logger.debug("GUI changed buffer: #{inspect(contents)}")
            File.write!(Path.join([state.dir, state.file]), contents)
            [{:noreply, state}]

          %{typ: :debug_message, data: messages} ->
            Logger.debug("GUI debug message: #{inspect(messages)}")
            [{:noreply, state}]

          %{typ: :navigate_up} ->
            Logger.debug("GUI navigate up")
            parent = Path.dirname(state.dir)
            list_files(Path.dirname(parent))
            [{:noreply, %{state | dir: Path.dirname(parent)}}]

          something_else ->
            Logger.error("Unknown event from GUI: #{inspect(something_else)}")
            []
        end
      end)

    case result do
      [] -> {:noreply, state}
      [reply | _] -> reply
    end
  end

  def handle_info(msg, state) do
    Logger.warn("Unknown message from GUI: #{inspect(msg)}")
    {:noreply, state}
  end

  def set_available_files(gui, paths) do
    GenServer.call(gui, {:set_available_files, paths})
  end

  def set_buffer(gui, buffer) do
    GenServer.call(gui, {:set_buffer, buffer})
  end

  def quit(gui), do: GenServer.call(gui, {:quit})

  def handle_call({:set_available_files, paths}, _from, state) do
    send_message(&Editor.Glue.set_available_files_json/2, paths, state)
    {:reply, :ok, state}
  end

  def handle_call({:set_buffer, contents}, _from, state) do
    send_message(&Editor.Glue.set_buffer_json/2, contents, state)
    {:reply, :ok, state}
  end

  def handle_call({:quit}, _from, state) do
    Port.close(state.port)
    {:stop, :shutdown, state}
  end

  def terminate(reason, _state) do
    Logger.debug("Terminating GUI: #{inspect(reason)}")
    # Port.close(state.port)
    System.stop(0)
  end
end
