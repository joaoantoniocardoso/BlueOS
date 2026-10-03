# Recorder (Draft 2)

Rust Recorder Service: capture Domain, data plane Task, and MCAP output. Vocabulary and naming live in
[`GLOSSARY.md`](../../../GLOSSARY.md) at the repository root (do not redefine terms here).

## Run locally

```bash
blueos recorder --recorder-path /path/to/folder --zenoh-endpoint tcp/127.0.0.1:7447
```

On BlueOS images the `recorder` symlink calls the multicall `blueos` binary with the `recorder` feature.

## Layout

| Crate | Role |
|-------|------|
| `logic/capture` | Record gate, active recording state |
| `logic/recorder` | Recorder Domain |
| `logic/api` | Wire conversions |
| `adapters/mcap` | `McapFile`, channel descriptors |
| `adapters/storage` | Recordings folder paths |
| `app` | Service wiring, Tasks, and feature folders (D-23) |

### Application crate (`app/src/`)

Service-wide modules at the top, then one folder per Block (`capture/`, `cameras/`, `library/`), with the same role
names inside every Block folder (`handlers`, `io`, `tasks`).

| Path | Role |
|------|------|
| `service.rs`, `cli.rs`, `context.rs`, `settings.rs`, `endpoints.rs`, `io.rs` | Kernel wiring shared across Blocks |
| `tasks/mavlink.rs` | MAVLink ingress (feeds capture and cameras) |
| `capture/tasks/data_plane/` | Data plane Task and sample plan |
| `cameras/io.rs` | Camera MAVLink egress |
| `library/handlers.rs` | Custom Commands and the `index` IO query |
| `library/io.rs` | Library blocking IO (rescan, delete) |
| `library/tasks/operations.rs` | Repair and snapshot reconcile Task |

Sans-IO components (`logic/paths`, `logic/schema-gate`) have no application folder; the feature that uses them imports
them directly.

## Tests

```bash
cargo test -p blueos-recorder-app
cargo test -p blueos-recorder-capture
cargo test -p blueos-recorder-domain
```
