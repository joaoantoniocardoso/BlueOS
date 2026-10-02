# BlueOS Rust service: ergonomics review

Companion to `doc/architecture/rust-service-overview.md` (how it works) and `doc/architecture/rust-service-review.md`
(correctness bugs). This document is about a different question: **how hard is this code to learn and to use?** It is
the north for the next POC.

**The bar.** A junior developer must be able to read a service, understand what it does, and add an endpoint during
their first month. Today the author of the POC needed more than 4 hours to analyze the Recorder.

**Scope.**

- the multicall binary: `core/app/blueos/src/main.rs`
- the Recorder app: `core/services/recorder/app/src/`
- the Recorder logic: `core/services/recorder/logic/policy`, `core/services/recorder/logic/library`
- the Kernel API (`core/libs/app/service/src/builder.rs`) and the `Domain` trait (`core/libs/logic/cqrs/src/lib.rs`),
  as far as they shape the code above
- baseline for comparison: the previous prototype,
  `~/BlueRobotics/microservices_core_prototype/crates/blueos-core/services/ardupilot_manager`

References are `path:line` from the repository root, as of commit `17a3f639d`.

**Status.** Owner notes and orchestrator notes are collected. The deep-dive reviews (R1 to R8, see the end) are done
and merged into the themes they cover. `# Summary of recommendations` lists one recommendation per theme.

**How each theme is written.**

- **Owner notes:** what the POC author does not like, or wants in the next POC.
- **Orchestrator notes:** facts checked in the code while collecting. They explain or extend the owner notes.
- **Deep-dive questions:** what the reviewer must answer.

---

# Owner direction for the next POC

These are the principles that the next POC must follow. The themes E1 to E12 below are the evidence.

## P1. The Service owns an App, and the App owns everything else

**Owner notes**

- A BlueOS Service owns an App.
- The App owns the settings, the CLI, logging, the tokio runtime, and the Kernel.
- The App does the wiring and owns the state.

**Orchestrator notes**

- Today each of these pieces has a different owner:
  - the tokio runtime: each service's `run` (`core/services/recorder/app/src/lib.rs:128`, E2)
  - the CLI: each service, with its own conventions (E3)
  - the settings: the example loads them, the Kernel loads them a second time (review M4), and the Recorder does not
    load them at all (E11)
  - logging: the builder, inside `run_with_session` (`core/libs/app/service/src/builder.rs:365`)
  - the Zenoh session: the builder or the service, depending on the service (E2)
  - background tasks: nobody (E5)
- **Name clash.** "App" already has two meanings: `blueos_cqrs::App`, which is Snapshot + Jobs
  (`core/libs/logic/cqrs/src/lib.rs:157`, and the vocabulary in `doc/architecture/rust-service-overview.md`), and the
  `app/` layer folder. This proposal would add a third. The next POC must give "App" one meaning and rename the others.

**Deep-dive questions**

- Where is the line between Service and App? For example: is the Service the process entry and the App the in-process
  owner?
- What does "the App owns the state" include: the Domain Snapshot, the job graph, the background tasks of E5?

**Deep-dive findings (R1)**

- **The Kernel is a function, not a type.** "Kernel" in the docs points at `pub async fn run` with 18 parameters
  (`core/libs/app/service/src/runtime.rs:331`). `KernelState` (`core/libs/app/service/src/runtime.rs:310`) is private
  and holds only part of the state. The Inbox (`:437`), the adapter task handles (`:782`, `:849`, `:892`), the
  liveliness token (`:356`), and the log guard (`:360`) are local variables. A reader has no object to open.
- **The Kernel itself detaches tasks.** The three adapter spawners return `()` and drop their handles
  (`core/libs/app/service/src/runtime.rs:782`, `:849`, `:892`). Only the persist worker is owned and drained (`:441`,
  `:699`). The Recorder's detached tasks copy the Kernel's own style.
- **D-04 and the code disagree.** D-04 says tokio lives "in the kernel only" (`doc/architecture/decisions.md:141`),
  but each service builds its runtime, and the Recorder's `app/` crate spawns long-running tasks.
- **`ServiceBuilder` has three jobs:** collecting registrations, disk IO at registration time (`.settings`), and being
  the process entry (`run()` opens the session and installs logging). This is why `.settings` cannot chain.
- **Recommended ownership, split by lifetime:**

| Owner | Lifetime | Owns |
|---|---|---|
| `Service` | the process | CLI values, logging, the tokio runtime, the settings folder, the Session, the Kernel, the exit code |
| `Kernel` (public type) | one run of the loop | the Inbox, adapter tasks, service tasks, timers, the persist worker, the liveliness token, the state |
| `DomainState<D>` (today `blueos_cqrs::App`) | every Command | the Snapshot and the job graph, data only |

  The rule: **the Kernel owns everything that can be stopped; the state owns everything that can be copied.** Tasks
  cannot live in the state, because the state is cloned on every `Io` and `Persist` Effect.
- **Naming:** do not create a third "App". Rename `blueos_cqrs::App` to `DomainState` (its doc comment already says
  "Holds domain snapshot and job graph", `core/libs/logic/cqrs/src/lib.rs:156`), and keep the `app/` folder name.
  About 120 mechanical renames, done in one commit first.
- **Recommended shape:** a `Service` trait with `type Domain`, `type Arguments: clap::Args`, `const NAME`,
  `const VERSION`, and a **pure** `build(context) -> ServiceBuilder`. The Kernel provides `entry::run::<S>()`, which
  does everything that every service repeats today. `ServiceContext` hands `build` the parsed CLI, the open session,
  the settings loaded once, and a `spawn` for Kernel-owned tasks.

```rust
impl blueos_service::Service for Example {
    type Domain = PumpDomain;
    type Arguments = crate::cli::ExampleArguments;

    const NAME: &'static str = "example";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    fn build(context: &ServiceContext<Self::Arguments>) -> Result<ServiceBuilder<PumpDomain>, ServiceError> {
        let settings: ExampleSettings = context.settings()?;
        let builder = context.builder(DomainState::new(PumpSnapshot { settings, ..Default::default() }));
        let builder = crate::endpoints::register(builder);
        Ok(crate::tasks::register(builder, context))
    }
}
```

  `VERSION` stays one line per service because `env!("CARGO_PKG_VERSION")` expands in the crate that writes it.

**Deep-dive findings (R2)**

- **Correction:** the owner note "the App does the wiring and owns the state" inverts the current design. `.app()` moves
  `App<D>` into the builder (`core/libs/app/service/src/builder.rs:102`), `run` moves it into `KernelState`
  (`core/libs/app/service/src/runtime.rs:479`), and `run()` returns only `Result<(), ServiceError>`
  (`core/libs/app/service/src/builder.rs:330`). There is no accessor, so P1 is an inversion of today's design, not a
  refinement of it.
- **The builder is a one-way funnel, so in-process code has to smuggle data out.** Every builder method consumes `self`,
  and the only handle the Kernel ever gives back is `ShutdownHandle` (`core/libs/app/service/src/shutdown.rs:10`),
  obtained through a `&mut self` escape hatch (`core/libs/app/service/src/builder.rs:91`). Root cause: there is no read
  path out of the Kernel, so the Recorder publishes its tap filter from inside a State selector
  (`core/services/recorder/app/src/lib.rs:258`, E5) and a reader cannot tell that this is a workaround.
- **The Recorder opens its own Session for no technical reason.** `Session` is `Arc<Backend>` and its `Clone` is an
  `Arc::clone` (`core/libs/adapters/comms/src/lib.rs:140`), but the builder has a setter and no getter (`:120`). One
  accessor would give the same clones the Recorder gets from `core/services/recorder/app/src/lib.rs:160`, so the two
  services bootstrap differently for an invisible reason.
- **The Kernel cannot default `service_info.version`.** Both services pass `env!("CARGO_PKG_VERSION")`
  (`core/services/recorder/app/src/lib.rs:206`, `core/services/example/app/src/lib.rs:57`), which expands in the calling
  crate. Every "Kernel defaults" plan (E4) therefore needs associated consts written in the service crate, not `Option`
  defaults. R1 reached the same conclusion.
- **Answer, where the line falls:** `Service` is the process entry and owns only the `main` contract, arguments in and
  `ExitCode` out. The App is the in-process owner: the settings loaded once, the parsed CLI, logging, the tokio runtime,
  the dependency `Context` (E13), the Kernel, and the task handles.
- **Answer, what "the App owns the state" includes:** the App owns the lifecycle of the state and the Kernel keeps the
  exclusive write access. The App should hold a Command sender (E6) and a State subscription, not the Snapshot itself;
  direct mutable access would break the single-writer property that makes the Inbox safe
  (`doc/architecture/rust-service-overview.md:105`). Background tasks belong to the App as a named list with restart
  policies (E5), not to `tokio::spawn` call sites. Write that distinction down before renaming anything, or P1 reads as
  "put a `Mutex<Snapshot>` in the App", which undoes D-04.
- **Recommendation:** split `run_with_session` (`core/libs/app/service/src/builder.rs:339`) so that `build()` returns a
  `Kernel<D>` value exposing `session()`, `commands()`, `shutdown()` and `run(self)`, about 60 lines, and design it so
  that a later `Service` trait can drive it unchanged. The 18-argument `runtime::run`
  (`core/libs/app/service/src/runtime.rs:331`) becomes a named struct as a side effect. Rejected: keeping the funnel and
  adding getters only, about 20 lines, which does not fix task ownership.
- Decided (owner): R1. Adopt the `Service` trait with a Kernel-provided `entry::run::<S>()` directly. R2's
  `build()`/`run()` split and the public `Kernel` type are part of that design, not a separate first step.

## P2. Log from the first line

**Owner notes**

- The App must log from the beginning.

**Orchestrator notes**

- Logging starts in `run_with_session` (`core/libs/app/service/src/builder.rs:365`), after `Session::open`
  (`core/libs/app/service/src/builder.rs:334`). Everything before that is lost.
- In the Recorder, these messages are never printed:
  - a tokio runtime build failure (`core/services/recorder/app/src/lib.rs:128` to `:139`)
  - the `parse_cli` warnings (`core/services/recorder/app/src/cli.rs:40`)
  - a `create_dir_all` failure, or a `Session::open` failure (`core/services/recorder/app/src/lib.rs:160`)
  - the final `error!("recorder: {error}")` that reports those failures
- The example has the same problem (review L10).

**Deep-dive questions**

- Logging to Zenoh needs a session, and opening the session can fail. Design a two-stage start: console logging first,
  then the Zenoh log layer once the session exists. Should early records be buffered and replayed to Zenoh?

**Deep-dive findings (R1)**

- **Corrections:** `create_dir_all` is at `core/services/recorder/app/src/lib.rs:156`. The final
  `error!("recorder: {error}")` (`:146`) is lost only for failures before `builder.run()`; Kernel failures are printed,
  because the subscriber outlives the call. The Recorder passes its own session, so `Session::open` in the builder never
  runs for it.
- **The two-stage design already exists.** `init` installs the console layer and a Zenoh layer that does nothing
  until a global slot is filled (`core/libs/adapters/logging/src/zenoh_layer.rs:13`, `:55`). The Kernel fills it later
  (`core/libs/app/service/src/runtime.rs:360`). Only stage 1 is called too late, because its arguments (name,
  verbosity) are builder fields.
- **Records between the two stages are dropped**, and replaying them is blocked: `LogRecord` has no timestamp
  (`core/libs/adapters/logging/src/zenoh_layer.rs:26`); the time is read at encode time in the Kernel
  (`core/libs/app/service/src/runtime.rs:1106`).
- `init` returns `Result` but cannot fail: it discards `try_init` with `let _ =`
  (`core/libs/adapters/logging/src/lib.rs:34`), and a second call is silently ignored (`:24`).
- The "service_info is required" and "app is required" errors (`core/libs/app/service/src/builder.rs:343`, `:364`)
  come before `init`, so the first mistakes of a new service author print nothing.
- **Recommendation:** call `init` at the first line of the entry, with a `-v` count read from the raw arguments
  (about ten lines; it cannot fail, unlike the full `clap` parse). Buffer early records in the existing 1024-entry
  channel and replay them when Zenoh attaches. Add a timestamp to `LogRecord`, stamped when the record is created.

## P3. A panic must not crash the Service

**Owner notes**

- A panic must not crash the Service. The Service should be able to recover by itself (optional).

**Orchestrator notes**

- The release profile uses `panic = "unwind"` (`core/Cargo.toml:43`). The Kernel never calls `catch_unwind`.
- Today there are two failure modes, and they are opposite:
  - A panic in the Inbox loop ends the process. Examples: a Domain handler, a State selector, or a query decode
    (review H1).
  - A panic in a spawned task kills only that task, and nobody notices, because the handle is dropped. Examples: an IO
    task (review M10), an Adapter task, the Recorder's detached tasks (E5).
- Recovering inside the Inbox loop is not free. `App::handle` changes the Snapshot in place, so after a panic the
  Snapshot can be half-changed. A recovery needs a copy of the App taken before `handle`, which review H2 already asks
  for.

**Deep-dive questions**

- What is the unit of recovery: a task, the Kernel, or the whole process?
- Which restart policy (backoff, maximum retries), and who decides it (the Kernel default, or the service)?
- What does a client see during a recovery (the `status` State, rejected acks)?
- How does this fit with the container supervisor that restarts the process today?

**Deep-dive findings (R3)**

- **Corrections:** "a panic in a spawned task kills only that task, and nobody notices" is too strong. The default hook
  prints the panic to stderr, but it never passes through `tracing`, so it is absent from the Zenoh `log` key
  (`core/libs/adapters/logging/src/zenoh_layer.rs:54`), from MCAP, and from the frontend. The accurate wording is
  "invisible to the BlueOS log pipeline". The Inbox loop is also the safest place to panic: `IoContext::drop`
  (`core/services/recorder/app/src/lib.rs:103`) still finishes the MCAP file during the unwind, and
  `core/run-service.sh:142` restarts a non-zero exit. The list of task kinds is incomplete; add the comms subscription
  and queryable pumps (`core/libs/adapters/comms/zenoh/src/lib.rs:74`, `:94`, `:174`), the per-State query task
  (`core/libs/adapters/comms/src/state.rs:27`), the log publisher (`core/libs/adapters/logging/src/zenoh_layer.rs:97`),
  and the MCAP writer thread (`core/services/recorder/adapters/mcap/src/writer.rs:129`).
- **A panic is the one event the BlueOS log pipeline cannot carry.** Logging is attached only as a `tracing` layer and
  no panic hook is installed anywhere. Root cause: nothing bridges `std::panic` into `tracing`, so the failures that
  killed a task are the only ones missing from the MCAP file the Recorder writes for post-dive analysis.
- **`std::sync::Mutex` poisoning turns one panic into a permanent cascade.** `.expect()` on `lock()` is the repo-wide
  idiom (`core/services/recorder/app/src/lib.rs:97`, `:99`, `core/services/recorder/app/src/library_io.rs:47`,
  `core/libs/app/service/src/runtime.rs:148`), so after a panic while a guard is held every later lock panics too, in
  tasks that did not fail. The one exception, `IoContext::drop` (`core/services/recorder/app/src/lib.rs:106`), returns
  on a poisoned lock and therefore silently skips finishing the MCAP file.
- **A clean exit disables the supervisor.** `core/run-service.sh:148` breaks the restart loop when the service exits 0,
  and `:152` waits 5 s otherwise. So a half-dead process with a dead tap task is never restarted, while an intentional
  `Ok(())` return means "never start me again".
- **Recovery has no vocabulary on the wire.** `STATUS_DEGRADED` is defined in
  `core/libs/idl/interfaces/blueos_msgs/msg/ServiceStatus.msg` and no Rust code publishes it; both services hardcode
  `STATUS_READY` (`core/services/recorder/app/src/lib.rs:216`). A client watching a recovery sees a service that claims
  to be ready.
- **The Recorder panics on a pre-1970 clock inside an IO task** (`core/services/recorder/app/src/lib.rs:121`, `:124`,
  reached from the `OpenSession` arm at `:306`). The task dies, no `IoComplete` is sent, and `session_active` stays
  false with no error reaching the Domain or the client. This is review M10 with a concrete trigger. Unverified on
  hardware: whether a Pi without an RTC reports a pre-1970 time before NTP sync.
- **The model to copy is already in the tree.** The library IO blocking tasks check `JoinError` and turn it into a
  failed operation (`core/services/recorder/app/src/library_io.rs:184`, `:213`, `:268`, `:330`). It is the only place
  where a panic becomes a Domain-visible outcome.
- **Answers.** The unit of recovery is the task, with the Inbox loop as a second, special unit. Recovering the loop is
  the review H2 copy plus `catch_unwind` with `AssertUnwindSafe` around `handle` and the selectors
  (`core/libs/app/service/src/runtime.rs:958`), so once H2 is done the marginal cost is close to zero, and `futures`
  0.3 is already a workspace dependency (`core/Cargo.toml:75`). Restart policy: a Kernel default of exponential backoff
  from 100 ms to 30 s, unlimited restarts for long-running tasks and `Never` for one-shot tasks, overridable per task;
  three loop panics inside one minute means exit non-zero. Clients see `STATUS_DEGRADED` with the failing task name in
  `detail`, merged into the `status` State by the Kernel, plus the existing rejected-ack path (`:640`) for the Command
  in flight. The container supervisor stays the outer ring, because in-process recovery only has to beat a process
  restart, and a process restart costs 5 s plus a lost MCAP tail plus a library rescan.
- **Recommendation:** supervise tasks in the Kernel, which is the machinery E5 needs anyway, and recover the Inbox loop
  with the H2 copy, while keeping "unrecoverable means exit non-zero" as the invariant so the container supervisor stays
  meaningful. Install the panic hook and fix the poisoned locks first; without them a recovery system has no telemetry
  and cascades. Rejected: crash-only, where every dropped MAVLink fact costs a full restart and a split MCAP file; and
  `tokio-util` `TaskTracker`, which covers shutdown and joining but not restart, panic-to-Command or status reporting,
  and is not a workspace dependency today.

## P4. The examples answer "how do I do X?"

**Owner notes**

- The examples must cover every corner, so that each "how do I do X?" has an answer.

**Orchestrator notes**

- Today the example covers: Command, Query, State, Event, jobs, timers, settings, and IO.
- It does not show:
  - an IO query
  - a Command with a non-IDL body
  - reacting to another service's topics
  - calling another service's Command
  - sending a Command from in-process code
  - a long-running background task, such as a data plane
  - extending the CLI
  - composing two Domains
- The Recorder needed all of these, and built each one outside the Kernel (E5, E6, E7).

**Deep-dive questions**

- Write the list of "how do I do X?" questions, and map each one to an example. One example service, or several small
  ones?

**Deep-dive findings (R6)**

- **Correction:** the list of eight gaps is incomplete. Six more mechanisms exist in the builder and are also absent
  from the example: `on_start` (`core/libs/app/service/src/builder.rs:79`), `on_shutdown` (`:85`), `shutdown_handle`
  (`:91`), `session` injection for tests (`:120`), typed service errors (the example returns `Result<(), String>`,
  `core/services/example/app/src/lib.rs:47`), and structured logging. The example has two log calls in 1187 lines and
  both put the value in the message (`:43`, `:124`), so under P9 it currently teaches the wrong thing.
- **The example's integration test does not test the example's wiring.** `run_async` builds the chain and calls
  `builder.run()` (`core/services/example/app/src/lib.rs:125`), which opens a real session, so the test re-declares the
  whole chain by hand (`core/services/example/app/tests/kernel_integration.rs:26` to `:89`). The two have already
  drifted: the test's `jobs` selector uses `job.status as u8` (`:46`) while production maps through
  `internal_status_to_idl` (`core/services/example/app/src/lib.rs:181`). This is the same root cause R1 found for E2,
  and it means the file a new developer is told to copy is never executed by a test.
- **The example teaches a DRY violation that the Recorder then scales up.**
  `core/services/example/logic/pump/src/phase.rs:1` says the values must stay aligned with the IDL constants by hand,
  although the codegen already emits them (`core/libs/idl/codegen/src/lib.rs:429`) and D-02 allows a `logic/` crate to
  depend on `blueos-idl` (`doc/architecture/decisions.md:93`). The Recorder follows the lesson in `library_state_to_idl`
  (`core/services/recorder/app/src/lib.rs:631`); see E8.
- **The example ships a no-op presented as required wiring.** `.cli(Argv::default())`
  (`core/services/example/app/src/lib.rs:65`) is never paired with `.cli_command`, so it does nothing, and a reader
  copying the example copies it (E3).
- **"Minimal but complete" is 1187 lines, 19 files and three Cargo packages for eight endpoints**
  (`core/services/example/README.md:3`). Eight of the 19 files are 10 lines or shorter, so D-20's "one concept per file"
  has produced files too small to carry a concept and a reader must open nine of them to follow one Command.
- **There is no answer to "how do I add a new service".** `core/services/example/README.md` mentions neither
  `core/app/blueos/src/main.rs`, nor the feature in `core/app/blueos/Cargo.toml`, nor `core/tools/nginx/nginx.conf`, nor
  `core/start-blueos-core`. E1 counts four edit sites in `main.rs` alone.
- **Answer, the question list: 34 questions, and 14 of them have no answer inside `core/services/example/`.** The
  missing ones are partial job progress, an IO query, a non-IDL Command body, sending a Command from in-process code,
  subscribing to another service's topic, calling another service's Command, a long-lived background task, composing two
  Domains, extending the CLI, running a Command at startup, cleaning up at shutdown, typed errors, structured logging,
  and registering a new service. Every one is a mechanism the Recorder needed, and for each one the Recorder wrote a
  private answer that is now the pattern a new developer will copy.
- **Recommendation:** keep one `example-minimal` under 150 lines that answers only the basics (one Command, one Query,
  one State, state, a Domain unit test, the frontend call), and move everything else into a cookbook of numbered
  entries under `core/services/example/cookbook/NN-<topic>.rs`, each a self-contained test against the channel backend,
  with a README table mapping all 34 questions to an entry and a test asserting that every question resolves to a file
  that compiles. That makes "the example answers every question" mechanically checkable and keeps the thing you copy
  small. Rejected: growing the single example to roughly 3000 lines and 35 files, which destroys the one job it has;
  and four or five small services, which costs four binaries, four features and four nginx entries and raises the
  question "which one do I copy?". Required with any option: split `run_async` into a pure `build(...)` and a thin
  `run` so the example's real wiring is finally covered (see E2 and P6).

## P5. Ergonomics comes first

**Owner notes**

- Ergonomics has priority.

**Orchestrator notes**

- The bar at the top of this document can be measured. Candidate measures:
  - the number of concepts to learn before adding one Command (E12)
  - the number of files to touch to add one Command
  - the lines of wiring per endpoint

**Deep-dive questions**

- Pick the measures, measure them on the example, the Recorder, and the prototype, and set targets for the next POC.

**Deep-dive findings (R6)**

- **Correction:** the three candidate measures are the right ones but they miss the quantity that dominates both of
  them, the framework surface a reader must read before writing the first line. Here that is about 2550 lines
  (`core/libs/logic/cqrs/src/lib.rs` 364, `core/libs/logic/jobs/src/lib.rs` 583,
  `core/libs/app/service/src/builder.rs` 396, `core/libs/app/service/src/runtime.rs` 1207, which
  `doc/architecture/rust-service-overview.md:473` tells the reader to read). In the prototype it is about 712 lines of
  `commonwealth`, and the rest is stock axum, serde and utoipa.
- **Transferability must be weighted, not just counted.** 21 of the POC's 24 concepts are repo-local, against 1 of the
  prototype's 8. A developer who learns axum extractors can use that knowledge anywhere; `Decision`, `io_from_job`,
  `JobSpec`, `TapPolicy` and `command_allow_empty` are worth nothing outside this repository. Counting concepts without
  that weighting understates the gap.
- **Answer, the six measures and their values.** C is the concepts to add one Command end to end, F the files
  hand-edited, W the wiring lines per endpoint, S the lines of the smallest complete service, R the framework lines a
  reader must read first, T the time to a first endpoint.

| Measure | POC example | POC Recorder | Prototype `disk_usage` | Target |
|---|---|---|---|---|
| C: concepts for one Command | 24 (27 with a State and an Event) | 24 plus 6 service-private | 8 | <= 8 |
| F: files hand-edited per Command | 8 (13 with a State and an Event) | 6 to 8 | 2 | <= 3 |
| W: wiring lines per endpoint | 27 | about 70 | 13 | <= 12 |
| S: service floor | 1187 lines / 19 files / 3 crates | 7474 / 23 / 6 | 197 lines / 10 files / 2 crates | <= 250 lines, 1 crate |
| R: framework read set | about 2550 lines, all repo-local | same | about 712 repo-local plus axum | <= 600 repo-local |
| T: time to first endpoint | 1 to 2 days | the owner needed 4+ hours to read it | 1 to 2 hours | <= 2 hours |

  W for the example is `SetLevel` measured end to end: `.msg` 4 lines, `api.lock` 1, the Command variant and match arm
  2, the handler 15, and `core/services/example/app/src/lib.rs:68` to `:72` 5. W for the Recorder is 1114 wiring lines
  (`lib.rs` 716, `inject.rs` 307, `cli.rs` 76, `error.rs` 15) over 12 endpoints. The T row is an estimate, not a
  measurement: the honest way to get it is to hand both repositories to a developer who has seen neither.
- **Recommendation:** adopt all six measures with the targets above, and enforce three of them mechanically in
  `.hooks/lib/rust_checks.sh`: S as a test that the minimal example stays under 250 lines, W as a test that each
  service's registration block stays under `12 * endpoint_count` lines (the denominator is already built from the
  registrations, `core/libs/app/service/src/builder.rs:350`), and F as a check that a pull request adding one `.msg`
  touches at most three hand-edited Rust files. A measure nobody runs is a wish; these three are cheap and they fail
  loudly.

## P6. Test-driven development for the next draft

**Owner notes**

- The code is heavily undertested. The next draft must be written with TDD.
- Rust code coverage reports from day zero, integrated in CI.

**Orchestrator notes**

- There are 162 Rust tests for about 17,100 lines of Rust in the Kernel, the shared libraries, the example, and the
  Recorder (test code included).

| Crate | Tests | Lines |
|---|---|---|
| `core/libs/app/service` (Kernel) | 17 | 2,728 |
| `core/libs/adapters/comms` | 14 | 1,745 |
| `core/libs/idl` | 16 | 1,846 |
| `core/services/example` | 10 | 1,046 |
| `core/services/recorder/app` | 13 | 2,517 |
| `core/services/recorder/adapters/mavlink` | 0 | 346 |

- None of H1, H2, M1, M2, or M10 from the correctness review has a test (review summary).
- The Recorder bootstrap (`run_async`, about 340 lines) has no test. It cannot have one as written: it opens a real
  Session and starts detached tasks (E2, E5).
- The tests call `sleep(` 35 times. Timing-based tests are slow and can be flaky.
- The clock is read inline in many places (E10), so time-dependent logic outside the Domain is hard to test.
- CI runs `cargo test` (`.github/workflows/test-and-deploy.yml:118`), but it measures coverage only for Python
  (`core/htmlcov`, `.github/workflows/test-and-deploy.yml:47`). Rust coverage is not measured.

**Deep-dive questions**

- Which design changes make TDD possible? Candidates: an injected clock, an in-process sender to the Inbox, a Kernel
  test harness that runs without sleeps.
- Which test layers should exist (Domain unit tests, Kernel harness, service integration on the channel backend), and
  what does each one own?
- Rust coverage in CI: which tool, where it runs (the Rust job at `.github/workflows/test-and-deploy.yml:114` to
  `:118`, or `.hooks/lib/rust_checks.sh:51` so it also runs locally), how the report is shown on a PR, and whether a
  minimum (overall, or per changed line) fails the build.

**Deep-dive findings (R7)**

- **Corrections, measured at this commit.** There are 163 test functions, of which 160 run and 3 are skipped by
  `#[ignore]` (`core/libs/adapters/comms/zenoh/src/lib.rs:303`, `:339`, `:376`). The tree is 17,569 lines of Rust,
  14,259 under `src/` and 3,295 under `tests/`. The per-crate table lists 5 of 25 crates and omits the best-tested
  ones: `libs/api` 17 tests, `recorder/logic/library` 18, `recorder/adapters/mcap` 16, `libs/adapters/settings` 12. The
  sleeps are 36 calls of which 33 execute. "Timing-based tests are slow" is wrong: the whole suite finishes in 1.579 s
  under nextest, and the cost is determinism, not time. Of 17 `now()` sites, 5 use `tokio::time::Instant` and are
  already mockable with `tokio::time::pause()`; the 12 that use `std::time` are not.
- **Coverage is 59.67% of lines, and the gap is not spread evenly.** The `logic/` crates are 77 to 99% covered and the
  `app/` crates are 0 to 9%: `core/services/recorder/app/src/lib.rs` is at 9.13% region coverage,
  `core/services/example/app/src/lib.rs` at 0.00%, `core/services/recorder/app/src/inject.rs` and `cli.rs` at 0.00%,
  `core/services/recorder/adapters/mavlink` at 0.00% across all four files.
- **The production Zenoh backend has zero executed coverage** (`core/libs/adapters/comms/zenoh/src/lib.rs`, 245 lines).
  The only three tests that would cover it are `#[ignore]`-gated on a local zenohd that CI never installs, so
  `ChannelBackend` is the only exercised backend. Since it dispatches a query to the first matching queryable only
  (`core/libs/adapters/comms/channel/src/broker.rs:131`) while Zenoh sends it to all of them, the Kernel tests are
  structurally incapable of catching review L4.
- **`tokio`'s `test-util` is already a dev-dependency and nothing uses it** (`core/libs/app/service/Cargo.toml:30`).
  There is not one `start_paused`, `time::pause` or `time::advance` in the workspace. The 16 sleeps in
  `core/libs/app/service/tests/kernel.rs` could become `#[tokio::test(start_paused = true)]` today, with no library
  change. This is the cheapest single improvement available.
- **The integration tests re-implement the production wiring instead of calling it**, so the wiring has no coverage at
  all; this is the same finding as P4 and E2, and it is the root cause of the 0 to 9% app layer, not a lack of effort.
- **16 of 17 Kernel tests tear down with `.abort()`**, so `on_shutdown` never runs, no task is joined, and a leaked task
  is indistinguishable from a clean one. Only `graceful_shutdown_via_handle`
  (`core/libs/app/service/tests/kernel.rs:706`) asserts the service returns `Ok`, so task-leak and lifecycle regressions
  cannot fail this suite.
- **`spawn_blocking` will fight a paused clock.** Tokio's auto-advance fires when the runtime is idle, and a
  blocking-pool task does not keep it busy, so a paused-clock harness can advance past a blocking file operation. There
  are 11 such calls, including `core/services/recorder/app/src/library_io.rs:77` and
  `core/libs/app/service/src/runtime.rs:447`. Unverified by experiment, reasoned from tokio's auto-advance contract; the
  design consequence holds either way, which is that file IO must sit behind an injectable port.
- **There is effectively one executing doctest in the workspace.** The Kernel's 100-line teaching example
  (`core/libs/app/service/src/lib.rs:19`) is a `no_run` block and two others are `ignore`. Separately,
  `core/libs/idl/codegen` is 908 lines with no tests of its own, and it holds both the H1 decode bug (`:523`) and
  `is_append_only_evolution` (`:864`).
- **Answer, the design changes that make TDD possible, in order:** paused-clock tests, which need no library change;
  splitting `run_async` into `build(config) -> ServiceBuilder<D>` and `run(builder)`; `command_sender()` mirroring
  `shutdown_handle()` (E6); an injectable `Clock` with `now_wall()` and `now_mono()` (E10); Kernel-owned tasks with a
  readable `tasks_running()` (E5); an Effect and publish recorder in the harness; file IO behind a port; and the Kernel
  loading settings once and handing the value to the app (E11, review M4). The first two cost little and are what
  currently keeps the app layer untested.
- **Answer, the test layers.** L1 Domain unit tests own the decisions, with no runtime. L2 owns codecs and contracts,
  including hostile CDR input and the `api.lock` gate. L3 is the Kernel harness and owns everything the Inbox loop
  promises, on the channel backend with a paused clock, an injected clock, an in-process sender and an Effect recorder,
  with no sleeps and no polling. L4 owns service wiring: that the real `build()` registers exactly the endpoints
  `ServiceInfo` advertises. L5 owns backend conformance, one shared test body run against both backends, with a zenohd
  container. L6 owns device and system tests. Today `kernel.rs` is L3 and L5 at once, done with sleeps, which is why it
  is slow to write, flaky to run, and still blind to L4 and review L13.
- **Answer, coverage in CI.** Use `cargo-llvm-cov` with nextest, measured here at 53.6 s warm; reject tarpaulin
  (miscounts generics and inlined code) and grcov. Run it in CI only, folded into the existing `rust-tests` job, which
  today builds the test binaries twice (`.github/workflows/test-and-deploy.yml:112` to `:118`); do not put it in
  `.hooks/lib/rust_checks.sh:51`, because llvm-cov sets different `RUSTFLAGS` and invalidates the developer's target
  directory. Report with `--summary-only` into `$GITHUB_STEP_SUMMARY` plus `--fail-under-lines`, so no bot comment and
  no third-party token. Gate on per-layer floors, not one overall number: a single 60% gate is already satisfied and
  would let the app layer stay untested forever. Start the floors at today's measured values (`libs/logic` 90%,
  `libs/app/service` 72%, `services/*/logic` 70%, `services/*/app` 0% but must rise every pull request, workspace 59%)
  in a committed ratchet file that may only increase, and add changed-lines gating once the workspace number clears
  about 80%. Gate on lines or regions, not branches: the branch column came back empty on stable.
- **Recommendation:** retrofit, then harness-first. Land the paused-clock change and the `build()` seam on the current
  POC to get a measured baseline and a real seam, then design the next Kernel around the L3 harness and use the
  retrofitted tests as its acceptance suite, porting H1, H2, M1, M2 and M10 as failing tests before the service is
  written. Rejected: retrofit alone, which leaves the app layer shaped so that tests must copy the wiring; and a
  harness-first rewrite alone, which risks designing the harness against imagined services.

## P7. Code quality tools from day zero

**Owner notes**

- Integrate a set of code quality tools from day zero, such as [rustqual](https://lib.rs/crates/rustqual).

**Orchestrator notes**

- `.hooks/lib/rust_checks.sh` runs today:
  - `cargo fmt --check` (`:44`)
  - `cargo clippy -D warnings`, with default lints only (`:47`); `core/Cargo.toml` has no `[workspace.lints]`
  - `cargo test` (`:51`)
  - `cargo deny check bans`, without advisories or licenses (`:55`)
  - `cargo check` on a `no_std` target for the logic crates (`:109`, `:113`)
  - `cargo semver-checks`, which is skipped for every crate today (`:116` to `:132`, review M8)
- rustqual (version 1.8.3, about 3,900 downloads) checks seven dimensions: IOSP, complexity, DRY, single
  responsibility, coupling, test quality, and architecture. Several findings in this document belong to those
  dimensions:
  - complexity and single responsibility: `run_async` is one 340-line function (E2)
  - DRY: the `InjectedCommand` copy of `RecorderCommand` (E6), three `SystemAndComponent` types (E8), the calendar code
    written twice (E10)
  - architecture: the layer rule in `doc/architecture/decisions.md` is checked only by folder convention
  - test quality: P6

**Deep-dive questions**

- Which tool set, and which gate for each tool (fail the build, or report only)? Candidates besides rustqual: workspace
  `clippy` lint groups (`pedantic`, selected `restriction` lints), `cargo deny` advisories and licenses,
  `cargo machete` for unused dependencies, and a check that enforces the layer dependency rule.
- Run rustqual on the current POC: does it flag the findings above? Use the result to calibrate its thresholds.
- rustqual is young. What is the risk of depending on it, and what is the fallback?

**Deep-dive findings (R7)**

- **Addition:** there is also no `rustfmt.toml` and no `clippy.toml` anywhere in the repository, so every formatting and
  lint default is whatever the toolchain ships.
- **`cargo deny check advisories licenses` fails today, in 0.89 seconds.** Three vulnerabilities: `lz4_flex 0.10.0`
  (RUSTSEC-2024-0436, decompression can leak uninitialised memory), reached through MCAP compression, which is exactly
  the path the Recorder feeds attacker-influenceable data into; and `quick-xml 0.39.4` via the `mavlink-bindgen` build
  dependency (RUSTSEC-2026-0194 and RUSTSEC-2026-0195, remote memory exhaustion). One unmaintained crate, `paste`. One
  unlicensed workspace crate, `blueos-idl-codegen`. The 371 license rejections are a configuration artefact, not 371
  problems: `core/deny.toml` has no `[licenses]` section, so cargo-deny 0.20 rejects everything. The real set is 25
  expressions, all permissive or permissively dual-licensed; only `option-ext 0.2.0` is single-licensed weak copyleft
  (MPL-2.0), and the 31 Eclipse-licensed crates are all zenoh and all `EPL-2.0 OR Apache-2.0`.
- **`core/libs/idl/codegen/Cargo.toml` is the only manifest that does not inherit `[workspace.package]`.** It hard-codes
  its version and edition and omits `license` entirely, which is what produces the unlicensed error above and means a
  workspace-wide edition or license change silently skips this crate.
- **Clippy with `pedantic` plus `nursery` produces about 360 new warnings in 18.2 seconds**, 280 pedantic and 80
  nursery. The useful ones: 10 `cast_possible_truncation`, the same family as H1's 32-bit wraparound; 6
  `significant_drop_tightening`, the nearest thing to a deadlock signal clippy offers; 3 clone problems; 4
  `needless_pass_by_value`; 2 `too_many_lines`, `run_async` among them; 1 `wildcard_imports`. The noise is about 15
  `must_use_candidate`, 8 `missing_const_for_fn` and 6 `doc_markdown`.
- **Zero `await_holding_lock` hits, which is a real positive result.** `std::sync::Mutex` is used in async code in six
  places (`core/libs/app/service/src/runtime.rs:122`, `core/services/recorder/app/src/library_io.rs:23`,
  `core/services/recorder/app/src/lib.rs:97`, `:99`, `core/libs/adapters/comms/channel/src/broker.rs:18`,
  `core/libs/adapters/comms/zenoh/src/query_responder.rs:9`), and no guard is held across an await. That lint is
  warn-by-default and already passes, so it is worth denying to keep it locked in.
- **13 `#[allow(...)]` attributes, none with a reason.** `clippy::allow_attributes_without_reason` would flag all 13 and
  is the one lint that makes suppressions reviewable; `#[allow(dead_code)]` at
  `core/services/recorder/app/src/cli.rs:65` is the silently ignored `--zkey` of E3, and a reason field would have
  forced the author to write down why.
- **`cargo +nightly udeps` finds one real unused dependency and one instructive false positive.** `tempfile` in
  `core/services/example/app` dev-dependencies is genuinely unused. `serde_arrays` in `libs/idl` is emitted by the
  codegen as `#[serde(with = "serde_arrays")]` into `OUT_DIR` (`core/libs/idl/codegen/src/lib.rs:419`), and no current
  `.msg` has a fixed array large enough to trigger it, so the dependency is correct only by coincidence and no tool can
  see the link. Any unused-dependency gate needs an explicit ignore entry with a comment here.
- **Answer, the gates.** Fail on `cargo fmt`, clippy with `[workspace.lints]`, `cargo deny check bans licenses sources`,
  `cargo nextest run` plus `cargo test --doc` separately, coverage with per-layer floors, an unused-dependency check on
  stable (`cargo machete` or `cargo-shear`, not nightly `udeps`), the existing layer check, `typos-cli`, and
  `cargo auditable build`. Report only: `cargo bloat`, the nightly job, and the young quality tools. Advisories are the
  exception and should report on a pull request and fail on the existing 6-day cron
  (`.github/workflows/test-and-deploy.yml:10`), because otherwise the first new RUSTSEC in a transitive dependency turns
  every open pull request red and the team learns to ignore the check. `cargo semver-checks` is a no-op until
  `blueos-idl` is publishable (E9), so either publish it or replace the gate with the `api.lock` field-signature check
  of review M8.
- **Answer, rustqual on this POC: not run, and the calibration risk is the reverse of what the note assumes.** rustqual
  was not installed and nothing about it was measured, so everything here is unverified. From its documented defaults it
  would almost certainly fire on `run_async` (`max_function_lines = 60` against 340 lines) and on the duplicated
  calendar code (E10), and its Test Quality dimension can consume the lcov file llvm-cov already produces. But
  `rustqual --init` writes thresholds calibrated to the current codebase, so run on this POC it would set
  `max_function_lines` near 340 and bless `run_async`. Write `rustqual.toml` by hand at the documented defaults.
- **Answer, the risk and the fallback.** Version 1.8.3, about 3,900 downloads, one maintainer, and it parses Rust
  source, so a toolchain bump can block every pull request. Mitigation: pin the exact version, gate only on specific
  rule IDs that are green today and never on the composite score, keep it `continue-on-error` for a full release cycle,
  and cache the binary. The fallback is that nothing it offers is load-bearing: clippy's nursery and restriction lints
  cover complexity, `ast-metrics` covers coupling, coverage floors cover test quality, and the highest-value dimension,
  architecture layers, is already enforced better by `.hooks/lib/rust_checks.sh:60` to `:103`, in bash, with no vendor.
- **Recommendation:** fix the gates already paid for first (the `[licenses]`, `[advisories]` and `[sources]` sections in
  `core/deny.toml`, `license.workspace = true` in the codegen manifest, `[workspace.lints]`, nextest, coverage), which
  is under a day and catches 3 CVEs and about 360 invisible clippy findings; then add the mature extras (`cargo
  machete`, `typos-cli`, `cargo auditable`, a nightly report job); then add rustqual, ast-metrics and thailint
  report-only and pinned. `cargo-quality` is rejected outright at 242 total downloads. The thing not to do is gate on
  rustqual on day zero: it would be the only single-maintainer tool with the power to block a release.

## P8. Supreme code quality: the tool set

**Owner notes**

- All the basics: fmt, deny, check, clippy, audit, bloat, auditable.
- Use nextest for tests.
- No unsafe code, so skip the unsafe-focused tools (miri, kani, mirai, rudra, and similar).
- Add protection against:
  - incompatible licenses in dependencies
  - deadlocks
  - memory leaks
  - thread leaks
  - lifecycle problems
  - unused dependencies and features
  - dead code
  - anti-patterns ([Rust patterns book, appendix C](https://www.rust-patterns.com/book/33-appendix-c-anti-patterns.html))
- Candidate tools named by the owner: ASTMetrics, thailint, rust-san, cargo-careful, rust-audit, rust-deny, luckbud,
  cargo-check-deadlock, tangleguard, rustqual, cargo-quality, valgrind, a spell checker.

**Orchestrator notes**

- Checked on crates.io (2026-10-01):

| Tool | crates.io | Version | Downloads |
|---|---|---|---|
| `cargo-nextest` | yes | 0.9.146 | 13.0 M |
| `cargo-audit` (named "rust-audit") | yes | 0.22.2 | 12.6 M |
| `cargo-llvm-cov` (P6) | yes | 0.9.1 | 8.1 M |
| `cargo-deny` (named "rust-deny") | yes | 0.20.2 | 6.1 M |
| `cargo-machete` | yes | 0.9.2 | 3.0 M |
| `cargo-hack` | yes | 0.6.45 | 1.9 M |
| `cargo-udeps` | yes | 0.1.61 | 1.5 M |
| `cargo-auditable` | yes | 0.7.6 | 1.1 M |
| `typos-cli` | yes | 1.50.3 | 1.0 M |
| `cargo-shear` | yes | 1.14.0 | 0.5 M |
| `cargo-bloat` | yes | 0.12.1 | 0.4 M |
| `cargo-spellcheck` | yes | 0.15.7 | 0.3 M |
| `cargo-careful` | yes | 0.4.10 | 0.2 M |
| `cargo-check-deadlock` | yes | 1.2.2 | 19 k |
| `rustqual` | yes | 1.8.3 | 3.9 k |
| `cargo-quality` | yes | 0.6.0 | 242 |
| ASTMetrics, thailint, rust-san, luckbud, tangleguard | no | | |

- The names not on crates.io may be tools published elsewhere, or other names. Two likely matches, to confirm:
  "luckbud" may be lockbud (a deadlock detector), and "rust-san" may be the rustc sanitizers (`-Zsanitizer=leak`,
  `-Zsanitizer=thread`). Both need a nightly toolchain.
- Candidate tool for each protection, to be confirmed by the reviewer:

| Protection | Candidates |
|---|---|
| licenses, advisories | `cargo deny check licenses advisories` (today only `bans` runs, P7), `cargo audit` |
| deadlocks | `cargo-check-deadlock`, lockbud, clippy `await_holding_lock` |
| memory leaks | LeakSanitizer, valgrind on test binaries |
| thread leaks, data races | ThreadSanitizer |
| lifecycle problems | no standard tool; needs a design where the Kernel owns every task (P1, E5) and a test that asserts all tasks end at shutdown |
| unused dependencies | `cargo-machete`, `cargo-shear`, `cargo-udeps` (nightly) |
| unused features | `cargo-hack` feature checks |
| dead code | rustc `dead_code`, `unreachable_pub`; forbid `#[allow(dead_code)]` (one exists today, E3) |
| anti-patterns | clippy lint groups mapped to the appendix, rustqual, cargo-quality |
| spelling | `typos-cli`, `cargo-spellcheck` |
| binary size, SBOM | `cargo-bloat`, `cargo-auditable` |

- Some tools need nightly (sanitizers, lockbud, `cargo-udeps`). CI needs a pinned nightly job next to the stable build.

**Deep-dive questions**

- Identify ASTMetrics, thailint, and tangleguard, and confirm the two likely matches above.
- For each protection: the tool, the CI job (stable or nightly), the gate (fail or report), and the run time.
- Run each candidate on the current POC. Does it find something real, or only noise?

**Deep-dive findings (R7)**

- **Answer, the five names identified** (high confidence for all five). ASTMetrics is **ast-metrics**
  (`ast-metrics.dev`), a single static Go binary that reads Rust plus six other languages, reports cyclomatic
  complexity, maintainability, coupling, instability and bus factor from git history, has a linter mode with a non-zero
  exit, and offers `ast-metrics review` to diff a branch against its base and report only regressions. thailint is
  **thai-lint** (`pip install thailint`, PyPI 0.25.0, MIT), tree-sitter based, with Rust rules `unwrap-abuse`,
  `clone-abuse` and `blocking-async`, plus language-agnostic `dry`, `nesting`, `srp` and `file-placement`. tangleguard
  is **TangleGuard**, a commercial layer and cycle checker driven by a YAML rules file, **closed source today**. luckbud
  is confirmed as **lockbud**, a MIR-level deadlock and panic detector that needs a pinned nightly. rust-san is
  confirmed as the **rustc sanitizers** (`-Zsanitizer=address,leak,thread,memory`), nightly only.
- **Corrections to the candidate table.** `cargo audit` and `cargo deny check advisories` read the same RUSTSEC
  database, so running both is duplicated maintenance for no extra coverage; pick `cargo deny`, which is already a
  dependency and also covers bans, licenses and sources. And unused dependencies do not need the nightly job after all:
  `cargo machete` and `cargo-shear` do `cargo-udeps`'s job on stable.
- **`cargo-deny` is built from source in CI while `cargo-semver-checks` is installed prebuilt.**
  `.github/workflows/test-and-deploy.yml:96` runs `cargo install --locked cargo-deny@0.20.2`, compiling it and its whole
  dependency tree on every cache miss; twelve lines later `:99` uses `taiki-e/install-action@v2` for semver-checks.
  Switching is free and removes minutes from a cold run.
- **There is no `.config/nextest.toml`, so the per-test timeout nextest gives for free is unused**, and no Rust job has
  `timeout-minutes`. Both are one-liners, and both turn a hang from a full runner burn into a 10-second failure.
- **`[graph] exclude-dev = true` (`core/deny.toml:4`) hides dev-dependency advisories** while build dependencies stay in
  scope, which is why both quick-xml advisories are correctly reported but a vulnerable test-only dependency would be
  silent. Separately, `strip = true` (`core/Cargo.toml:44`) means a sanitizer finding could not be reproduced against a
  shipped binary, so a sanitizer plan needs its own profile with symbols.
- **Answer, the protections that have no tool at all are the two the owner cares most about.** Memory leaks: there is no
  `unsafe` code, so LeakSanitizer and valgrind cannot see the real bug, which is review M9's unbounded job-graph growth;
  the protection that works is a bounded-memory assertion in the Kernel harness. Thread leaks: ThreadSanitizer detects
  data races, not leaks, and has known false positives on the tokio work-stealing scheduler; the actual defect is 16
  detached `tokio::spawn` calls with dropped handles, which becomes `assert_eq!(0, tasks_running())` once the Kernel
  owns its tasks (P1, E5). Lifecycle problems: same answer, and it is the right one. Deadlocks:
  `clippy::await_holding_lock` is already clean and should be denied, `significant_drop_tightening` is a style signal,
  and lockbud will only model the blocking `std::sync` locks. Dead code: the coverage report is already a dead-code
  detector, since `inject.rs` and the MAVLink adapter at 0.00% are either dead or untested and both answers need action.
- **Answer, anti-patterns: 12 of appendix C's 19 have at least partial clippy coverage, 4 have none, and 1 is
  complete.** `unsafe_code = "forbid"` is free and makes skipping miri, kani and rudra defensible rather than assumed.
  The gap that actually bites here is stringly-typed APIs: `EndpointInfo.kind` is a `String`, IDL enums arrive as `u8`
  constants, and `inject.rs` encodes enums as strings with silent default arms
  (`core/services/recorder/app/src/inject.rs:121`, `:140`). No tool will close that one; it belongs in the IDL decision
  of E8.
- **Answer, the CI layout: seven jobs.** A stable lint job (fmt, clippy with the workspace lints, the layer check, the
  `no_std` and wasm32 checks); a test job (`cargo llvm-cov nextest`, the per-layer floor script, `cargo test --doc`); a
  supply-chain job (all four `cargo deny` checks, `cargo machete`, `typos`); a report-only quality job (rustqual,
  thailint, ast-metrics); a report-only pinned-nightly job (udeps, branch coverage, sanitizers, lockbud); a backend
  conformance job with a zenohd service container for the L5 layer of P6; and the existing cross-build job extended with
  `cargo auditable build` and `cargo bloat`. Jobs 1, 2, 3 and 6 gate the merge and run in parallel, so the blocking Rust
  path is the test job, with a target of 8 minutes. Two conditions make that hold: install every tool with
  `taiki-e/install-action` and never `cargo install`, and keep coverage out of the pre-push hook. Four distinct
  `Swatinem/rust-cache` keys are needed, because plain, coverage-instrumented, nightly and per-target cross fingerprints
  genuinely differ. CI durations are unverified: no run of the workflow in the last 40 on `bluerobotics/BlueOS` contains
  a Rust job, so the estimates extrapolate from warm local measurements.
- **Recommendation:** make the mature set the day-zero bar (the fixes above plus nextest with a timeout config,
  coverage with floors, `cargo machete`, `typos`, `cargo auditable`, and the conformance job), and add the exploratory
  tier in the same pull request as report-only jobs that never gate. Rejected: `cargo-check-deadlock` and TangleGuard,
  which duplicate checks the repository already has better; `cargo-quality`; and spending the nightly job's 15 minutes
  on sanitizers that cannot see reference cycles or detached tasks while the Recorder app sits at 9% coverage. Every
  tool that found something real found it in under 20 seconds, and the two sharpest findings came from tools the project
  already depends on but does not fully invoke.

## P9. Guidance for developers and their agents

**Owner notes**

Developers and their agents must be guided towards:

- best-in-class idiomatic Rust
- KISS
- TDD (P6)
- careful use of architectural patterns: do not introduce new patterns, and document every new one
- documented code
- new dependencies always added with `default-features = false`, plus only the needed features
- a declaration order that lets a reader understand a file top-down, in a single pass (see "Declaration order" below)
- imports always grouped and chained, in hierarchical blocks: std, third-party crates, `blueos` crates, owned modules,
  then relative paths
- no re-exports, ever
- structured logging, always
- readability first:
  - group blocks of code by intent
  - name variables by meaning, not by type
  - no abbreviations, even obvious ones: `Err(error)` or `Err(reason)`, never `Err(err)`
- clones for an async block go in a new scope, so they do not pollute the outer scope:

```rust
// Good
tokio::spawn({
    let session = session.clone();
    async move { session.run().await }
});

// Bad
let session = session.clone();
tokio::spawn(async move { session.run().await });
```

**Declaration order**

The goal is that a reader understands a file in one pass from top to bottom, without jumping to the end and then back
up. The order:

1. All type declarations. A type comes before the types it uses.
2. All `impl` blocks, in the same order as their types.
3. Free functions. A caller comes before the functions it calls.

```rust
struct A {
    b: B,
    c: C,
}

struct B {
    c: C,
}

struct C;

impl A { /* ... */ }

impl B { /* ... */ }

impl C { /* ... */ }
```

When the uses form a cycle (a method of `C` calls `B`), a single pass is impossible. Then follow the main direction of
use, as far as possible.

**Orchestrator notes**

- The owner's personal editor rules will be removed. The rules in this section replace them, so the two earlier
  conflicts are settled: the declaration order is the one above, and logging is structured.
- The current code is far from several of these rules:
  - `default-features = false` is used by 4 of the 20 third-party workspace dependencies in `core/Cargo.toml` (R8).
  - There are 37 `pub use` lines in `core/libs`, `core/services`, and `core/app`. Some rename types (E7).
  - Logging is mixed: about 40 calls use structured fields (`error!(%error, "...")`), and about 29 put values in the
    message (`error!("recorder: {error}")`).
  - The clone-in-scope pattern is used in some places (the `.state` and `.io` closures in
    `core/services/recorder/app/src/lib.rs`) and not in others (`core/services/recorder/app/src/lib.rs:408` to `:411`).
- There is no `rustfmt.toml`. Stable rustfmt cannot express the five import groups. The `group_imports` option is
  nightly-only, and it gives three groups (std, external, crate). The five-group order needs nightly rustfmt plus a
  custom check, or a custom check alone.
- The rules live only in the owner's personal editor rules (`~/.cursor/rules`). Other developers and their agents do
  not get them. The repository has `AGENTS.md`, but it covers the Python and Vue code.
- The declaration order is not settled for constants, trait `impl` blocks (before or after the inherent `impl`), and
  `#[cfg(test)]` modules.

**Deep-dive questions**

- Which rules can a tool enforce (rustfmt, clippy lints such as `allow_attributes_without_reason`, a custom check),
  and which can only be written as guidance?
- Where does the guidance live so that developers and agents both read it: `AGENTS.md`, a Rust section in
  `doc/architecture/decisions.md`, editor rules committed to the repository?
- Write one before and after example for each rule, taken from this POC.
- Can a tool check the declaration order? Clippy's ordering lint (`arbitrary_source_item_ordering`, to confirm) sorts
  by kind and name, not by use, so it would fight this rule. Is a custom check from the use graph feasible, and where
  do constants, trait `impl` blocks, and test modules go?

**Deep-dive findings (R8)**

- **Corrections.** `[workspace.dependencies]` has 43 lines, not 54 (`core/Cargo.toml:47` to `:89`); five carry
  `default-features = false` but one of those is a path dependency, so of the 20 third-party dependencies only 4 have it
  (`:78`, `:82`, `:84`, `:89`). The logging split is 97 `tracing` calls: 46 structured, 31 with the value interpolated
  into the message, 3 mixing both, and 17 with a constant message, so 34 violations rather than 29. Stable rustfmt does
  **preserve** blank-line-separated import groups and sorts only within each one, verified by round-tripping a
  three-group file, so the five groups are maintainable on stable plus a membership check and no nightly is needed; on
  nightly, `group_imports = "StdExternalCrate"` would actively merge `blueos_*` with third-party crates and `crate::`
  with `self::`. For the clone-before-`async move` rule the score is 12 violations and 2 compliant sites, both in a test
  file (`core/libs/app/service/tests/kernel.rs:452`, `:744`), so no production `tokio::spawn` in the repository uses the
  scoped form.
- Resolved: R8's count is the correct one. The orchestrator's "54" counted every matching line in `core/Cargo.toml`,
  not only `[workspace.dependencies]`. There are 43 dependency lines; 4 of the 20 third-party dependencies use
  `default-features = false`.
- **`clippy::arbitrary_source_item_ordering` enforces the useful half of the declaration order, once configured.** It
  has two independent halves. `module-item-order-groupings` in `clippy.toml` declares an ordered list of item kinds and
  matches the owner's rule for free. The alphabetical half is what fights it, and it is on by default:
  `source-item-ordering` defaults to `["enum", "impl", "module", "struct", "trait"]`, which demands alphabetical struct
  fields, enum variants and impl methods. That default must be narrowed to `["module", "trait"]`, because field order is
  the CDR wire order under D-05 and D-06, so obeying the lint would be an append-only API break. This is the one place
  where a default clippy setting could silently break the product.
- **The Kernel teaches the wrong logging style.** 28 of the 34 message-interpolated log calls are in
  `core/libs/app/service/src/runtime.rs` (`:423`, `:452`, `:535`, `:639`, `:802`, `:983` and others), while the Recorder
  is 46 structured against 1 interpolated. Developers and agents read the Kernel to learn the API, so it is the
  highest-leverage file to fix.
- **The import-group violations are one alphabetical block, not sloppiness.** 34 of 91 files put `blueos_*` and
  third-party crates in a single sorted block, and because `blueos_api` sorts before `bytes`, `clap`, `serde`, `tokio`
  and `tracing`, alphabetical order inside one block always produces the wrong group order
  (`core/libs/app/service/src/runtime.rs:23`, `core/libs/app/service/src/builder.rs:13`,
  `core/libs/logic/cqrs/src/lib.rs:58`). The fix is one blank line per file. Separately, 72 places split one crate root
  over several top-level `use` lines, costing 160 extra lines.
- **The code is consistently leaf-first and callee-last, which is the exact opposite of the rule.** 65 places declare a
  used type before the type that contains it (20 in `core/services/recorder/logic/policy/src/lib.rs`, 15 in
  `core/services/recorder/logic/library/src/lib.rs`), and 189 free-function pairs declare the callee after the caller,
  with not one file following caller-first. Adopting the rule is a real reversal of the dominant Rust idiom, not a
  cleanup, and without a `--fix` it is the rule that will get suppressed into irrelevance.
- **`#[cfg(test)]` placement is already inconsistent and the existing lint cannot see it.** 14 files put an inline
  `mod tests` at the bottom, while 3 declare `#[cfg(test)] mod tests;` near the top with items after it
  (`core/services/recorder/logic/library/src/lib.rs:9`, `core/libs/adapters/settings/src/lib.rs:20`,
  `core/services/example/logic/pump/src/lib.rs:31`). `clippy::items_after_test_module` fires for the inline form and not
  for the external one, verified by experiment.
- **650 undocumented public items in hand-written code, and 10 of 24 crate roots have no `//!`.** The worst are
  `core/services/recorder/logic/policy/src/lib.rs` (109) and `core/services/recorder/logic/library/src/lib.rs` (62);
  `RecorderCommand` (`policy/src/lib.rs:86`) has no doc on the enum and none on any variant, which is E7's complaint.
- **The no-abbreviation rule already passes, and the real naming weakness is different.** Zero hits for `Err(e)`,
  `Err(err)`, `cmd`, `ctx`, `tx`, `rx`, `idx`, and `clippy::min_ident_chars` reports 0 violations workspace-wide, so the
  rule should be locked in with a lint before it regresses rather than restated as advice. What does bite is naming by
  type: `core/services/recorder/app/src/lib.rs:286` and `:288` bind two different values to `bytes` in one closure, and
  no lint catches that.
- **Three architectural patterns are in the code and in no decision record:** the self-query command injection of E6
  (`core/services/recorder/app/src/inject.rs`), the `Drop`-finishes-the-MCAP-file RAII of `IoContext`
  (`core/services/recorder/app/src/lib.rs:94`, `:104`), and `Ros2ddsGate`
  (`core/services/recorder/logic/policy/src/ros2dds_gate.rs`). A fourth, the side-effecting State selector (`:258`),
  does not just go undocumented, it contradicts D-03's statement that a selector is a pure read. So "document every new
  pattern" is already broken three times and nothing checks it.
- **Answer, what a tool can enforce.** Lints cover re-exports (`clippy::pub_use`, 37 hand-written sites), glob imports
  (1), the kind-group half of the declaration order (99 violations in 46 files), the test-module position (inline form
  only), documented items (`missing_docs`), short names (0), shadowing, `#[allow]` reasons (12 bare, plus 21 where
  `#[expect]` would be correct), and the `no_std` discipline (`clippy::std_instead_of_core`, 54). Only a custom check
  can do the five import groups, hierarchical chaining, the use-graph and call order, structured logging, and the
  clone-before-`async move` rule; the same `syn`-based tool covers all five in roughly 150 to 250 lines, dropping
  intra-cycle edges with Tarjan, which is exactly the owner's "best effort on cycles" clause. Nothing can check KISS,
  intent grouping, naming by meaning, or "no new undocumented pattern". Unverified: the 65, 189 and 99 counts come from
  a column-0 regex prototype rather than real parsing, whose known weakness is matching type names inside doc comments.
- **Answer, the unsettled placements.** Constants and type aliases go right after the imports, because they are the
  vocabulary a reader needs first and never depend on local types, which is what the code already does
  (`core/libs/app/service/src/runtime.rs:31`). Trait `impl` blocks come before the inherent `impl` for a given type, so
  all contract information sits together under the `derive` attributes; the clippy lint cannot express this, so it stays
  guidance plus the custom check. `#[cfg(test)] mod tests` goes last, always, which also keeps it out of the way of the
  kind-group configuration, where `mod` belongs to the first group.
- **"No re-exports, ever" cannot be obeyed as written.** Four cases in the repository are required by other rules:
  `core/libs/adapters/logging/src/lib.rs:13` re-exports the `tracing` macros, which the owner's own log-macros rule
  explicitly permits; `core/libs/adapters/cli/src/lib.rs:6` re-exports `clap` derives that must be in scope at the use
  site; `core/libs/idl/src/lib.rs:18` is the only way to surface `include!`-ed `OUT_DIR` code (E9); and
  `core/services/example/logic/pump/src/lib.rs:18` to `:28` exists precisely because D-20 mandates one concept per file.
  The rule should be restated as what it protects: ban renaming re-exports (the one real violation,
  `core/services/recorder/logic/policy/src/lib.rs:15`, `:16`, which caused E7's two-names-per-type confusion), ban
  re-exporting another crate's domain types to spare a caller a dependency, and allow a crate-root or adapter facade
  behind `#![expect(clippy::pub_use, reason = "...")]`, so every new one argues for itself in a pull request.
- **Recommendation:** put the full rule text in a new `doc/architecture/rust-style.md`, keep the imperative checklist
  once inside marker comments, and mirror that block verbatim into `AGENTS.md` and a committed
  `.cursor/rules/rust-blueos.mdc` scoped with `globs: core/**/*.rs`, with `CLAUDE.md` pointing at `AGENTS.md` and a
  20-line drift check in `.hooks/lib/rust_checks.sh` that fails when the three copies disagree. That is the same
  generated-block-plus-check idiom the project already accepted for `core/interfaces/api.lock`. Make the teaching
  example the executable copy, passing every rule with zero `#[allow]`, because an agent copying the example inherits
  the style more reliably than from any prose. Support it with `[workspace.lints]`:

```toml
[workspace.lints.rust]
missing_docs = "deny"
unreachable_pub = "deny"

[workspace.lints.clippy]
allow_attributes = "deny"
allow_attributes_without_reason = "deny"
alloc_instead_of_core = "deny"
arbitrary_source_item_ordering = "deny"   # with the narrowed core/clippy.toml above
min_ident_chars = "deny"                  # 0 violations today; free insurance
pub_use = "deny"                          # with an #![expect] at the four facade roots
shadow_unrelated = "deny"
std_instead_of_core = "deny"
wildcard_imports = "deny"
```

  Roll out cheapest first: the four lints with 0 to 12 violations in one small pull request, then the import grouping
  (34 files, mechanical), then structured logging (34 calls, 28 of them in one Kernel file), then clone-before-spawn
  (12), then `unreachable_pub` and `std_instead_of_core`, then the declaration order with the kind groups first, and
  `missing_docs` last, starting at `warn` and denied per crate as each is documented. Open for the owner: whether the
  use-graph order is worth reversing 65 plus 189 sites, whether nightly rustfmt is acceptable for
  `imports_granularity = "Crate"` (the only tool that writes chained imports for you), and whether `missing_docs`
  extends to private items, where `clippy::missing_docs_in_private_items` adds roughly 340 more and the existing
  `AGENTS.md` rule ("no docstrings unless the function is non-obvious") says the opposite.

## P10. Newtypes and type-driven state machines

**Owner notes**

- Promote newtypes and type-driven design, including type-state state machines.

**Orchestrator notes**

- **One state is stored in two fields that can disagree.** `RecorderSnapshot` has `session_active: bool` and
  `session: Option<RecordingSession>` (`core/services/recorder/logic/policy/src/lib.rs:75`). `SessionFinished` clears
  `session` but leaves `session_active` set (`core/services/recorder/logic/policy/src/lib.rs:332` to `:334`). During a
  rotation the Snapshot says "active, with no session", and the `recording` State publishes that. One enum, for example
  `Idle`, `Opening`, `Active(ActiveRecording)`, `Finishing`, cannot hold that combination.
- **An outcome is stored as three fields with eight combinations, of which three are legal.**
  `RecordingOperationEvent` has `succeeded: bool`, `cancelled: bool`, and `error: String`
  (`core/services/recorder/logic/library/src/lib.rs:75`), and `OperationFinished` branches on the combinations. An enum
  `Succeeded`, `Cancelled`, `Failed { reason }` removes the branches.
- **Paths are validated again and again instead of parsed once.** `validate_relative_recording_path` is called at 7
  sites in the Recorder. A `RecordingPath` newtype, built once at the boundary, makes an unvalidated path impossible to
  pass into the Domain ("parse, don't validate").
- **Units travel as bare numbers:** `now_unix_seconds: i64`, `now_millis: u64`, `status_interval_hertz: f32`,
  `recording_time_ms: u32`, `video_status: u8`. Nothing stops seconds from being passed as milliseconds.
  `core::time::Duration` exists in `no_std`.
- **Timer ids are one untyped number space.** `CAPTURE_STATUS_TIMER` is `TimerId(1)`
  (`core/services/recorder/logic/policy/src/lib.rs:228`) and `RESCAN_TIMER` is `TimerId(200)`
  (`core/services/recorder/logic/library/src/lib.rs:30`). The two blocks share the same Kernel timer map; they avoid a
  collision only because someone picked 200.
- **Likely bug from the same cause (to verify):** all video streams share the one `CAPTURE_STATUS_TIMER`. Each
  `StartCapture` cancels it and re-arms it for its own stream
  (`core/services/recorder/logic/policy/src/lib.rs:424` onward), and the Kernel replaces a timer with the same id. With
  two cameras recording, only the last one gets status ticks. A timer key typed per stream would make this visible.
- **Required builder fields are checked at run time.** "service_info is required" and "app is required"
  (`core/libs/app/service/src/builder.rs:343`, `:364`) are runtime errors, printed before logging starts (P2). A
  type-state builder makes them compile errors.
- Related: IDL enums arrive as `u8` constants (E8), and `JobId(0)` means both "no job" and "the first job" (review L5).

**Deep-dive questions**

- Where does each tool fit? Type-state with generic parameters suits builders and resource handles (an MCAP file that
  is open versus finished). The Domain Snapshot must stay one `Clone` type that the Inbox owns, so its state machine
  should be an enum, not a generic type-state. Confirm this split and write it as a guideline.
- Which newtypes should the shared libraries provide (time, ids, validated paths), and which belong to each service?
- How do enums with data map to the IDL, which today has only constants (E8)?

**Deep-dive findings (R9)**

- Confirmed bug, by a failing test in a scratch copy: every video stream shares one `CAPTURE_STATUS_TIMER`
  (`TimerId(1)`, `core/services/recorder/logic/policy/src/lib.rs:228`), armed per stream (`:458`). The Kernel keys
  armed timers by `TimerId` alone (`core/libs/app/service/src/runtime.rs:313`, `:992`). With two cameras only the last
  one gets ticks. A broadcast `StartCapture` arms the same id twice in one Decision. `StopCapture` on one camera
  cancels the other camera's timer.
- Confirmed, and permanent rather than transient: `SessionFinished` leaves `session_active` true. Rotation emits
  `FinishSession` and `OpenSession` as two tasks spawned concurrently (`runtime.rs:967`). A late `SessionFinished`
  wipes the new session. The Recorder then publishes "active, empty file" and drops every `SessionBytesWritten`.
- New bug: `CaptureStatusTick` reschedules itself with the same `now_millis` (`policy/src/lib.rs:400`), so
  `recording_time_ms` reports 0 forever.
- New, by reading only (needs a device to confirm): the same rotation race can make `FinishSession` close the file that
  `OpenSession` just created (`core/services/recorder/app/src/lib.rs:329`, `:333`).
- Twenty spots where illegal states are representable. The teaching example is one of them: `self_test_phase: u8`,
  `self_test_active: bool` and `self_test_root_job: Option<JobId>` must agree in four places.
- Rule: an enum for state you store, type-state for state you move. The Snapshot must stay one `Clone` enum, because it
  is a field of `App<D>` and `handle_command` takes `&mut Snapshot` in any state. Type-state fits builders and resource
  handles; R9 compiled a type-state builder where a missing required field is a compile error.
- R9 drafted an eight-rule guideline text for `rust-style.md` (raw report, P10 section).

## P11. Borrow before cloning

**Owner notes**

- Always prefer borrowing, references, and `Cow`, and well-scoped containers, over copying and cloning.

**Orchestrator notes**

- `.clone()` appears 76 times in the Recorder, 41 times in the Kernel, 27 times in comms, and 9 times in the example.
  `.to_vec()`, `.to_string()`, and `.to_owned()` appear 61, 20, 57, and 5 times in the same places.
- Copies on hot paths:
  - The tap clones the whole `TapPolicy`, including a `BTreeSet<String>` of video topics, for **every** sample on `**`
    (`core/services/recorder/app/src/tap.rs:416`). Borrowing the watch value for the duration of the check is enough.
  - The tap copies every raw MAVLink payload into a `Vec` before parsing it (`core/services/recorder/app/src/tap.rs:419`).
  - The Kernel clones the whole App for every `Io` and every `Persist` Effect
    (`core/libs/app/service/src/runtime.rs:973`, `:1029`; review M9). The Recorder's `.io` closure then reads one field
    of it.
  - Payloads are copied on publish and on receive, against D-09's zero-copy promise (review L8).
- Some clones are a symptom of design, not of style: the `Session` and channel clones before each detached task (E5)
  exist because nothing owns the tasks, and the JSON copies in `inject.rs` exist because there is no in-process sender
  (E6).

**Deep-dive questions**

- Which clones are cheap and intended (`Arc`, `Session` handles, `Bytes`), and which copy real data? Write the rule so
  that it separates the two.
- Which APIs force a clone today (closures that take owned values, the `.io` signature that takes `App<D>` by value,
  `String` fields where `&str` or `Cow<'_, str>` would do), and what would the borrowing version look like?
- Can a lint catch part of it (clippy `redundant_clone`, `needless_pass_by_value`, `implicit_clone`; to confirm)?

**Deep-dive findings (R9)**

- The worst hot-path copy was not in the notes. The tap builds a full `ChannelDescriptor` for every sample
  (`core/services/recorder/app/src/tap.rs:443`), copying the whole schema text, and the writer drops it when the channel
  is already known (`writer.rs:213`). For non-embedded schemas that is a blocking `fs::read_to_string` per sample. For
  JSON topics it is a parse and reserialize per sample.
- The App clone per `Io` Effect (`runtime.rs:973`) copies the whole recording index, and the consumer reads one field.
  R9 compiled the fix: `.io` takes `&App<D>`. This works with a `'static` spawned task, because only the `async move`
  block has to be `'static`; the closure copies the field it needs before the block.
- Clippy with five lints over the workspace: 28 sites. `clone_on_ref_ptr` 15, `needless_pass_by_value` 10,
  `redundant_clone` 3, `implicit_clone` and `unnecessary_to_owned` 0. All 15 `clone_on_ref_ptr` hits are correct cheap
  clones. No lint found any of the seven real hot-path copies. Enable the lints for readability; they do not replace
  review for this class of problem.
- Rule: a handle clone bumps a reference count and is free; a data copy allocates and needs a reason. The reader must be
  able to tell them apart at the call site (`Arc::clone(&session)` for handles). R9 drafted a seven-rule guideline text.

## P12. Reuse mature projects before building

**Owner notes**

- We are building a valuable boilerplate, but most of the mechanisms we need may already exist, and some may be
  battle-tested. Our code is open source, and we have no shame in reusing and depending on other projects that are
  mature enough.
- None of these points is new: event-driven services, CQRS, jobs, DTOs, dependency injection, an IDL, and Rust, Python
  and TypeScript generation from `.msg` files. Someone must have built them already.
- In other languages, established frameworks do all of this with about 2k lines on top of their core.

**Orchestrator notes**

- The POC writes all of these mechanisms itself: the Kernel and Inbox loop (`core/libs/app/service`), the CQRS traits
  (`core/libs/logic/cqrs`), the job tree, the timers, the settings store, the `.msg` parser and code generators, and
  the calendar code (E10).
- The rule that a new dependency must be mature, `no_std` where it enters logic crates, and have minimal features (P9)
  still applies. The question is which mechanisms clear that bar.

**Deep-dive questions**

- For each mechanism, which mature Rust crates exist (maturity, maintenance, license, `no_std`, async runtime), and
  would adopting one shrink our code or only move it?
- Is there a whole framework (actors, CQRS, event sourcing, ROS 2 style IDL tooling, Zenoh-native frameworks) that
  covers most of it, so that BlueOS writes only the 2k-line extension?
- Which existing tools generate Rust, Python and TypeScript from one IDL, and can they read `.msg` or replace it?
- What must stay ours, and why?

**Deep-dive findings (R11)**

- No Rust framework covers "an event-driven service with a versioned IDL over a pub/sub bus". The 2k-line frameworks
  in other languages are request-response web frameworks; Rust has those too (axum, tower). Zenoh-Flow is dead (last
  release 2023-09). dora-rs and Copper (`cu29`) replace the process model that D-01, D-02 and D-10 fixed. Eclipse
  uProtocol (`up-rust`) replaces the API contract itself. The Zenoh ROS 2 crate `hiroz` is 0.2.0, so D-24's deferral
  holds; `roslibrust` 0.26 already has a `hiroz` feature, which is the cheap way to try it later.
- Adopt (about 600 of our lines go):
  - `tokio-util` `TaskTracker` and `CancellationToken` for the task registry and shutdown (E5). Compiled: it survives a
    simulated adapter panic and drains cleanly, and gives the restart that P3 asks for.
  - `tokio-util` `DelayQueue` for `Effect::Schedule`. Keyed cancel also removes the shared-timer bug class.
  - `backon` for retry and backoff, which we wrote twice.
  - `zenoh-keyexpr` for key matching in the test broker.
  - `roslibrust_codegen` for `.msg` parsing, replacing `ros2_message` 0.0.5 (one owner, stale since 2024, 81 downloads
    per quarter) at the root of the IDL. Compiled on the real 34-file tree; it needs a `package.xml` per package.
  - Python: generated dataclasses plus `pycdr2`, about 80 generator lines instead of about 250.
- Keep ours, with the reason:
  - Kernel and Inbox loop: `ractor` and `kameo` force `async` handlers, which breaks sans-IO logic (D-03).
  - CQRS traits: every candidate (`cqrs-es` and others) is event sourcing with a store, `async`, and `std`.
  - Jobs tree: every jobs crate (`apalis` and others) is a durable worker queue, not a flow shown in the UI.
  - Session state machine: `statig` compiles `no_std`, but a plain enum did the same four tests in 77 lines against 126.
  - Settings: D-11 needs byte compatibility with Python `appdirs`; `config` and `figment` do not give it.
  - DI: `builder.context(T)` stays (E13).
  - CDR codec: `cdr-encoding` matches our test vectors byte for byte, but does not build `no_std`.
  - Rust and TypeScript emission: `roslibrust_codegen` output is `std`-only and tied to its runtime; `ts-rs`, `typeshare`
    and `specta` read Rust types, not `.msg`.
- Size: about 15,000 Rust lines remain of 15,596. About 4,000 are platform mechanism and about 1,300 are the IDL; the
  rest is the Recorder's domain and tests. The achievable form of the 2k target is per service: with the E12 budget, a
  small service is 150 to 300 lines and a Recorder-sized one about 2,000.

---

# Findings

## E1. The multicall entry point

**Owner notes**

- `core/app/blueos/src/main.rs:11` calls `run` twice, and the reason is not clear.
- The feature gate should be on the `match` arms in `run` (`core/app/blueos/src/main.rs:25`), not on duplicated
  `dispatch_*` function pairs.

**Orchestrator notes**

- The two calls try two invocation styles: a symlink name in `argv[0]` (`recorder ...`), then a subcommand
  (`blueos recorder ...`). The file does not say so.
- If a service is compiled out and is started through its symlink, the first call prints "rebuild with --features",
  then the second call treats the user's first argument as a service name.
- Adding one service touches four places in this file (the usage `eprint!`, a `match` arm, two `dispatch_*`
  functions), plus a feature in `core/app/blueos/Cargo.toml`.
- The two services have different entry signatures: `blueos_example::run` returns `()` and always exits with 0
  (review L10), while `blueos_recorder::run` returns `ExitCode` (`core/app/blueos/src/main.rs:33`,
  `core/app/blueos/src/main.rs:46`).

**Deep-dive questions**

- What is the smallest `main.rs` that keeps both invocation styles, and where should the per-service entry signature
  be defined so that every service has the same one?

**Deep-dive findings (R1)**

- **Correction:** adding a service takes six edits, not five: four in `main.rs`, plus a feature line **and** an
  optional dependency line in `core/app/blueos/Cargo.toml`.
- **Root cause of the double call:** the "not compiled in" stubs return `false`, the same value as "unknown name"
  (`core/app/blueos/src/main.rs:28`, `:42`, `:56`). `main` cannot tell them apart, so it tries the second style and
  then also prints the usage.
- **The default build has no example.** `default = ["recorder"]` (`core/app/blueos/Cargo.toml:13`). The teaching
  example (D-20) needs `--features example`.
- **`blueos --help` and `--version` exit with code 2**, because `main` matches arguments by hand.
- **Exit codes are flattened differently per service:** a Recorder failure becomes exit 1, an example failure exit 0.
- **Recommendation:** resolve the name once, keep a list of known names that is never feature-gated, put the
  `#[cfg]` on the `match` arms, and use the `Service` trait from P1 as the one entry signature:

```rust
const KNOWN: &[&str] = &["example", "recorder"];

fn main() -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().collect();
    let Some((name, arguments)) = entry::resolve(&arguments) else {
        return entry::usage(KNOWN);
    };
    match name {
        #[cfg(feature = "example")]
        "example" => entry::run::<blueos_example::service::Example>(arguments),
        #[cfg(feature = "recorder")]
        "recorder" => entry::run::<blueos_recorder::service::Recorder>(arguments),
        other => entry::unknown_or_missing(other, KNOWN),
    }
}
```

  A known name that reaches the last arm can only be compiled out, so it gets one clear message. `--help`, `-h`, and
  `--version` are reserved first arguments that exit with 0. A self-registering inventory (`linkme`, `inventory`) was
  rejected: a new dependency and pattern, for two services.

## E2. The service entry and bootstrap

**Owner notes**

- `run` (`core/services/recorder/app/src/lib.rs:128`) will be the same in every service: build a tokio runtime, run,
  map the result to an exit code. It should be part of the service interface.
- `lib.rs` is a bootstrap. That is fine, but it should be named as such (bootstrap or build), and have a fixed place in
  the service folder layout.
- From `lib.rs`, a reader cannot tell what the service is or what it does.

**Orchestrator notes**

- `run_async` is one function of about 340 lines (`core/services/recorder/app/src/lib.rs:152` to `:495`). It holds
  all the wiring of the service.
- The module comment (`core/services/recorder/app/src/lib.rs:1` to `:12`) talks about D-03, D-09, and shared memory.
  It does not say what the Recorder does for a user.
- The Kernel can open the Zenoh session itself (`core/libs/app/service/src/builder.rs:330` to `:335`), but the
  Recorder opens its own (`core/services/recorder/app/src/lib.rs:160`) because its detached tasks need clones. The
  builder does not cover the Recorder's needs, so the Recorder works around it.

**Deep-dive questions**

- Which parts of `run` and `run_async` are the same for every service, and what would a Kernel-owned entry point look
  like?
- What should the folder layout of a service be so that "what it does" (Domain) and "how it is wired" (bootstrap) are
  found in obvious places?

**Deep-dive findings (R1)**

- **Correction:** the module comment is not empty. It names D-15 and the recording policy. It is written for an
  architecture reviewer, though. The user-facing sentence ("records the Zenoh backbone to MCAP files") is only in
  `core/services/recorder/README.md:3`.
- **The example's integration test does not test the shipped service.** It rebuilds a `ServiceBuilder` by hand
  (`core/services/example/app/tests/kernel_integration.rs:25`), copies the `jobs` closure, and registers one of the three
  Commands. A new endpoint added to `lib.rs` never reaches the test. Root cause: there is no "build only" function to
  call. This is the strongest argument for splitting a pure `build()` from `run()`.
- **`lib.rs` holds five roles:** the crate root, the entry, the 344-line bootstrap, the settings DTO, and the IDL
  conversions.
- **Steps that every service repeats** (the Kernel entry should own them): convert the arguments, parse the CLI, build
  the runtime, map the verbosity, `block_on`, map the error to an exit code, open the session, build `ServiceInfo`, load
  the settings, `builder.run()`.
- **Recommended layout inside `app/src/`** (no change to D-02 or to the folder check):

```text
core/services/<name>/
  logic/<block>/      what it does: Domain, Snapshot, Commands, Queries, Events, Jobs (no_std)
  adapters/<thing>/   how it touches the world: devices, files, protocols
  app/
    README.md         one page for a user: what the service does, which keys it serves
    src/
      lib.rs          module list only, under 20 lines
      service.rs      impl Service: NAME, VERSION, Arguments, build()
      cli.rs          the service's extra clap::Args
      endpoints.rs    every .command / .query / .state / .event / .io registration
      messages.rs     every From / Into between an IDL Message and a Domain type (E8, E13)
      tasks.rs        long-running tasks, declared to the Kernel (E5)
      settings.rs     the SettingsSchema and its two mappings
```

  To add an endpoint, a developer edits `endpoints.rs`, plus `messages.rs` if a conversion is needed, and nothing
  else. Each registration group is a `register(builder) -> builder` function, so `endpoints.rs` can later be split into
  one file per endpoint without other changes.

## E3. The CLI

**Owner notes**

- `parse_cli` (`core/services/recorder/app/src/cli.rs:36`) and `zenoh_key_overrides`
  (`core/services/recorder/app/src/cli.rs:66`) will be repeated in every service.
- There is no Zenoh configuration on the CLI.
- `verbose` and the Zenoh configuration should be common arguments that every service has. Each service extends them.

**Orchestrator notes**

- A common struct already exists: `blueos_cli::Common` (`core/libs/adapters/cli/src/lib.rs:15`) has `--config` and a
  counted `-v`. The example uses it (`core/services/example/app/src/lib.rs:33`). The Recorder does not: it defines
  `verbose: bool` (`core/services/recorder/app/src/cli.rs:15`) and maps it to 0 or 1
  (`core/services/recorder/app/src/lib.rs:154`), so `-vv` is not possible. Two services, two CLI conventions.
- There are three CLI mechanisms: `blueos_cli::Common`, `ServiceBuilder::cli(Argv)` with `.cli_command`
  (`core/libs/app/service/src/builder.rs:126`, `:132`), and a private `clap::Parser` per service. The Recorder uses
  only the last one.
- `--zkey` is accepted and silently ignored: `zenoh_key_overrides` is dead code behind `#[allow(dead_code)]`
  (`core/services/recorder/app/src/cli.rs:65`).
- Zenoh can be configured only through the `ZENOH_CONFIG` environment variable
  (`core/libs/adapters/comms/zenoh/src/config.rs:4`).
- `parse_cli` expands environment variables in every argument with `shellexpand`
  (`core/services/recorder/app/src/cli.rs:40`). The reason is not written down.
- `recorder_directory` and `schema_directory` are one-line wrappers, each used once.

**Deep-dive questions**

- Design the common CLI: which arguments every service must have (verbosity, Zenoh config, settings folder, ...), how
  a service extends it, and who parses it (the Kernel or the service).

**Deep-dive findings (R1)**

- **Addition:** the Recorder's crate does not even depend on `blueos-cli`; it uses `clap` directly
  (`core/services/recorder/app/Cargo.toml:26`).
- **The Kernel's own CLI mechanism is dead.** No service calls `.cli_command`, and the only `.cli(...)` call passes
  an empty `Argv` (`core/services/example/app/src/lib.rs:65`). It still costs two builder fields, two of the 18 `run`
  arguments, and a branch at start (`core/libs/app/service/src/runtime.rs:526`).
- **No service can reach a remote Zenoh router.** `Endpoint::Remote` exists
  (`core/libs/adapters/comms/driver/src/lib.rs:21`) but only tests use it. `ZENOH_CONFIG` replaces the whole config,
  which also silently drops the shared-memory transport that D-09 needs.
- **No service can be told where its settings live.** `--config` is used as a folder, but its help says "JSON5 config
  file". The Recorder has no such argument.
- **`shellexpand` solves a problem the shipped launch line does not have** (`core/start-blueos-core:119` contains no
  `$`). A literal `$` in a path is silently rewritten. `shellexpand` and `tempfile` are pinned in the crate
  (`core/services/recorder/app/Cargo.toml:32`, `:42`), not in the workspace.
- **Recommended common CLI**, parsed by the Kernel; each service adds a `type Arguments: clap::Args`:

```rust
#[derive(clap::Args, Clone, Debug)]
pub struct Common {
    /// Raise the log level: -v for debug, -vv for trace.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    /// Folder that holds <service>/settings-<N>.json. Default: the user config folder.
    #[arg(long, value_name = "DIR")]
    pub settings_path: Option<PathBuf>,
    /// Zenoh router to connect to.
    #[arg(long, value_name = "ENDPOINT", default_value = "tcp/127.0.0.1:7447")]
    pub zenoh_endpoint: String,
    /// Full Zenoh JSON5 config file. Replaces --zenoh-endpoint.
    #[arg(long, value_name = "FILE", env = "ZENOH_CONFIG")]
    pub zenoh_config: Option<PathBuf>,
    /// Override one Zenoh config key, for example --zenoh-set transport/shared_memory/enabled=false
    #[arg(long = "zenoh-set", value_name = "PATH=JSON5")]
    pub zenoh_overrides: Vec<String>,
}
```

  `--config` becomes `--settings-path`. `env = "ZENOH_CONFIG"` keeps the variable and shows it in `--help`.
  `--zenoh-set` replaces the dead `--zkey`; it assumes zenoh 1.9.0 can set one key path (unverified). Service
  arguments use `PathBuf`, which deletes the one-line wrappers. Then delete `.cli`, `.cli_command`, and `Argv`, and drop
  `shellexpand`.

## E4. Builder size and API asymmetry

**Owner notes**

- The builder chain is massive (`core/services/recorder/app/src/lib.rs:202` to `:407`). The Command registrations are
  easy to understand.
- The `.io` registration is clearer, because its `match` is explicit.
- The Domain has `Command`, `Event`, `Query`, and `IoRequest`, and each one is wired with a different strategy. There
  is no common shape. Each Command needs its own `.command()` call, while `.io()` needs one call for all requests.
- `.settings` cannot be chained.
- Zenoh is a special path. Should it be an IO like any other, but required or built in?

**Orchestrator notes**

- There are six registration strategies:

| Kind | Strategy | Example |
|---|---|---|
| Command | one call per name, closure Message to Command | `core/services/recorder/app/src/lib.rs:225` |
| Opaque Command | one call per name, closure bytes to Command | `core/services/recorder/app/src/lib.rs:257` |
| State | one call per name, selector App to Message | `core/services/recorder/app/src/lib.rs:268` |
| Event | one call per name, a filter closure and an encode closure | `core/services/recorder/app/src/lib.rs:271` |
| IO query | one call per name, async closure, no App | `core/services/recorder/app/src/lib.rs:281` |
| IO | one call for all requests, async closure with a `match` | `core/services/recorder/app/src/lib.rs:293` |

- The Event registration needs two closures that match the same variant twice. The encoder has an unreachable
  `let ... else { return Err("event filter mismatch") }` (`core/services/recorder/app/src/lib.rs:271` to `:280`).
- `.settings` returns `Result` (`core/libs/app/service/src/builder.rs:143`) because it creates a `SettingsManager`
  (disk access) at registration time. Registration and IO are mixed.
- Boilerplate that the Kernel could default:
  - `service_info` repeats the service name and `CARGO_PKG_VERSION`, and passes empty `endpoints` that the Kernel
    fills later (`core/services/recorder/app/src/lib.rs:204`).
  - `.jobs(|_| JobList { jobs: Vec::new() })` for a service without jobs (`core/services/recorder/app/src/lib.rs:224`).
  - `.status(...)` that always says `STATUS_READY` (`core/services/recorder/app/src/lib.rs:215`).
- `.io` returns `Result<Command, Command>` (`core/libs/app/service/src/builder.rs:321`). Every IO must produce a
  Command, so the Recorder has no-op Commands `Ack` and `IoFailed`
  (`core/services/recorder/logic/policy/src/lib.rs:113`, `:342`). `IoFailed` carries nothing, so the Domain cannot tell
  which IO failed. It is returned from seven places in the `.io` closure.
- The `Domain` trait has seven associated types and `io_from_job` (`core/libs/logic/cqrs/src/lib.rs:122` to `:146`).
  A Domain without queries or jobs must still write placeholders (see E7).
- `run_with_session` passes 18 positional arguments to `runtime::run` (`core/libs/app/service/src/builder.rs:366` to
  `:385`). This is internal, but a reader who follows `builder.run()` lands there.

**Deep-dive questions**

- Is one shape possible for all four kinds (Command, Query, Event, IO)? For example, one explicit `match` per
  direction, or routing generated from the IDL. Compare at least two options, with their cost.
- Should Zenoh be an Adapter like device IO, with the Kernel providing a default one?

**Deep-dive findings (R2)**

- **Corrections:** `IoFailed` is returned from nine places, not seven; add
  `core/services/recorder/app/src/library_io.rs:86` and `:90` to the seven in the `.io` closure. `Ack` has four sites
  (`core/services/recorder/app/src/lib.rs:356`, `:374`, `:392`, `core/services/recorder/app/src/library_io.rs:231`). The
  constant `.status` is true of the example as well (`core/services/example/app/src/lib.rs:141`), so no service in the
  repository has a status that carries information and a Kernel default would lose nothing. The note on `.io` also
  understates the problem: the Kernel throws the `Result` away.
- **The `Result<Command, Command>` of `.io` is discarded by the Kernel.**
  `core/libs/app/service/src/runtime.rs:978` reads `Ok(command) | Err(command) => command`, so both arms produce the
  same value on the same code path. Root cause: the Domain's only failure channel is another Command, so `Ok` and `Err`
  collapse to one type and the Kernel has nothing left to distinguish. The signature asks every IO author to make a
  choice with zero effect, nine times in the Recorder, and a junior reasonably concludes that `Err` means the Kernel
  will handle the failure.
- **Failure is expressible in some registrations and not in their neighbours.** `.state`, `.status` and `.jobs` take
  infallible closures (`core/libs/app/service/src/builder.rs:261`, `:296`, `:307`) while `.command`, `.query`,
  `.io_query` and `.event` take fallible ones (`:165`, `:215`, `:235`, `:277`). The infallible ones still fail, inside
  `message_to_payload` (`:390`), and that error kills the Inbox loop (`core/libs/app/service/src/runtime.rs:1054`,
  review M2). Root cause: each method was added with whatever signature fit that day, and there is no rule such as
  "every registration returns `Result`".
- **`message_to_payload` exists twice, identical** (`core/libs/app/service/src/builder.rs:390` and
  `core/libs/app/service/src/runtime.rs:1175`), because there is no single Message-to-wire seam.
- **The Kernel's own minimal example is 103 lines for one Command** (`core/libs/app/service/src/lib.rs:19` to `:122`),
  with no query, no event, no real state, and the empty-jobs boilerplate still present (`:108`). This is the first thing
  a junior reads from `cargo doc`, so the floor for "a service" is already a page.
- **`command_opaque` silently costs discoverability.** It is the only one of the three `.command*` variants that sets
  `request_schema: String::new()` (`core/libs/app/service/src/builder.rs:208`), which the Kernel publishes as an
  endpoint with no schema (`core/libs/app/service/src/runtime.rs:243`), so the D-24 inspector cannot build a form for
  it. The decode strategy is encoded in the method name instead of in the request type, and the `ServiceInfo`
  consequence is invisible at the call site.
- **Zenoh is not an adapter; `Session` is a closed enum with 14 hand-written match arms.** `enum Backend`
  (`core/libs/adapters/comms/src/lib.rs:55`) has two `cfg` variants and seven `CommsBackend` methods, each a two-arm
  match (`:62` to `:138`), even though the trait already exists and `Backend` implements it (`:63`). The enum was chosen
  to avoid a `dyn` indirection and then wrapped in `Arc<Backend>` anyway (`:141`), so the indirection is paid
  regardless.
- **Queries are addressed by registration index, Commands by key string** (`core/libs/app/service/src/runtime.rs:498`,
  `:682` against `:518`), which makes registration order a load-bearing, undocumented contract.
- **Answer, one shape for all four kinds.** Four options, measured against the Recorder's 199-line chain with 21
  registration calls (`core/services/recorder/app/src/lib.rs:202` to `:400`) plus the detached `.settings` at `:402` and
  `.on_start` at `:488`.

| Option | Chain lines | Calls | New concepts | Keeps `ServiceInfo` | Macro needed |
|---|---|---|---|---|---|
| Today | 199 plus 2 detached | 21 | 6 strategies | yes | no |
| 1. One `match` per direction | ~170 | 4 | 1 | no, needs a second list | no |
| 2. Routing generated from the IDL | 1 | 1 | 1 | yes, generated | yes, in the codegen |
| 3. Extractors, as in axum | ~25 | 21 | 3 | yes | yes, per arity |
| 4. One endpoint trait per kind | ~170 | 21 | 5, symmetrical | yes | no |

- **Answer, splitting the `Domain` trait: do it, but change `Decision` first.** `Decision<D: Domain>`
  (`core/libs/logic/cqrs/src/lib.rs:65`) uses only `D::Event`, `D::Command` and `D::IoRequest`, while `Effect` is
  already generic over `<Command, IoRequest>` (`:100`). Making `Decision` generic over the three types deletes
  `LibraryDomain`'s entire `Domain` impl (`core/services/recorder/logic/library/src/lib.rs:168` to `:196`), shrinks
  `map_library_decision`, and removes the behavioural divergence where the trait impl passes `None` as the active
  session. Then split `Domain` into `Domain` plus `Queries` plus `JobDomain`, so `.query` requires `D: Queries` and the
  job machinery requires `D: JobDomain`; associated type defaults are unstable in Rust, so the split is the only way.
- **Answer, `.io`'s return type:** `Result<Option<Command>, IoError>`, with the Kernel turning `Err` into
  `Domain::io_failed(request, error) -> Command`. It deletes `Ack` (4 sites) and `IoFailed` (9 sites), gives the Domain
  the one thing `IoFailed` cannot carry today, which is *which* request failed and why, and combined with awaiting the
  IO task it also fixes review M10. Rejected: keeping `Result<Command, Command>` and making the Kernel use it, which
  still leaves the service inventing `Ack`; and `Option<Command>`, which deletes `Ack` but leaves failure with no
  channel.
- **Answer, `.settings`:** store the description and build the `SettingsManager` inside `run_with_session`, next to
  `Session::open` (`core/libs/app/service/src/builder.rs:334`), where a failure already returns `Err`. `.settings`
  becomes `-> Self`, the Recorder's three statements merge back into one expression, and this is exactly where review
  M4's "load once, hand the value to the app" fix belongs, so the two changes are one change.
- **Answer, Kernel defaults:** delete `.jobs` and let the Kernel derive `JobList` from `application.jobs`, which it
  already reads for the ack (`core/libs/app/service/src/runtime.rs:1163`) and which also fixes the quiet contradiction
  where an absent `.jobs` means no state at all (`:372`) against D-12's promise of a free `jobs` state; default
  `.status` to `STATUS_READY` and keep the method as a detail override; and replace `service_info` with a small
  `ServiceDescriptor` trait carrying `NAME`, `VERSION` and `CAPABILITIES` as associated consts in the service crate,
  since `version` cannot be defaulted (P1). That turns 21 lines of the Recorder into a 5-line `impl`.
- **Answer, Zenoh:** do not model it as an `Effect::Io`-style adapter. It is the Kernel's spine, used for liveliness,
  logging, four kinds of State, the `info` queryable, every adapter and every event publish
  (`core/libs/app/service/src/runtime.rs:357` to `:1086`), and `Effect::Io` results come back through the Inbox one at a
  time while State publishing must not. Do replace `enum Backend` with `Arc<dyn CommsBackend>`, which deletes the 14
  match arms, lets the Kernel default to Zenoh while tests pass any backend, and makes a third transport an added file.
- **Recommendation:** six independent steps in order. `Decision` over three types; split `Domain`; `.io` returns
  `Result<Option<Command>, IoError>`; `.settings` returns `Self`; Kernel defaults plus `ServiceDescriptor`; then one
  registration shape, which should be Option 2 as the spine with Option 4 as the hand-written fallback. Option 2 is the
  only one where the endpoint list and the wiring are the same artifact, which makes `ServiceInfo`, the inspector and
  the frontend consistent by construction; Option 4 covers what the IDL cannot describe, with the best compile errors
  and no new runtime concept. Rejected: Option 1, because names and schemas move inside a closure body and D-24
  endpoint discovery breaks; Option 3, which has the best ergonomics but buys them with a macro, a new framework
  concept and compile errors bad enough that axum needs `#[diagnostic::on_unimplemented]` to paper over them.

## E5. Hidden wiring: channels, tasks, lifecycle

**Owner notes**

- On a first top-down read, it is not clear what the senders, the contexts, and the policy are for.
- The `tap_*` clones (`core/services/recorder/app/src/lib.rs:408` to `:411`) should be inside a block around the
  spawn, before `async move`. The repository should have a Rust rule for this.
- The `tokio::spawn` calls look like service tasks. They should be declared as such, and their handles should be stored
  in the service, so that they can be inspected and have a clear lifecycle (RAII).

**Orchestrator notes**

- Before the builder, `run_async` creates four channels (`core/services/recorder/app/src/lib.rs:163`, `:170`, `:172`,
  `:190`) and an `IoContext` with two `Mutex` fields and a watch sender (`core/services/recorder/app/src/lib.rs:94`).
  The `Drop` of `IoContext` finishes the MCAP file (`core/services/recorder/app/src/lib.rs:104`). None of these has a
  comment that says its role.
- The data plane gets its filter through a **side effect inside a State selector**: the `recording` selector sends the
  `TapPolicy` on a watch channel every time States are published (`core/services/recorder/app/src/lib.rs:258` to
  `:266`). A selector should be a pure read. Removing or renaming that State would silently break recording gating.
- The initial `TapPolicy` is a literal (`core/services/recorder/app/src/lib.rs:163` to `:168`) that duplicates
  `TapPolicy::from_snapshot`.
- There are three detached `tokio::spawn` calls in `lib.rs` (`core/services/recorder/app/src/lib.rs:413`, `:436`,
  `:482`), and one per liveliness query in `core/services/recorder/app/src/tap.rs:367`. The handles are dropped. The
  Kernel does not know about them: they are not stopped on shutdown, and their failures are only logged.
- When `run_data_plane` returns, the supervisor logs and then waits for the next writer change
  (`core/services/recorder/app/src/lib.rs:413` to `:433`). Recording stops silently until the next session rotation.

**Deep-dive questions**

- Draw the real data flow of the Recorder: every task, channel, and who sends to whom.
- Propose a "service task" concept owned by the Kernel (declaration, handles, shutdown, restart policy, errors).
- Write the Rust rule for clones before `async move`, with a before and after example.

**Deep-dive findings (R3)**

- **Corrections:** the four channels and the `IoContext` are only what `run_async` creates. A running Recorder has 9
  message channels and 7 mutexes; the others are the Inbox (256) and persist (64) queues
  (`core/libs/app/service/src/runtime.rs:437`, `:439`), the liveliness get results (32,
  `core/services/recorder/app/src/tap.rs:94`), the MCAP write queue (4096,
  `core/services/recorder/adapters/mcap/src/writer.rs:128`) and the log records (1024,
  `core/libs/adapters/logging/src/zenoh_layer.rs:85`). The `Drop` of `IoContext` gives up silently on a poisoned lock
  (`core/services/recorder/app/src/lib.rs:106`). The initial `TapPolicy` literal is dead the moment the first
  `publish_all_states` runs (`core/libs/app/service/src/runtime.rs:496`). And "recording stops silently until the next
  session rotation" is wrong in a way that makes it worse: a rotation does not restart the tap either.
- **A session rotation never reaches the data plane (high).** The supervisor awaits `tap::run_data_plane` inside its own
  loop (`core/services/recorder/app/src/lib.rs:420`), and `run_data_plane` returns only when the `**` subscription ends
  (`core/services/recorder/app/src/tap.rs:223`). After a rotation the tap still holds the old `Arc<McapWriterHandle>`,
  whose thread has exited, so `write_message` returns `Disconnected`
  (`core/services/recorder/adapters/mcap/src/writer.rs:252`) and the tap logs once per sample forever
  (`core/services/recorder/app/src/tap.rs:275`) while the new file gets nothing and the published `recording` state
  looks perfect. Root cause: the writer handle is passed by value into a long-running loop while the change
  notification is a `watch` the caller can only poll between loops. Not reproduced on a device; the D-23 verification
  list covers SIGINT but not a rotation.
- **`Effect::Io` effects are unordered, and the Recorder's rotation depends on their order (high).** The Kernel spawns
  one independent task per effect (`core/libs/app/service/src/runtime.rs:967`) while the Domain returns
  `[FinishSession, OpenSession]` for a rotation (`core/services/recorder/logic/policy/src/lib.rs:253`). The two race on
  `IoContext.session` (`core/services/recorder/app/src/lib.rs:329` writes, `:333` takes), and if `OpenSession` wins then
  `FinishSession` closes the new file while the Snapshot reports it as active. Root cause: the Domain expresses a
  sequence as a `Vec<Effect>` and the Kernel treats the vector as a set; nothing in the `Effect` vocabulary can say
  "after", and the documented tool for sequencing is the job graph, which the Recorder does not use. Not reproduced on
  a device.
- **The tap gate updates only on the Applied path**, because the selector that sends the `TapPolicy` runs inside
  `publish_all_states` (`core/libs/app/service/src/runtime.rs:623`) and the error path at `:638` publishes nothing. So
  review H2 is, for the Recorder, also "the Snapshot says recording and the data plane says not recording".
- **Three sources of truth for the current writer:** `RecorderSnapshot.session`
  (`core/services/recorder/logic/policy/src/lib.rs:324`), `IoContext.session`
  (`core/services/recorder/app/src/lib.rs:97`) and the `writer_watch` value (`:98`). The two findings above are both
  disagreements between two of the three, nothing reconciles them, and no test can, because only the Snapshot is
  observable from outside. Root cause: the data plane needs a live handle, which cannot live in a `no_std` Snapshot, so
  a parallel ownership chain grew next to it with no stated invariant.
- **MAVLink facts are edge-triggered and lossy, so one dropped fact freezes the armed gate.** The adapter emits
  `ArmedChanged` only on a change (`core/services/recorder/adapters/mavlink/src/lib.rs:105`) and the tap drops facts
  when the 256-slot channel is full with a `warn!` only (`core/services/recorder/app/src/tap.rs:425`). Losing one arm
  transition means either no MAVLink for the whole dive or MAVLink recorded while disarmed, until the next transition.
  Root cause: a level signal is carried as edges over a lossy channel, and the consumer then blocks on a 2 s network
  round trip per fact (`core/services/recorder/app/src/lib.rs:466`, E6), which is what makes the drop reachable.
- **Nothing stops the background tasks at shutdown, and the MCAP file is finished under them.** The Kernel's drain
  covers in-flight `Effect::Io` only (`core/libs/app/service/src/runtime.rs:557`), so the three Recorder tasks end only
  when the tokio runtime is dropped (`core/services/recorder/app/src/lib.rs:143`), after `IoContext` has already
  finished the file. Samples in that window are lost with an error log, and no test can assert "all tasks ended"
  because no handle exists.
- **`SessionBytesWritten` has no producer and cannot have one.** It is declared
  (`core/services/recorder/logic/policy/src/lib.rs:112`), handled (`:336`) and published
  (`core/services/recorder/app/src/lib.rs:573`), but nothing constructs it and `encode_injected` has no arm for it
  (`core/services/recorder/app/src/inject.rs:235`), so `RecordingState.session_bytes_written` is a permanent 0. It is
  direct proof that the E6 door blocks features: the data plane knows the byte count and has no way to tell the Domain.
- **The error branch of the `TapPolicy` send is unreachable** (`core/services/recorder/app/src/lib.rs:263`), because the
  original sender lives in `run_async`'s frame for the whole life of the service. A reader spends time on handling that
  cannot happen, and the bad clone form (below) is what hides the channel lifetime that proves it.
- **Answer, the Recorder's real data flow.** There are 15 task kinds plus two blocking threads. Only three are stopped
  at shutdown: the Inbox loop (it is the driver), the persist worker (queue closed and joined,
  `core/libs/app/service/src/runtime.rs:699`) and the in-flight IO tasks (drained for up to 5 s,
  `core/libs/app/service/src/shutdown.rs:51`). The rest are dropped with the runtime, and the four Recorder-owned ones
  are the costly ones.

| Recorder-owned task | path:line | Stopped at shutdown? | On failure |
|---|---|---|---|
| tap supervisor, and `run_data_plane` inside it | `core/services/recorder/app/src/lib.rs:413` | no | recording stops permanently |
| discovery loop | `core/services/recorder/app/src/lib.rs:436` | no | no MAVLink facts; the armed gate freezes |
| library progress injector | `core/services/recorder/app/src/lib.rs:482` | no | repair progress stops; the UI bar sticks |
| liveliness `get`, one per topic | `core/services/recorder/app/src/tap.rs:367` | no | that topic stays on the fallback lane |

- **Answer, the Rust rule for clones.** A value cloned only so an `async move` block or a `move` closure can own it must
  be bound inside a block attached to the spawn, never in the enclosing scope, so the enclosing scope gains no name it
  does not use. The repository already uses the good form three times, all in the same file as the worst violation
  (`core/services/recorder/app/src/lib.rs:259`, `:282`, `:294` against `:408` to `:411`), so the file disagrees with
  itself. Mechanical enforcement needs a custom check, since no clippy lint covers it: flag a `let` whose initialiser
  ends in `.clone()` or `Arc::clone(..)` and whose only use is inside a nearby `spawn` argument. The before and after
  pair is in P9's owner notes; the violation list is 13 sites across `core/services` and `core/libs`, with
  `core/services/example` and `core/app/blueos` clean.
- Resolved: R3 counted 13 sites that break the clone-before-`async move` rule and R8 counted 12. One scan finds 13
  sites (19 clone lines), 11 of them outside tests, so the difference is grouping and test files:
  `core/libs/app/service/src/runtime.rs:996`, `core/libs/adapters/comms/src/state.rs:25`,
  `core/services/recorder/app/src/library_io.rs:123`, `:160`, `:235`, `core/services/recorder/app/src/tap.rs:366`,
  `core/services/recorder/app/src/lib.rs:259`, `:284`, `:435`, `:481`, `:695`, and in tests
  `core/libs/app/service/tests/kernel.rs:454`, `:746`.
- **Recommendation:** a Kernel-owned task registry, declared next to the other registrations, with handles stored in
  `KernelState` beside `timers` (`core/libs/app/service/src/runtime.rs:313`).

```rust
pub enum Restart {
    Never,
    OnFailure { backoff: Backoff, max_attempts: Option<u32> },
    Always { backoff: Backoff },
}

pub struct TaskContext<D: Domain> {
    pub session: Session,
    pub commands: CommandSender<D>,   // E6
    pub shutdown: ShutdownSignal,     // resolves on SIGTERM or a trigger
}

impl<D: Domain + 'static> ServiceBuilder<D> {
    /// Long-running task owned by the kernel: started after the adapters, stopped before the
    /// inbox drains, restarted per `restart`.
    pub fn task<Run, Fut>(self, name: &'static str, restart: Restart, run: Run) -> Self
    where
        Run: Fn(TaskContext<D>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), String>> + Send + 'static;
}
```

  The shutdown order is: set the shutdown signal so tasks leave their loops, dispatch the `on_shutdown` Command, drain
  `Effect::Io` for 5 s, join the task handles with the remaining budget, then abort the stragglers and name them in a
  `warn!`. Stopping the tasks before the shutdown Command would break `StopRecording`, and stopping them after the IO
  drain lets the tap write to a finished file, so signal-first and join-last fixes both. Task health becomes a map the
  Kernel merges into the `status` State, which deletes the hand-written `.status` closure from both services (E4) and
  gives P3 its `STATUS_DEGRADED`. The Recorder then has three `.task(...)` calls instead of three detached spawns, the
  supervisor loop disappears because `Restart::Always` is the supervisor, and `run_data_plane` is forced to select on
  the `watch` receiver, which is the fix for the rotation bug above. Implement the cancellation semantics on the
  existing `watch<bool>` (`core/libs/app/service/src/shutdown.rs:31`) so no new dependency is needed, and in the same
  change make a Decision's `Io` effects run sequentially in one task. Rejected: a `TaskGuard` RAII helper owned by the
  service, which fixes the dropped handles but not shutdown ordering, restart or status, and leaves every service
  writing its own supervisor; and `tokio-util`'s `TaskTracker` plus `CancellationToken`, which is well tested but gives
  no restart, no panic-to-status and no typed Command sender, and is a new workspace dependency.

## E6. The "Internal" command

**Owner notes**

- It is not clear what the `Internal` opaque command (`core/services/recorder/app/src/lib.rs:257`) is for.

**Orchestrator notes**

- In-process producers reach the Domain through Zenoh. MAVLink facts from the tap and progress from library IO are
  sent as a Zenoh query to the service's own key `blueos/v1/recorder/command/Internal`, with a JSON body
  (`core/services/recorder/app/src/lib.rs:518`). `decode_injected` turns them back into Commands
  (`core/services/recorder/app/src/inject.rs:74`).
- **Root cause:** the Kernel gives in-process code no way to send to the Inbox. Zenoh is the only door. The builder
  docs themselves warn that self-queries can be lost (`core/libs/app/service/src/builder.rs:75` to `:78`).
- The cost is `inject.rs`, 307 lines:
  - a JSON copy of part of `RecorderCommand` (`InjectedCommand`, `core/services/recorder/app/src/inject.rs:28`),
  - hand-written encode and decode (`core/services/recorder/app/src/inject.rs:150`, `:74`),
  - enums sent as strings, with silent default arms (`_ => RecordingOperationKind::Delete` at
    `core/services/recorder/app/src/inject.rs:121`, `_ => CaptureCommandKind::RequestCaptureStatus` at `:140`).
- **Trust:** `Internal` is a public endpoint and is listed in `ServiceInfo`. Any Zenoh client, including a browser
  tab, can send `ArmedChanged` or `LibraryOperationFinished` and corrupt the Domain state.
- `fact_to_injected_command` (`core/services/recorder/app/src/inject.rs:255`) is a MAVLink-fact-to-Command mapping,
  not an injection step. The name misleads.
- Each injected Command is a query round trip through zenohd with a 2-second timeout
  (`core/services/recorder/app/src/lib.rs:528`). It runs inside the discovery loop, so a slow reply stalls discovery.

**Deep-dive questions**

- What should the Kernel offer so that in-process producers send Commands directly? What happens to `inject.rs`?

**Deep-dive findings (R3)**

- **Correction, the root cause is sharper than stated.** The Inbox channel is created inside `runtime::run`
  (`core/libs/app/service/src/runtime.rs:437`), which runs after the builder has been consumed
  (`core/libs/app/service/src/builder.rs:367`), so no sender can exist before `run` and the builder exposes none.
  Contrast `shutdown_handle` (`core/libs/app/service/src/builder.rs:91`), which solves the identical ownership problem
  by creating its channel in the builder. Note also that the doc warning about lost self-queries sits on `on_start`
  (`:75`), the only sanctioned alternative, and `on_start` works only once at startup.
- **Correction:** `fact_to_injected_command` does not produce an `InjectedCommand` at all, it returns a
  `RecorderCommand` (`core/services/recorder/app/src/inject.rs:255`), so the name is wrong twice.
- **`Internal` is an unauthenticated write port into the Domain, advertised with no schema.** A browser tab on the
  vehicle network can send `{"kind":"armed_changed","armed":false}` and stop MAVLink recording, or forge
  `library_scan_completed` and put fabricated file names into the `library` State, which the frontend turns into
  `/userdata/recorder/<path>` URLs. Root cause: the only in-process door is a public endpoint, and the IDL-based trust
  boundary of D-05 and D-06 does not cover a JSON body, so `api.lock` has no entry for it and the D-06 gates cannot see
  it either. Not reproduced on a device.
- **Silent default arms turn a typo into a different Command.** Any unknown operation string maps to
  `RecordingOperationKind::Delete` (`core/services/recorder/app/src/inject.rs:121`) and any unknown capture command to
  `RequestCaptureStatus` (`:140`). A forged or mis-encoded operation clears the in-flight repair, snapshot and delete
  sets and triggers a rebuild of the library entries (`core/services/recorder/logic/library/src/lib.rs:365`), so a
  running repair loses its progress in the UI.
- **`encode_injected` and `decode_injected` are not inverse, and the gap fails silently.** Both return `Err` for
  non-injectable commands (`core/services/recorder/app/src/inject.rs:214`, `:235`) and `inject_internal_command`
  discards that error with `Err(_) => return` (`core/services/recorder/app/src/lib.rs:521`). Adding a Command and
  forgetting the encode arm produces a feature that compiles, runs and does nothing, which is exactly the shipped
  `SessionBytesWritten` bug in E5.
- **The loopback crosses the shutdown gate, so in-flight results are dropped silently.** While shutting down the Inbox
  rejects every Command that has a reply channel (`core/libs/app/service/src/runtime.rs:599`), and the injector only
  logs when the Zenoh query itself fails (`core/services/recorder/app/src/lib.rs:532`), while a rejected ack is a
  successful query. A repair that finishes during the 5 s drain reports into a void and the library state then disagrees
  with the disk until the next scan. Root cause: the injector treats "Zenoh delivered it" as "the Domain accepted it".
- **The door has no backpressure, only a 2 s timeout** (`core/services/recorder/app/src/lib.rs:528`). The Inbox is a
  256-slot `mpsc` that gives real backpressure for free; routing through Zenoh replaces that with a fixed timeout and a
  dropped message.
- **Answer, what the Kernel should offer:** move the Inbox pair into the builder, exactly as `shutdown_handle` already
  does, and hand out a typed sender.

```rust
/// Sends domain commands into this service's inbox from in-process producers.
/// Obtain before `run`; valid for the life of the service.
#[derive(Clone)]
pub struct CommandSender<D: Domain> { inbox: mpsc::Sender<InboxMessage<D>> }

impl<D: Domain> CommandSender<D> {
    /// Queues `command`. Applies backpressure when the inbox is full.
    pub async fn send(&self, command: D::Command) -> Result<(), SendError>;
    /// Queues `command` without waiting; fails fast when the inbox is full.
    pub fn try_send(&self, command: D::Command) -> Result<(), SendError>;
    /// Queues `command` and waits for the domain's verdict.
    pub async fn send_awaiting_ack(&self, command: D::Command) -> Result<CommandAck, SendError>;
}
```

  `send_awaiting_ack` reuses the oneshot the adapters already use (`core/libs/app/service/src/runtime.rs:812`), so the
  reject path and the ack shape are identical to an external Command.
- **Answer, what happens to `inject.rs`: it is deleted.** `InjectedCommand`, `InjectedSystem`,
  `InjectedScannedRecording`, the four hand-written converters and both codecs go
  (`core/services/recorder/app/src/inject.rs:8` to `:252`), and with them two of the three copies of
  `SystemAndComponent` and one copy of `ScannedRecording` (E8). `fact_to_injected_command` is renamed
  `command_from_fact` and moves next to the MAVLink facts, with the clock passed in instead of read inline (E10). The
  `Internal` registration (`core/services/recorder/app/src/lib.rs:257`) and `inject_internal_command` (`:518`)
  disappear, so the endpoint leaves `ServiceInfo`. The discovery loop sends directly, which turns the 2 s stall into
  real backpressure. The library progress task (`:482`) disappears entirely, because `LibraryIoContext.progress_sender`
  is already an `mpsc::Sender<RecorderCommand>` (`core/services/recorder/app/src/library_io.rs:25`) that simply has
  nowhere to send. And `SessionBytesWritten` becomes implementable. Net: roughly 320 lines deleted, one public endpoint
  removed, two tasks and one channel removed, three duplicated types removed.
- **Recommendation:** expose `ServiceBuilder::command_sender()` for anything the service constructs itself, and put the
  same `CommandSender` in the `TaskContext` of E5 so Kernel-owned tasks need no extra wiring. A sender only through
  `TaskContext` would be stricter but blocks adapters that are not tasks, and `LibraryIoContext` is exactly that.
  Rejected: keeping the Zenoh loopback on an internal key, which keeps `inject.rs`, the round trip and the silent
  default arms, adds a second keyspace to explain, and is not the case D-08 deferred IPC for, since the producer and
  the consumer are the same process. Add the two matching entries to the example (P4 lists both as missing): "send a
  Command from in-process code" and "a long-running background task". Those two plus `CommandSender` are what would
  have stopped `Internal` from being written.

## E7. Domain ontology: policy, library, tap, inbox

**Owner notes**

- "Served outside the inbox" (`core/services/recorder/logic/library/src/lib.rs:146`): the Inbox is never explained in
  the Recorder.
- Should `LibraryDomain` be called `CqrsDomain`? The ontology is confusing.
- `RecorderCommand` (`core/services/recorder/logic/policy/src/lib.rs:86`) has no doc comments. It is not clear what
  `SetPolicy` does.
- "Tap" means nothing to the reader.
- Looking at the Recorder code, it is not clear how to separate "policy" from "library".

**Orchestrator notes**

- `LibraryDomain` implements `Domain` (`core/services/recorder/logic/library/src/lib.rs:166`), but the Kernel never runs
  it. It exists so that `handle_library_command` can return `Decision<LibraryDomain>`. The trait then forces
  placeholders:
  - `LibraryQuery {}` (`core/services/recorder/logic/library/src/lib.rs:157`)
  - `LibraryQueryView` (`:160`)
  - `LibraryJobSpec {}` (`:163`)
  - an `io_from_job` that returns a fake `Scan` (`:193`)
- Its `handle_command` passes `None` as the active session (`core/services/recorder/logic/library/src/lib.rs:182`),
  which the real path (`dispatch_library`) does not. So the trait implementation behaves differently from production.
  It is used by one test (`core/services/recorder/logic/library/src/tests.rs:414`).
- `RecorderDomain` has the same problem. `RecorderQuery::TapPolicy` and `TapPolicyView`
  (`core/services/recorder/logic/policy/src/lib.rs:214` to `:222`) are never registered with `.query`, so they are
  dead code. `io_from_job` returns a fake `OpenSession` (`core/services/recorder/logic/policy/src/lib.rs:419`).
- The crate `blueos-recorder-policy` holds the whole Recorder Domain: session lifecycle, the MAVLink camera protocol,
  and library dispatch. "Policy" has three meanings:
  - `RecordingPolicy`, the user settings (`core/services/recorder/logic/policy/src/lib.rs:45`)
  - `TapPolicy`, the filter derived for the data plane (`core/services/recorder/logic/policy/src/lib.rs:152`)
  - the crate name
- Library Commands exist twice: `RecorderCommand::RepairRecording`
  (`core/services/recorder/logic/policy/src/lib.rs:120`) and `RecorderCommand::Library(LibraryCommand::RepairRecording)`
  (`:134`). The Domain maps the first one to the second one (`:349` to `:356`), and three `map_library_*` functions
  translate the results back.
- The policy crate re-exports library types under new names (`OperationKind as RecordingOperationKind`,
  `RecordingState as LibraryRecordingState`, `core/services/recorder/logic/policy/src/lib.rs:12` to `:17`). One type
  has two names depending on the crate.
- `RecorderCommand` is one flat enum that mixes four kinds of input, with no grouping and no docs:
  - client intents (`StartRecording`)
  - IO results (`SessionOpened`, `Ack`, `IoFailed`)
  - timer ticks (`CaptureStatusTick`)
  - facts observed on the bus (`ArmedChanged`)

**Deep-dive questions**

- Propose an ontology for the Recorder: which Domains exist, what each one owns, and how they compose without a fake
  `Domain` implementation.
- Should the `Domain` trait be split, so that a Domain without queries or jobs does not need placeholders?
- Glossary: which names (Inbox, Tap, Policy, Library, Kernel, ...) must be renamed or defined, and where should they
  be defined so that a reader meets them before using them?

**Deep-dive findings (R4)**

- **Correction:** "the trait then forces placeholders" is wrong. The trait forces the four associated types, not
  placeholder values. `type View = ()` is legal and `match query {}` / `match *job_spec {}` are total for uninhabited
  types, both compiled with `rustc --edition 2024`. So `LibraryQueryView`
  (`core/services/recorder/logic/library/src/lib.rs:160`), the fake `Scan` (`:194`) and the fake `OpenSession`
  (`core/services/recorder/logic/policy/src/lib.rs:420`) are authoring choices and latent bugs that nobody had to write.
- **Corrections:** "Policy has three meanings" is five occurrences of three meanings; add the IDL message
  `core/libs/idl/interfaces/blueos_recorder_msgs/msg/RecordingPolicy.msg:1` and the public command `SetPolicy`
  (`core/services/recorder/app/src/lib.rs:225`). The renaming re-exports are at
  `core/services/recorder/logic/policy/src/lib.rs:13` to `:18`, not `:12` to `:17`. And the policy crate also holds the
  ROS 2 DDS schema gate (`core/services/recorder/logic/policy/src/ros2dds_gate.rs:1`), so it covers four unrelated
  concerns, not three.
- **`Decision<D: Domain>` is the single root cause of the fake Domain.** Its three fields need only `D::Event`,
  `D::Command` and `D::IoRequest` (`core/libs/logic/cqrs/src/lib.rs:65` to `:69`), yet the bound forces any reusable
  sans-IO block to supply all seven associated types in order to return a decision. Every future composed service
  (wizard, calibration, D-01) will hit this and copy `LibraryDomain`. R2 found the same root cause from the API side;
  see E4.
- **The empty `RecorderJobSpec` makes the Recorder advertise a jobs endpoint it can never fill.**
  `pub enum RecorderJobSpec {}` (`core/services/recorder/logic/policy/src/lib.rs:224`) plus the boilerplate `.jobs`
  registration (`core/services/recorder/app/src/lib.rs:224`) become a published `jobs` endpoint
  (`core/libs/app/service/src/runtime.rs:207`), so the D-24 inspector sees a state that is structurally always empty.
  Root cause: `jobs` is a mandatory-looking builder step rather than an opt-in capability.
- **`Session` means three things inside `run_async`, and this is the worst readability problem in the service.** The
  Zenoh connection (`core/services/recorder/app/src/lib.rs:160`), `Mutex<Option<McapSession>>`, which is an MCAP file
  (`:97`), and `application.snapshot.session`, a `RecordingSession` value object (`:218`). No glossary can fix it; only
  renaming can.
- **`RecordingState` names two unrelated types and both are visible in one file.** The IDL message is the recorder
  session state (`core/libs/idl/interfaces/blueos_recorder_msgs/msg/RecordingState.msg:1`, imported at
  `core/services/recorder/app/src/lib.rs:36`), while the library's is a per-file lifecycle enum
  (`core/services/recorder/logic/library/src/lib.rs:34`, imported as `LibraryRecordingState` at `:48`), and the function
  that bridges them maps to the `RecordingFile` constants instead (`:631`).
- **`logic/` holds two kinds of sans-IO machine with two calling conventions, and only one has a name.**
  `RecorderDomain` is driven by the Inbox as `Command -> Decision`; `Ros2ddsGate` is driven by the tap as
  `Ros2ddsGateInput -> Vec<Ros2ddsGateOutput>` (`core/services/recorder/logic/policy/src/ros2dds_gate.rs:51`, used at
  `core/services/recorder/app/src/tap.rs:34`). The architecture vocabulary
  (`doc/architecture/rust-service-overview.md:16` to `:32`) has no word for the second.
- **The same library command arrives by two different transports.** `OperationFinished` and `ScanCompleted` come back as
  an IO result (`core/services/recorder/app/src/library_io.rs:93`, `:353`), while `RepairProgress` goes out on an mpsc
  channel and then through JSON over Zenoh to `Internal` (`:136`, `core/services/recorder/app/src/lib.rs:482`). Two
  transports for two variants of one enum, chosen by nothing the reader can see. The cause is E6; the ontology makes it
  invisible.
- **`TapPolicy::from_snapshot` is a read model with no name and no path to the reader.** It is a `Snapshot -> filter`
  projection (`core/services/recorder/logic/policy/src/lib.rs:160`), identical in role to a `View`, yet it is reached by
  a side effect inside a State selector while the `Query` and `View` vocabulary built for exactly this goes unused.
- **"Inbox" is used in the Recorder before it is defined anywhere a Recorder reader looks**
  (`core/services/recorder/logic/library/src/lib.rs:146`, `core/services/recorder/README.md:52`); the only definition is
  `doc/architecture/rust-service-overview.md:24`, which the README does not link.
  `core/services/recorder/app/src/tap.rs` has no module doc at all, so "tap" is defined only in a sentence in another
  file.
- **Answer, the ontology: one Domain and four Blocks, where a Block does not implement `Domain`.** The Domain is what
  the Kernel runs and owns the root Snapshot, the public Command enum and the IO request enum. A Block is a sans-IO
  reducer the Domain composes, owning a sub-snapshot and its own command, event and IO enums. For the Recorder that is
  `logic/recorder` as the Domain, plus `capture` (session lifecycle, armed flag, bytes written, the record gate),
  `cameras` (the MAVLink camera protocol of D-15), `library` (unchanged in scope) as Blocks, and `schema-gate`, which is
  not a Block but a pure state machine driven by an adapter. Composition works with no fake Domain:

```rust
// core/libs/logic/cqrs
pub struct Outcome<Event, Command: Clone, IoRequest: Clone> {
    pub events: Vec<Event>,
    pub effects: Vec<Effect<Command, IoRequest>>,
    pub rejection: Option<String>,
}

pub type Decision<D> =
    Outcome<<D as Domain>::Event, <D as Domain>::Command, <D as Domain>::IoRequest>;

// and in the Recorder Domain, the whole composition:
RecorderCommand::Library(command) => library::handle(&mut snapshot.library, command, active)
    .map(RecorderEvent::RecordingOperation, RecorderCommand::Library, RecorderIo::Library),
```

  `Outcome::map` lives in `blueos-cqrs` and is tested once for everyone, which deletes `LibraryDomain`, its four
  placeholders, and the 40 lines of `map_library_decision`, `map_library_event` and `map_library_effects`
  (`core/services/recorder/logic/policy/src/lib.rs:523` to `:562`). Two rules come with it: delete the flat library
  duplicates (`:120` to `:133`) so the builder closures map straight to `RecorderCommand::Library(..)`, which removes
  about 50 lines and the question "which of the two do I send?"; and group `RecorderCommand` by origin
  (`Request`, `IoResult`, `Tick`, `Observed`), because the four kinds have different trust levels (E6) and the grouping
  is what documents the enum, with prose doc comments as the second line of defence.
- **Answer, splitting the trait: yes, but the first move is smaller.** Use uninhabited types and total matches today,
  which deletes both fake IO requests and the placeholder view with no API change. Then split into `Domain` plus
  `DomainQueries` plus `DomainJobs`, with `ServiceBuilder::query` and `.jobs` carrying the extra bounds, so a minimal
  service declares four associated types and one method. Rejected mechanically, not as a preference: associated type
  defaults in a trait are unstable (E0658), compiled to be sure, so "just give the trait defaults" is not available on
  stable.
- **Answer, the glossary: rename first, because a glossary that explains three meanings of `Session` is a bug report.**
  `McapSession` becomes `McapFile`, `RecordingSession` becomes `ActiveRecording`, `TapPolicy` becomes `RecordGate`, the
  IDL `RecordingState` becomes `RecorderSessionState`, the library `RecordingState` becomes `RecordingFileState`,
  `RecordingPolicy` becomes `RecorderSettings` (and is then the only one, see E11), `OperationKind` is named
  `RecordingOperationKind` at its definition so both renaming re-exports die, `blueos-recorder-policy` becomes
  `blueos-recorder-domain`, `LibraryDomain` is deleted, `Ack` and `IoFailed` become `IoSucceeded` and
  `IoFailed { request }` so the Domain can react (E4), and `Internal` is deleted with E6. Definitions live in four
  places: the architecture vocabulary stays the one normative table and gains `Block`, `Outcome`, `data plane` and
  `tap`; each service README gets a short "words used in this service" table; every `logic/` crate root opens with one
  sentence of what the block owns in user terms rather than decision IDs; and every Command enum carries an enum-level
  doc stating the four origins.
- **Recommendation:** do the uninhabited-type cleanup now, because the fake `OpenSession` is a loaded gun, then land
  `Outcome` in the shared library together with the `logic/policy` split and the renames, with the trait split
  alongside. `Outcome` is the one that must land in `blueos-cqrs`, because every service after the Recorder composes
  blocks.

## E8. Type conversions and duplicated types

**Owner notes**

- `recording_*_from_*` (`core/services/recorder/app/src/lib.rs:564`, `:610`, `:640`) should use `From` and `Into`.

**Orchestrator notes**

- There are three copies of `SystemAndComponent`:
  - the policy one (`core/services/recorder/logic/policy/src/lib.rs:39`)
  - the MAVLink adapter one (`core/services/recorder/adapters/mavlink/src/lib.rs:24`)
  - `InjectedSystem` (`core/services/recorder/app/src/inject.rs:69`)

  They are converted by four hand-written functions (`to_policy`, `from_policy`, `to_mavlink_system`, and inline code in
  `fact_to_injected_command`).
- `ScannedRecording` is copied as `InjectedScannedRecording` (`core/services/recorder/app/src/inject.rs:8`) and
  converted by hand in both directions.
- IDL enums arrive as `u8` constants, so the code maps them by hand (`library_state_to_idl`,
  `core/services/recorder/app/src/lib.rs:631`).
- The orphan rule may be why `From` was not used: the app crate owns neither the IDL types (`blueos_idl`) nor the
  Domain types (policy crate).

**Deep-dive questions**

- Where can the conversions live so that `From` and `Into` work? Should the Domain use the IDL types directly for
  shared value types? Should the IDL generate Rust enums?

**Deep-dive findings (R4)**

- **Correction:** the orphan rule is a hard constraint for the app crate but it is direction-specific, not absolute.
  `impl From<&RecorderSnapshot> for idl::RecordingState` compiles in the policy crate, because a local type as the trait
  parameter satisfies the rule; it is only in the app crate, which owns neither side, that it is illegal (E0117,
  verified with a standalone `rustc` probe).
- **The layer gate leaves no crate that `logic/` and `adapters/` can share, so the duplication is structurally forced.**
  A `logic/` crate may depend only on `libs/logic` or a sibling logic crate (`.hooks/lib/rust_checks.sh:86` to `:89`),
  and an `adapters/` crate only on other adapters or `libs/idl` (`:90`). The intersection is empty, so
  `SystemAndComponent` cannot be defined once with today's gate. This is not a developer oversight and no code review
  will fix it.
- **The gate and D-02 disagree, in both rows, in opposite directions.** `doc/architecture/decisions.md:93` says
  `logic/` may depend on `blueos-idl` and the gate forbids it; `:94` does not mention `blueos-idl` for adapters and the
  gate allows it. Since `doc/architecture/decisions.md:7` requires code and decisions to be fixed together, one of the
  two is stale. R2 found the same contradiction from the DI side; see E13.
- **Even with the gate relaxed, a naive `logic -> blueos-idl` dependency breaks the `no_std` gate.** The workspace
  declares `blueos-idl` with default features (`core/Cargo.toml:59`) and the default set is `["std"]`
  (`core/libs/idl/Cargo.toml:21`), while the hook builds every logic crate for `thumbv7em-none-eabihf` and
  `wasm32-unknown-unknown` with default features (`.hooks/lib/rust_checks.sh:105`). So any logic crate using the IDL
  must write `default-features = false`, which `doc/architecture/decisions.md:306` already asks for generally.
- **The recording-file state enum exists in four representations, and the TypeScript one is hand-written:** the `.msg`
  constants (`core/libs/idl/interfaces/blueos_recorder_msgs/msg/RecordingFile.msg:4`), the generated Rust consts
  (`core/libs/idl/codegen/src/lib.rs:429`), the domain enum (`core/services/recorder/logic/library/src/lib.rs:34`), and
  `core/frontend/src/libs/recorder/constants.ts:13`. The generator emits constants to TypeScript only inside the schema
  text, never as values. `OperationKind` is worse at five, because `inject.rs` adds JSON strings. A new `STATE_*` value
  needs four coordinated edits in two languages and nothing fails if one is missed.
- **The real cost of the hand-written conversions is the silent default arms, not the verbosity.** Any unknown operation
  string becomes `Delete` (`core/services/recorder/app/src/inject.rs:118`) and any unknown capture command becomes
  `RequestCaptureStatus` (`:137`), so a typo deletes a recording instead of failing. A generated or derived conversion
  cannot do this.
- **One MAVLink command id has three representations inside the service:** the `MAV_CMD_*` constants in the Domain
  (`core/services/recorder/logic/policy/src/lib.rs:34`), `CaptureCommandKind` re-mapped to `u32` by
  `capture_command_id` (`:564`), and `mavlink::MavCmd` in the adapter
  (`core/services/recorder/adapters/mavlink/src/lib.rs:161`), re-matched once more in
  `core/services/recorder/app/src/inject.rs:292`. The Domain cannot use `MavCmd` because of the gate, which is why the
  constants were copied.
- **Answer, where the conversions can live: exactly two viable homes.** The logic crate that owns the Domain type, which
  needs the gate change plus `default-features = false`; or a newtype in the app crate (`struct StateField(u8)`), which
  buys the `From` syntax and nothing else. A third option is closed by policy rather than by Rust: `blueos-idl` is
  published (`core/libs/idl/Cargo.toml:14`) and `doc/architecture/decisions.md:171` forbids a published crate from
  depending on unpublished workspace crates, so the impls cannot live there.
- **Answer, should the Domain use IDL types directly: only for leaf value types.** Keeping domain-owned types with
  `From` impls in the logic crate preserves exhaustive matching exactly where the rules live, and the conversion sits
  next to the type it belongs to with one round-trip test per pair. Using IDL types throughout would delete about 110
  lines outright, but it makes D-06's append-only rule a Domain constraint, replaces Rust enums with `u8`, and makes the
  Domain ungreppable because the types are generated into `OUT_DIR` (E9). The carve-out is pure scalar wrappers with no
  domain rules, such as `builtin_interfaces/Time`, which `core/services/recorder/app/src/lib.rs:603` currently wraps by
  hand.
- **Answer, should the IDL generate Rust enums: yes, gated on a documented convention.** Generate an enum when a field
  `x` has sibling constants named `X_*` of the same type, with a mandatory `Unknown(u8)` variant so D-06 forward
  compatibility holds and an older reader survives a new value, and with shared test vectors per D-24
  (`doc/architecture/decisions.md:623`) so the Rust, TypeScript and Python codecs agree. Then `library_state_to_idl` and
  `internal_status_to_idl` become `From` impls and every future copy of them never gets written. Rejected: a newtype
  with associated consts, which loses the exhaustiveness that is the whole point. Ship the smaller change in the same
  pull request regardless, which is emitting the constants to TypeScript as values, because the hand-written
  `core/frontend/src/libs/recorder/constants.ts:13` is a silent divergence waiting to happen.
- **Recommendation:** fix the gate and D-02 together first, because nothing else in E8 is possible without it; then
  delete two of the three `SystemAndComponent` types, leaving the survivor in the service's `logic/` and allowing an
  `adapters/` crate to depend on its own service's `logic/`, which is the normal hexagonal direction and whose
  prohibition created the three copies; then generate the enums; then replace every `*_from_*` free function with a
  `From` impl plus a round-trip test; and delete `InjectedScannedRecording` and `InjectedSystem` along with `inject.rs`
  itself (E6). Expected effect: `inject.rs`'s 307 lines go away and the roughly 110 mapping lines at
  `core/services/recorder/app/src/lib.rs:557` to `:664` become derived or `From` impls next to their types.

## E9. Generated IDL types are hard to navigate

**Owner notes**

- `RecordingLibrary` (`core/services/recorder/app/src/lib.rs:268`) is a generated type. The editor finds it, but from
  the terminal it is hard to find.

**Orchestrator notes**

- The types are generated into `OUT_DIR` and included with `include!` (`core/libs/idl/src/lib.rs:15`). A `rg` over the
  repository never finds `struct RecordingLibrary`. The `.msg` file is the only source that a reader finds.

**Deep-dive questions**

- Compare the options: commit the generated code with a freshness check, generate into `src/`, or keep `OUT_DIR` and
  document a lookup path. Check what other large Rust projects do.

**Deep-dive findings (R5)**

- **Correction:** the `.msg` file is not the only source a reader finds. `core/libs/idl/typescript/messages.d.ts` (201
  lines, one interface per message) and `core/libs/idl/typescript/schemas.ts` (386 lines) are committed and greppable.
  And the `.msg` files are reachable by two paths, because `core/interfaces/<package>` is a symlink farm pointing into
  `core/libs/idl/interfaces/` while `doc/architecture/decisions.md:106` describes `core/interfaces/` as the source.
- **Correction:** `rg` fails for three compounding reasons, not one. The output is in `OUT_DIR`; the target directory is
  outside the repository entirely, so even searching `target/` from the repository root cannot work; and five live
  copies of `generated_mod.rs` exist at once under different build-hash directories, plus one per extra target.
- **The generated Rust is unformatted token soup, one line per message, and that is the real reason it is unreadable.**
  The codegen writes `format!("...{}", tokens)` where `tokens` is a `TokenStream`
  (`core/libs/idl/codegen/src/lib.rs:346`), and `TokenStream::to_string()` puts everything on one line with spaces
  around every `::`. The generated `recording_library.rs` is 13 lines, and line 13 holds the struct, both codec impls,
  the `Message` impl and the full embedded schema text; identifiers read `crate :: msg :: ...`. This also decides the
  committing question, because committed as-is every `.msg` change is a one-line diff no reviewer can read. Both
  reference implementations format: `prost-build` makes `prettyplease` a default feature, and `mavlink-bindgen`, already
  a transitive BlueOS dependency, shells out to `rustfmt`.
- **The build script writes into the source tree, and the freshness check cannot fail.** `core/libs/idl/build.rs:8`
  passes the `typescript` folder into the codegen, which writes four committed files there, and the guard
  (`core/libs/idl/tests/typescript_stale.rs:30`) asserts that the generated mtime is at least the `build.rs` mtime.
  Running the test necessarily builds `blueos-idl`, whose build script has just rewritten all four files, so the
  assertion is always true; it also returns early when `BLUEOS_IDL_REGEN_TYPESCRIPT=1`. Editing a `.msg` never touches
  `build.rs`, and a fresh checkout sets every mtime to checkout time, so the comparison carries no information either
  way. Root cause: an mtime comparison instead of a content comparison, combined with a build script that writes
  outside `OUT_DIR`. Any recommendation to commit the generated Rust must not repeat this.
- **`blueos-idl` cannot be published, so D-05 is unmet.** `cargo publish --dry-run -p blueos-idl` fails because
  `blueos-idl-codegen` specifies no version: the codegen is `publish = false`
  (`core/libs/idl/codegen/Cargo.toml:6`) and is declared as a bare path dependency (`core/Cargo.toml:60`), while
  `doc/architecture/decisions.md:167` says the crate is published and `:171` forbids a published crate from depending on
  an unpublished one. Measured cost of that build dependency: a 38-crate build closure (including `regex`, `md-5` and
  `syn`) against a 9-crate runtime closure, and 6.13 s of clean build. `mavlink` does this correctly with a pinned
  published build dependency.
- **The generated catalog is 508,927 bytes of string literals in one `match`** across 218 lines, behind the `catalog`
  feature the Recorder turns on (`core/services/recorder/app/Cargo.toml:15`), while the generated `msg/` tree is 76,272
  bytes across 41 files. That splits the committing question into two very different artifacts.
- **`cargo doc` works and is the cheapest lookup path that exists today**, but it bakes the absolute `OUT_DIR` into the
  doc tree, so the published source view is per-machine and not reproducible.
- **Dead code in the codegen that clippy cannot see:** the loop at `core/libs/idl/codegen/src/lib.rs:297` fills a
  `lines` vector that `:320` discards with `let _ = lines;`, which suppresses the unused warning so
  `cargo clippy -D warnings` passes. This is a concrete calibration case for P8's dead-code protection.
- **Answer, what notable Rust projects do, and the split is consistent.** windows-rs commits 672 generated `.rs` files
  into `src/` with no build-script codegen; aws-sdk-rust commits generated crates and keeps the generator in another
  repository; `prost-types` commits into `src/` with a `@generated` marker; `prost-build` and `mavlink` use `OUT_DIR`
  for other people's builds, both formatted; ros2-rust generates a whole crate into the colcon build directory. tonic is
  unverified. The rule the split follows: codegen that is a library for other people's builds stays in `OUT_DIR`, and
  codegen whose output is the product users read gets committed. `blueos-idl` is the second kind, since
  `doc/architecture/decisions.md:166` publishes it precisely so extensions and ROS 2 nodes read those types. Unverified:
  whether windows-rs or aws-sdk-rust run a freshness check on their committed output.
- **Recommendation: commit the generated code into `src/`, with four conditions.** Run the output through
  `prettyplease` in the codegen tool, which adds a dependency to the tool and not to `blueos-idl`, because option A is
  only worth having if the committed code is reviewable. Turn the codegen into an explicit
  `cargo run -p blueos-idl-codegen -- --write`, wired into `.hooks/pre-push --fix`; this is not a new pattern, it is the
  `BLUEOS_IDL_UPDATE_LOCK=1` contract the team already accepted for `api.lock`
  (`core/libs/idl/tests/api_lock.rs:34`). Make the check regenerate into a temporary directory and byte-compare, failing
  with the exact fix command, and replace `typescript_stale.rs` with the same comparison so that gate becomes real.
  Commit `msg/`, `schema_catalog.rs` and the TypeScript, then delete `core/libs/idl/build.rs` and the build dependency,
  which is what makes the crate publishable. On the 509 KB: the 212 vendored `.msg` files it derives from are already
  committed, so a ROS distro bump is already a large reviewed diff. Rejected: generating into `src/` from `build.rs`,
  which is what produced the broken freshness check. Fallback if committing 509 KB is refused: keep `OUT_DIR`, still
  format the output, publish the codegen with an `=` pinned version as mavlink does, and document the lookup path plus
  `cargo doc -p blueos-idl --open` in `core/libs/idl/README.md`. Two cleanups belong with either option: resolve the
  `core/interfaces` symlink farm to one real location, and delete the dead `lines` loop.
- Decided (owner): R5. Commit all generated code, Rust and TypeScript, behind an explicit regeneration command and a
  CI byte comparison. The committed output is also a gate against unintended changes: any change to an IDL file, to
  the codegen, or to a codegen dependency shows up as a reviewed diff, and CI fails when the committed files and the
  regenerated files differ.

## E10. Reinvented library code and `no_std`

**Owner notes**

- Most of `core/services/recorder/logic/library/src/timestamp.rs` looks like work that a common library (probably
  chrono) already does.
- Shouldn't `#![no_std]` be in `Cargo.toml` instead of in the source files?

**Orchestrator notes**

- The calendar math is written twice: `epoch_days_to_civil` (`core/services/recorder/logic/library/src/lib.rs:465`)
  and `civil_days_since_epoch` (`core/services/recorder/logic/library/src/timestamp.rs:107`). Chrono is already a
  workspace dependency (`core/Cargo.toml:73`) and is used in the app (`core/services/recorder/app/src/lib.rs:118`).
  The logic crate rewrites it because it is `no_std`.
- `generate_filename` converts `SystemTime` to seconds and then to chrono
  (`core/services/recorder/app/src/lib.rs:118` to `:126`).
- "Now" is computed in at least four places: `unix_time_now` (`core/services/recorder/app/src/lib.rs:596`),
  `now_millis` (`core/services/recorder/app/src/tap.rs:243`), inline in `fact_to_injected_command`, and inline in
  `handle_sample` (`core/services/recorder/app/src/tap.rs:440`).
- File names use four timestamp formats (`recorder_YYYYMMDD_HHMMSS`, `.snapshot-...Z`, `.copy-...Z`, `_split_`), all
  parsed by hand.

**Deep-dive questions**

- Can the logic crates use chrono (or `time`) with `default-features = false`? What is the smallest change that
  deletes the hand-written calendar code?
- `no_std` stays (owner decision). It is a crate attribute in Rust, so `Cargo.toml` cannot set it. It is already
  enforced: the pre-push hook builds the logic crates for a `no_std` target (`.hooks/lib/rust_checks.sh:109`).

**Deep-dive findings (R5)**

- **Correction:** the calendar math is not "written twice". `epoch_days_to_civil`
  (`core/services/recorder/logic/library/src/lib.rs:465`) and `civil_days_since_epoch`
  (`core/services/recorder/logic/library/src/timestamp.rs:107`) are the two directions of Howard Hinnant's
  civil-from-days pair. The accurate statement is that about 62 lines of calendar arithmetic are hand-written across two
  files of one crate, covering both directions, with no range validation anywhere.
- **Correction:** "now" is computed in six places in the recorder app, not four, plus one in the Kernel, in three units
  and with two different error policies: `core/services/recorder/app/src/lib.rs:119` (seconds, `.expect("clock")`,
  panics), `:596` (seconds), `core/services/recorder/app/src/tap.rs:243` (millis), `:436` (nanos),
  `core/services/recorder/app/src/inject.rs:261` (millis), `core/services/recorder/app/src/library_io.rs:373`
  (seconds), and `core/libs/app/service/src/runtime.rs:1123` (sec plus nanosec). All but the first use
  `unwrap_or_default`.
- **Correction:** four filename timestamp formats are parsed but only two are produced here,
  `recorder_%Y%m%d_%H%M%S.mcap` (`core/services/recorder/app/src/lib.rs:125`) and the snapshot form
  (`core/services/recorder/logic/library/src/lib.rs:449`). `.copy-...Z` and `_split_...` are parse-only, and a fifth
  convention, `.recover` (`core/services/recorder/adapters/storage/src/folder.rs:14`), the parser does not know about.
- **Correction on the `no_std` sub-note:** correct as written, and the likely irritation behind the owner's question is
  that in two crates the attribute sits after the module documentation (`core/libs/logic/cqrs/src/lib.rs:50`,
  `core/libs/logic/jobs/src/lib.rs:22`), so a reader meets 20 to 50 lines of prose before the constraint. The idiomatic
  conditional form used by chrono and zenoh-keyexpr is `#![cfg_attr(not(feature = "std"), no_std)]`.
- **The hand-written parser accepts invalid dates and signed digit fields.** There is no range validation
  (`core/services/recorder/logic/library/src/timestamp.rs:85`) and each field is parsed with `str::parse::<i64>`
  (`:98`), which accepts a leading `+` or `-`. Compiled side by side with chrono over 1,900,289 samples from 1990 to
  2050 the two agree exactly on both format and parse, so the arithmetic is right and only validation is missing:
  `20250230_000000` silently becomes March 2 and `0000+111_000000` reads `+1` as month 1. It matters because
  `created_unix_seconds_from_filename` feeds the library sort order
  (`core/services/recorder/logic/library/src/lib.rs:528`), so a bad name silently misplaces a recording and a far-past
  value pins it to the bottom forever.
- **Three case-insensitive `.mcap` rules disagree, in two languages, and it is a live user-visible bug.** Validation
  lowercases the whole path (`core/services/recorder/logic/library/src/path.rs:28`), naming strips only exact `.mcap`
  or `.MCAP` (`core/services/recorder/logic/library/src/lib.rs:444`), and the frontend strips with `/\.mcap$/i`
  (`core/frontend/src/libs/recorder/view-logic.ts:187`). Measured by running all three: `live.Mcap` is accepted, the
  backend produces `live.Mcap.snapshot-...Z.mcap` and the browser expects `live.snapshot-`, so the snapshot succeeds and
  the browser never recognises its own download, falling into the timeout path of `doc/architecture/decisions.md:560`.
  Root cause: the suffix rule is data, written three times in two languages, with a fourth definition in the storage
  adapter (`core/services/recorder/adapters/storage/src/folder.rs:13`) that the logic crate cannot reach.
- **The exponential backoff loop is written twice, 15 lines each** (`core/services/recorder/app/src/tap.rs:96` and
  `:114`), differing only in the subscribe call and the log message, both retrying forever with no jitter and no cap.
  Root cause: there is no retry helper anywhere in `libs/`, so each adapter writes its own loop.
- **The in-process test backend reimplements Zenoh key matching with different semantics.**
  `core/libs/adapters/comms/driver/src/key_match.rs:2` is a 36-line recursive matcher used only by the channel test
  broker, while `zenoh-keyexpr` 1.9.0 is already in `core/Cargo.lock:3610`, is `no_std`-capable, supports the `$*`
  sub-chunk wildcard the hand-written matcher does not, and is not covered by the `cargo-deny` ban, which names only the
  `zenoh` crate. A test double with different semantics from the production backbone is the exact failure class
  `doc/architecture/decisions.md:567` records on the device. Separately, `**` is matched by full backtracking, so
  several `**` in one expression is exponential; key expressions are service-controlled today, so this is a latent cost
  rather than an exploit.
- **Two hand-rolled recursive directory walks can overflow the stack.**
  `core/services/recorder/adapters/storage/src/folder.rs:126` and `:169` both recurse on `path.is_dir()`, which follows
  symlinks, with no depth limit, so a symlink loop is unbounded recursion and `panic = "unwind"` does not catch a stack
  overflow. `walkdir` is already in the lockfile, does not follow symlinks by default and takes a depth limit. The two
  suffix checks in these walks also disagree with each other, one case-insensitive and one not.
- **The `std` feature of `blueos-idl` is obsolete**: its only effect is `impl std::error::Error`
  (`core/libs/idl/src/error.rs:22`), and `core::error::Error` is stable, compiled under `#![no_std]` for
  `thumbv7em-none-eabihf` with no features. Deleting the feature also deletes the special `--no-default-features` branch
  of the `no_std` gate (`.hooks/lib/rust_checks.sh:112`).
- **`user_config_dir` is hand-rolled XDG and should stay** (`core/libs/adapters/settings/src/manager.rs:180`), because
  D-11 demands byte compatibility with Python's `appdirs` and `dirs::config_dir()` returns `None` when `HOME` is unset
  while this falls back to a container path. The gap is that the comment names `appdirs` and never says why `dirs` was
  rejected, so the next reader will "fix" it.
- **Answer, chrono in the logic crates: yes, verified by compiling.**
  `chrono = { version = "=0.4.41", default-features = false, features = ["alloc"] }` compiles for both gated targets.
  Formatting needs only `alloc` and parsing is not feature-gated at all, and the only crate added to the runtime closure
  is `num-traits`, which chrono already takes with default features off. Measured binary cost: +57,344 bytes total and
  +53,284 of `.text` on two stripped thin-LTO musl release binaries, about +0.4% of the roughly 14 MB per-target binary,
  and an upper bound because the probe used runtime format strings. `time` 0.3 cannot format without `std`, so it would
  do half the job. One Cargo gotcha shapes the change: a member cannot turn off default features of an inherited
  workspace dependency, so `core/Cargo.toml:73` must be inverted to carry `default-features = false` with `alloc`, and
  the app and adapter crates ask for `features = ["std", "clock"]`; that also brings the line in line with P9.
- **Answer, the smallest change that deletes the hand-written calendar code:** invert the workspace line, add
  `chrono.workspace = true` to the library logic crate, replace `format_snapshot_timestamp` and `epoch_days_to_civil`
  (`core/services/recorder/logic/library/src/lib.rs:456` to `:482`) with one chrono format call, and replace the five
  parsing helpers (`core/services/recorder/logic/library/src/timestamp.rs:73` to `:119`) with two
  `NaiveDateTime::parse_from_str` calls, one per format. That deletes about 62 lines of arithmetic and about 30 of
  hand-rolled validation, takes `timestamp.rs` from 119 lines to roughly 40, and the existing tests
  (`core/services/recorder/logic/library/src/tests.rs:67`, `:307`) already cover all four name shapes and become the
  regression check. One behaviour change to accept deliberately: an invalid date now returns `None` and falls back to
  the file mtime, which is the fix for the validation bug and not a regression.
- **Answer, one Clock owned by the App, with a wall method and a monotonic method.** D-03 already forbids reading a
  clock inside logic (`doc/architecture/decisions.md:130`) and the Domain commands already carry `now_unix_seconds`, so
  the Domain is not the problem; nobody owns the read. One unit at the boundary, nanoseconds since the epoch, with named
  conversions instead of three units chosen per call site, and one error policy instead of `.expect("clock")` in one
  place and `unwrap_or_default()` in six. This is also the precondition P6 names, and it removes the need for most of
  the 35 `sleep` call sites. One caveat: a Clock only pays for itself if the Kernel hands it to handlers and IO
  closures, because today the `.io` closure receives a clone of the whole App, so a Clock added without that plumbing
  becomes an eighth way to read the time. Tie it to E13 and do not land it alone.
- **Recommendation:** take chrono with `alloc` in the logic crates, and do the `.mcap` suffix rule first because it is a
  live bug, defining the rule once with a shared JSON test vector checked by both `cargo test` and vitest, using the
  mechanism `doc/architecture/decisions.md:623` already established. Then replace `key_match.rs` with `zenoh-keyexpr`,
  replace the two directory walks with `walkdir` and make their suffix checks agree, add one retry helper with jitter
  and a cap to `libs/`, delete the `std` feature of `blueos-idl`, and add the one-line comment that says why `dirs` was
  rejected. Keep the custom CDR codec and the `no_std` path splitting: both have real reasons (D-06 trailing-field
  defaults and the string-padding fix; no established `no_std` path crate), so neither is reinvention. On the `no_std`
  placement question, keep the attribute where it is but move it above the module documentation. Rejected: a shared
  `blueos-time` crate wrapping the existing math, which deduplicates nothing real because the two functions are inverses
  and keeps the missing validation; and property tests against chrono as a dev-dependency, which would have caught the
  validation bug but leaves two implementations in the tree forever and still needs chrono.

## E11. Settings are not loaded or saved (found while collecting)

**Orchestrator notes**

- The initial Snapshot uses `RecordingPolicy::default()` (`core/services/recorder/app/src/lib.rs:193`). The saved
  settings are never loaded at start.
- `SetPolicy` returns `Decision::new()` with no `Effect::Persist` (`core/services/recorder/logic/policy/src/lib.rs:245`
  to `:248`). The Kernel writes settings only on `Effect::Persist` (`core/libs/app/service/src/runtime.rs:1023`). So
  `UpdateSettings` changes the running policy, but never writes it to disk.
- `RecorderSettings` (`core/services/recorder/app/src/lib.rs:66`) duplicates `RecordingPolicy`. The version exists
  three times: the `VERSION` field, the `SettingsSchema::VERSION` constant, and a hard-coded `VERSION: 1` in
  `settings_from_snapshot` (`core/services/recorder/app/src/lib.rs:660`).
- **Root cause:** the settings API asks each service to remember three things: load at start, emit `Persist` in every
  settings Command, and map both ways. Nothing enforces any of them. The example does all three; the Recorder misses
  two.

**Deep-dive questions**

- Confirm the bug on a running service. What settings API would make it impossible to miss?

**Deep-dive findings (R4)**

- **Corrections:** all four notes are verified and the bug is larger. The version exists in four places, not three: the
  DTO field (`core/services/recorder/app/src/lib.rs:67`), the `Default` literal (`:75`), the schema constant (`:83`) and
  the hardcoded literal in `settings_from_snapshot` (`:660`). "No `Effect::Persist` in the Recorder" is verified by
  exhaustion: the only occurrence in `core/services/recorder/` is an unreachable pass-through arm
  (`core/services/recorder/logic/policy/src/lib.rs:559`).
- **`auto_start_recording` does nothing, even before the persistence bug.** It is read exactly once, from the hardcoded
  default, before the builder runs (`core/services/recorder/app/src/lib.rs:193`) and turned into an `on_start` command
  (`:488`). No Domain handler ever reads `snapshot.policy.auto_start_recording`. So the UI toggle at
  `core/frontend/src/components/recorder/RecorderPolicyCard.vue:24` has no effect in the running process, is not
  persisted, and reverts on restart. It is a restart-required field in fact, but `restart_required_fields()` returns
  `&[]` (`core/services/recorder/app/src/lib.rs:89`), so no `RestartRequired` event is possible and D-11
  (`doc/architecture/decisions.md:293`) is not met.
- **The Kernel loads the settings file and throws the value away, for every service.**
  `core/libs/app/service/src/runtime.rs:120` builds a `SettingsManager`, which is `with_load(.., true)`
  (`core/libs/adapters/settings/src/manager.rs:37`) and therefore reads the disk (`:60`), and `manager.settings()` is
  never called by the Kernel; its only use of the manager is at persist time
  (`core/libs/app/service/src/runtime.rs:151`). The information needed to fix E11 is already in the Kernel's hand at
  startup, which makes this the cheapest possible fix point.
- **Registering settings writes a defaults file for a service that never reads it.** `SettingsManager::load` saves
  defaults when the file is missing (`core/libs/adapters/settings/src/manager.rs:95`, `:146`), so merely calling
  `.settings(...)` creates `<config>/recorder/settings-1.json`. `core/start-blueos-core:119` passes no `--config` and
  the Recorder passes `None` (`core/services/recorder/app/src/lib.rs:403`), so the folder is the user config directory,
  which `core/Dockerfile:135` documents as shadowed by a host bind mount. Unverified: the exact path
  `/root/.config/recorder/settings-1.json` is inferred, because `HOME` in the running core container was not confirmed.
- **The published `settings` State is derived from the App, so it always shows defaults and disagrees with disk**
  (`core/libs/app/service/src/runtime.rs:1061`, `core/services/recorder/app/src/lib.rs:658`). Combined with review M5,
  where `watchSettings` never delivers updates, the user sees the change accepted, the UI keeps it until reload, and it
  is gone after a restart. This is the user-visible symptom.
- **The service has two public doors for the same setting, and the UI exposes both.** `SetPolicy`
  (`core/services/recorder/app/src/lib.rs:225`) and the Kernel's `UpdateSettings` (`:404`) both produce
  `RecorderCommand::SetPolicy`, while the UI offers "Apply policy" (`RecorderPolicyCard.vue:51`) and a separate save
  path that sends `UpdateSettings` (`core/frontend/src/views/RecorderView.vue:220`). The UI implies "apply now" against
  "save" and the backend implements neither distinction. D-11 says settings changes arrive as `UpdateSettings`, so
  `SetPolicy` is a second door that bypasses the settings concept.
- **The Recorder is accidentally immune to review M3 and the example is not**, because the Kernel has no opinion about
  `VERSION`: the Recorder hardcodes it on the way out (`core/services/recorder/app/src/lib.rs:660`) and requires it on
  the way in with no serde default (`:67`), while the example copies the client value
  (`core/services/example/app/src/settings_schema.rs:65`). Two services, two incompatible conventions, neither enforced.
- **`RecorderSettings` lacks `deny_unknown_fields`**, so misspelled settings keys are silently ignored
  (`core/services/recorder/app/src/lib.rs:64` against `core/services/example/app/src/settings_schema.rs:7`).
- **Answer, the bug is confirmed by code trace, not on a device.** The trace: default policy into the Snapshot; the
  Kernel loads the file and discards it; `UpdateSettings` decodes into `RecorderCommand::SetPolicy`; `SetPolicy` assigns
  and returns `Decision::new()` with zero effects (`core/services/recorder/logic/policy/src/lib.rs:245`); the Kernel
  runs the empty effect list, acks accepted, and publishes the `settings` State from the App; `persist_blocking`
  (`core/libs/app/service/src/runtime.rs:145`) is never called; a restart returns to the start. No test touches
  `SetPolicy`, settings or `Effect::Persist` in either recorder crate; the only `Effect::Persist` assertions in the
  repository are in the example and in the Kernel's own test domain, so the mechanism is tested and its use is not. To
  confirm on a DUT: read `blueos/v1/recorder/settings`, send `UpdateSettings` with
  `record_mavlink_only_when_armed: false`, check that the settings file on disk still shows `true`, restart the
  container, and observe the revert.
- **Answer, a settings API that cannot be got wrong: leave the service exactly one obligation, which is to say how the
  document relates to the Snapshot.** Everything else moves into the Kernel.

```rust
pub trait SettingsPort {
    type Domain: Domain;
    type Document: SettingsSchema;

    fn into_snapshot(document: Self::Document, snapshot: &mut <Self::Domain as Domain>::Snapshot);
    fn from_snapshot(snapshot: &<Self::Domain as Domain>::Snapshot) -> Self::Document;
    fn updated(document: Self::Document) -> <Self::Domain as Domain>::Command;
}
```

  The Kernel then loads the document once, which it already does, and calls `into_snapshot` before the first command, so
  load cannot be forgotten because the service never builds those Snapshot fields itself. On `UpdateSettings` it
  validates `VERSION` and rejects otherwise, which fixes review M3 once for every service, dispatches
  `Port::updated(document)` through the Inbox, and persists `from_snapshot` unconditionally before the ack, so persist
  cannot be forgotten because the Domain is not responsible for it. It publishes the `settings` State from the same
  `from_snapshot`, so the state cannot disagree with the disk, and it diffs the two documents against
  `restart_required_fields()` and emits `RestartRequired` itself. Both mapping directions are trait methods, so neither
  can be omitted, and `Effect::Persist` leaves the settings path entirely.
- **Recommendation:** the `SettingsPort` trait, plus a Kernel-shipped `assert_settings_round_trip` helper and one
  integration test on the channel backend that sends `UpdateSettings`, restarts the service and asserts the new value is
  in the Snapshot, which catches this class of bug for every future service; plus merging `SetPolicy` into
  `UpdateSettings` so there is one way to change a setting; plus making `auto_start_recording` either live or declared
  restart-required. Stretch: delete `RecorderSettings` entirely and serialize the Domain's own `RecordingPolicy` behind
  a serde feature, which makes the version exist once and removes E13's DTO inside a DTO, at the cost of coupling the
  file format to Domain field names, acceptable only if the migration hook is actually used, and today both services
  return `Ok(())` from it. If the current branch must ship first, take the 5-line fix (load like the example does, add
  `Effect::Persist` to the `SetPolicy` arm) **and** the `auto_start_recording` fix, because a persisted setting that
  still does nothing is a worse user experience than one that obviously resets.

## E12. Overall complexity

**Owner notes**

- The library is too asymmetric. It needs a lot of understanding before one can use it.
- The prototype `ardupilot_manager` was already more complex than wanted. This POC is worse.

**Orchestrator notes**

- The Recorder app has 2,023 lines in 6 files (`lib.rs` 716, `tap.rs` 505, `library_io.rs` 404, `inject.rs` 307,
  `cli.rs` 76, `error.rs` 15). The two logic crates have 2,096 lines. The Kernel runtime alone has 1,207 lines.

**Deep-dive questions**

- Count the concepts that a new developer must learn to add one Command end to end, here and in the prototype.
- What did the POC add over the prototype, and what did each addition buy?
- Propose a target for the next POC: the concept budget, and the shape of a minimal service.

**Deep-dive findings (R6)**

- **Correction, the comparison with the prototype is worse than "the POC is worse".** By raw lines the two are close:
  `ardupilot_manager` is 8541 lines and the Recorder is 7474. But 6223 of the prototype's lines are in `src/stack/`,
  which is real domain work (25 REST endpoints, firmware install, board detection, four flight-controller drivers, SITL,
  process management), and its service shell plus API surface together are 1950 lines. The Recorder serves 12 endpoints
  with a 1114-line shell. Per endpoint that is 93 lines of Recorder wiring against 28 lines of prototype wiring, a 3.3x
  regression on the one thing the Kernel was supposed to make cheap.
- **Correction, the Kernel did not absorb complexity, it relocated it.** The POC added 9116 lines under `core/libs/`
  where the prototype had 4197 lines of `commonwealth`. If those 4919 new framework lines had removed service-level
  work, service wiring would be smaller; it is 3.3x larger.
- **The floor rose 6x while the ceiling stayed flat.** The prototype's smallest complete service is `disk_usage`: 197
  lines, 10 files, 2 crates, 4 endpoints, and a one-line `main.rs`. The POC's smallest is the example at 1187 lines, 19
  files, 3 crates, 8 endpoints, which is 148 lines per endpoint against 49. For a service the size of the prototype's
  `beacon` (181 lines) the three-crate layout would cost more in `Cargo.toml` plumbing than the service contains in
  logic.
- **Six registration strategies are really eleven.** `ServiceBuilder` exposes `command`, `command_allow_empty`,
  `command_opaque`, `query`, `io_query`, `state`, `event`, `io`, plus the special-cased `status`, `jobs` and `settings`
  (`core/libs/app/service/src/builder.rs:165` to `:327`), each with a different closure signature. The prototype has
  one, `routes!`. A developer's first question, "which one do I use?", has eleven answers here and one there.
- **100% of Domains written so far contain dead placeholder types.** `LibraryQuery {}`, `LibraryQueryView`,
  `LibraryJobSpec {}` and a fake `io_from_job` (`core/services/recorder/logic/library/src/lib.rs:157` to `:193`), and
  `RecorderQuery::TapPolicy` with `TapPolicyView`, never registered and never called
  (`core/services/recorder/logic/policy/src/lib.rs:214` to `:222`). That is the trait's shape, not a Recorder mistake.
- **The example's cost structure already predicted the Recorder's.** Nine of its 19 files are needed to express one
  Command's data path, and the Recorder is the same structure at 6x the scale. Nothing went wrong in the Recorder that
  was not visible in the example at 1187 lines; the owner's four hours went on rediscovering which of the eleven
  registration strategies and seven associated types mattered.
- **Answer, what the POC added and what each addition bought.** Earning their place: the IDL, which is the one addition
  that clearly pays, since it buys real ROS 2, Foxglove and MCAP interop; `api.lock`, which turns an API break into a CI
  failure instead of a vehicle failure; the sans-IO Domain, whose tests run with no runtime, no mocks and no sleeps,
  which is genuinely better than anything in the prototype; the single-inbox Kernel, which killed the prototype's raw
  pointers, `unsafe impl Send` and blocking RPC in dispatch (D-19); the `Jobs` graph, for cancellable inspectable
  multi-step flows; the multicall binary; and settings and logging, which were ported from the prototype rather than
  invented. Pure cost: the three-crate layout per service, the eleven registration shapes, the seven mandatory
  associated types, and the missing in-process sender, which is the direct cause of `inject.rs` and of a public
  `Internal` endpoint any browser tab can drive. Those four account for essentially all of the gap measured in P5.
- **Answer, the concept budget is eight:** the `.msg` file and the append-only rule; the Command enum variant; the
  handler that mutates the snapshot and returns a `Decision`; `Decision` itself; `Effect`; the Snapshot; one
  registration mechanism; and `sendCommand` on the frontend. Everything currently on the list of 24 and not in those
  eight must be deleted, defaulted or generated.
- **Answer, the shape of a minimal service: one crate, two modules, with the registration carried by a derive.**

```rust
/// Everything that can change the state. One variant per Command endpoint.
#[derive(Clone, Command)]
pub enum Command {
    #[blueos(message = SetTargetRequest)]
    SetTarget { target: f32 },
    #[blueos(internal)]                   // reachable in-process, not published on the bus
    SampleRead { celsius: f32 },
    #[blueos(timer)]
    SampleTick,
}

/// Wiring. Every endpoint the derives could not infer is one line here.
pub fn build() -> Service<Thermometer> {
    Service::new("thermometer")
        .state::<ThermometerState>()      // field names match the .msg; no selector closure
        .io(adapters::run)
        .on_start(Command::SampleTick)
}

blueos_service::main!(build);             // CLI, runtime, logging, shutdown, exit code
```

  Measured against the P5 targets: one crate, about 70 lines, three concepts visible on screen, two files touched to add
  a Command (the `.msg` and `lib.rs`) and four wiring lines. The layer split becomes a module boundary inside one crate,
  with the existing `no_std` target build (`.hooks/lib/rust_checks.sh:109`) pointed at the module instead of the crate;
  services above roughly 800 lines may opt back into separate crates, and the Recorder would.
- **Recommendation:** adopt the budget of eight and the one-crate shape for small services, and collapse the eleven
  registration methods into a `#[derive(Command)]` on the Command enum, keeping IDL codegen only for the
  Message-to-Domain-type conversions (E8). The derive keeps the registration where the Domain is, which is where a
  reader looks, and the codegen kills the hand-written conversions and the hand-copied constants. Rejected: the
  extractor pattern, which is the idiom most Rust developers know but is itself several hundred lines of framework and
  so fights the framework-read-set target; and IDL-generated registration as the spine, because the naming convention
  becomes load-bearing and renaming a `.msg` silently moves an endpoint. Three risks to state plainly: a derive macro is
  new machinery and P9 forbids undocumented new patterns, so it needs its own decision entry before anyone writes it;
  structural field matching between a `.msg` and an enum variant will produce bad compile errors unless the macro is
  careful, and that error quality is the difference between the proposal working and being resented; and collapsing to
  one crate weakens the mechanical layer check D-02 was proud of, so the `no_std` module build has to be proven to catch
  the same mistakes before the crate split is removed, not after.
- Resolved: R6 proposed default associated types for the `Domain` trait. R2 and R4 compiled it: associated type
  defaults are unstable Rust (E0658). Only a trait split or uninhabited types are possible (see E4 and E7).
- Decided (owner): R4. Split the Recorder's `logic/policy` crate into crates named after what they own (capture,
  cameras, library, schema gate). R6's one-crate layout for small services is not adopted.
- Decided (owner): the R10 hybrid. Each service commits an `endpoints.toml` that generates a handler trait, with no
  proc macro. The generated code is committed and byte-compared (E9 decision), and an `endpoints.lock` gates renames.
  R6's `#[derive(Command)]` and R2's naming-convention routing are not adopted.

**Deep-dive findings (R10)**

- R2's naming convention cannot name the endpoints that exist. Of the Recorder's 12 endpoints, 7 follow one rule
  (strip `Command`). The other 5 need four rules that contradict each other, or none: `recording` from
  `RecordingState`, `library` from `RecordingLibrary`, `operation` from `RecordingOperation`, `index` from
  `RecordingIndexRequest`, and `Internal`, which has no message. In the example, `EmptyRequest` is the request of three
  endpoints (`core/services/example/app/src/lib.rs:72`, `:73`, `:76`). So R2 needs a manifest anyway, and with a
  manifest a renamed `.msg` can no longer move an endpoint silently.
- R6's derive cannot call `ServiceBuilder`, because the Command enum lives in `logic/` and D-02 forbids `logic/` from
  depending on `app/`. It must emit a trait and data too. Both options end in the same shape.
- Derive risks that remain after mitigation: it covers 3 of the 7 wire kinds (3 more macros, 400 to 600 lines), and a
  `.msg` with no variant cannot be detected. Error quality needs four techniques together (`syn::Error::new_spanned`,
  `Error::combine`, `quote_spanned!`, trybuild). Without `quote_spanned!` a field type mismatch points at the
  `#[derive]` line with no field name; without `Error::combine` the second mistake is hidden. Both were reproduced.
- Recommendation: a committed per-service `endpoints.toml` that generates a handler trait, with no proc macro. The
  generated code is committed and byte-compared (E9 decision). A separate `endpoints.lock`, keyed on the wire key with a
  public or internal column, gates renames. rustc reports the mistakes: E0046 "missing: recording" for a forgotten
  handler, E0053 "expected RecordingOperation, found RecordingState" for a wrong type, and E0277 customised with
  `#[diagnostic::on_unimplemented]` (stable since Rust 1.78.0, checked on 1.98.1). Two new concepts for a junior,
  250 to 350 lines of codegen (estimate), no new workspace member or crate.
- Side gains, compiled: an event endpoint becomes one `fn(&Event) -> Option<Message>` and the unreachable
  "event filter mismatch" arm goes (`core/services/recorder/app/src/lib.rs:271`); queries stop being addressed by
  registration index (`runtime.rs:498`); `async fn` satisfies an `impl Future + Send` trait method.
- What would change it: the one-crate layout for small services (rejected by the E7 decision), a Kernel that must give
  a handler more than three or four things, or a service with about 40 endpoints.

## E13. Implicit DTOs, and no dependency injection

**Owner notes**

- The DTOs are implicit rather than explicit.
- The libraries give no tooling for dependency injection, so each service writes and wires everything itself. If Rust
  libraries for injection exist, we should not have to write this ourselves.

**Orchestrator notes**

- The DTOs are the IDL Messages (generated, E9). Each Message is mapped to a Domain type somewhere different:
  - inside the builder closures (`core/services/recorder/app/src/lib.rs:225` to `:257`)
  - in free functions (`recording_*_from_*`, E8)
  - inside the `.io` closure
  - in `inject.rs` (E6)

  No single place lists which Message maps to which Domain type.
- Some DTOs are not in the IDL at all:
  - `InjectedCommand` is a JSON DTO (E6). Its endpoint publishes an empty `request_schema` in `ServiceInfo`
    (`core/libs/app/service/src/builder.rs:200` to `:210`), so clients cannot discover its shape.
  - `RecorderSettings` is a serde DTO (`core/services/recorder/app/src/lib.rs:66`), carried as a JSON string inside the
    `document_json` field of the IDL `SettingsEnvelope` (`core/services/recorder/app/src/lib.rs:588`). It is a DTO
    inside a DTO.
- Dependencies are wired by hand: the Recorder builds `IoContext` and `LibraryIoContext`, creates channels, and clones
  the Session into each task (E5). The Kernel cannot hand dependencies (Session, clock, settings, a task spawner) to a
  handler. The `.io` closure gets a full clone of the App instead of the few things it needs.
- Rust options, checked on crates.io (2026-10-01):

| Option | Kind | Version | Downloads | Last release |
|---|---|---|---|---|
| `shaku` | compile-time container | 0.6.3 | 249 k | 2026-08 |
| `dill` | runtime container | 0.17.1 | 70 k | 2026-09 |
| `nject` | compile-time, zero cost | 0.5.1 | 28 k | 2026-06 |
| `teloc` | compile-time container | 0.2.0 | 13 k | 2021-11 (unmaintained) |
| `froodi` | IoC container | 1.0.0-beta.18 | 5 k | 2026-04 |

- There is also the extractor pattern used by axum and bevy, which needs no container. A handler declares typed
  parameters, such as `State<T>` or `Session`, and the framework supplies them. This is the usual Rust answer to
  dependency injection, and it fits a Kernel that already calls the handlers.

**Deep-dive questions**

- Should the DTOs be explicit? For example: one mapping module per service, `From` and `Into` between each Message and
  its Domain type (E8), and no DTO outside the IDL.
- Compare the extractor pattern with a container crate (`shaku`, `nject`) for the Kernel: code a service has to write,
  compile errors, test doubles (P6), and the cost of a new pattern (P9).

**Deep-dive findings (R2)**

- **Correction:** the crates.io table is unverified. None of `shaku`, `dill`, `nject`, `teloc` or `froodi` is in the
  local registry and crates.io was not queried, so no version, download count or release date in it is confirmed. The
  extractor row is verified: `FromRequestParts` and `FromRequest` are at `axum-core-0.5.6/src/extract/mod.rs:51` and
  `:76`.
- **The pre-push check forbids what D-02 and D-05 allow, and that is the root cause of the orphan-rule problem.** A
  crate in a `logic/` folder may depend only on `libs/logic` or sibling logic in the same service
  (`.hooks/lib/rust_checks.sh:86` to `:89`), and `blueos-idl` lives in `libs/idl`, so `logic -> blueos-idl` is a
  violation, confirmed by running the script's own classifier against the three dependency shapes. But
  `doc/architecture/decisions.md:93` says logic may depend on `blueos-idl`, and D-05 (`:167`) made the crate `no_std`
  plus `alloc` explicitly so that `logic/` may use it; the adapters rule one line below already carves out `libs/idl`
  (`.hooks/lib/rust_checks.sh:90`), so the omission looks accidental. The consequence: the one crate where E8's `From`
  wish is expressible is the crate the gate blocks. R4 found the same contradiction from the type side; see E8.
- **Handlers get everything the Kernel has and nothing they need.** `.io` receives `App<D>` by value
  (`core/libs/app/service/src/builder.rs:320`) and the Kernel clones the whole Snapshot plus the job graph for every
  `Effect::Io` (`core/libs/app/service/src/runtime.rs:973`), while the Recorder's closure uses exactly one field of that
  clone (`core/services/recorder/app/src/lib.rs:297`). Everything it really needs arrives by closure capture through a
  hand-built `IoContext` (`:94`, built at `:180`). Root cause: there is no way for a handler to declare what it needs,
  so the Kernel hands over all it has and the service smuggles the rest in. The Recorder has already written the
  dependency container; the Kernel just does not know about it.
- **Every registration is type-erased at the point of registration**, as `Arc<dyn Fn ...>` in ten builder methods and
  eight type aliases (`core/libs/app/service/src/runtime.rs:43` to `:67`), several with `Pin<Box<dyn Future>>` in the
  return type. Because the erasure happens in the builder and not at the call site, adding one dependency means changing
  the signature of every closure in every service. That is the opposite of injection, and it is why an extractor pattern
  cannot be added incrementally later.
- **There is no seam for a test double, so the Kernel's own tests need 275 lines of scaffolding**: a full fake Domain
  with all seven associated types and a 110-line `spawn_test_service` before the first assertion
  (`core/libs/app/service/tests/kernel.rs:34` to `:325`), shared by seventeen tests. The Recorder cannot be tested at
  this level at all, because its dependencies are constructed inside `run_async`, which also opens a real session and
  spawns three detached tasks. P6 notes the function has no test; this is the structural reason.
- **The settings shape is the one DTO no generated type describes.** `RecorderSettings` is serde-only
  (`core/services/recorder/app/src/lib.rs:66`), and the field list the UI uses to warn about restarts is produced by
  walking `serde_json` object keys at runtime (`core/libs/app/service/src/runtime.rs:1193`). So the restart-required
  contract D-11 promises to put in the IDL (`doc/architecture/decisions.md:295`) is in fact reconstructed from a JSON
  string at publish time.
- **Answer, should the DTOs be explicit: yes, and the blocker is a one-line condition, not Rust.** Fix the hook to allow
  `libs/idl` for logic crates, then add one crate per service, `logic/api`, that depends on `blueos-idl` and on the
  service's own logic crates and contains only `From` impls; the script's existing "sibling logic in the same service"
  clause already permits it, so no further gate change is needed. That keeps the pure Domain crate IDL-free, which
  preserves the D-03 purity claim, and keeps the app crate mapping-free. About 100 lines move out of the app crate and
  become `From` impls in one file a reader can list: the four `recording_*` functions, `library_state_to_idl`,
  `settings_policy`, `settings_from_snapshot`, and the seven Request-to-Command closures. Pair it with generated Rust
  enums for the IDL constant groups (E8) and an IDL-declared settings document, which also deletes the runtime key walk.
  Rejected: gathering the mappings into one `mapping.rs` inside the app crate, which is zero-risk but keeps them free
  functions with nothing enforcing completeness; and putting IDL types in the Snapshot, which is smaller still but pins
  the Domain to the wire format so a D-06 constraint becomes a Domain constraint.
- **Answer, extractors against a container: no container crate.** The services do not have the problem a container
  solves. A container pays off with a deep graph of swappable implementations behind traits; the Recorder has exactly
  two IO contexts and the example has none, so adding a dependency to inject two structs is the wrong trade, and
  `teloc` being unmaintained is a reminder of the maintenance risk in that niche (unverified figures). The extractor
  pattern is genuinely better ergonomics and is the pattern a Rust developer is most likely to know already, but it
  costs a macro, an arity story, compile errors bad enough that axum needs `#[diagnostic::on_unimplemented]` to make
  them readable (`axum-core-0.5.6/src/extract/mod.rs:48`), and a new concept to document under P9. It is worth adopting
  the day the Kernel has many different things to supply to many different handlers; today it has one, `App`, and the
  service supplies everything else.
- **Recommendation:** pass a service-defined `Context` by reference and stop cloning `App` into IO.

```rust
// the service owns this; it is today's IoContext, unchanged
struct RecorderContext { /* paths, mcap config, session mutex, writer watch, comms, library */ }

builder
    .context(RecorderContext::new(...)?)
    .io(|context: &RecorderContext, snapshot: &RecorderSnapshot, request| async move {
        match request { /* as today, minus the Arc clone dance */ }
    })
```

  Three things fall out: the `Arc`-clone-into-closure ritual (`core/services/recorder/app/src/lib.rs:294`) goes away;
  the per-IO `App` clone becomes a borrow of the one field the handler reads, which also helps review M9; and a test
  builds a `RecorderContext` with a `tempdir` and calls the handler directly, which is the missing seam. It is one new
  concept, it is a struct the Recorder already wrote, and it has no macro, no container and no new failure mode.
  Revisit extractors only if the Kernel grows past three or four things to supply, and if it does, copy axum's
  diagnostic attribute from the start rather than after the first junior hits the error. Ordering note: the `Context`
  change and E4's registration-shape change touch the same signatures, so do `Context` first, because it is smaller,
  independently valuable, and whichever registration shape wins has to carry a dependency parameter anyway.

---

# Summary of recommendations

| ID | Recommendation (one line) | Source |
|---|---|---|
| P1 | Split `build()` from `run()` so the Kernel is a value the Service owns, then grow it into one `Service` trait entry. | R1, R2 |
| P2 | Call `init` on the first line of the entry, buffer the early records, and add a timestamp to `LogRecord`. | R1 |
| P3 | Supervise tasks in the Kernel and recover the Inbox loop with the H2 copy, keeping "unrecoverable means exit non-zero". | R3 |
| P4 | One `example-minimal` under 150 lines plus a compiled cookbook entry per "how do I do X" question, checked by a test. | R6 |
| P5 | Adopt the six measures (concepts, files, wiring lines, service floor, framework read set, time) and enforce three in the hook. | R6 |
| P6 | Retrofit paused-clock tests and a `build()` seam now, then design the next Kernel around an L3 harness. | R7 |
| P7 | Finish the gates already paid for (all four `cargo deny` checks, `[workspace.lints]`, the missing license), then the extras. | R7 |
| P8 | Seven CI jobs with prebuilt tool installs, per-layer coverage floors and `cargo auditable`; sanitizers stay report-only. | R7 |
| P9 | One `rust-style.md` with a checklist mirrored into `AGENTS.md` and a committed Cursor rule, plus lints and a `syn` checker. | R8 |
| P10 | Fix the shared capture timer and the session flag, enum for stored state, type-state for builders and handles. | R9 |
| P11 | `.io` takes `&App<D>`, one descriptor per channel not per sample, `Arc::clone` for handles, clone lints on. | R9 |
| P12 | No framework fits; adopt `tokio-util` tasks and timers, `backon`, `zenoh-keyexpr`, `roslibrust_codegen`, `pycdr2`. | R11 |
| E1 | Resolve the name once, keep a never-gated list of known names, `#[cfg]` on the `match` arms, one entry signature. | R1 |
| E2 | Split the bootstrap into a pure `build()` and a thin `run()`, with one fixed `app/src/` layout per service. | R1 |
| E3 | One Kernel-parsed common CLI that each service extends with `type Arguments: clap::Args`. | R1 |
| E4 | `Decision` over three types, split `Domain`, `.io -> Result<Option<Command>, IoError>`, `.settings -> Self`, Kernel defaults, then a committed `endpoints.toml` that generates a handler trait. | R2, R10 |
| E5 | A Kernel-owned task registry with handles, shutdown ordering, restart policy and health in `status`, plus sequential `Io` effects. | R3 |
| E6 | Create the Inbox pair in the builder, hand out `CommandSender<D>`, and delete `inject.rs` and the `Internal` endpoint. | R3 |
| E7 | Uninhabited types now, then `Outcome<Event, Command, IoRequest>` with one Domain and four Blocks, plus the renames. | R4 |
| E8 | Fix the layer gate and D-02 together, keep one `SystemAndComponent`, generate enums from `.msg` constants, use `From`. | R4 |
| E9 | Format the generated Rust, commit it behind an explicit regeneration command and a byte comparison, delete `build.rs`. | R5 |
| E10 | Take chrono with `default-features = false, features = ["alloc"]` in the logic crates, fix the `.mcap` rule first, one Clock. | R5 |
| E11 | A `SettingsPort` trait where the Kernel owns load, persist, publish and version validation, plus one restart test. | R4 |
| E12 | A budget of eight concepts: one crate for small services, one registration shape, four fewer associated types, an in-process sender. | R6 |
| E13 | Explicit DTOs in one `logic/api` crate per service, and `builder.context(T)` instead of a container or extractors. | R2 |

---

# Known bugs left for the next draft

The owner decided not to fix these in this POC. The next draft must cover each one with a test first (P6).

| Bug | Where | Status |
|---|---|---|
| All video streams share one capture status timer, so only the last camera gets ticks, and stopping one camera cancels the other's timer. | `core/services/recorder/logic/policy/src/lib.rs:228`, `:458`; `core/libs/app/service/src/runtime.rs:313` | confirmed by test (R9) |
| On rotation, a late `SessionFinished` wipes the new session but leaves `session_active` true, so every `SessionBytesWritten` is dropped. | `policy/src/lib.rs:332`; `runtime.rs:967` | confirmed by test (R9) |
| `CaptureStatusTick` reschedules with the same `now_millis`, so `recording_time_ms` is always 0. | `policy/src/lib.rs:400` | confirmed by test (R9) |
| The rotation race may make `FinishSession` close the file `OpenSession` just created. | `core/services/recorder/app/src/lib.rs:329`, `:333` | code reading only, needs a device |

---

# Deep-dive reviews

| Reviewer | Themes | Status |
|---|---|---|
| R1 | P1, P2, E1, E2, E3: ownership, early logging, entry point, bootstrap, CLI | merged |
| R2 | P1, E4, E13: builder and API shape, Zenoh as an Adapter, DTOs and dependency injection | merged |
| R3 | P3, E5, E6: panic recovery, wiring, tasks, lifecycle, `Internal` | merged |
| R4 | E7, E8, E11: ontology, types, settings | merged |
| R5 | E9, E10: generated code, reinvented code, `no_std` | merged |
| R6 | P4, P5, E12: example coverage, ergonomics measures, complexity against the prototype | merged |
| R7 | P6, P7, P8: testability, the TDD plan, coverage, and the code quality tool set in CI | merged |
| R8 | P9: guidance for developers and agents, and what tools can enforce | merged |
| R9 | P10, P11: newtypes and type-driven state machines, borrowing before cloning | merged |
| R10 | E4 decision: mitigations for the derive macro and for IDL-generated routing | merged |
| R11 | P12: mature projects that could replace our own mechanisms | merged |
