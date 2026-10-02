# BlueOS Rust Event-Driven Architecture: Decision Record

Status: accepted, target of the second draft, last updated 2026-10-02.

This is the master decision record for introducing Rust, event-driven services, and a versioned IDL API
into BlueOS. Every contributor (human or AI agent) working on `core/libs/`, `core/services/<rust service>/`,
`core/libs/idl/`, or the frontend `blueos-api` library must read it first, together with `GLOSSARY.md` for the
vocabulary. When code contradicts a decision, either fix the code or update the decision here in the same change.

This record always states the current target. Superseded text is deleted; git keeps the history.

## Drafts and evidence

- **Draft 1** lived on branch `pocs/blueos-service`, up to commit `17a3f639d`. It proved the architecture and
  shipped a Recorder that ran on a Raspberry Pi 4, but its code is not the base for draft 2 (D-20).
- The evidence that shaped draft 2, written against that commit and kept unchanged in `docs/architecture/draft-1/`
  (its `path:line` references, including those into this file, point at that commit):
  - `rust-service-overview.md`: how draft 1 works.
  - `rust-service-review.md`: correctness findings (H1, M1 to M11, L1 to L13).
  - `rust-service-ergonomics-review.md`: owner direction (P1 to P12) and findings (E1 to E13).
- Draft 2 map: [`docs/architecture/rust-service-overview.md`](../architecture/rust-service-overview.md) (D-20).
- **Draft 2** is the target described here.

## Source reference

For the exact wording of a decision, retrieve the design conversation (author machine,
`~/.cursor/projects/home-joaoantoniocardoso-BlueRobotics-worktrees-BlueOS-docker-quartz-geyser-BlueOS-docker/agent-transcripts/<id>/<id>.jsonl`):

| Transcript id | Content |
|---|---|
| `11effae0-a983-42fb-ad8c-d8f350be2bf0` | Draft 1 design |
| `52237482-724f-4979-85b6-d6325dab8126` | Draft 1 implementation, reviews, D-21 and D-23 |
| `066e6fa2-c022-43cf-9b55-aa000441bfc2` | Draft 2 decisions (D-25 to D-32 and every amendment) |

Related repositories (author machine, `~/BlueRobotics/`):

| Repository | Role |
|---|---|
| `microservices_core_prototype` | Earlier full Rust port of ardupilot_manager. Baseline for the ergonomics measures (D-31). Its architecture is not the target. |
| `blueos-recorder` | Current Recorder (Rust, zenoh + mcap). Rewritten in this repository and then retired. |
| `zBlueberry` | Peer experiment: `.msg` files to serde structs. Its name must not be used. |
| `radcam-manager` | Reference CI: matrix `cross` musl build, binaries copied into the image. |

## Index

- D-01 Goals and migration strategy
- D-02 Crate layout and dependency boundaries
- D-03 Sans-IO logic: why `logic/` never needs async
- D-04 Runtime and the Kernel
- D-05 IDL: ROS2 `.msg` + CDR, committed generated code, published crates
- D-06 Schema evolution policy and API-break gates
- D-07 Key space
- D-08 Public API vs internal IPC; what the frontend talks
- D-09 Zero-copy and containers
- D-10 Comms contract
- D-11 Settings: Python compatibility, owned by the Kernel
- D-12 Standard per-service keys and the future service manager
- D-13 Logging
- D-14 Frontend library and the Vue 3 transition
- D-15 Recorder migration
- D-16 CI and deploy
- D-17 Python side of the migration
- D-18 REST gateways for migrated services
- D-19 Retired
- D-20 Process: how draft 2 is built
- D-21 Breaking changes for users and extension developers
- D-22 Retired
- D-23 Recording library: retire `recorder_extractor`, rebuild the Records frontend
- D-24 Zenoh inspector: BlueOS API, ROS 2 and Foxglove in one tool
- D-25 Service, Kernel and DomainState
- D-26 Endpoint manifest and generated wiring
- D-27 Tasks, Projections and the reconcile pattern
- D-28 Durable state
- D-29 Panics and recovery
- D-30 Code quality gates, tests and the Rust style guide
- D-31 Ergonomics goals
- D-32 Reused projects and what stays ours
- Open items

---

## D-01 Goals and migration strategy

Context: BlueOS today is a REST-based, non-distributed set of Python microservices with a per-service,
versioned JSON settings library.

Decision:

- Move to an **event-driven, non-distributed** microservice system.
- **Zenoh is the single communication backbone** for all traffic: data, logs, video, messaging, API
  (commands/queries), MAVLink, ROS2.
- **Recorder** conditionally records the backbone into MCAP.
- A **versioned BlueOS IDL API** built on ROS2 message definitions and serialization (D-05).
- An **internal-only IPC** keyspace, unversioned, allowed because the deploy guarantees coherence (D-08).
- New services are written in Rust with **CQRS-lite (no event sourcing) + jobs**.
- The frontend of new services uses the event-driven API, not REST.

Migration order:

1. Recorder is rewritten here as a BlueOS service, with a reworked frontend.
2. Setup wizard + calibration move from the frontend into a Rust backend service, redesigned to be
   product-oriented: guide users from zero to ready-to-fly, not only vehicle/BlueOS setup. Not in draft 2.
3. Python services migrate one by one until the Python venv disappears from the image.
4. Each migrated service gets a gateway translating the event-driven API to its current REST API (D-18).
5. A future system-wide service manager controls settings, command-line arguments, and lifecycle
   (enable/disable/start/stop/restart) of all services (D-12).

Consequences: every decision below is judged against "can Python and Rust services be swapped one at a time
with no user-visible breakage".

## D-02 Crate layout and dependency boundaries

Decision: every Rust crate lives in exactly one of three folders, in shared libs and services alike.

| Folder | Content | May depend on (workspace crates) |
|---|---|---|
| `logic/<block>` | A Domain or a Block: pure code, `#![no_std]` + `alloc`, no IO, no IDL types | `libs/logic/`, sibling logic crates of the same service |
| `logic/api` | Conversions between Messages and Domain types, and the settings document. Pure, `no_std` | `libs/logic/`, `blueos-idl`, the service's own logic crates |
| `adapters/` | Code touching the outside world | `libs/adapters/`, `blueos-idl`, its own service's `adapters/` and `logic/` |
| `app/` (service) | The `Service` implementation, the endpoint manifest, Tasks (D-25 to D-27) | anything in `libs/`, its own service |
| `app/blueos` (workspace) | Multicall binary | `libs/`, service `app/` crates via cargo features |

- A service is split into crates named after what they own: one Domain crate, one crate per Block, one
  `logic/api` crate, and its adapters. The one-crate layout for small services was rejected, so that the folder
  check stays mechanical.
- Single Cargo workspace at `core/Cargo.toml` with a single `core/Cargo.lock`. No per-service lock files.
- Naming: every Rust package is hyphenated, `blueos-<name>` for libs (`blueos-service`, `blueos-comms-zenoh`)
  and `blueos-<service>-<part>` for services (`blueos-recorder-domain`, `blueos-recorder-library`,
  `blueos-recorder-api`). Every workspace crate is listed in `[workspace.dependencies]` and members depend on it
  with `workspace = true`, never a relative `path`. Python packages keep their names.
- Test-only features (the comms `channel` backend and the `blueos-service` `testing` harness) are enabled in
  `[dev-dependencies]` only, so they never reach the shipped binary through feature unification.
- Layout: `core/libs/{logic,adapters,app}/...`, `core/services/<name>/{logic/<block>,logic/api,adapters/<thing>,app}`,
  `core/libs/idl/interfaces/` (`.msg` sources, D-05), `core/app/blueos` (multicall binary).
- One multicall binary `blueos`; each service is invoked as `blueos <service>` or via a symlink named after
  the service (argv[0]). Each service still runs in its own process. `main` resolves the name once, keeps a list
  of known names that is never feature-gated, puts the `#[cfg(feature)]` on the `match` arms, and calls
  `entry::run::<S>()` for every service (D-25). A known name that is compiled out gets one clear message;
  `--help`, `-h` and `--version` exit with 0.
- No crate depends on another service's crates. Services talk only over comms.
- Enforcement: `.hooks/lib/rust_checks.sh` (folder rules via `cargo metadata` + `jq`, `cargo deny` bans so
  `zenoh` stays behind the comms zenoh driver, a `cargo tree` check without dev edges that fails when a test-only
  feature is enabled, and a build of every `logic/` crate for `thumbv7em-none-eabihf` and `wasm32-unknown-unknown`
  so any IO dependency fails to compile).

Rationale: the folder decides the dependency rules, which keeps hexagonal boundaries mechanical rather than
reviewed by hand. Keeping IDL types out of the Domain and Block crates means the wire format (D-06) never becomes
a Domain constraint; `logic/api` is the one place a reader lists to see every mapping. Allowing adapters to depend
on their own service's logic is the normal hexagonal direction; forbidding it produced three copies of one type
in draft 1. The multicall binary keeps image size and the incremental update layer small.

## D-03 Sans-IO logic: why `logic/` never needs async

Context: reviewers will ask whether real services (calibration, wizard, recorder) can be written without
async in the domain.

Decision: yes. `logic/` is written in the **sans-IO** style (same approach as `quinn-proto`, `rustls`, and
the core of `h2`). Async is only about *waiting*; logic never waits, it represents waiting as data and
returns Effects. The Kernel (D-04) performs the waiting.

| Need | How it is modeled without async |
|---|---|
| Wait for an IO result (RPC, MAVLink command, file) | The Decision carries `Effect::Io(request)`; the Kernel runs it and feeds the result back as an IO result Command. |
| Timeouts, delays, retries | `Effect::Schedule { after, key, command }`; the Kernel delivers `command` as a Tick later. Cancel by key. Timer keys are a type each Domain declares (for example `CaptureStatus(stream)`), never a shared number space. |
| Current time | The Kernel reads the injected Clock once per Command and passes `now` (wall and monotonic) to every handler. Commands carry a time only when the time is part of the fact itself. Logic never reads a clock. |
| Random ids | Allocated by the caller or by deterministic counters. |
| Multi-step flows (wizard, calibration) | Explicit state machines and `blueos-jobs` graphs (`Sequence`, `Parallel`, cancellation) instead of `await` chains. |
| Heavy CPU (e.g. compass ellipsoid fit) | A pure function in `logic/`, executed by the Kernel as an IO request on a blocking thread so the Inbox stays responsive. |
| High-rate data (MCAP writing, video, sonar) | **Control plane vs data plane**: logic owns the policy, a Task applies it on the hot path through the reconcile pattern (D-27). Payloads never go through the Inbox, which also preserves zero-copy (D-09). |

Shape of the logic:

- A **Domain** is what the Kernel runs. A **Block** is a reusable sans-IO reducer that a Domain composes; it does
  not implement the `Domain` trait. Both return an `Outcome<Event, Tick, IoRequest, TimerKey>` (domain events and
  Effects, or a rejection with a typed reason); the Domain's own Outcome is its `Decision`. The only Command an
  Outcome carries is the Tick of a `Schedule` Effect. `Outcome::map` lifts a Block's Outcome into the Domain's types
  with the Domain's enum constructors, so composing a Block needs no fake Domain and no hand-written mapping
  functions.
- The `Domain` trait is split: `Domain` (Snapshot, one type per Command origin, Event, IoRequest, timer key,
  `handle`), plus `DomainQueries` and `DomainJobs`. A Domain without queries or jobs does not mention them.
  `DomainJobs` lives in `blueos-jobs`, next to the `Jobs` it returns, and the Domain keeps its `Jobs` in its
  Snapshot, so the transaction clone rolls them back with the rest of the DomainState.
  Associated type defaults are unstable Rust, so the split is the only way. A Domain or Block with no IO requests,
  no domain events, no timers or no Commands of one origin sets that type to an uninhabited type
  (`core::convert::Infallible` or an empty enum), never a placeholder; a `match` then needs no arm for it.
- A Command is grouped by origin: **Request** (from a Command endpoint), **IO result**, **Tick**, and
  **Observed fact** (from a Task). The Domain declares one type per origin and `handle` receives
  `Command<Request, IoResult, Tick, ObservedFact>`, so the grouping is checked by the compiler: wire code can build
  only a Request (D-26), a timer delivers only a Tick, and an IO executor returns only an IO result. A client can
  never forge an IO result or an observed fact. The grouping is also what documents the Commands.
- A Command that is invalid for the current Snapshot is rejected, never accepted and ignored.
- Domain state that must stay consistent is one enum, not several fields that can disagree: an enum for state
  that is stored, type-state for builders and resource handles (D-30).

Cost: a flow that would be five lines of `await` becomes explicit states.

Benefit (why we accept the cost): the state is inspectable, publishable to the UI, cancellable, resumable after
a restart (D-28), and unit-testable without a runtime, mocks, or sleeps. The product flows we want (wizard,
calibration) need exactly these properties.

## D-04 Runtime and the Kernel

Decision:

- tokio is accepted. Isolating from it buys nothing because `zenoh` already runs its own tokio runtime.
- The runtime is built by `entry::run::<S>()` (D-25), never by a service. Only the Kernel (`libs/app/service`),
  adapters and a service's `app/` IO code are async.
- **One Inbox (`mpsc`, 256 slots) per service.** Every Command reaches the Domain through it: Requests from the
  Command endpoints, IO results, Ticks, and Observed facts sent by Tasks through `CommandSender` (D-27).
- **Every Command is a transaction.** The Kernel clones the DomainState before `handle`. If `handle` or a
  Projection panics, or an Effect fails synchronously, the Kernel restores the clone, drops the domain events,
  and rejects the ack. A rejection by the Domain also restores the clone, so a rejected Command never changes the
  Snapshot. Large, rarely changed parts of a Snapshot go behind `Arc` so the clone stays cheap.
- Order after a successful `handle`: run the synchronous part of the Effects, update the Projections and publish
  the States, send the ack, then publish the Events. A client that reads a State right after the ack sees the new
  value.
- **The `Io` Effects of one Decision run in order, in one task.** Each one reports its own result, even if an
  earlier one failed. Different Decisions run concurrently. The Kernel awaits each IO task: an IO executor returns
  `Result<Option<Command>, IoError>`, and an error or a panic becomes `Domain::io_failed(request, error)`, so a
  job never stays Running forever and the Domain always learns which request failed.
- **Timers** live in a `tokio-util` `DelayQueue` that the Inbox loop polls itself. An expired timer is handled in
  the loop and never queued, so a cancel or re-arm always wins and no registration outlives its timer.
- IO code receives the service's **Context** by reference, together with the Snapshot it needs; the Kernel never
  clones the whole DomainState into IO.
- Startup order: declare every queryable and await it (a failure stops startup), put the `on_start` Commands into
  the Inbox, publish the initial States, and declare the liveliness token last. So "alive" means every endpoint
  answers, and no client Command runs before the startup Commands.
- A publish or encode error is logged and never stops the Inbox loop. A State value is stored for deduplication
  only after it was sent.
- Shutdown order (on SIGTERM, SIGINT or a trigger): signal the Tasks to leave their loops, dispatch the
  `on_shutdown` Command, drain in-flight IO for up to 5 s, join the Tasks with the remaining budget, then abort
  the stragglers and name them in a warning.
- The Zenoh backend is an `Arc<dyn CommsBackend>`: the Kernel defaults to Zenoh, tests pass the channel backend.
- No `unsafe`, no blocking call inside the Inbox loop, typed errors (no magic payloads such as `b"FAILED"`).

## D-05 IDL: ROS2 `.msg` + CDR, committed generated code, published crates

Decision:

- Message definitions are **ROS2 `.msg` files** under `core/libs/idl/interfaces/`, versioned per package, with one
  `package.xml` per package. That folder is the one real location; it sits inside the crate so the crate can be
  published.
- Serialization is **CDR** (ROS2 default). Zenoh `Encoding` is `application/cdr;<schema_name>`, the convention
  the Recorder uses to write MCAP channels with `ros2msg` schemas. Strings are not padded after their terminator,
  as in standard CDR (TypeScript, Python, Foxglove).
- `.msg` parsing uses `roslibrust_codegen`. The code generator (`blueos-idl-codegen`) emits: Rust types with a CDR
  codec, the schema text per type (`SCHEMA_NAME`, `SCHEMA`, and `blueos_idl::schema(name)` for every message),
  defaults for missing trailing fields (D-06), TypeScript types, and TypeScript constants as values.
- Schema text lists the root definition first and its dependencies after it, under `MSG: package/Name` headers,
  which is what `@foxglove/rosmsg` and MCAP readers expect.
- When a field `x` has sibling constants `X_*` of the same type, the generator emits a Rust enum with an
  `Unknown(raw)` variant, so an older reader survives a new value. Domain enums convert from it in `logic/api`.
- Decoders never trust a length from the payload: preallocation is bounded by the remaining bytes, offsets use
  checked arithmetic, and hostile inputs (such as a sequence length of `0xFFFFFFFF`) are regression tests.
- **All generated code is committed**, Rust (formatted with `prettyplease`) and TypeScript, by an explicit
  `cargo run -p blueos-idl-codegen -- --write`, wired into `.hooks/pre-push --fix`. CI regenerates into a temporary
  folder and byte-compares; a difference fails with the command that fixes it. There is no `build.rs`. Any change
  to a `.msg`, to the generator, or to a generator dependency is therefore a reviewed diff.
- Two crates are **published on crates.io** so extensions and ROS2 nodes can use them:
  - `blueos-idl`: generated types + codec + embedded schemas, `no_std` + `alloc`, with no `std` feature
    (`core::error::Error` is enough).
  - `blueos-api`: key conventions (D-07), encoding strings, optional zenoh helpers (D-10).
- Names `blueos-idl`, `blueos-api` (and `blueos`, to reserve) were free on crates.io on 2026-09-24.
- No zBlueberry branding anywhere.
- Published crates never depend on unpublished workspace crates; committing the generated code is what removes the
  generator from `blueos-idl`'s dependencies.
- The Recorder must record every ROS 2 and Foxglove message, not only BlueOS ones. The upstream definitions
  (ROS 2 Jazzy interface packages and the Foxglove SDK's `schemas/ros2`) are vendored unmodified in
  `core/libs/idl/catalog/` and exposed as schema text only, by the `blueos-idl` `catalog` feature. They get no
  types and no `api.lock` entries, since they are not BlueOS API. `foxglove.Name`, the name the Foxglove SDK
  and mavlink-camera-manager publish, resolves to `foxglove_msgs/msg/Name`.

Rationale: `.msg` + CDR gives native ROS2 interop and Foxglove/MCAP support out of the box; embedding the
schema removes the need for the Recorder to find `.msg` files on disk. The generated types are the product that
extensions read, so they are committed where `rg` and reviewers find them, like `prost-types` and windows-rs.

## D-06 Schema evolution policy and API-break gates

Context: plain CDR has no schema evolution (fields are positional). DDS XTypes (appendable/mutable, XCDR2)
can evolve but must be declared in `.idl` with annotations, `.msg` cannot express it, support across ROS2
middleware is uneven, and `rmw_zenoh` uses plain CDR. Protobuf/FlatBuffers are MCAP-friendly but not ROS2 topic
formats.

Decision: accept the limitation, with a strict policy:

- A published `v1` message that is only ever sent on its own is **append-only**: new fields may only be added
  at the end.
  - New writer, old reader: works (decoders stop after the fields they know).
  - Old writer, new reader: our generated decoders (Rust, TypeScript, Python) fill missing trailing fields
    with defaults. Native ROS2 nodes will fail; this is documented.
- A message used as a field or as a sequence element of another message is **frozen**. A decoder fills missing
  fields only at the end of the whole buffer, so an appended field in a nested message shifts every byte after it.
  Changing a frozen message requires a new message type.
- Any other change (remove, reorder, retype, rename a field; change semantics) requires a **new message type
  or a new major version** of the package/keyspace.
- Versioning is the major version in the key (D-07) plus the gates below. No type hash travels with a message:
  under append-only evolution a new writer and an old reader have different hashes by design, so a hash check
  could only break the compatibility this policy promises.

Test gates (mandatory, run in `.hooks/pre-push` and CI):

- `core/libs/idl/api.lock` stores the field signature of every message and every endpoint key with its
  Message types (D-26). A test compares the current tree with it: a change other than an append to a top-level
  message, a change to a frozen message, or a removed or renamed endpoint key fails unless the major version was
  bumped. The test computes the set of frozen messages itself. Updating the lock is an explicit, reviewed step.
- `cargo semver-checks` on the published crates once they are on crates.io. Until then `api.lock` is the gate.

## D-07 Key space

Decision:

- Public, versioned API keys: `blueos/v1/<service>/...`.
- Internal IPC keys (when introduced): `blueos/@ipc/...`, unversioned.
- The draft 1 zenoh topic layout is not preserved (D-21). Python producers move to the IDL too (D-17).
- Exact sub-structure (commands, state, events, jobs, settings, info) is defined by D-10 and D-12, generated from
  each service's endpoint manifest (D-26), and lives in `blueos-api` as constants/helpers, never as ad-hoc strings
  in services or in the frontend.

## D-08 Public API vs internal IPC; what the frontend talks

Decision: **the frontend talks the versioned IDL API**, not the internal IPC.

Rationale:

- Zero-copy (the main reason for IPC) does not reach the browser: `zenoh-ts` runs over a websocket.
- Deploy coherence does not hold in the browser: a tab left open across an update, Cockpit, or a mobile
  client are out of sync with the backend.
- Dogfooding: if the BlueOS frontend is built on the public API, extensions and third parties can be too.
- Recordings stay decodable: the Recorder records the backbone; schema-less IPC payloads produce MCAP files that
  other versions cannot read.

IPC:

- Purpose: let BlueOS internals change without breaking the external API; zero-copy where it matters.
- Deferred until a concrete internal-only consumer in another process needs it. Code inside one service's
  process never uses the backbone to reach its own Domain; it uses `CommandSender` and Projections (D-27).
- Internal debug views may read IPC keys; product UI may not.

## D-09 Zero-copy and containers

Context: the concrete zero-copy pipeline today is MCM (mavlink-camera-manager) to the Recorder. Extensions
(e.g. ping sonar) should benefit too. A custom `zenohd` build with shared memory (SHM) enabled replaces the
current one. Zenoh uses SHM automatically when payloads exceed ~3 KB, both peers are on the same host, and
both support it (MCM, Recorder, BlueOS zenoh do).

Decision:

- Comms never copies payloads on the way through, on publish or on receive: no framing prefix, the payload is a
  cheap-clone bytes type (D-10) that hands back its inner buffer without a copy and offers a borrowed accessor,
  and the CDR reader borrows its input. The Recorder keeps the refcounted `ZBytes` path up to the MCAP write.
- **Extensions need `IpcMode: host` in their Docker permissions to share SHM. This must be documented** in
  the extension developer docs (Kraken permissions, extension template) and in the `blueos-api` README.
  Document the narrower alternative too: bind-mounting `/dev/shm` (host IPC mode shares the whole host IPC
  namespace).
- Core already shares SHM because it binds all of `/dev/` (`bootstrap/startup.json.default`).
- Without either setting, extensions silently fall back to network transport; the docs must say so.

## D-10 Comms contract

Decision (implemented in `libs/adapters/comms*`, constants in `blueos-api`):

- Connect as a zenoh **client** to the local `zenohd` (`tcp/127.0.0.1:7447` by default, D-25 CLI), like Python's
  `commonwealth/utils/zenoh_helper.py`. No peer-mode listen/connect fallback.
- No payload framing. Correlation and metadata go in Zenoh **attachments**; Zenoh queries already correlate
  replies.
- `Sample` exposes key, payload (cheap clone), encoding, timestamp, attachment.
- Key expressions with wildcards (`**`, `*`, `$*`) are matched with `zenoh-keyexpr`, in the Zenoh backend and in
  the channel test backend alike.
- **State** keys = queryable (current value, for late joiners) + update stream, published only on change.
  Clients **subscribe first, then query**, and ignore the query reply if a sample has already arrived; the other
  order loses an update published between the two.
- **Commands** are Zenoh queries; the reply is an IDL `CommandAck { accepted, job_id, reason }`. `job_id` is the
  root job this Command created, or 0 when it created none; job ids start at 1. Progress and results come as
  Events or States, not in the reply.
- **Queries** (CQRS read side) are Zenoh queries answered from the Snapshot.
- Queryables reply with their declared key, not the query's, so a wildcard `get` tells replies apart. A `get` on a
  command key runs the command, so tools never `get` wider than `*/state/*`, `*/settings` and `*/jobs`.
- Liveliness token per service (D-12).
- In-process channel driver kept for tests. It sends a query to every matching queryable, as Zenoh does, and
  removes closed subscribers.

## D-11 Settings: Python compatibility, owned by the Kernel

Context: Python services use `commonwealth.settings` (legacy `settings.py`/`manager.py` and the Pydantic
`managers/pydantic_manager.py` + `bases/pydantic_base.py`): files named `settings-<VERSION>.json` in
`appdirs.user_config_dir(<project>)`, a `VERSION` field, migrations from older versions, refusal of versions
from the future, highest-version-first loading, temp-file cleanup.

Decision:

- Rust settings are **100% compatible** so a Rust and a Python implementation of the same service can be
  swapped with no breakage (same folder, file names, JSON shape, `VERSION`, migration semantics). The folder
  fallback is hand-written XDG because `dirs::config_dir()` returns `None` without `HOME`; the code says so.
- **Golden-file tests**: fixtures written by the Python library are loaded and round-tripped by Rust.
- Scope: each service only reads/writes its own settings. The settings document type lives in the service's
  `logic/api` crate (D-02) and denies unknown fields.
- **The Kernel owns settings; a service only says how the document relates to its Snapshot.** The service
  implements a settings port with three mappings (document into Snapshot, Snapshot into document, document into the
  update Command) and a constant list of restart-required fields. The Kernel:
  - loads the document once at startup, before the first Command, and puts it into the Snapshot;
  - on `UpdateSettings`, rejects a `VERSION` other than the service's own, applies the update in the Domain on the
    cloned DomainState (D-04), persists the document derived from the Snapshot, and on a write failure restores
    the clone and rejects; it then publishes the `settings` State from the same document and acks;
  - writes atomically (temporary file, fsync, rename, fsync of the folder), because vehicles lose power often.
- `Effect::Persist` does not exist: no Domain can forget to persist, and the published State cannot disagree with
  the file.
- **One door per setting**: settings change only through `UpdateSettings`; no service-specific Command changes
  a setting.
- On the wire the document stays JSON inside `SettingsEnvelope`, the same document as the file. Settings evolve
  through `VERSION` migrations, which must not be coupled to API stability (D-06).
- Restart-required fields: the Kernel keeps the document the process is running with and the saved one, and
  publishes the fields that differ as part of the `settings` State, so a UI that opens later still sees that a
  restart is pending, and changing a field back clears it.
- Command-line arguments are a third category, owned by the future service manager, not stored in the
  service's settings file.

## D-12 Standard per-service keys and the future service manager

Decision: the Kernel gives every service, for free:

- Liveliness token `blueos/v1/services/<name>` (alive/dead), declared last at startup (D-04).
- `info` queryable at `blueos/v1/<name>/query/info` (name, version, build, capabilities, and every endpoint from
  the manifest, D-26).
- `status` State: ready by default; degraded, with the failing Task named in `detail`, while a Task is restarting
  or after an Inbox loop recovery (D-29).
- `settings` State + `UpdateSettings` command, including the pending restart fields (D-11).
- `jobs` State, only for Domains that have jobs, derived by the Kernel from the DomainState: the live job graphs
  plus a bounded history of finished ones. Finished root graphs are dropped after a retention count. A Domain
  with jobs opts in with `ServiceBuilder::jobs`. Job ids start at 1; the ack of a Command that started a root Job
  carries its id, and every other ack carries 0.
- `log` stream (D-13).

These names are reserved: the endpoint manifest cannot reuse them.

The future system-wide service manager (settings, command-line arguments, start/stop/restart/enable/disable)
is then just a client of these keys plus process/container control. Nothing service-specific is needed.

## D-13 Logging

Decision:

- `tracing` everywhere, always with structured fields (D-30). The logging adapter publishes each record as a
  foxglove `Log` message (CDR, via `blueos-idl`) on the service's `log` key. The Recorder records these; the
  frontend console and extension-log views read them.
- Logging starts on the first line of `entry::run`, with the `-v` count read from the raw arguments, so it cannot
  fail and every later failure is printed. The console layer works at once; the Zenoh layer attaches when the
  Session exists, and records produced before that are buffered and replayed. Each record is timestamped when it
  is created.
- The Zenoh layer ignores `zenoh*` targets, so publishing a log line at trace level cannot generate more log lines.
- A panic hook routes every panic into `tracing`, so panics reach the `log` key and the MCAP file (D-29).

## D-14 Frontend library and the Vue 3 transition

Decision:

- `core/frontend/src/libs/blueos-api/` is **plain TypeScript with no Vue imports**. It exposes: subscribe to a
  State then query it; send a command and get `accepted/rejected + job_id`; watch jobs; decode/encode IDL types.
  States, settings and jobs share one watcher helper.
- The library reaches the backbone only through a `Transport` with two operations, `subscribe` and `get`.
  `zenohTransport` implements it over the `zenoh-ts` Session; vitest drives the library with an in-memory fake, so
  no unit test needs the network, a timer or a sleep.
- `watchState` subscribes, then queries, and ignores the reply for a key that already had an update (D-10). A
  wildcard key follows each matching key on its own. `sendCommand` returns the decoded `CommandAck`: a rejection is an
  ack, while no reply or an error reply throws a typed error.
- Each service gets a typed TypeScript client generated from its endpoint manifest (D-26): keys, request and
  response types, States and Events. The frontend writes no key strings and copies no constants by hand.
- A thin Vue 2 wrapper (mixin or store module) sits on top.
- BlueOS will move to Vue 3; that migration only replaces the wrapper with a composable. The core library is
  unchanged.
- Decoding uses `@foxglove/rosmsg2-serialization` with the embedded schema text and generated types.
- WASM builds of the Rust codec are **not** used for now (D-24 says when they will be).

## D-15 Recorder migration

Decision:

- Rewrite `blueos-recorder` in this repository and retire the external repository afterwards.
- Crates (D-02):

| Crate | Folder | Owns |
|---|---|---|
| `blueos-recorder-domain` | `logic/recorder` | The Recorder Domain: the root Snapshot, the public Command enum, composition of the Blocks |
| `blueos-recorder-capture` | `logic/capture` | Block: the active recording, the armed flag, bytes written, the record gate |
| `blueos-recorder-cameras` | `logic/cameras` | Block: the MAVLink camera protocol (capture commands and status replies) |
| `blueos-recorder-library` | `logic/library` | Block: the recording library (D-23) |
| `blueos-recorder-schema-gate` | `logic/schema-gate` | A pure state machine, driven by the data plane, that holds ROS 2 samples until their schema is known (D-24) |
| `blueos-recorder-api` | `logic/api` | Conversions and `RecorderSettings` |
| adapters | `adapters/{mcap,mavlink,storage}` | MCAP writing, MAVLink parsing, the recordings folder |

- Names: `RecorderSettings` (the one settings type), `ActiveRecording` (the recording the Domain wants),
  `McapFile` (an open MCAP file), `RecordGate` (the Projection the data plane follows), `RecorderSessionState`
  (the IDL message for the recording state), `RecordingFileState` (the lifecycle of one file in the library),
  `RecordingOperationKind`. "Session" means only the Zenoh connection.
- The data plane is a Task that owns the `McapFile` and follows the `RecordGate` through the reconcile pattern
  (D-27): it opens, rotates and finishes files itself and reports opened, finished and bytes written as
  Observed facts. There is one source of truth for the current file.
- MAVLink facts (armed state) are Observed facts that carry the full current value and are re-sent periodically,
  so a dropped one heals (D-27). Each camera has its own capture status timer key.
- Preserve the contract MCM relies on (`--recorder=external`): `video/...` topics, MAVLink camera capture
  commands and status replies, and "record MAVLink only while armed".
- Keep the zero-copy path (D-09). The data plane builds a channel descriptor once per channel, not per sample.
  MCAP schemas come from the embedded IDL schemas (D-05) with the existing JSON fallback.
- `auto_start_recording` is either applied live or declared restart-required; it is never silently ignored.
- Frontend reworked on `blueos-api` (D-14).
- radcam-manager stays in its own repository.
- Runs as `recorder --recorder-path /usr/blueos/userdata/recorder` (symlink to the `blueos` multicall binary),
  same arguments and slot as the retired binary in `core/start-blueos-core`.
- Device acceptance checklist (test layer L6, D-30), run before merging: MCM `--recorder=external`; armed gating;
  video over SHM; MAVLink capture replies with two cameras; a session rotation while recording; `docker stop`
  mid-recording; an image for `linux/arm/v7` built from CI artifacts; repair cancel by hand; the library rescan
  with hundreds of recordings; snapshot and repair under heavy write load.

## D-16 CI and deploy

Decision (radcam-manager model):

- A matrix CI job cross-builds with `cross` for `aarch64-unknown-linux-musl`,
  `armv7-unknown-linux-musleabihf`, `x86_64-unknown-linux-musl`, in parallel with the Python pipeline, with
  `cargo auditable build` and a `cargo bloat` report.
- The Docker image installs the single `blueos` binary (selected by `TARGETARCH`) plus service symlinks in the
  **last layer**, for cache reuse and small incremental updates. The per-target binaries are bind-mounted
  (`RUN --mount=type=bind,source=target/build`), not copied, so the other architectures' binaries never land
  in a layer; `core/.dockerignore` re-includes only `target/build/*/*/release/blueos`.
- Shipped features: `recorder` only. The teaching example and the cookbook are never shipped.
- Release binaries are about 14 MB per target (musl, stripped, thin LTO).
- Local builds: `cd core && ./build_cross.sh`.
- The Rust gates and CI jobs are in D-30.

## D-17 Python side of the migration

Decision:

- Python producers/consumers (commonwealth zenoh helper and logs, kraken zenoh handlers) use `blueos/v1/` keys
  and CDR IDL payloads.
- Python uses **runtime `.msg` parsing** plus CDR instead of a third codegen target, since Python is being
  phased out. It is a small pure-Python parser and codec in `commonwealth/utils/blueos_idl.py` (stdlib
  `struct` only), checked against the shared CDR test vectors (D-24).
- Rejected: `rosbags` pulls `numpy`, `apsw`, `lz4`, `zstandard` and `ruamel-yaml` into the image (none present
  before), `numpy`/`apsw` have no armv7 wheels, and it needed workarounds to get the D-06 trailing-field defaults.
  `pycdr2` with generated dataclasses is pure Python but has had no release since 2022-12; our codec is already
  tested and Python is going away.
- The `.msg` files reach the image through the existing `COPY libs` (`/home/pi/libs/idl/interfaces`);
  `BLUEOS_IDL_INTERFACES` overrides the path.
- Without the `.msg` files, a Python service logs once and skips liveliness and `info`.
- Frontend consumers of those keys (Zenoh inspector, console logger, extension logs) are updated together.

## D-18 REST gateways for migrated services

Decision: when a Python service is migrated, a gateway translates the event-driven API to that service's
current REST API so existing clients keep working. Gateways are adapters with no domain logic; they are
dropped once no client needs them. Not built in draft 2.

## D-19 Retired

Draft 1's findings on the first POC. Every resolution is now stated in the decision it belongs to.

## D-20 Process: how draft 2 is built

Decision:

- Draft 2 starts on a new branch from `master`. It is written **test-first**, and the Kernel is designed around
  its test harness (layer L3, D-30).
- **Porting rule.** Only data is copied as-is: test vectors, Python settings fixtures, the vendored ROS 2 and
  Foxglove catalog, `.msg` files and `api.lock`. Code is rewritten test-first, with draft 1 read only as a
  reference. A draft 1 module may be copied unchanged only if it already passes every day-zero gate (D-30) with
  no `#[allow]` and passes a review against the Rust style guide.
- Every finding of the correctness review and every known Recorder bug lands as a failing test before the code
  that fixes it. The known Recorder bugs: all cameras share one capture status timer; a late `SessionFinished`
  after a rotation wipes the new session; `recording_time_ms` is always 0; a rotation can close the file it just
  opened.
- Build order: the foundation (gates and CI, IDL and codegen, Kernel with its harness, comms, logging,
  settings), then the teaching example, then the Recorder (backend and frontend).
- The teaching example is `core/services/example/`:
  - `example-minimal`: one Command, one Query, one State, a Domain unit test and the frontend call, about
    150 lines (a goal, D-31).
  - A cookbook of numbered entries, `core/services/example/cookbook/NN-<topic>.rs`, each a self-contained test
    on the channel backend, answering every "how do I do X?" question (34 today, listed in the ergonomics review
    P4). A README table maps each question to its entry, and a test asserts that every question resolves to an
    entry that compiles.
  - The example's real `build()` is what its tests exercise; no test rebuilds the wiring by hand.
  - The example passes every rule of the style guide with zero `#[allow]`; an agent copying it inherits the style.
  - The example README covers adding a new service: the `main.rs` arm, the feature, nginx and
    `core/start-blueos-core`.
- Once the Kernel exists, a new architecture overview replaces the draft 1 one in `docs/architecture/`: how a Service
  works end to end, and a "where it is in the code" table keyed by the `GLOSSARY.md` terms. Service READMEs link to
  the glossary and never redefine its words.
- The stacked-PR split is decided once the Recorder runs on a device.

## D-21 Breaking changes for users and extension developers

Decision: no legacy bridge (D-07). These changes go into the release notes and the extension developer docs.
Anything still using the old keys (Cockpit, Foxglove bridges, extensions) gets no data and no error.

| Before | After |
|---|---|
| `services/<service>/log`, JSON `foxglove.Log` | `blueos/v1/<service>/log`, `application/cdr;foxglove_msgs/msg/Log` |
| `extensions/logs/<identifier>`, plain text | `blueos/v1/kraken/log/extension/<sanitized identifier>`, CDR `foxglove_msgs/msg/Log` |
| Python REST-over-Zenoh `<service>/<path>` | `blueos/v1/<service>/http/<path>` (still JSON) |
| `kraken/extension/logs/request` | `blueos/v1/kraken/http/extension/logs/request` |
| No discovery | Liveliness `blueos/v1/services/<service>`, CDR `ServiceInfo` at `blueos/v1/<service>/query/info` |
| `blueos-recorder` release binary | `recorder` symlink to the in-image `blueos` multicall binary |
| `/recorder-extractor/v1.0/*` REST API | Recorder library API under `blueos/v1/recorder/` (D-23) |

Native ROS2 subscribers get plain CDR without XTypes: they cannot read messages from an older writer that
lacks trailing fields (D-06).
Extensions publishing to the Recorder must follow the `IpcMode` note in D-09.

## D-22 Retired

Draft 1's branch review outcomes. Fixes are stated in the decisions they touch, and the evidence is in
`docs/architecture/draft-1/rust-service-review.md` and `docs/architecture/draft-1/rust-service-ergonomics-review.md`.

## D-23 Recording library: retire `recorder_extractor`, rebuild the Records frontend

Context: the Python `recorder_extractor` (branch `video_player_tidy2`, commits `614d2a67a` and `43d273355` in
`~/BlueRobotics/BlueOS-docker`) turned into an MCAP catalog with repair, snapshot and delete, and the Records page
into an in-browser MCAP player. Both talked REST and polled.

Decision:

- The Recorder owns its recordings. The library is a Block, `core/services/recorder/logic/library`
  (`blueos-recorder-library`), composed into the Recorder Domain. `recorder_extractor` is deleted with its uv
  workspace entries, nginx location and `start-blueos-core` line.
- **Bytes stay on nginx.** `/userdata/recorder/<path>` serves files with HTTP ranges (CORS exposes
  `Accept-Ranges` and `Content-Range`); browsers need ranges and downloads, which Zenoh does not give them.
  This is the one exception to D-08: the IDL API carries the catalog and control, never recording bytes.
- **Event-driven, no polling.** The library is a State; outcomes are Events; the frontend watches both.
- Native repair: the `mcap` crate rewrites a recording in-process (no `mcap` CLI subprocess; progress is the
  exact read offset). Output goes to a `.recover` temporary file renamed over the original; cancel removes the
  temporary file and leaves the original untouched; leftovers, nested ones included, are discarded at startup. A
  repair running at shutdown is not cancelled; the 5 s drain applies.
- A recording still being written is downloaded through `SnapshotRecording`: the same rewrite writes an indexed
  copy `<stem>.snapshot-<UTC>Z.mcap` next to it, and the browser downloads it from nginx once the `operation`
  Event names it. The download also completes from the `library` State and times out.
- The recording-file suffix rule (case-insensitive `.mcap`) and the file name timestamp formats are defined once
  and shared with the frontend through a test vector (D-24). Timestamps are parsed and formatted with `chrono`.
- The Recorder knows which file it is writing. Other files are rescanned on a timer and after each operation; the
  State is republished only when it changes. ponytail: timer rescan (5 s); switch to inotify if external writers
  or large folders make it costly.
- `RecordingFile.allowed_operations` is published by the library from the same rules that reject Commands, so the
  frontend never duplicates them.
- The `index` IO query is answered by an adapter outside the Inbox (reads that need disk but no Domain state),
  one request at a time per query name, with a timeout. The walk reports the size seen at its start; the frontend
  asks again when `library` reports a new size.
- Frontend layering mirrors the backend (D-02, D-14):
  - `src/libs/mcap/logic/`: pure TypeScript, no DOM, no network (record parsing, keyframe index, frames,
    codec parameters, CSV, muxing). Unit-tested with vitest in Node.
  - `src/libs/mcap/adapters/`: IO behind small interfaces (`ByteSource` over `fetch` ranges, WebCodecs/MSE
    players, canvas thumbnails, thumbnail cache). The index source is an interface; the recorder client
    implements it with the `index` query.
  - `src/libs/recorder/`: framework-agnostic recorder client on the generated client (D-14). No Vue imports.
  - Vue 2 components (`components/records/*`, `RecordsView.vue`) only bind these to templates. Records shows an
    explicit empty state when the Recorder is not running.

API (keys under `blueos/v1/recorder/`, messages in `blueos_recorder_msgs`):

| Kind | Name | Message |
|---|---|---|
| state | `library` | `RecordingLibrary` (`RecordingFile[]`, newest first) |
| command | `RepairRecording` / `CancelRepair` / `DeleteRecording` / `SnapshotRecording` | `...Command { path }` |
| event | `operation` | `RecordingOperation` (repair, snapshot, delete; succeeded, cancelled, failed with a reason, output path) |
| io query | `index` | `RecordingIndexRequest` -> `RecordingIndex` (paged chunk index + raw metadata records) |

Rejections (from the Python rules): repair when already repairing, already indexed, being written or written
less than 10 s ago; cancel when not repairing; delete while being written, repaired or already being deleted;
snapshot of a missing file; any path that is absolute, contains `..`, is not `.mcap`, or is not in the library.
A path is parsed once at the boundary into a validated recording path type; the Domain never receives an
unvalidated path.

## D-24 Zenoh inspector: BlueOS API, ROS 2 and Foxglove in one tool

Context: the Zenoh inspector is meant to become the default tool to interact with Zenoh: discover services, read
State, send Commands, and show ROS 2 and Foxglove traffic. Specialized views (video now; table, plot, vehicle
frame later) must plug into compatible data.

Decision:

- Layering mirrors D-23: `src/libs/zenoh-inspector/logic/` (pure TypeScript, vitest), `adapters/` (zenoh-ts,
  `blueos-api`, `requestAnimationFrame`), a framework-agnostic controller, and Vue 2 components that only bind.
  Ports live in `logic/types.ts`.
- **Views are a registry** of `{ id, label, supports(topic), priority }`. JSON is always available; the video
  player is the default for `CompressedVideo` and `video/` topics. New views are registry entries.
- **Services describe their API.** `ServiceInfo` carries `EndpointInfo[] endpoints`: kind, name, key, request and
  response schema, generated from the endpoint manifest (D-26), so the inspector can list every Command, Query,
  State and Event and build a command form from the request schema. Python services publish an empty list.
- The inspector sends Commands and Queries to BlueOS services, plus raw JSON/text queries to any key (the
  Python `http/` gateway). Requests to ROS 2 services are not sent (rmw_zenoh needs its request attachment).
- **Schema resolution**, in this order, in TypeScript and in the Recorder:
  1. the encoding suffix `application/cdr;<pkg>/msg/<Name>`;
  2. the transport: the rmw_zenoh data key `<domain>/<topic>/<pkg>::msg::dds_::<Name>_/<RIHS01 hash>`, or the
     `@ros2_lv` liveliness token of rmw_zenoh or `zenoh-plugin-ros2dds`;
  3. BlueOS IDL, then the vendored ROS 2 and Foxglove catalog (D-05), also emitted to TypeScript;
  4. otherwise raw bytes (inspector: size and hex preview, decoded again when the type arrives).
- ROS 2 publishers set no Zenoh encoding, so samples arrive as `zenoh/bytes`. `zenoh/bytes`, an empty encoding
  and bare `application/cdr` are ROS 2 candidates only when the payload starts with a CDR encapsulation header.
  Only a ros2dds publisher token marks a key as ROS 2; other keys outside `blueos/v1` are "Other", and the video
  view is chosen by the `video/` prefix because the camera manager sends `application/cdr` without a schema.
- ROS 2 liveliness is joined per topic from the set of alive publisher tokens; subscriber, service and node
  tokens are listed as entities without data.
- Decoding by reflection stays on `@foxglove/rosmsg` and `@foxglove/rosmsg2-serialization`. No new codec.
- The Recorder, for a `ros2dds` sample that arrives before its token: a liveliness `get` on the router for that
  topic, a bounded per-topic queue (2 s or 64 samples, payloads held by reference, D-09) flushed with the resolved
  schema, and on timeout a schema-less channel followed by a second channel with the schema once it resolves
  (MCAP allows several channels per topic). A type with no known schema lands on the schema-less fallback channel
  instead of being dropped. This is the schema gate of D-15.
- ROS 2 name parsing is a pure crate, `core/libs/logic/ros2-names` (`blueos-ros2-names`), mirrored in
  TypeScript.
- **Shared test vectors instead of WebAssembly** for logic that exists in more than one language: one JSON
  fixture per concern, checked by `cargo test`, vitest and pytest where applicable: ROS 2 names, BlueOS keys,
  CDR with D-06 defaults, and the recording file names (D-23).
- WebAssembly is adopted for `logic/` crates when the browser first needs domain rules that a query cannot
  replace (wizard or calibration steps offline, settings validation). D-02 keeps those crates `no_std` without
  IO and builds them for `wasm32-unknown-unknown` so the door stays open.

Rejected or deferred:

- **Rerun, Foxglove and RViz become BlueOS extensions**, not core. Each needs its own data layout or a gateway on
  the vehicle. Publishing standard ROS 2 messages (`sensor_msgs`, `geometry_msgs`, `diagnostic_msgs`,
  `foxglove_msgs`) at the producers serves all of them.
- **Hiroz** (ZettaScale, pure-Rust ROS 2 on Zenoh) is deferred to an experiment branch: 0.2.0, experimental,
  mostly one author, turns on zenoh `unstable`/`internal`, opens its own session, and its hash-in-key matching
  conflicts with D-06. `roslibrust` 0.26 has a `hiroz` feature, which is the cheap way to try it later. dora-rs
  and copper-rs replace the process model and are not ROS 2 over Zenoh.

## D-25 Service, Kernel and DomainState

Context: in draft 1 the runtime, the CLI, the settings, logging, the Session and background tasks each had a
different owner, "Kernel" was an 18-argument function, and "App" had three meanings.

Decision:

- Ownership is split by lifetime. **The Kernel owns everything that can be stopped; the DomainState owns
  everything that can be copied.** Tasks cannot live in the DomainState, because it is cloned for every Command
  (D-04).

| Owner | Lifetime | Owns |
|---|---|---|
| Service | the process | CLI values, logging, the tokio runtime, the settings folder, the Session, the Context, the Kernel, the exit code |
| Kernel (a public type) | one run of the Inbox loop | the Inbox, endpoint adapters, Tasks, timers, Projections, the liveliness token, the DomainState |
| DomainState (draft 1 `blueos_cqrs::App`) | every Command | the Snapshot and the Jobs, data only |

- "App" names only the `app/` folder.
- Every service implements one trait:

```rust
impl blueos_service::Service for Example {
    type Domain = PumpDomain;
    type Arguments = crate::cli::ExampleArguments;

    const NAME: &'static str = endpoints::NAME;
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn build(context: &ServiceContext<Self::Arguments>) -> Result<ServiceBuilder<PumpDomain>, ServiceError> {
        /* pure: register endpoints (D-26), Tasks and Projections (D-27); no IO, no spawning */
    }
}
```

- `entry::run::<S>()` does everything every service used to repeat: start logging (D-13), parse the CLI, build
  the runtime, open the Session, load the settings (D-11) and the durable state (D-28), call `build`, run the
  Kernel, and map the result to an exit code (D-29). `build` receives that Session through
  `ServiceContext::session()` so the Context and IO executors can publish before any Task runs, because an IO
  Effect can run before the first Task starts. `VERSION` is a constant in the service crate because
  `env!("CARGO_PKG_VERSION")` expands where it is written.
- Tests call `build` directly, so the wiring a test exercises is the wiring that ships.
- The Kernel parses a common CLI, and each service extends it with `type Arguments: clap::Args`:
  `-v` (counted), `--settings-path <DIR>`, `--zenoh-endpoint <ENDPOINT>` (default `tcp/127.0.0.1:7447`),
  `--zenoh-config <FILE>` (also `ZENOH_CONFIG`), `--zenoh-set <PATH=JSON5>` (repeatable). Paths are `PathBuf`;
  arguments are not shell-expanded.
- Inside `app/src/`, a service keeps a fixed layout documented in the example README: `lib.rs` (module list only),
  `service.rs` (`impl Service`), `cli.rs`, `tasks.rs`, the generated `endpoints.rs` (D-26) and the custom
  handlers in `handlers.rs`.

## D-26 Endpoint manifest and generated wiring

Context: draft 1 had eleven registration methods with different closure shapes, so a junior's first question
("which one do I use?") had eleven answers, and the endpoint list was rebuilt at runtime from closures.

Decision:

- Each service commits `app/endpoints.toml`, listing every public endpoint: its kind (command, query, io query,
  state, event), name and Message types. The key is derived, never written: `blueos/v1/<service>/<kind>/<name>`
  (D-07), and an IO query is a query on the wire. `services/tank` is the test Service of every shape:

```toml
service = "tank"

[command]
Drain = { request = "blueos_example_msgs/msg/EmptyRequest" }
SetLevel = { request = "blueos_example_msgs/msg/SetLevelRequest", custom = true }

[query]
Level = { request = "blueos_example_msgs/msg/EmptyRequest", response = "blueos_example_msgs/msg/LevelQueryResponse" }

[io_query]
Probe = { request = "blueos_example_msgs/msg/EmptyRequest", response = "blueos_example_msgs/msg/LevelQueryResponse" }

[state]
tank = { message = "blueos_example_msgs/msg/LevelQueryResponse" }

[event]
LevelChanged = { message = "blueos_example_msgs/msg/LevelQueryResponse" }
```

- The generator (the same tool and the same commit-and-compare rule as D-05: `blueos-idl-codegen --write`
  regenerates, `--check-endpoints` runs in `.hooks/pre-push` and CI) emits, committed:
  - `logic/api/src/endpoints.rs`: the `Conversions` trait, one function per endpoint named after it in snake
    case. The Domain implements it in `logic/api`: a Message becomes a Request or a Query, a Response becomes the
    reply Message (`<name>_response`), the Snapshot becomes a State Message, and a domain event becomes an Event
    Message or `None`. `From` impls cannot do this: the Message and the Domain type are both foreign to
    `logic/api`, so the orphan rule forbids the impl, and two endpoints that share a Message would need two.
  - `app/src/endpoints.rs`: the Service `NAME`, `register`, which declares every endpoint on the
    `ServiceBuilder`, and the `Handlers` trait. `Handlers` has a method only for an endpoint marked `custom` (a
    Command or a Query whose conversion can refuse the Message or needs the Context) and for every IO query,
    which is answered by IO outside the Inbox. States and Events are never custom. A custom Query's reply still
    converts in `Conversions`.
  - the `ServiceInfo` endpoint list (D-12, D-24);
  - the typed TypeScript client (D-14).
- Mistakes are compile errors. A Domain without `impl Conversions`, or a type without `impl Handlers`, gets an
  `#[diagnostic::on_unimplemented]` message that names the Service and lists every function with its endpoint.
  A missing function is E0046 and a wrong Message type is E0053, both naming the function, which is the
  endpoint's name. `services/tank/app/tests/compile_errors.rs` pins each error. The generator rejects two
  endpoints that generate the same function (names are unique in a Service, whatever their kind, and compared
  in snake case), the reserved names of D-12 (`Info` and `info` alike), a name that is not an identifier, a
  Message that is not in `blueos-idl`, and a field the format does not have.
- Adding an endpoint touches the `.msg`, the manifest and the Domain (its `logic/` crates, `Conversions`
  included); the app crate changes only in its generated file, or in `handlers.rs` for a custom endpoint.
- `api.lock` (D-06) records every endpoint key with its Message types; a removed or renamed key is an API break.
- No proc macro and no routing by naming convention. A naming convention could not name the endpoints that exist
  and would move an endpoint silently when a `.msg` is renamed; a derive macro would add a new framework concept
  with poor compile errors.

Rationale: the endpoint list and the wiring are the same artifact, so `ServiceInfo`, the inspector and the
frontend client are consistent by construction, and adding an endpoint touches the `.msg`, the manifest and the
Domain.

## D-27 Tasks, Projections and the reconcile pattern

Context: draft 1 gave in-process code no way to reach its own Domain except a public Zenoh endpoint (`Internal`)
with a hand-written JSON codec, and fed the data plane through a side effect inside a State selector. Background
tasks were detached `tokio::spawn` calls that nobody stopped.

Decision:

- A **Task** is declared in `build` with a name and a restart policy: `Never`, `OnFailure { backoff,
  max_attempts }` or `Always { backoff }`. Defaults: exponential backoff from 100 ms to 30 s with jitter, `Always`
  for long-running Tasks and `Never` for one-shot ones. The Kernel owns the handles (`tokio-util` `TaskTracker`)
  and the cancellation (`CancellationToken`), stops Tasks in the shutdown order of D-04, and merges their health
  into the `status` State (D-12). A test can assert that no Task is left running after shutdown.
- A Task receives a context with the Session, a `CommandSender`, the shutdown token, the service Context, and the
  Projections it follows.
- **`CommandSender<D>`** puts Commands into the service's own Inbox: `send` (waits when the Inbox is full),
  `try_send` (fails fast), and `send_awaiting_ack` (waits for the Domain's verdict, with the same ack and rejection
  as an external Command). The builder also hands it to adapters that are not Tasks.
- A **Projection** is a pure function from the Snapshot to a value. The Kernel recomputes it after every applied
  Command and delivers it as a `watch` receiver, deduplicated by equality. A published State is a Projection
  sent on the bus. Projections and State selectors never have side effects.
- **Reconcile pattern** for a long-lived resource (a file, a device, a stream): the Domain writes the state it
  wants into its Snapshot; the one Task that owns the resource follows it through a Projection, opens, changes and
  closes the resource itself, and reports what really happened as Observed facts. The Domain never holds a live
  handle, and no `Io` Effect touches that resource.
- **Observed facts carry the full current value** (for example `armed: true`), not a change. Producers send on
  change and re-send periodically, and the Domain handles them idempotently, so a dropped fact heals by itself.

Rationale: one owner per resource and one source of truth. In draft 1 the current MCAP writer had three
sources of truth, which produced the rotation bugs listed in D-20.

## D-28 Durable state

Context: D-03 promises flows that are resumable after a restart (wizard, calibration), and draft 1 persisted
nothing but settings.

Decision:

- A Domain opts in with [`DomainDurable`](../../core/libs/logic/domain/src/lib.rs): a dedicated `DurableState` type and accessors, separate from observed facts and re-derivable Snapshot fields. Its Jobs are persisted with it when the Service chains `durable_state_with_jobs`.
  Observed facts, live data and anything that can be re-derived are never persisted.
- The durable part derives `serde` directly in the logic crate (`default-features = false`, `alloc`), because it
  has no external compatibility to keep.
- It is written as versioned JSON to `<config>/<service>/state-<N>.json`, next to the settings document. That
  folder is mounted from the host, so it survives core updates.
- It is written after an applied Command that changed it, at most once per second, and once at shutdown, with the
  same atomic write as settings (D-11). The ack does not wait for it.
- An unreadable file, or one with another version, is logged, moved aside, and replaced by a fresh state. There
  are no migrations: it is recoverable state, not user configuration.
- On restore, timers and in-flight IO are gone. The Kernel marks every Running or Cancelling Job leaf
  [`Interrupted`](../../core/libs/logic/jobs/src/lib.rs) (`STATUS_INTERRUPTED` on the wire), then delivers one
  "restored" Tick from `DomainDurable::restored_tick`, so the Domain re-arms its timers and decides, per flow,
  whether to retry or fail. The Kernel never re-runs IO by itself, because IO is not idempotent.

## D-29 Panics and recovery

Decision:

- The unit of recovery is the Task (its restart policy, D-27), plus the Inbox loop as a special unit (the
  transaction restore, D-04).
- A panic hook sends every panic through `tracing` (D-13), so it reaches the `log` key and the MCAP file.
- A poisoned lock is never `.expect`ed; one panic must not cascade into every task that shares the lock.
- An Inbox loop recovery marks the service degraded. Three loop panics within one minute make the process exit
  non-zero.
- Exit codes: anything unrecoverable exits non-zero, so `core/run-service.sh` restarts the service. Exit 0 means
  an intentional stop, which the supervisor does not restart. The container supervisor stays the outer ring.

Rationale: a process restart costs about 5 s, the tail of the MCAP file and a library rescan, so recovering in
process is worth it; but a half-dead process that claims to be ready is worse than a restart, so whatever cannot
be recovered still exits.

## D-30 Code quality gates, tests and the Rust style guide

Decision:

- **Tests.** Draft 2 is written test-first (D-20). Layers: L1 Domain and Block unit tests (no runtime); L2 codecs
  and contracts (hostile CDR input, `api.lock`, generated-code comparison, `trybuild` compile-fail tests of a
  generated API); L3 the Kernel harness on the channel backend with a paused clock, an injected Clock,
  `CommandSender` and an Effect recorder, with no sleeps and no polling; L4 service wiring, through the real
  `build()` (D-25); L5 backend conformance, one shared test body run against the channel backend and a zenohd
  container; L6 device and system tests (D-15). File IO in the Kernel sits behind a port, so a paused clock never
  races a blocking thread.
- **Gates that fail the build:** `cargo fmt`; clippy with `[workspace.lints]` (including `unsafe_code = "forbid"`,
  `missing_docs` for public items, `unreachable_pub`, `allow_attributes` and `allow_attributes_without_reason`,
  `pub_use`, `min_ident_chars`, `shadow_unrelated`, `std_instead_of_core`, `alloc_instead_of_core`,
  `wildcard_imports`, `await_holding_lock`, the clone lints, and `arbitrary_source_item_ordering` configured in
  `core/clippy.toml` to order item kinds only, never fields or variants, because field order is CDR wire order);
  the in-repo `syn` checker (below); `cargo deny check bans licenses sources`; `cargo nextest run` with a per-test
  timeout, plus `cargo test --doc`; coverage with `cargo-llvm-cov` and per-layer floors kept in a committed ratchet
  file that may only rise; `cargo machete`; `typos`; the folder check and the `no_std` and wasm32 builds (D-02);
  `api.lock` and the generated-code comparison (D-05, D-06, D-26); `cargo auditable build`.
- **Advisories** (`cargo deny check advisories`) report on pull requests and fail on the scheduled run, so a new
  advisory in a transitive dependency does not turn every open pull request red.
- **Report only:** `cargo bloat`; a pinned nightly job (`cargo udeps`, branch coverage, sanitizers, lockbud);
  `rustqual` (exact version pinned, `rustqual.toml` written by hand at its documented defaults), thai-lint and
  ast-metrics.
- CI has seven Rust jobs: lint, test (with coverage), supply chain, report-only quality, report-only nightly,
  backend conformance (with a zenohd service container), and the cross build (D-16). Tools are installed prebuilt
  (`taiki-e/install-action`), never with `cargo install`. Coverage stays out of the pre-push hook because it
  changes `RUSTFLAGS` and invalidates the developer's target folder.
- **Style guide.** The full text lives in `docs/architecture/rust-style.md`. Its checklist is mirrored, between
  markers, into `AGENTS.md` and a committed `.cursor/rules/rust-blueos.mdc` (globs `core/**/*.rs`), and a drift
  check in `.hooks/lib/rust_checks.sh` fails when the copies disagree. It covers:
  - idiomatic Rust, KISS, test-first, documented public items (private items optional; the `AGENTS.md` rule
    against docstrings applies to Python and TypeScript only);
  - new dependencies with `default-features = false` and only the features needed;
  - no abbreviated names; names by meaning, not by type; structured logging always;
  - imports in five blank-line-separated groups (std, third-party, `blueos` crates, owned modules, relative paths),
    chained per crate;
  - declaration order readable top-down in one pass: constants and type aliases right after the imports, then
    types (a type before the types it uses), then `impl` blocks in the same order with trait `impl`s before the
    inherent one, then free functions (a caller before its callees), and `#[cfg(test)] mod tests` last; when uses
    form a cycle, follow the main direction of use;
  - re-exports: no renaming re-export, no re-export of another crate's domain types; a facade is allowed only
    behind `#![expect(clippy::pub_use, reason = "...")]`;
  - values cloned for an `async move` block or a `move` closure are bound in a block attached to the spawn;
  - newtypes and type-driven state (an enum for stored state, type-state for builders and resource handles,
    `Duration` for units, parse at the boundary);
  - borrowing before cloning: a handle clone (`Arc::clone(&session)`) is free, a data copy needs a reason;
  - every new architectural pattern gets a decision entry here before it is used.
- The `syn` checker enforces what clippy cannot: the five import groups and chaining, the caller-first order,
  structured logging, and the clone-before-spawn rule. It gates from day zero, because draft 2 starts clean.
  Formatting uses stable `rustfmt`, which preserves blank-line-separated import groups.

## D-31 Ergonomics goals

Decision: the bar is that a junior developer can read a service, understand what it does, and add an endpoint in
their first month. These measures are **goals, not gates**: CI reports them and never fails on them, because what
reality needs cannot be known in advance, and quality wins over line count.

| Measure | Draft 1 example | Prototype `disk_usage` | Goal |
|---|---|---|---|
| Concepts to add one Command end to end | 24 | 8 | 8 or fewer |
| Wiring lines per endpoint | 27 | 13 | 12 or fewer |
| Smallest complete service (hand-written lines, `Cargo.toml` and `endpoints.toml` included, any number of crates) | 1187 | 197 | 250 or fewer |
| Framework code a reader must read first (repository-local lines) | about 2550 | about 712 | 600 or fewer |
| Time to a first endpoint | 1 to 2 days | 1 to 2 hours | 2 hours or less |

The goal for files touched per Command is set once the endpoint manifest (D-26) exists.

## D-32 Reused projects and what stays ours

Decision: our code is open source, and we reuse mature projects rather than write mechanisms ourselves. No Rust
framework covers "an event-driven service with a versioned IDL over a pub/sub bus" (Zenoh-Flow is dead; dora-rs
and Copper replace the process model; Eclipse uProtocol replaces the API contract).

Reused:

| Need | Project |
|---|---|
| Task lifetimes, cancellation, timers | `tokio-util` (`TaskTracker`, `CancellationToken`, `DelayQueue`); already compiled in through `zenoh` |
| Retry with backoff and jitter | `backon` |
| Key expression matching | `zenoh-keyexpr` |
| `.msg` parsing | `roslibrust_codegen` |
| Calendar math in logic crates | `chrono` with `default-features = false, features = ["alloc"]`; app crates add `std` and `clock` |
| Directory walks | `walkdir` (no symlink following, depth limit) |
| Generated code formatting | `prettyplease`, in the generator only |

Ours, with the reason:

- Kernel and Inbox loop: actor frameworks (`ractor`, `kameo`) force `async` handlers, which breaks sans-IO (D-03).
- CQRS traits: every candidate is event sourcing with a store, `async` and `std`.
- Jobs: every jobs crate is a durable worker queue, not a flow shown in the UI.
- Settings: D-11 needs byte compatibility with Python `appdirs`, which `config` and `figment` do not give.
- Dependency injection: a service Context passed by reference (D-04); no container crate.
- CDR codec: `cdr-encoding` matches our test vectors but does not build `no_std`, and D-06 needs trailing-field
  defaults.
- Rust and TypeScript emission: `roslibrust_codegen` output is `std`-only and tied to its runtime; `ts-rs`,
  `typeshare` and `specta` read Rust types, not `.msg`.
- Python codec: D-17.
- State machines: plain enums; `statig` compiles `no_std` but took more code for the same tests.

## Open items

- `mavlink-codec` is a git dependency. Publish or vendor it only when a crate that depends on it has to be
  published.
- Document the `IpcMode` note (D-09) in the external extension docs and the extension template.
- Zenoh inspector: `get_type_description` queries for types outside the catalog, a `blueos/v1/schemas/**`
  queryable for extension schemas, ROS 2 service requests, and the table, plot and vehicle-frame views.
- The frontend `lint` script ignores `.ts` files; fixing the existing backlog is its own pull request.
