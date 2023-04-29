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
        send(port, {self(), {:command, "Hello, Thread!\n"}})
        port
      end

    {:ok, %{port: port}}
  end

  def handle_info({:DOWN, _, :port, _, _}, state) do
    Logger.debug("GUI port died")
    {:stop, :shutdown, state}
  end

  def handle_info({_port, {:data, data}}, state) do
    case Jason.decode(data) do
      {:ok, %{"event" => "exit"}} ->
        Logger.debug("GUI exited")
        {:stop, :shutdown, state}

      {:ok, %{"error" => message}} ->
        Logger.error("Error from GUI: #{message}")
        {:noreply, state}

      {:ok, message} ->
        Logger.info("Unknown data from GUI: #{inspect(message)}")
        {:noreply, state}

      {:error, _} ->
        Logger.error("Invalid JSON from GUI: #{inspect(data)}")
        {:noreply, state}
    end
  end

  def handle_info(msg, state) do
    Logger.debug("Unknown message from GUI: #{inspect(msg)}")
    {:noreply, state}
  end

  def output(s), do: output(__MODULE__, s)

  def output(gui, s) do
    GenServer.call(gui, {:output, s})
  end

  def handle_call({:output, s}, _from, state) do
    send(state.port, {self(), {:command, "#{s}\n"}})
    {:reply, :ok, state}
  end
end
