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
- Implementation, reviews, D-21 to D-23: transcript `52237482-724f-4979-85b6-d6325dab8126`, same folder.

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
- D-21 Breaking changes for users and extension developers
- D-22 Branch review outcomes
- D-23 Recording library: retire `recorder_extractor`, rebuild the Records frontend

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
- Naming: every Rust package is hyphenated, `blueos-<name>` for libs (`blueos-service`, `blueos-comms-zenoh`)
  and `blueos-<service>[-<block>]` for services (`blueos-recorder`, `blueos-recorder-policy`). Every
  workspace crate is listed in `[workspace.dependencies]` and members depend on it with `workspace = true`,
  never a relative `path`. Python packages keep their names.
- Test-only backends (the comms `channel` feature) are enabled in `[dev-dependencies]` only, so they never
  reach the shipped binary through feature unification.
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
| Multi-step flows (wizard, calibration) | Explicit state machines and `blueos-jobs` graphs (`Sequence`, `Parallel`, cancellation) instead of `await` chains. |
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
- The Recorder must record every ROS 2 and Foxglove message, not only BlueOS ones. The upstream definitions
  (ROS 2 Jazzy interface packages and the Foxglove SDK's `schemas/ros2`) are vendored unmodified in
  `core/libs/idl/catalog/` and exposed as schema text only, by the `blueos-idl` `catalog` feature. They get no
  types and no `api.lock` entries, since they are not BlueOS API. `foxglove.Name`, the name the Foxglove SDK
  and mavlink-camera-manager publish, resolves to `foxglove_msgs/msg/Name`.

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
- `info` queryable at `blueos/v1/<name>/query/info` (name, version, build, capabilities).
- `status` state.
- `settings` state + `UpdateSettings` command (D-11).
- `jobs` state (job graph status from `blueos-jobs`).
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
- Runs as `recorder --recorder-path /usr/blueos/userdata/recorder` (symlink to the `blueos` multicall binary),
  same arguments and slot as the retired binary in `core/start-blueos-core`.

## D-16 CI and deploy

Decision (radcam-manager model):

- A matrix CI job cross-builds with `cross` for `aarch64-unknown-linux-musl`,
  `armv7-unknown-linux-musleabihf`, `x86_64-unknown-linux-musl`, in parallel with the Python pipeline.
- The Docker image installs the single `blueos` binary (selected by `TARGETARCH`) plus service symlinks in the
  **last layer**, for cache reuse and small incremental updates. The per-target binaries are bind-mounted
  (`RUN --mount=type=bind,source=target/build`), not copied, so the other architectures' binaries never land
  in a layer; `core/.dockerignore` re-includes only `target/build/*/*/release/blueos`.
- Shipped features: `recorder` only. The teaching example is never shipped.
- Release binaries are about 14 MB per target (musl, stripped, thin LTO).
- Local builds: `cd core && ./build_cross.sh` (see `core/services/recorder/README.md`).
- Rust checks (fmt, clippy, tests, deny, folder rules, no_std build, API-break gates) run in `.hooks/pre-push`
  and CI.

## D-17 Python side of the migration

Decision:

- Python producers/consumers (commonwealth zenoh helper and logs, kraken zenoh handlers) switch to
  `blueos/v1/` keys and CDR IDL payloads.
- Python uses **runtime `.msg` parsing** plus CDR instead of a third codegen target, since Python is being
  phased out. It is a small pure-Python parser and codec in `commonwealth/utils/blueos_idl.py` (stdlib
  `struct` only), tested against bytes produced by the Rust codec for every message.
- `rosbags` was evaluated and rejected: it pulls `numpy`, `apsw`, `lz4`, `zstandard` and `ruamel-yaml` into
  the image (none present before), `numpy`/`apsw` have no armv7 wheels, and it needed workarounds to get the
  D-06 trailing-field defaults.
- The `.msg` files reach the image through the existing `COPY libs` (`/home/pi/libs/idl/interfaces`);
  `BLUEOS_IDL_INTERFACES` overrides the path.
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

Native ROS2 subscribers get plain CDR without XTypes: they cannot read messages from an older writer that
lacks trailing fields (D-06).
Extensions publishing to the recorder must follow the `IpcMode` note in D-09.

## D-22 Branch review outcomes

Two reviews ran over the branch: a Rust review against the team rules, and a BlueOS blast-radius review that
walked every change up to its FastAPI, nginx, frontend and `start-blueos-core` entry points.

Fixed:

| Finding | Resolution |
|---|---|
| No SIGTERM/SIGINT handling: stopping the container left the MCAP file unfinished. | Kernel `ServiceBuilder::on_shutdown(command)`: on signal (or `ShutdownHandle::trigger()`) it dispatches the command, drains in-flight IO and pending settings writes for up to 5 s, then returns. Recorder dispatches `StopRecording`; `McapSession` also finishes on `Drop`. |
| `Effect::Persist` awaited disk inside the inbox (D-04). | One FIFO persist worker; the inbox never waits on disk; last write wins. |
| Zenoh query replies called `.wait()` on runtime threads. | Awaited. |
| In-process test broker was a process-wide singleton; kernel tests needed a global mutex. | One broker per `ChannelBackend::pair()`; mutex removed. That exposed a real race: two services attaching the Zenoh log publisher at once, the loser failed to start. The publisher is now one atomic `OnceLock` (one service per process; later ones reuse it). |
| Zenoh network tests silently passed without a router. | `#[ignore]`, and they fail loudly when run. |
| Recorder shipped the `channel` test backend, used `anyhow`, `.expect` on startup, magic `status: 2`, swallowed IO errors, blocking MCAP open/finish and a blocking tap send on tokio workers. | Zenoh-only prod features, `thiserror`, exit code on failure, `STATUS_READY`, `IoFailed` + logs, `spawn_blocking`, `try_send` with drop counting, supervised background loops, subscribe retry. |
| Duplicated MAVLink topic constants. | Single copy in `logic/policy`. |
| `blueos-api` had no tests. | Exact-string tests for every helper. |
| Python services died at startup without the `.msg` files. | Log once and skip liveliness/info. |
| `cargo semver-checks` never ran in CI. | Installed in the pre-push job. |
| Mixed crate naming and relative `path` dependencies; `blueos-service` and `blueos-logging` enabled the comms `channel` test backend in normal dependencies, so it still shipped. | Hyphenated names and `workspace = true` everywhere (D-02); `channel` only in dev-dependencies. |
| D-12 did not name the `info` key. | `blueos/v1/<name>/query/info`. |

Kept as is, with the reason:

- `Clone + Send` on the `Domain` associated types: the kernel clones `App` into IO tasks and stores commands
  in timers. Removing them needs `Arc<App>` in the kernel; not worth it now.
- The frontend `lint` script still ignores `.ts`: including it reports 447 errors and 23 warnings, almost all in
  existing code (354 auto-fixable) or unresolved imports from the uninitialized `MAVLink2Rest` submodule. The
  new TypeScript under `libs/blueos-api/`, `components/recorder/` and `tests/` lints clean. Fix the backlog in
  its own PR.

Open:

- `mavlink-codec` is a git dependency. It does not block merging; publish or vendor it only when a crate that
  depends on it has to be published.
- Comms fan-out clones key/encoding strings per subscriber (`Arc<str>` would avoid it); the Recorder tap still
  copies MAVLink payloads before the policy gate because `Payload` has no borrowed slice accessor.
- Document the `IpcMode` note (D-09) in the external extension docs and the extension template.
- Not proven on a vehicle: MCM `--recorder=external`, armed gating, video over SHM, MAVLink capture replies,
  `docker stop` mid-recording, and an `linux/arm/v7` image built from CI artifacts. Run these on a DUT before
  merging the Recorder PR.

Proposed stacked-PR split (each rebuilt as clean history, without the POC add/remove churn):

1. Rust foundation: Cargo workspace, Rust checks in `.hooks/pre-push` and CI, `logic/{jobs,cqrs}`,
   `blueos-idl` + `api.lock` gate, `blueos-api`, comms, logging, settings, kernel, this decision record.
2. Teaching example: `core/services/example`, the `blueos` multicall binary, the frontend `blueos-api` library,
   the example developer view, the `AGENTS.md` walkthrough.
3. Python and frontend on the versioned IDL keys (the D-21 breaking changes). Depends only on PR 1.
4. Recorder backend and delivery: recorder crates, cross-build CI job, Dockerfile last layer,
   `start-blueos-core`, removal of the external recorder bootstrap.
5. Recorder frontend.

## D-23 Recording library: retire `recorder_extractor`, rebuild the Records frontend

Context: on `~/BlueRobotics/BlueOS-docker` branch `video_player_tidy2`, commit `614d2a67a` ("WIP: backend")
turns the Python `recorder_extractor` into an MCAP catalog (states `recording/ready/needs_repair/repairing`,
a paged chunk-index walk, repair through `mcap recover` with progress read from `/proc`, cancel, a recovered
download of a recording still being written, delete), and `43d273355` ("WIP: frontend") replaces the
Records page with an in-browser MCAP player, CSV export and thumbnails that read chunk bodies with HTTP
ranges from nginx (`libs/mcap/*`, `components/records/*`, `RecordsView.vue`), and drops the ffmpeg/Broadway
decoders from the Zenoh inspector. Both talk REST (`/recorder-extractor/v1.0/...`) and poll.

Decision:

- The Recorder service owns its recordings. The library is a new sans-IO block,
  `core/services/recorder/logic/library` (`blueos-recorder-library`), composed into the recorder domain.
  `recorder_extractor` is deleted with its uv workspace entries, nginx location and `start-blueos-core` line.
- **Bytes stay on nginx.** `/userdata/recorder/<path>` serves files with HTTP ranges (CORS exposes
  `Accept-Ranges` and `Content-Range`); browsers need ranges and downloads, which Zenoh does not give them.
  This is the one exception to D-08: the IDL API carries the catalog and control, never recording bytes.
- **Event-driven, no polling.** The library is a state; outcomes are events; the frontend watches both.
- Native repair: the `mcap` crate rewrites a recording in-process (no `mcap` CLI subprocess, no `/proc` offset
  hack; progress is the exact read offset). Output goes to a `.recover` temp file renamed over the original;
  cancel removes the temp and leaves the original untouched; leftovers are discarded at startup.
- A recording still being written is downloaded through `SnapshotRecording`: the same rewrite writes an
  indexed copy `<stem>.snapshot-<UTC>Z.mcap` next to it (the naming the Python service already parsed), and the
  browser downloads it from nginx once the `operation` event names it.
- The recorder knows which file it is writing, so `STATE_RECORDING` needs no `lsof`/open-file scan. Other
  files are rescanned on a timer and after each operation; the state is republished only when it changes.
  ponytail: timer rescan (5 s); switch to inotify if external writers or large folders make it costly.
- Kernel additions needed by this and reusable by every service:
  - `Decision::reject(reason)`: a domain refuses a command synchronously; the `CommandAck` carries
    `accepted = false` and the reason (Python's 409s). A rejecting decision has no events or effects.
  - `ServiceBuilder::io_query(name, handler)`: an async query answered by an adapter outside the inbox, for
    reads that need disk but no domain state (the index walk). One request at a time per query name.
- Frontend layering mirrors the backend (D-02, D-14):
  - `src/libs/mcap/logic/`: pure TypeScript, no DOM, no network (record parsing, keyframe index, frames,
    codec parameters, CSV, muxing). Unit-tested with vitest in Node.
  - `src/libs/mcap/adapters/`: IO behind small interfaces (`ByteSource` over `fetch` ranges, WebCodecs/MSE
    players, canvas thumbnails, thumbnail cache). The index source is an interface; the recorder client
    implements it with the `index` query.
  - `src/libs/recorder/`: framework-agnostic recorder client on `blueos-api` (library state, operation events,
    commands, index source, `/userdata/recorder` URLs). No Vue imports.
  - Vue 2 components (`components/records/*`, `RecordsView.vue`) only bind these to templates, so the Vue 3
    move replaces them without touching the libraries. `store/records.ts` and REST types are removed.
- `.mcap-harness/` probes become vitest tests where they check behavior; the rest is dropped.

API (keys under `blueos/v1/recorder/`, messages in `blueos_recorder_msgs`):

| Kind | Name | Message |
|---|---|---|
| state | `library` | `RecordingLibrary` (`RecordingFile[]`, newest first) |
| command | `RepairRecording` / `CancelRepair` / `DeleteRecording` / `SnapshotRecording` | `...Command { path }` |
| event | `operation` | `RecordingOperation` (repair, snapshot, delete; succeeded, cancelled, error, output path) |
| io query | `index` | `RecordingIndexRequest` -> `RecordingIndex` (paged chunk index + raw metadata records) |

Rejections (from the Python rules): repair when already repairing, already indexed, being written or written
less than 10 s ago; cancel when not repairing; delete while being written or repaired; snapshot of a
missing file; any path that is absolute, contains `..`, is not `.mcap`, or is not in the library.

Orchestration (Composer 2.5 agents, one git worktree each, merged by cherry-pick):

1. Contract (done by the orchestrator): the messages above, the codegen fix for primitive arrays, this entry.
2. In parallel: kernel additions; recorder adapters (index walk, footer, native rewrite, storage scan);
   frontend `libs/mcap` port and Zenoh inspector player.
3. In parallel: library logic + recorder wiring + retirement of the Python service; recorder client and
   Records frontend.
4. Blast-radius and Rust reviews, a fix pass, and the outcome recorded here.

Outcome of step 4:

- Fixed from the reviews: a panicking repair/snapshot/delete task now ends the operation as failed instead of
  leaving it in flight; a second delete of the same path is rejected; nested `.recover` leftovers are
  discarded; a missed `operation` event no longer hangs a snapshot download (it also completes from the
  `library` state and times out); Records shows an explicit empty state when the Recorder is not running.
- Kept: the index walk reports the size seen at the start of the walk (pages continue from the last offset
  and the frontend asks again when `library` reports a new size); the first repair progress arrives after
  1 s; a repair running at shutdown is not cancelled (the 5 s drain applies, leftovers are discarded next
  start); `/recorder-extractor/v1.0/*` is gone for extensions (D-21 break).
- Found on a Raspberry Pi 4 (the `video_player_tidy2` core image with the Recorder and nginx config swapped in,
  the frontend served by the Vite dev server), none of which the
  channel-backend tests exercised, all fixed:
  - Commands that a service sends to its own keys at startup raced the declaration of those queryables over
    Zenoh; the library never initialized and auto-start relied on a 500 ms sleep. `ServiceBuilder::on_start`
    dispatches startup commands straight into the inbox.
  - The Rust CDR codec padded every string to 4 bytes and required that padding when reading, unlike
    standard CDR (TypeScript, Python, Foxglove): commands from the browser ending in a string were
    rejected, and a `bool`/`uint8`/`uint16` after a string was misplaced for other readers.
  - Generated schema text listed dependencies first with `MSG: package/msg/Name` headers, which
    `@foxglove/rosmsg` cannot resolve; every nested same-package message (`RecordingLibrary`, `JobList`,
    `SettingsEnvelope`, ...) failed to decode in the browser, and MCAP readers took the first dependency
    as the root. The root definition now comes first, dependencies under `MSG: package/Name`.
  - The Recorder embedded schemas from a hand-kept list, so newer messages were recorded without one.
    `blueos_idl::schema(name)` is generated for every message.
- Verified on the device: library state lists every file with the right state and updates live; repair of a
  64 MB recording on the Pi takes about 7 s with progress and remaining time at 1 Hz; rejections return
  their reasons; snapshot of the recording being written produces an indexed copy served by nginx with
  ranges and the exposed CORS headers; delete from the page; the player opens a repaired recording from
  its index; SIGINT finalizes the active recording.
- Still unproven: repair cancel by hand on the device (repairs finished before a click), the 5 s rescan with
  hundreds of recordings, and a snapshot/repair under heavy write load.
