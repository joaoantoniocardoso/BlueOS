# Example Rust service (D-20)

A minimal but complete BlueOS service: simulated pump level control, a three-step self-test job graph, scheduled
timeouts, Python-compatible settings, and a pirate-mode frontend page. Use it as the template for new Rust
services and for AI agents; read `doc/architecture/decisions.md` first.

## Layout (D-02)

| Path | Role |
|---|---|
| `logic/pump/` | `#![no_std]` domain: commands, queries, events, jobs, settings, snapshot |
| `adapters/simulated_pump/` | Fake hardware (async delay only; no `logic/` dependency) |
| `app/` | `ServiceBuilder` wiring, CDR codecs, CLI entry (`example::run`) |

The multicall binary is `core/app/blueos` (`cargo run -p blueos --features example -- example`). This service is
**not** registered in `start-blueos-core`; run it manually against a local `zenohd`.

## Sans-IO model (D-03)

Logic never awaits. Long work is modeled as:

1. `handle_command` enqueues a [`JobGraph`](../../libs/logic/jobs/src/lib.rs) and/or returns `Effect::Schedule`.
2. The kernel runs `Effect::Io` in the adapter and posts `PumpCommand::JobIoFinished` back to the inbox.
3. Timeouts deliver `PumpCommand::SelfTestTimedOut` from a kernel timer.

Start reading at `logic/pump/src/domain.rs` (`Domain` impl), then `command.rs` (transitions), `jobs.rs` (graph),
and `snapshot.rs` (published fields).

## Command flow (end to end)

```text
Vue (ExampleServiceView) -> blueos-api sendCommand
  -> zenoh query blueos/v1/example/command/<Name> (CDR payload)
  -> comms adapter -> kernel inbox
  -> App::handle -> PumpDomain::handle_command
  -> effects: Io / Schedule / Persist
  -> publish state (pump, status, jobs, settings) + events
  -> frontend watchState / watchJobs receives updates
```

Queries use `blueos/v1/example/query/Level` (read model from snapshot). Standard keys (`status`, `jobs`, `settings`,
`UpdateSettings`, liveliness, `info`, `log`) come from the kernel (D-12).

## IDL messages

Sources: `core/libs/idl/interfaces/blueos_example_msgs/msg/`. After editing:

```bash
cargo build -p blueos-idl
BLUEOS_IDL_UPDATE_LOCK=1 cargo test -p blueos-idl api_lock_matches_interfaces -- --nocapture
```

## Settings (D-11)

File shape: `settings-1.json` with `VERSION`, `max_level`, `self_test_timeout_seconds` (runtime), and
`device_model` (restart-required). `UpdateSettings` persists via `Effect::Persist` and may emit
`RestartRequired` on `device_model` changes.

## Run locally

```bash
# zenoh router (if not already running on 7447)
zenohd --config <your-config>

RUSTC_WRAPPER= cargo run -p blueos --features example -- example -v
```

Frontend (pirate mode): **Example Rust service** under `/tools/example-service`.

## Tests

```bash
RUSTC_WRAPPER= cargo test -p blueos-example-pump
RUSTC_WRAPPER= cargo test -p blueos-example
```

Logic tests cover every command path without a runtime. `app/tests/kernel_integration.rs` exercises the channel
backend: command ack, jobs state, IO completion, and `SelfTestCompleted` event.
