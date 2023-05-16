defmodule Editor.GUI do
  use GenServer
  require Logger

  def start_link(:please) do
    GenServer.start_link(__MODULE__, :ok, name: __MODULE__)
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
        list_files(port, dir)
        port
      end

    {:ok, %{port: port, serial: 0, file: nil, dir: dir}}
  end

  defp list_files(port, dir) do
    glob = Path.join([dir, "*"])
    paths = Path.wildcard(glob)
    Logger.debug("Listing files: #{inspect(paths)}")
    message = Editor.Glue.set_available_files_json(paths, 0)
    send(port, {self(), {:command, "#{message}\n"}})
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
            if File.dir?(file) do
              list_files(state.port, file)
              [{:noreply, %{state | dir: Path.join([state.dir, file])}}]
            else
              Logger.debug("GUI clicked file: #{inspect(file)}")
              serial = send_message(&Editor.Glue.open_file_json/2, file, state)
              [{:noreply, %{state | serial: serial, file: file}}]
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
            list_files(state.port, Path.dirname(parent))
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

  def set_available_files(paths), do: set_available_files(__MODULE__, paths)

  def set_available_files(gui, paths) do
    GenServer.call(gui, {:set_available_files, paths})
  end

  def set_buffer(buffer), do: set_buffer(__MODULE__, buffer)

  def set_buffer(gui, buffer) do
    GenServer.call(gui, {:set_buffer, buffer})
  end

  def quit, do: quit(__MODULE__)
  def quit(gui), do: GenServer.call(gui, {:quit})

  def handle_call({:set_available_files, paths}, _from, state) do
    serial = send_message(&Editor.Glue.set_available_files_json/2, paths, state)
    {:reply, :ok, %{state | serial: serial}}
  end

  def handle_call({:set_buffer, contents}, _from, state) do
    serial = send_message(&Editor.Glue.set_buffer_json/2, contents, state)
    {:reply, :ok, %{state | serial: serial}}
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

  defp send_message(fun, arg, state) do
    serial = state.serial + 1
    message = fun.(arg, serial)
    send(state.port, {self(), {:command, "#{message}\n"}})
    serial
  end
end
