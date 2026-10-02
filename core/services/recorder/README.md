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
| `logic/api` | `RecorderSettings`, wire conversions |
| `adapters/mcap` | `McapFile`, channel descriptors |
| `adapters/storage` | Recordings folder paths |
| `app` | Service wiring and data plane Task |

## Tests

```bash
cargo test -p blueos-recorder-app
cargo test -p blueos-recorder-capture
cargo test -p blueos-recorder-domain
```
