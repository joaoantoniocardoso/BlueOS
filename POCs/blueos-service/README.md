# BlueOS Service POC

Hexagonal, CQRS (no event sourcing), jobs-oriented BlueOS service kernel.

This POC exists to answer: **what is a Service, how is the code organized, and how do we write one?**

The minimum real-world example is **vehicle calibration** as a backend service talking to a stub **Autopilot** service over BlueOS Comms (Zenoh primary, in-process channels for unit tests).

## Layout

Every Rust crate lives in one of three folders, in the shared libraries and in each service alike:

- `logic/`: pure code with no I/O (`#![no_std]`).
- `adapters/`: code that touches the outside world.
- `app/`: code that connects logic to adapters and builds the binary.

The shared crates live in BlueOS core under [`core/libs/`](../../core/libs), in the `core/` Cargo workspace. This POC keeps only the two example services, laid out as `services/<name>/{logic/<block>,adapters/<thing>,app}`, and depends on core by path. Neither service needs its own adapter.

```text
core/libs/
  logic/jobs                 blueos_jobs -- pure job state machine
  logic/cqrs                 blueos_cqrs -- CQRS: Domain/Effect ports and the App engine
  adapters/comms             blueos_comms -- Session, Session::open, feature-selects the driver
  adapters/comms/driver      blueos_comms_driver -- Driver / Dispatcher / Frame (private to comms)
  adapters/comms/zenoh       blueos_comms_zenoh -- ZenohDriver (private to comms)
  adapters/comms/channel     blueos_comms_channel -- ChannelDriver (private to comms)
  adapters/cli               blueos_cli -- clap
  adapters/configs           blueos_configs -- JSON5 via serde
  adapters/logging           blueos_logging -- tracing
  app/service                blueos_service -- owns App + Adapters, runs the loop
POCs/blueos-service/services/
  calibration/logic/sensors  calibration_sensors -- gyro/baro/stationary handlers
  calibration/app            calibration -- calibration binary
  autopilot/logic/preflight  autopilot_preflight -- preflight stub handlers
  autopilot/app              autopilot -- autopilot binary
```

A crate's folder decides who it may depend on (dev-dependencies excluded):

| A crate in | may depend on workspace crates in |
|---|---|
| `logic/` | `libs/logic/` |
| `adapters/` | `libs/adapters/` and its own service's `adapters/` |
| `app/` | anything in `libs/` and its own service |

No crate depends on another service's crates. A crate nested in another crate's folder, like the comms drivers, is private to that crate and its siblings. [`deny.toml`](deny.toml) only covers crates.io crates: `zenoh` stays behind `blueos_comms_zenoh`.

Every `logic/` crate is built for a target without `std`, so a dependency on `tokio`, `zenoh` or any other I/O crate fails to compile.

## Check

```sh
POCs/blueos-service/.hooks/check-boundaries
```

It runs the same Rust checks as BlueOS's `.hooks/pre-push` does for `core/` (`.hooks/lib/rust_checks.sh`): `cargo fmt --check`, `clippy -D warnings`, `cargo test`, `cargo deny check bans`, the folder rules, and a build of every `logic/` crate for `thumbv7em-none-eabihf`. It needs `cargo install --locked cargo-deny@0.20.2`, `rustup target add thumbv7em-none-eabihf` and `jq`.
