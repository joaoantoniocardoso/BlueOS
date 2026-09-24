# BlueOS Rust Event-Driven Architecture: Decision Record

Status: accepted (WIP branch `pocs/blueos-service`), last updated 2026-09-24.

This is the master decision record for introducing Rust, event-driven services, and a versioned IDL API
into BlueOS. Every contributor (human or AI agent) working on `core/libs/`, `core/services/<rust service>/`,
`core/interfaces/`, or the frontend `blueos-api` library must read it first. When code contradicts a
decision, either fix the code or update the decision here in the same change.

## Source reference

The decisions below were made in a design conversation. For exact wording, retrieve it from:

- Transcript id: `11effae0-a983-42fb-ad8c-d8f350be2bf0`
- Path (author machine):
  `~/.cursor/projects/home-joaoantoniocardoso-BlueRobotics-worktrees-BlueOS-docker-quartz-geyser-BlueOS-docker/agent-transcripts/11effae0-a983-42fb-ad8c-d8f350be2bf0/11effae0-a983-42fb-ad8c-d8f350be2bf0.jsonl`

Related repositories (author machine, `~/BlueRobotics/`):

| Repository | Role |
|---|---|
| `microservices_core_prototype` | Earlier full Rust port of ardupilot_manager. Source of real-world code to port: `crates/blueos-core/commonwealth/src/{settings,zenoh_node,tracing_init}` and the `xtask` cross-build. Its architecture is not the target. |
| `blueos-recorder` | Current Recorder (Rust, zenoh + mcap). To be rewritten in this repository and retired. |
| `zBlueberry` | Peer experiment: `.msg` files to serde structs via `build.rs` + `ros2_message`. Seed for the IDL codegen. Its name must not be used. |
| `radcam-manager` | Reference CI: matrix `cross` musl build, binaries copied into the image. |

## Index

- D-01 Goals and migration strategy
- D-02 Crate layout and dependency boundaries
- D-03 Sans-IO logic: why `logic/` never needs async
- D-04 Runtime: tokio in the kernel only
- D-05 IDL: ROS2 `.msg` + CDR, published crates
- D-06 Schema evolution policy and API-break gates
- D-07 Key space
- D-08 Public API vs internal IPC; what the frontend talks
- D-09 Zero-copy and containers
- D-10 Comms contract
- D-11 Settings: Python compatibility, runtime vs restart
- D-12 Standard per-service keys and the future service manager
- D-13 Logging
- D-14 Frontend library and the Vue 3 transition
- D-15 Recorder migration
- D-16 CI and deploy
- D-17 Python side of the migration
- D-18 REST gateways for migrated services
- D-19 Findings on the first POC and their resolution
- D-20 Process: examples first, PR split later

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

1. Recorder is rewritten here as a BlueOS service, with a reworked frontend (in progress).
2. Setup wizard + calibration move from the frontend into a Rust backend service, redesigned to be
   product-oriented: guide users from zero to ready-to-fly, not only vehicle/BlueOS setup. Not in this
   branch.
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
| `logic/` | Pure code, `#![no_std]` + `alloc`, no IO | `libs/logic/`, `blueos-idl` |
| `adapters/` | Code touching the outside world | `libs/adapters/`, its own service's `adapters/` |
| `app/` (service) | Wires logic to adapters, library crate | anything in `libs/`, its own service |
| `app/blueos` (workspace) | Multicall binary | `libs/`, service `app/` crates via cargo features |

- Single Cargo workspace at `core/Cargo.toml` with a single `core/Cargo.lock`. No per-service lock files.
- Layout: `core/libs/{logic,adapters,app}/...`, `core/services/<name>/{logic/<block>,adapters/<thing>,app}`,
  `core/interfaces/` (`.msg` sources), `core/app/blueos` (multicall binary).
- One multicall binary `blueos`; each service is invoked as `blueos <service>` or via a symlink named after
  the service (argv[0]). Each service still runs in its own process.
- No crate depends on another service's crates. Services talk only over comms.
- Enforcement: `.hooks/lib/rust_checks.sh` (folder rules via `cargo metadata` + `jq`, `cargo deny` bans so
  `zenoh` stays behind the comms zenoh driver, and a build of every `logic/` crate for
  `thumbv7em-none-eabihf` so any IO dependency fails to compile).

Rationale: the folder decides the dependency rules, which keeps hexagonal boundaries mechanical rather than
reviewed by hand. The multicall binary keeps image size and the incremental update layer small.

## D-03 Sans-IO logic: why `logic/` never needs async

Context: reviewers will ask whether real services (calibration, wizard, recorder) can be written without
async in the domain.

Decision: yes. `logic/` is written in the **sans-IO** style (same approach as `quinn-proto`, `rustls`, and
the core of `h2`). Async is only about *waiting*; logic never waits, it represents waiting as data and
returns effects. The kernel (D-04) performs the waiting.

| Need | How it is modeled without async |
|---|---|
| Wait for an IO result (RPC, MAVLink command, file) | Handler returns `Effect::Io(request)`; the kernel runs it and feeds the result back as a `Command` (e.g. `JobProgress`). |
| Timeouts, delays, retries | Handler returns `Effect::Schedule { after, command }`; the kernel arms a timer and delivers `command` later. Cancel by job id. |
| Current time, random ids | Passed in with the command (`now`, ids allocated by the caller or deterministic counters). Never read inside logic. |
| Multi-step flows (wizard, calibration) | Explicit state machines and `blueos_jobs` graphs (`Sequence`, `Parallel`, cancellation) instead of `await` chains. |
| Heavy CPU (e.g. compass ellipsoid fit) | A pure function in `logic/`, executed by the kernel as a job on a blocking thread so the inbox stays responsive. |
| High-rate data (MCAP writing, video, sonar) | **Control plane vs data plane**: logic owns the policy (which topics to record, when), adapters apply it on the hot path. Payloads never go through the inbox, which also preserves zero-copy (D-09). |

Cost: a flow that would be five lines of `await` becomes explicit states.

Benefit (why we accept the cost): the state is inspectable, publishable to the UI, cancellable,
resumable after a restart, and unit-testable without a runtime, mocks, or sleeps. The product flows we
want (wizard, calibration) need exactly these properties.

## D-04 Runtime: tokio in the kernel only

Decision:

- tokio is accepted. Isolating from it buys nothing because `zenoh` already runs its own tokio runtime.
- Only `libs/app/service` (the kernel) and adapters own async code.
- The kernel runs **one inbox (`mpsc`) per service**. Adapters only translate incoming samples/queries into
  `Command`s and send them to the inbox. IO effects run as spawned tasks that post their result back as a
  `Command`. Timers from `Effect::Schedule` are kernel-owned.
- No `unsafe` to share state with callbacks, no blocking RPC inside dispatch, errors are typed (no magic
  byte payloads such as `b"FAILED"`).

## D-05 IDL: ROS2 `.msg` + CDR, published crates

Decision:

- Message definitions are **ROS2 `.msg` files** under `core/interfaces/`, versioned per package.
- Serialization is **CDR** (ROS2 default). Zenoh `Encoding` is `application/cdr;<schema_name>`, which is
  already the convention the current Recorder uses to write MCAP channels with `ros2msg` schemas.
- Codegen is seeded from zBlueberry's `build.rs` (`ros2_message` parsing to serde structs) and extended with:
  a CDR codec, the schema text embedded per type (`SCHEMA_NAME`, `SCHEMA`), defaults for missing trailing
  fields (D-06), and TypeScript `.d.ts` output.
- Two crates are **published on crates.io** so extensions and ROS2 nodes can use them:
  - `blueos-idl`: generated types + codec + embedded schemas, `no_std` + `alloc` so `logic/` may use it.
  - `blueos-api`: key conventions (D-07), encoding strings, optional zenoh helpers (D-10).
- Names `blueos-idl`, `blueos-api` (and `blueos`, to reserve) were free on crates.io on 2026-09-24.
- No zBlueberry branding anywhere.
- Published crates never depend on unpublished workspace crates.

Rationale: `.msg` + CDR gives native ROS2 interop and Foxglove/MCAP support out of the box; embedding the
schema removes the need for Recorder to find `.msg` files on disk.

## D-06 Schema evolution policy and API-break gates

Context: plain CDR has no schema evolution (fields are positional). DDS XTypes (appendable/mutable, XCDR2)
can evolve but must be declared in `.idl` with annotations, `.msg` cannot express it, support across ROS2
middleware is uneven, and `rmw_zenoh` uses plain CDR. Type hashes (ROS2 Iron/Jazzy, RIHS01) detect a
mismatch but do not resolve it. Protobuf/FlatBuffers are MCAP-friendly but not ROS2 topic formats.

Decision: accept the limitation, with a strict policy:

- A published `v1` message is **append-only**: new fields may only be added at the end.
  - New writer, old reader: works (decoders stop after the fields they know).
  - Old writer, new reader: our generated decoders (Rust, TypeScript, Python) fill missing trailing fields
    with defaults. Native ROS2 nodes will fail; this is documented.
- Any other change (remove, reorder, retype, rename a field; change semantics) requires a **new message type
  or a new major version** of the package/keyspace.
- Each type carries a hash; it is put in the encoding or the attachment so mismatches fail loudly.

Test gates (mandatory, run in `.hooks/pre-push` and CI):

- A checked-in lock file of per-message structural hashes (`core/interfaces/api.lock`). A test fails if a
  message changed in any way other than append-only, unless its major version was bumped. Updating the lock
  is an explicit, reviewed step.
- `cargo semver-checks` on the published crates (`blueos-idl`, `blueos-api`).

## D-07 Key space

Decision:

- Public, versioned API keys: `blueos/v1/<service>/...`.
- Internal IPC keys (when introduced): `blueos/@ipc/...`, unversioned.
- The current zenoh topic layout is WIP; **no legacy is preserved**. Python producers move to the IDL too
  (D-17).
- Exact sub-structure (commands, state, events, jobs, settings, info) is defined by D-10 and D-12 and lives in
  `blueos-api` as constants/helpers, never as ad-hoc strings in services.

## D-08 Public API vs internal IPC; what the frontend talks

Decision: **the frontend talks the versioned IDL API**, not the internal IPC.

Rationale:

- Zero-copy (the main reason for IPC) does not reach the browser: `zenoh-ts` runs over a websocket.
- Deploy coherence does not hold in the browser: a tab left open across an update, Cockpit, or a mobile
  client are out of sync with the backend.
- Dogfooding: if the BlueOS frontend is built on the public API, extensions and third parties can be too.
- Recordings stay decodable: Recorder records the backbone; schema-less IPC payloads produce MCAP files that
  other versions cannot read.

IPC:

- Purpose: let BlueOS internals change without breaking the external API; zero-copy where it matters.
- Deferred until a concrete internal-only consumer needs it. Each service exposes its public API through one
  small API adapter mapping internal events to IDL messages.
- Internal debug views may read IPC keys; product UI may not.

## D-09 Zero-copy and containers

Context: the concrete zero-copy pipeline today is MCM (mavlink-camera-manager) to Recorder. Extensions
(e.g. ping sonar) should benefit too. A custom `zenohd` build with shared memory (SHM) enabled replaces the
current one. Zenoh uses SHM automatically when payloads exceed ~3 KB, both peers are on the same host, and
both support it (MCM, Recorder, BlueOS zenoh do).

Decision:

- Comms must never copy payloads on the way through: no framing prefix, payload is a cheap-clone bytes type
  (D-10). Recorder keeps the refcounted `ZBytes` path up to the MCAP write.
- **Extensions need `IpcMode: host` in their Docker permissions to share SHM. This must be documented** in
  the extension developer docs (Kraken permissions, extension template) and in the `blueos-api` README.
  Document the narrower alternative too: bind-mounting `/dev/shm` (host IPC mode shares the whole host IPC
  namespace).
- Core already shares SHM because it binds all of `/dev/` (`bootstrap/startup.json.default`).
- Without either setting, extensions silently fall back to network transport; the docs must say so.

## D-10 Comms contract

Decision (implemented in `libs/adapters/comms*`, constants in `blueos-api`):

- Connect as a zenoh **client** to the local `zenohd` (`tcp/127.0.0.1:7447`), like Python's
  `commonwealth/utils/zenoh_helper.py`. No peer-mode listen/connect fallback.
- No payload framing. Correlation and metadata go in Zenoh **attachments**; Zenoh queries already correlate
  replies.
- `Sample` exposes key, payload (cheap clone), encoding, timestamp, attachment.
- Key expressions with wildcards (`**`, `*`) are supported by the dispatcher.
- **State** keys = queryable (current value, for late joiners) + update stream. Clients query first, then
  subscribe.
- **Commands** are Zenoh queries; the reply is an IDL `CommandAck { accepted, job_id, reason }`. Progress
  and results come as events/state, not in the reply.
- **Queries** (CQRS read side) are Zenoh queries answered from the snapshot.
- Liveliness token per service (D-12).
- In-process channel driver kept for tests.

## D-11 Settings: Python compatibility, runtime vs restart

Context: Python services use `commonwealth.settings` (legacy `settings.py`/`manager.py` and the Pydantic
`managers/pydantic_manager.py` + `bases/pydantic_base.py`): files named `settings-<VERSION>.json` in
`appdirs.user_config_dir(<project>)`, a `VERSION` field, migrations from older versions, refusal of versions
from the future, highest-version-first loading, temp-file cleanup.

Decision:

- Rust settings must be **100% compatible** so a Rust and a Python implementation of the same service can be
  swapped with no breakage (same folder, file names, JSON shape, `VERSION`, migration semantics).
- Port the prototype's `commonwealth/src/settings/` (schema, manager, error, tests) as
  `libs/adapters/settings`, replacing the POC's read-only JSON5 `blueos_configs`.
- **Golden-file tests**: fixtures written by the Python library are loaded and round-tripped by Rust.
- Scope: each service only reads/writes its own settings.

Runtime vs restart-required settings:

- The settings file stays one struct per service (file format unchanged).
- The kernel loads it at startup and hands it to the domain.
- Changes arrive as `Command::UpdateSettings`. The handler applies what it can live, returns a persist
  effect, and emits `Event::RestartRequired { fields }` for anything that needs a restart.
- The settings IDL message marks which fields require a restart so the UI can warn before applying.
- Command-line arguments are a third category, owned by the future service manager, not stored in the
  service's settings file.

## D-12 Standard per-service keys and the future service manager

Decision: the kernel gives every service, for free:

- Liveliness token `blueos/v1/services/<name>` (alive/dead).
- `info` queryable (name, version, build, capabilities).
- `status` state.
- `settings` state + `UpdateSettings` command (D-11).
- `jobs` state (job graph status from `blueos_jobs`).
- `log` stream (D-13).

The future system-wide service manager (settings, command-line arguments, start/stop/restart/enable/disable)
is then just a client of these keys plus process/container control. Nothing service-specific is needed.

## D-13 Logging

Decision: `tracing` everywhere; the logging adapter publishes each record as a foxglove `Log` message
(CDR, via `blueos-idl`) on the service's `log` key, porting the prototype's `tracing_init/zenoh_layer.rs`.
Recorder records these; the frontend console and extension-log views read them.

## D-14 Frontend library and the Vue 3 transition

Decision:

- `core/frontend/src/libs/blueos-api/` is **plain TypeScript with no Vue imports**. It exposes: query state
  then subscribe; send a command and get `accepted/rejected + job_id`; watch jobs; decode/encode IDL types.
- A thin Vue 2 wrapper (mixin or store module) sits on top.
- BlueOS will move to Vue 3; that migration only replaces the wrapper with a composable. The core library is
  unchanged.
- Decoding uses `@foxglove/rosmsg2-serialization` with the embedded schema text and generated `.d.ts` types.
- WASM builds of the Rust codec are **not** used for now (extra toolchain in Vue/Vite for no gain). Revisit
  if shared validation logic (e.g. wizard) is needed.

## D-15 Recorder migration

Decision:

- Rewrite `blueos-recorder` as `core/services/recorder/{logic,adapters/mcap,adapters/mavlink,app}`; retire the
  external repository afterwards.
- Preserve the contract MCM relies on (`--recorder=external`): `video/...` topics, MAVLink camera capture
  commands and status replies, and "record MAVLink only while armed".
- Recording policy (armed state, per-stream video recording state, topic filters) is `logic/`; the zenoh
  tap and MCAP writer are adapters (control plane vs data plane, D-03).
- Keep the zero-copy path (D-09); MCAP schemas come from the embedded IDL schemas (D-05) with the existing
  JSON fallback.
- Frontend reworked on `blueos-api` (D-14).
- radcam-manager stays in its own repository.

## D-16 CI and deploy

Decision (radcam-manager model):

- A matrix CI job cross-builds with `cross` for `aarch64-unknown-linux-musl`,
  `armv7-unknown-linux-musleabihf`, `x86_64-unknown-linux-musl`, in parallel with the Python pipeline.
- The Docker image copies the single `blueos` binary (selected by `TARGETARCH`) plus service symlinks in the
  **last layer**, for cache reuse and small incremental updates.
- Rust checks (fmt, clippy, tests, deny, folder rules, no_std build, API-break gates) run in `.hooks/pre-push`
  and CI.

## D-17 Python side of the migration

Decision:

- Python producers/consumers (commonwealth zenoh helper and logs, kraken zenoh handlers) switch to
  `blueos/v1/` keys and CDR IDL payloads.
- Python uses **runtime `.msg` parsing** plus CDR (e.g. `rosbags`; license and fit to verify) instead of a
  third codegen target, since Python is being phased out.
- Frontend consumers of those keys (Zenoh inspector, console logger, extension logs) are updated together.

## D-18 REST gateways for migrated services

Decision: when a Python service is migrated, a gateway translates the event-driven API to that service's
current REST API so existing clients keep working. Gateways are adapters with no domain logic; they are
dropped once no client needs them. Not built in this branch.

## D-19 Findings on the first POC and their resolution

| Finding | Resolution |
|---|---|
| Kernel shares `App`/`Session` with callbacks through raw pointers and hand-written `unsafe impl Send`. | D-04 inbox kernel. |
| Synchronous RPC inside dispatch: long commands block everything including cancel; A->B->A deadlocks. | D-04, IO as spawned tasks. |
| RPC errors flattened to `b"FAILED"`. | Typed errors (D-04). |
| 4-byte correlation prefix on every payload, ad-hoc text payloads. | D-05, D-10: pure CDR, attachments. |
| Peer-mode listen/connect trick on `:7447` while `zenohd` already listens there. | D-10 client mode. |
| `Kind::State` equals `Stream`; late joiners see nothing. | D-10 state queryable. |
| Queries only in-process; CLI `snapshot` queries a fresh `App` (always defaults). | D-10 queries over comms. |
| Exact-string key dispatch, no wildcards. | D-10. |
| Stringly-typed `JobSpec`; handlers reparse payloads. | Generic `Jobs<Spec>`. |
| No job timeouts. | `Effect::Schedule` (D-03). |
| Read-only JSON5 configs, not compatible with Python settings. | D-11. |
| No liveliness/info/log over zenoh. | D-12, D-13. |
| Two workspaces, two lock files; no Rust in the image. | D-02, D-16. |
| POC calibration/autopilot stubs teach wrong patterns (stub autopilot, text payloads). | Replaced by the teaching example (D-20). |

## D-20 Process: examples first, PR split later

Decision:

- First reach the target ("perfect") code on this branch: reworked libs, `blueos-idl`/`blueos-api`, adapters,
  kernel, a **teaching example service**, and Recorder migrated (backend and frontend).
- The teaching example lives in `core/services/example/` and is the base for developers and their AI agents:
  one concept per file, heavily documented (command, query, state, event, job, schedule, settings runtime vs
  restart, frontend view), linked from `AGENTS.md`.
- Only then decide the stacked-PR split. The setup wizard is not part of this branch.
- The POC under `POCs/blueos-service/` is removed once the example replaces it.
