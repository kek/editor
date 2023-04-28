# NIF for Elixir.Editor

## To build the NIF module:

- Your NIF will now build along with your project.

## To load the NIF:

```elixir
defmodule Editor do
  use Rustler, otp_app: :editor, crate: "editor"

  # When your NIF is loaded, it will override this function.
  def add(_a, _b), do: :erlang.nif_error(:nif_not_loaded)
end
```

## Examples

[This](https://github.com/rusterlium/NifIo) is a complete example of a NIF written in Rust.

## References

- https://www.erlang.org/doc/man/erl_nif.html "Long-running NIFs"
- https://docs.rs/rustler/latest/rustler/
