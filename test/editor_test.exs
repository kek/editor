defmodule EditorTest do
  use ExUnit.Case
  doctest Editor

  test "adding by Rustler NIF" do
    assert Editor.add(1, 2) == 3
  end

  test "greets the world" do
    result = Editor.spawn_thread(self())
    assert result == {}

    assert_receive "Hello world"
  end

  test "use a resource" do
    resource = Editor.make_resource()
    assert Editor.read_resource(resource) == 42
  end

  test "use a channel" do
    channel = Editor.make_channel(self())
    Editor.send_on_channel(channel, 101)
    assert_receive 101
  end
end
