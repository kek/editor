defmodule Editor.GUI do
  use GenServer
  require Logger

  def start_link(:please) do
    GenServer.start_link(__MODULE__, :ok, name: __MODULE__)
  end

  def init(:ok) do
    port =
      if Mix.env() != :test do
        port = Port.open({:spawn, "priv/native/editor"}, [:binary])
        Port.monitor(port)
        send(port, {self(), {:command, "Hello, Thread!\n"}})
        port
      end

    {:ok, %{port: port}}
  end

  def handle_info({:DOWN, _, :port, _, _}, state) do
    {:stop, :shutdown, state}
  end

  def handle_info(msg, state) do
    Logger.debug("GUI: #{inspect(msg)}")
    {:noreply, state}
  end

  def output(s) do
    GenServer.call(__MODULE__, {:output, s})
  end

  def handle_call({:output, s}, _from, state) do
    send(state.port, {self(), {:command, "#{s}\n"}})
    {:reply, :ok, state}
  end
end
