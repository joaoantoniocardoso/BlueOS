# blueos-idl

Published ROS 2 message types for BlueOS with embedded `ros2msg` schemas and a `#![no_std]` CDR codec (D-05).

## Message sources

Canonical `.msg` files live in `interfaces/` inside this crate so `cargo package` ships them. Workspace
paths under `core/interfaces/` symlink here (see `core/interfaces/README.md`).

## Schema evolution (D-06)

- **Append-only** within a major version: new fields only at the end of the `.msg` file.
- **New writer, old reader**: decoders ignore trailing payload bytes.
- **Old writer, new reader**: generated Rust decoders default missing trailing fields.
- Any other change (remove, reorder, retype, rename) requires a **major version bump** in
  `core/interfaces/api.lock` and a new message type or package revision.
- Each type exposes `TYPE_HASH` (SHA-256 of schema name + ordered field signature).

### API-break lock

`core/interfaces/api.lock` stores `schema_name major field_signature_hash` per message. CI runs
`api_lock_matches_interfaces`; drift fails until the lock is refreshed intentionally:

```bash
BLUEOS_IDL_UPDATE_LOCK=1 cargo test -p blueos-idl api_lock_matches_interfaces -- --nocapture
```

After editing `.msg` files, rebuild to refresh TypeScript output:

```bash
cargo build -p blueos-idl
```

## Adding a message

1. Add `interfaces/<package>/msg/<Name>.msg` (use `#` comments).
2. `cargo build -p blueos-idl` (regenerates Rust + `typescript/`).
3. Update `api.lock` with the command above (or bump major for breaking changes).

## Consumers

| Consumer | How |
|---|---|
| Rust services / `logic/` | Depend on `blueos-idl`, use `Message::encode` / `decode` and `SCHEMA` for MCAP. |
| TypeScript (`blueos-api` frontend lib) | Import `core/libs/idl/typescript/messages.d.ts` types and `schemas.ts` with `@foxglove/rosmsg2-serialization`. |
| Python (transitional) | Parse the same `.msg` text at runtime (e.g. `rosbags`) with CDR payloads; keys from `blueos-api`. |

## Features

- `std` (default): enables `std::error::Error` on `blueos_idl::Error`.
- Without default features: suitable for `thumbv7em-none-eabihf` logic crates.
