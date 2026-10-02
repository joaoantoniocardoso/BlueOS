# BlueOS Rust service: draft 2 overview

This is the reader's map for draft 2 event-driven Rust services. Meanings live in [`GLOSSARY.md`](../../GLOSSARY.md)
at the repo root. Rules live in [`docs/adr/decisions.md`](../adr/decisions.md); this file links them and names code.
The draft 1 tour stays unchanged under [`docs/architecture/draft-1/rust-service-overview.md`](draft-1/rust-service-overview.md)
as evidence (D-20).

The walkthrough follows one `SetLevel` Command on `example-minimal` (`core/services/example/`). Integration tests
use the same path through `ExampleService::build` and `endpoints::register` (D-20, D-25).

## Layers

Services split by folder (D-02): `logic/` is pure, `adapters/` is IO, `app/` wires the Kernel and declares public
endpoints. The Domain never waits (D-03); the Kernel runs the Inbox loop and carries out Effects (D-04).

## `SetLevel` end to end

### Client Request

A browser or test sends a Command on the backbone. The teaching frontend uses the generated client in
`core/frontend/src/libs/blueos-api/services/example.ts`: the `SetLevel` descriptor names the key
`blueos/v1/example/command/SetLevel` (D-07, D-26). `sendCommand` in `core/frontend/src/libs/blueos-api/command.ts`
CDR-encodes a `SetLevelRequest` and performs a Zenoh query (D-10, D-14). The payload type is generated in
`core/libs/idl/src/generated/msg/blueos_example_msgs/set_level_request.rs` from
`core/libs/idl/interfaces/blueos_example_msgs/msg/SetLevelRequest.msg` (D-05).

The same shape is exercised without a browser in `core/services/example/app/tests/endpoints.rs` via
`Harness::send("SetLevel", ...)`, which uses the channel comms backend instead of Zenoh (D-10).

HTTP is not on this Command path. Userdata files and nginx ranges are the exception for recording bytes (D-08, D-23).

### Command endpoint adapter

At startup, `Kernel::boot` in `core/libs/app/service/src/kernel/mod.rs` declares a queryable on the Command key
(D-10). `serve_command` receives each query, decodes the body with the closure registered in
`core/services/example/app/src/endpoints.rs` (generated from `core/services/example/app/endpoints.toml`, D-26).
That closure calls `Conversions::set_level` in `core/services/example/logic/api/src/lib.rs`, which maps the Message
to `PumpRequest::SetLevel`. Invalid bodies never enter the Inbox (D-04).

The adapter enqueues a `Delivery` on the Inbox (`core/libs/app/service/src/inbox.rs`): a `Command::Request`, plus
a `CommandReply::Query` so the Kernel can answer with `CommandAck` when the Command finishes (D-10).

### Inbox and Domain Decision

`Kernel::run` receives Deliveries one at a time (D-04). Before `Pump::handle`, the Kernel clones
`PumpSnapshot` so a failure restores the prior DomainState (D-04, D-25). `handle` lives in
`core/services/example/logic/domain/src/lib.rs`. It pattern-matches `Command::Request(PumpRequest::SetLevel(level))`,
rejects levels above `MAX_LEVEL` with `Outcome::Rejected`, or applies the level with `Outcome::Applied` and no
Effects. That `Outcome` is the Domain's `Decision` (`blueos_domain` in `core/libs/logic/domain/src/lib.rs`, D-03).

Unit tests in the same file call `Pump::handle` with no Kernel (D-20).

### Kernel Effects, States, ack, Events

On `Applied`, `apply_sync_effects` in `core/libs/app/service/src/kernel/effects.rs` runs synchronous Effects (none
for this Command). Each declared State is a Projection of the Snapshot: `Conversions::pump` builds the `PumpState`
Message from `PumpSnapshot` (`core/services/example/logic/api/src/lib.rs`). `publish_states` sends only changed
values (D-10). `complete_command_reply` returns `CommandAck { accepted, job_id, reason }` with `job_id` zero when
no Job started (D-10). `example-minimal` declares no Event endpoints and `Pump::Event` is uninhabited, so nothing
is published as an Event after the ack.

On `Rejected`, the Snapshot stays at the backup, States are not republished, and the ack carries the rejection
reason string from the Domain error (D-04).

Clients that watch `pump` use `watchState` in `core/frontend/src/libs/blueos-api/watch.ts` (subscribe, then query,
D-10, D-14).

### Process startup (where this Service sits)

`blueos_service::entry::run` in `core/libs/app/service/src/entry/run.rs` starts logging (D-13), parses the CLI
(D-25), opens the Zenoh backend as the Session (D-25), loads settings and durable state when registered (D-11,
D-28), calls `ExampleService::build` in `core/services/example/app/src/service.rs`, then `Kernel::start` and
`Kernel::run` (D-25). `example-minimal` does not register settings, durable state, Tasks, or Jobs; the Recorder
shows those paths below.

## Settings, durable state, Tasks, Projections

These do not appear in `example-minimal`, but every shipped Service uses the same Kernel hooks.

**Settings document.** Each service defines its document in `logic/api` and registers `.settings(...)` on
`ServiceBuilder`. The Kernel loads the file through `SettingsDriver` in `core/libs/app/service/src/settings.rs`,
using `blueos-settings` in `core/libs/adapters/settings/` for Python-compatible files (D-11). `UpdateSettings` is
served like any Command (`serve_update_settings` in `kernel/mod.rs`). The published `settings` State uses the same
JSON as the file inside `SettingsEnvelope`.

**Durable state.** Opt-in via `ServiceBuilder::durable_state` (D-28). `core/libs/app/service/src/durable_state.rs`
debounces writes to `ServiceStateStore` in `core/libs/adapters/settings/src/service_state.rs`. Restore may deliver a
restored Tick; the Kernel does not re-run IO for interrupted Jobs (D-28).

**Jobs.** When the Domain implements `DomainJobs`, the Kernel keeps `Jobs` inside the Snapshot
(`core/libs/logic/jobs/src/lib.rs`), publishes the standard `jobs` State, and puts the root `job_id` in the ack
(D-04, D-12).

**Tasks and Projections.** Long-running work is declared with `ServiceBuilder::task` and supervised in
`core/libs/app/service/src/tasks.rs` (D-27). Tasks receive a `CommandSender`, a `Session`, and typed
`Projection` receivers from `core/libs/app/service/src/projection.rs`. The Recorder data plane Task is
`run_data_plane` in `core/services/recorder/app/src/data_plane.rs`; it follows `RecordGate` from the capture Block
in `core/services/recorder/logic/capture/src/lib.rs` and reports `Observed fact` Commands (D-27). High-rate samples
stay on the data plane; the Inbox sees only control Commands and Observed facts (control plane vs data plane in the
Recorder Domain at `core/services/recorder/logic/recorder/src/lib.rs`).

**IO query endpoints.** Answered outside the Snapshot, in IO code with the service Context. See `RecorderHandlers::index`
in `core/services/recorder/app/src/handlers.rs` and `ServiceBuilder::io_query` in
`core/libs/app/service/src/builder.rs` (D-04).

**Custom endpoints.** Endpoints marked `custom = true` in the manifest get a handler trait method; the service maps
Messages in `handlers.rs` (D-26). `SetLevel` on `example-minimal` is not custom; the Recorder's `DeleteRecording`,
`RepairRecording`, `CancelRepair` and `SnapshotRecording` are, in `core/services/recorder/app/src/handlers.rs`.

## Where it is in the code

Each row names the crate and module that implements the term on this branch. Paths are under `core/` unless noted.

| Term | Crate / module |
|------|----------------|
| **Service** | `blueos-service`: `service::Service`, `entry::run` (`libs/app/service/src/service.rs`, `entry/run.rs`) |
| **Kernel** | `blueos-service`: `kernel` (`libs/app/service/src/kernel/mod.rs`) |
| **Domain** | `blueos-domain`: `Domain` trait (`libs/logic/domain/src/lib.rs`); example `Pump` (`services/example/logic/domain/src/lib.rs`) |
| **Block** | Composed logic with its own Snapshot slice; example `Capture` (`services/recorder/logic/capture/src/lib.rs`), lifted in `RecorderDomain` (`services/recorder/logic/recorder/src/lib.rs`). No separate `Block` trait. |
| **DomainState** | The Kernel's `snapshot: D::Snapshot` plus embedded `Jobs` when `DomainJobs` is implemented (`kernel/mod.rs`, `libs/logic/jobs/src/lib.rs`; D-25) |
| **Snapshot** | Per-Domain type, e.g. `PumpSnapshot` (`services/example/logic/domain/src/lib.rs`) |
| **Durable state** | `blueos-service`: `durable_state` (`libs/app/service/src/durable_state.rs`); store in `blueos-settings` (`libs/adapters/settings/src/service_state.rs`). Not used by `example-minimal`. |
| **Job** | `blueos-jobs` (`libs/logic/jobs/src/lib.rs`); Kernel `jobs` State wiring (`builder.rs`, `kernel/mod.rs`) |
| **Task** | `blueos-service`: `tasks` (`libs/app/service/src/tasks.rs`); example Recorder `run_data_plane` (`services/recorder/app/src/data_plane.rs`) |
| **Inbox** | `blueos-service`: `inbox` (`libs/app/service/src/inbox.rs`); loop in `kernel/mod.rs` |
| **Context** | Service-specific IO dependencies in `ServiceBuilder::context`; Recorder `RecorderContext` (`services/recorder/app/src/context.rs`). `example-minimal` uses `()` |
| **Command** | `blueos-domain`: `Command` enum (`libs/logic/domain/src/lib.rs`) |
| **Request** | `Command::Request` variant; client origin `PumpRequest` (`services/example/logic/domain/src/lib.rs`) |
| **IO result** | `Command::IoResult`; handled via `Domain::io_failed` (domain trait). No IO in `example-minimal` |
| **Tick** | `Command::Tick`; timers in `kernel/timers.rs`. Not used by `example-minimal` |
| **Observed fact** | `Command::ObservedFact`; Recorder `RecorderObservedFact` (`services/recorder/logic/recorder/src/lib.rs`) |
| **Projection** | Published States and in-process `ProjectionRegistry` (`libs/app/service/src/projection.rs`); `RecordGate` in capture Block (`services/recorder/logic/capture/src/lib.rs`) |
| **Outcome** | `blueos-domain`: `Outcome` (`libs/logic/domain/src/lib.rs`) |
| **Decision** | Type alias `Decision<D>` to `Outcome` in Domain types (`libs/logic/domain/src/lib.rs`) |
| **Effect** | `blueos-domain`: `Effect` (`libs/logic/domain/src/lib.rs`); applied in `kernel/effects.rs` and `kernel/io.rs` |
| **domain event** | Per-Domain event type, e.g. `RecorderEvent` (`services/recorder/logic/recorder/src/lib.rs`). `Pump::Event` is uninhabited in `example-minimal` |
| **Control plane** | Domain Commands and Snapshot, e.g. Recorder `RecorderDomain::handle` (`services/recorder/logic/recorder/src/lib.rs`) |
| **Data plane** | Recorder `run_data_plane` and MCAP writer path (`services/recorder/app/src/data_plane.rs`) |
| **Message** | `blueos-idl` generated types (`libs/idl/src/generated/`); schemas from `libs/idl/interfaces/` |
| **Session** | `Arc<dyn CommsBackend>` alias in `command_sender.rs`; Zenoh in `blueos-comms-zenoh` (`libs/adapters/comms-zenoh/`) |
| **Command endpoint** | `ServiceBuilder::command`, `serve_command` (`builder.rs`, `kernel/mod.rs`); manifest entry `SetLevel` in `services/example/app/endpoints.toml` |
| **Query endpoint** | `ServiceBuilder::query`, `serve_query` (`kernel/mod.rs`); `Level` on example |
| **IO query endpoint** | `ServiceBuilder::io_query`; Recorder `index` (`services/recorder/app/src/handlers.rs`) |
| **Endpoint manifest** | `app/endpoints.toml`; generator `blueos-idl-codegen` (`libs/idl/codegen/src/endpoints.rs`) |
| **Custom endpoint** | Manifest `custom = true`; handler trait in generated `endpoints.rs`, impl in `services/recorder/app/src/handlers.rs` |
| **State** | `ServiceBuilder::state`; published in `kernel/mod.rs`; example `pump` key in `endpoints.toml` |
| **Event** | `ServiceBuilder::event`; `publish_events` in `kernel/mod.rs`; Recorder `operation` in `services/recorder/app/endpoints.toml` |
| **Settings document** | Service schema in `logic/api`; file IO via `libs/adapters/settings/`; Kernel `settings.rs`. Recorder `services/recorder/app/src/settings.rs`. Not used by `example-minimal` |
