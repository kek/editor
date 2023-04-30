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

    {:ok, %{port: port}}
  end

  def handle_info({:DOWN, _, :port, _, _}, state) do
    Logger.debug("GUI port died")
    {:stop, :shutdown, state}
  end

  def handle_info({_port, {:data, data}}, state) do
    result =
      data
      |> String.split("\n")
      |> Enum.reject(&(&1 == ""))
      |> Enum.flat_map(fn line ->
        case Jason.decode(line) do
          {:ok, %{"event" => "exit"}} ->
            Logger.debug("GUI exited")
            [{:stop, :shutdown, state}]

          {:ok, %{"error" => message}} ->
            Logger.error("Error from GUI: #{message}")
            []

          {:ok, message} ->
            Logger.warn("Unknown data from GUI: #{inspect(message)}")
            []

          {:error, wtf} ->
            Logger.error("Invalid JSON from GUI: #{inspect(line)}: #{inspect(wtf)}")
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

  def output(gui, s) do
    GenServer.call(gui, {:output, s})
  end

  def quit(gui) do
    GenServer.call(gui, {:output})
  end

  def handle_call({:output, s}, _from, state) do
    send(state.port, {self(), {:command, "#{s}\n"}})
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
