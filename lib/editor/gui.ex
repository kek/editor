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
        paths = ["mix.exs", "Cargo.toml", "README.md"]
        message = Editor.NIF.set_available_files_json(paths, 0)
        send(port, {self(), {:command, "#{message}\n"}})

        port
      end

    {:ok, %{port: port, serial: 1}}
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

  def set_available_files(paths), do: set_available_files(__MODULE__, paths)

  def set_available_files(gui, paths) do
    GenServer.call(gui, {:set_available_files, paths})
  end

  def quit, do: quit(__MODULE__)
  def quit(gui), do: GenServer.call(gui, {:quit})

  def handle_call({:output, data}, _from, %{port: port, serial: serial} = state) do
    s = Editor.NIF.test_event_json(data, serial)
    send(port, {self(), {:command, "#{s}\n"}})
    {:reply, :ok, %{state | serial: serial + 1}}
  end

  def handle_call({:set_available_files, paths}, _from, %{port: port, serial: serial} = state) do
    message = Editor.NIF.set_available_files_json(paths, serial)
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
