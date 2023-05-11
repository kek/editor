defmodule Editor.Glue do
  @moduledoc """
  Documentation for `Editor.Glue`.
  """
  use Rustler, otp_app: :editor, crate: "editor"

  @doc ~S"""
  ## Examples

    ```
    iex> Editor.Glue.spawn_thread(self())
    {}
    iex> receive do msg -> msg end
    "Hello world"
    ```
  """

  def spawn_thread(_debug_pid), do: :erlang.nif_error(:nif_not_loaded)

  def make_number(_resource), do: :erlang.nif_error(:nif_not_loaded)

  @doc ~S"""
  ## Examples

    ```
    iex> resource = Editor.Glue.make_number(42)
    iex> Editor.Glue.read_resource(resource)
    42
    ```
  """
  def read_resource(_resource), do: :erlang.nif_error(:nif_not_loaded)

  def make_channel(_debug_pid), do: :erlang.nif_error(:nif_not_loaded)

  @doc ~S"""
  ## Examples

    ```
    iex> channel = Editor.Glue.make_channel(self())
    iex> Editor.Glue.send_on_channel(channel, 101)
    {}
    iex> receive do msg -> msg end
    101
    ```
  """
  def send_on_channel(_channel, _integer), do: :erlang.nif_error(:nif_not_loaded)

  def set_available_files_json(_path, _serial), do: :erlang.nif_error(:nif_not_loaded)

  def open_file_json(_path, _serial), do: :erlang.nif_error(:nif_not_loaded)

  def decode_event(_data), do: :erlang.nif_error(:nif_not_loaded)
end
