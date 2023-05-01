defmodule Editor do
  @moduledoc """
  Documentation for `Editor`.
  """
  use Rustler, otp_app: :editor, crate: "editor"

  # When your NIF is loaded, it will override this function.
  def add(_a, _b), do: :erlang.nif_error(:nif_not_loaded)

  def spawn_thread(_debug_pid), do: :erlang.nif_error(:nif_not_loaded)

  def make_resource(), do: :erlang.nif_error(:nif_not_loaded)

  def read_resource(_resource), do: :erlang.nif_error(:nif_not_loaded)

  def make_channel(_debug_pid), do: :erlang.nif_error(:nif_not_loaded)

  def send_on_channel(_channel, _integer), do: :erlang.nif_error(:nif_not_loaded)

  def something(_), do: :erlang.nif_error(:nif_not_loaded)
end
