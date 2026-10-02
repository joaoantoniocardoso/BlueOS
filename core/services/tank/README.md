# tank

The test Service of the generated endpoint registration (D-26). It is never shipped: no multicall entry, no image.

Its manifest, `app/endpoints.toml`, has an endpoint of every kind (Command, Query, IO query, State, Event), plain
and `custom`, so its tests cover every shape the generator emits.

- `logic/domain`: the Domain.
- `logic/api`: `impl Conversions for Tank`, one function per plain endpoint, and the generated `src/endpoints.rs`.
- `app`: the generated `src/endpoints.rs`, the custom endpoints in `src/handlers.rs`, and `impl Service` in
  `src/service.rs`. `tests/endpoints.rs` drives every endpoint as a client; `tests/compile_errors.rs` shows the
  compile error of each endpoint mistake.

Regenerate the endpoint code after changing the manifest:

```sh
cd core && cargo run -p blueos-idl-codegen -- --write
```
