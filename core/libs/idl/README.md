# blueos-idl

Published ROS 2 message types for BlueOS with embedded `ros2msg` schemas and a `#![no_std]` CDR codec (D-05).

## Message sources

Canonical `.msg`, `.srv` and `.action` files live in `interfaces/` (`core/libs/idl/interfaces/`) so `cargo package`
ships them.

Each part of a `.srv` (request, response) and of a `.action` (Goal, Job result, Feedback) is a message named as in
ROS 2, `<package>/srv/<Name>_Request` or `<package>/action/<Name>_Goal`, with the type `<Name>Request` or `<Name>Goal`
in Rust and TypeScript. `blueos_idl::schema("<package>/action/<Name>")` returns the schema text of the whole file,
every part in order.

## Schema evolution (D-06)

- **Append-only** within a major version: new fields only at the end of the `.msg` file, or of the part of a
  `.srv` or `.action`, each part being a top-level message.
- **New writer, old reader**: decoders ignore trailing payload bytes.
- **Old writer, new reader**: generated Rust decoders default missing trailing fields.
- Any other change (remove, reorder, retype, rename) requires a **major version bump** in
  `api.lock` and a new message type or package revision.
- Each type exposes `TYPE_HASH` (SHA-256 of schema name + ordered field signature).

### API-break lock

`api.lock` stores `schema_name major field_signature` per message. CI runs `api_lock_matches_interfaces`;
drift fails until the lock is refreshed intentionally:

```bash
cargo run -p blueos-idl-codegen --bin blueos-idl-print-lock > core/libs/idl/api.lock
```

After editing `.msg`, `.srv` or `.action` files, regenerate committed Rust and TypeScript:

```bash
cargo run -p blueos-idl-codegen --bin blueos-idl-codegen -- --write
```

After changing the generator, refresh the committed output of its `.srv` and `.action` fixtures
(`codegen/tests/fixtures/interface_kinds/`):

```bash
BLUEOS_IDL_UPDATE_FIXTURES=1 cargo test -p blueos-idl-codegen --test interface_kinds
```

## Adding a message

1. Add `interfaces/<package>/msg/<Name>.msg` (use `#` comments).
2. `cargo run -p blueos-idl-codegen --bin blueos-idl-codegen -- --write` (regenerates Rust + `typescript/`).
3. Update `api.lock` with the command above (or bump major for breaking changes).

## Consumers

| Consumer | How |
|---|---|
| Rust services / `logic/` | Depend on `blueos-idl`, use `Message::encode` / `decode` and `SCHEMA` for MCAP. |
| TypeScript (`blueos-api` frontend lib) | Import `core/libs/idl/typescript/messages.d.ts` types and `schemas.ts` with `@foxglove/rosmsg2-serialization`. |
| Python (transitional) | Parse the same `.msg` text at runtime (e.g. `rosbags`) with CDR payloads; keys from `blueos-api`. |

## Features

- No default features, and no `std` feature: the crate is `no_std` + `alloc`, suitable for `thumbv7em-none-eabihf`
  logic crates.
- `catalog`: adds `catalog::schema`, the schema text of the vendored ROS 2 and Foxglove messages.
