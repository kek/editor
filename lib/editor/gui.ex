defmodule Editor.GUI do
  use GenServer
  require Logger

  def start_link(:please) do
    GenServer.start_link(__MODULE__, :ok, name: __MODULE__)
  end

  def init(:ok) do
    Logger.debug("Starting GUI on #{inspect(self())}")

    port =
      if Mix.env() != :test do
        port = Port.open({:spawn, "priv/native/editor"}, [:binary])
        Port.monitor(port)
        Process.link(port)
        port
      end

    {:ok, %{port: port, serial: 0}}
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
        result = Editor.NIF.decode_event(line)
        Logger.debug("Decoded GUI event: #{inspect(result)}")

        case result do
          %{typ: :exit} ->
            Logger.debug("GUI exited")
            [{:stop, :shutdown, state}]

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

  def output(s), do: output(__MODULE__, s)

  def output(gui, data) do
    GenServer.call(gui, {:output, data})
  end

  def open_file(path), do: open_file(__MODULE__, path)

  def open_file(gui, path) do
    GenServer.call(gui, {:open_file, path})
  end

  def quit, do: quit(__MODULE__)
  def quit(gui), do: GenServer.call(gui, {:quit})

  def handle_call({:output, data}, _from, %{port: port, serial: serial} = state) do
    s = Editor.NIF.test_event_json(data, serial)
    send(port, {self(), {:command, "#{s}\n"}})
    {:reply, :ok, %{state | serial: serial + 1}}
  end

  def handle_call({:open_file, path}, _from, %{port: port, serial: serial} = state) do
    message = Editor.NIF.open_file_command_json(path, serial)
    send(port, {self(), {:command, "#{message}\n"}})
    {:reply, :ok, %{state | serial: serial + 1}}
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
