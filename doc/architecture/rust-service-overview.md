# BlueOS Rust service: how it works

This is a guided tour of a BlueOS Rust service. It follows the teaching example (`core/services/example`) and the
shared kernel (`core/libs/app/service`) that every Rust service uses.

- The reasons behind each design choice are in `doc/architecture/decisions.md`.
- Known bugs and problems found while writing this tour are in `doc/architecture/rust-service-review.md`.
- References are `path:line` from the repository root, as of commit `17a3f639d`.

# Part 1: The big picture

## 1.1 Vocabulary

These terms keep the same meaning in the whole text.

| Term | Meaning | Defined at |
|---|---|---|
| **Service** | One process. It runs one Domain inside the Kernel. | `core/services/example/` |
| **Domain** | Pure logic. It has no IO, no async code, and no clock. | `core/libs/logic/cqrs/src/lib.rs:122` |
| **Snapshot** | The data of the Domain (its state). | `core/services/example/logic/pump/src/snapshot.rs:8` |
| **Jobs** | A tree of multi-step work that is tracked next to the Snapshot. | `core/libs/logic/jobs/src/lib.rs:81` |
| **App** | Snapshot + Jobs. This is the only mutable state of a service. | `core/libs/logic/cqrs/src/lib.rs:157` |
| **Kernel** | The async runtime that owns the App. | `core/libs/app/service/src/runtime.rs:331` |
| **Inbox** | The one queue into the Kernel loop. The only way to reach the App. | `core/libs/app/service/src/runtime.rs:437` |
| **Adapter task** | An async task that converts a Zenoh request into an Inbox message. | `core/libs/app/service/src/runtime.rs:782` |
| **Session** | The connection to Zenoh. | `core/libs/adapters/comms/src/lib.rs:140` |
| **Message** | An IDL type (a ROS 2 `.msg`). It travels as CDR bytes. | `core/libs/idl/interfaces/` |
| **Command** | A Domain input value that may change the App (for example `PumpCommand`). | `core/services/example/logic/pump/src/command.rs:18` |
| **Decision** | The Domain output: domain events + Effects, or a rejection. | `core/libs/logic/cqrs/src/lib.rs:65` |
| **Effect** | An order from the Domain to the Kernel: `Io`, `Schedule`, `CancelSchedule`, `Persist`. | `core/libs/logic/cqrs/src/lib.rs:100` |
| **domain event** | A `PumpEvent` value. It exists only inside the process. | `core/services/example/logic/pump/src/event.rs:4` |
| **Registration** | A closure stored in `ServiceBuilder`. It converts between Messages and Domain types. | `core/libs/app/service/src/builder.rs:25` |

A service shows four endpoint kinds to the outside world:

| Endpoint | Direction | Zenoh primitive | Key |
|---|---|---|---|
| **Command endpoint** | client to service: "change something" | queryable, replies `CommandAck` | `blueos/v1/<service>/command/<Name>` |
| **Query endpoint** | client to service: "tell me something" | queryable, replies a Message | `blueos/v1/<service>/query/<Name>` |
| **State** | service to clients: "the current value" | publish on change + queryable for late clients | `blueos/v1/<service>/state/<name>` |
| **Event** | service to clients: "something happened" | publish only | `blueos/v1/<service>/event/<Name>` |

## 1.2 What a BlueOS service is

A BlueOS service is a process that:

- holds a state,
- changes that state only when a Command arrives,
- tells the world about the changes.

It talks to other programs only through Zenoh.

The code has three layers, and the folder decides the layer (`doc/architecture/decisions.md:87`):

```text
            Zenoh
              ^ |
              | v
+-------------------------------+
| app/    Kernel + Registrations|  core/libs/app/service/, core/services/example/app/
|         (async)               |
+-------------------------------+
         ^ Decision   | Command
         |            v
+-------------------------------+
| logic/  Domain (pure, no_std) |  core/services/example/logic/pump/
+-------------------------------+

  adapters/  IO code (devices)     core/services/example/adapters/simulated_pump/
```

**The core rule:** logic never waits (`doc/architecture/decisions.md:117`). When the Domain needs IO or time, it
returns an Effect. The Kernel does the work and sends the result back into the Inbox as a new Command.

## 1.3 The five mechanisms in one picture

```text
                 zenohd (router)
  requests  |                  ^  State / Event / log Messages
            v                  |
      [Adapter tasks]     [Publishers]
            |                  ^
            v                  |
      ===== Inbox loop (one message at a time) =====
            |                  ^
     handle(Command)       Decision
            v                  |
      [ App = Snapshot + Jobs, run by Domain ]
                    |
                 Effects
    Io -------> io task ----+
    Schedule -> timer task -+--> new Command --> Inbox
    Persist --> settings file (the Kernel waits)
```

1. **Initialization:** build the App, store the Registrations, connect to Zenoh, start the Adapter tasks, then start
   the Inbox loop.
2. **External communication:** Zenoh goes through one Session. Device IO goes through `Effect::Io`. The settings file
   goes through `Effect::Persist`.
3. **Command pipeline:** request, then Command, Inbox, Domain, Decision, Effects, ack, and finally publish.
4. **Query pipeline:** request, then Inbox, read the App, and reply. It never changes the App.
5. **Event emission:** domain events are collected while a Command runs. After the ack, they are filtered, encoded,
   and published.

Only the Inbox loop touches the App. So the App needs no locks, and all changes happen in a strict order.

---

# Part 2: Each mechanism in detail

## 2.1 Initialization

```text
main (multicall binary)          chooses the service by name
 -> example::run                 CLI + tokio runtime
    -> run_async
       load settings -> Snapshot
       App::new(Snapshot)
       ServiceBuilder + Registrations   (stores closures only)
       builder.run()
        -> Session::open         connect to zenohd
        -> logging init
        -> Kernel run()
             declare: liveliness, log, States, info, UpdateSettings
             create:  Inbox, persist worker
             publish all States once
             spawn:   Adapter tasks
             send on_start Commands to the Inbox
             Inbox loop (until shutdown)
```

**Node by node:**

1. **Binary entry.** `core/app/blueos/src/main.rs:5`. There is one binary for all services. It chooses the service
   by `argv[0]` (a symlink name) or by the first argument (`core/app/blueos/src/main.rs:11`). Then it calls
   `blueos_example::run` (`core/app/blueos/src/main.rs:34`).
2. **Service entry.** `core/services/example/app/src/lib.rs:36`. It parses the CLI (`--config`, `-v`; see
   `core/libs/adapters/cli/src/lib.rs:15`), builds the tokio runtime (`core/services/example/app/src/lib.rs:38`), and
   runs `run_async` (`core/services/example/app/src/lib.rs:47`).
3. **Initial Snapshot.**
   - Settings are loaded at `core/services/example/app/src/lib.rs:49`, then `core/services/example/app/src/lib.rs:129`,
     then `core/libs/adapters/settings/src/manager.rs:33`. The file is `settings-<VERSION>.json` and uses the same
     format as Python.
   - `App::new` runs at `core/services/example/app/src/lib.rs:50`, then `core/libs/logic/cqrs/src/lib.rs:173`. It
     creates the Snapshot and empty Jobs.
4. **Registrations.** These are at `core/services/example/app/src/lib.rs:61` to
   `core/services/example/app/src/lib.rs:122`. Each builder method only stores a closure:

| Builder method | Stores | Defined at |
|---|---|---|
| `.command` / `.command_allow_empty` | request bytes to Command | `core/libs/app/service/src/builder.rs:165`, `core/libs/app/service/src/builder.rs:183` |
| `.query` | request bytes + `&App` to response bytes | `core/libs/app/service/src/builder.rs:215` |
| `.io_query` | request bytes to async response (no App) | `core/libs/app/service/src/builder.rs:235` |
| `.state` / `.status` / `.jobs` | `&App` to State Message | `core/libs/app/service/src/builder.rs:261`, `core/libs/app/service/src/builder.rs:296`, `core/libs/app/service/src/builder.rs:307` |
| `.event` | filter + domain event to Message | `core/libs/app/service/src/builder.rs:278` |
| `.io` | IO executor: `IoRequest` to async Command | `core/libs/app/service/src/builder.rs:318` |
| `.settings` | settings runtime (loads a second `SettingsManager` that is used for writing) | `core/libs/app/service/src/builder.rs:143`, then `core/libs/app/service/src/runtime.rs:108` |
| `.on_start` / `.on_shutdown` | Commands for start and stop | `core/libs/app/service/src/builder.rs:79`, `core/libs/app/service/src/builder.rs:85` |

5. **Connect.** `core/libs/app/service/src/builder.rs:330` calls `Session::open` at
   `core/libs/app/service/src/builder.rs:334`. Tests pass a channel Session instead. The call goes to
   `core/libs/adapters/comms/src/lib.rs:153`, then `core/libs/adapters/comms/zenoh/src/lib.rs:31`, then
   `core/libs/adapters/comms/zenoh/src/config.rs:3`:
   - If `ZENOH_CONFIG` is set, it uses that file (`core/libs/adapters/comms/zenoh/src/config.rs:4`).
   - Otherwise it runs in client mode to `tcp/127.0.0.1:7447`, with shared memory on
     (`core/libs/adapters/comms/zenoh/src/config.rs:29`).
6. **Before the Kernel starts.** `core/libs/app/service/src/builder.rs:339`:
   - It fills `ServiceInfo.endpoints` from the Registrations (`core/libs/app/service/src/builder.rs:350`, then
     `core/libs/app/service/src/runtime.rs:179`).
   - It installs tracing (`core/libs/app/service/src/builder.rs:365`, then
     `core/libs/adapters/logging/src/lib.rs:23`).
7. **Kernel `run`.** `core/libs/app/service/src/runtime.rs:331`. It does these steps in this order:
   - liveliness token `blueos/v1/services/<name>` (`core/libs/app/service/src/runtime.rs:356`)
   - log publisher (`core/libs/app/service/src/runtime.rs:360`)
   - State handles: status (`core/libs/app/service/src/runtime.rs:362`), jobs
     (`core/libs/app/service/src/runtime.rs:372`), settings (`core/libs/app/service/src/runtime.rs:382`), custom
     States (`core/libs/app/service/src/runtime.rs:404`)
   - the standard `UpdateSettings` Command endpoint (`core/libs/app/service/src/runtime.rs:388`)
   - the `info` queryable, in its own task (`core/libs/app/service/src/runtime.rs:412`)
   - the Inbox, an mpsc channel of size 256 (`core/libs/app/service/src/runtime.rs:437`)
   - the persist worker (`core/libs/app/service/src/runtime.rs:440`)
   - the first publish of all States, so late clients always find a value
     (`core/libs/app/service/src/runtime.rs:496`)
   - the Adapter tasks: queries (`core/libs/app/service/src/runtime.rs:498`), IO queries
     (`core/libs/app/service/src/runtime.rs:508`), commands (`core/libs/app/service/src/runtime.rs:517`)
   - the CLI and `on_start` Commands, sent into the Inbox (`core/libs/app/service/src/runtime.rs:526`,
     `core/libs/app/service/src/runtime.rs:538`)
   - the shutdown signal and the Inbox loop (`core/libs/app/service/src/runtime.rs:554`,
     `core/libs/app/service/src/runtime.rs:556`)

**What the example depends on:**

| Crate | Layer | Role |
|---|---|---|
| `blueos-example-pump` | logic | the Domain |
| `blueos-cqrs` | logic | `Domain`, `App`, `Decision`, `Effect` |
| `blueos-jobs` | logic | the job graph |
| `blueos-idl` | logic | Messages and the CDR codec |
| `blueos-api` | shared lib | key names (`core/libs/api/src/lib.rs:22`) |
| `blueos-comms` + zenoh driver | adapters | Session |
| `blueos-settings` | adapters | the settings file |
| `blueos-logging` | adapters | tracing and log publishing |
| `blueos-cli` | adapters | CLI arguments |
| `blueos-example-simulated-pump` | adapters | fake hardware |
| `blueos-service` | app | the Kernel |

At runtime the service needs a `zenohd` on `127.0.0.1:7447` and a writable settings folder.

## 2.2 How the service talks to the outside world

There are exactly three ways out:

1. **Zenoh**, through the Session. This carries all Messages.
2. **`Effect::Io`**, through the IO executor. This is for devices and other IO.
3. **`Effect::Persist`**, through the persist worker. This writes the settings file.

```text
 Kernel code
    |  publish / declare_queryable / declare_state / declare_liveliness
 Session                    facade, cheap to clone
    |
 Backend = Zenoh | Channel  (Channel is for tests only)
    |
 zenoh::Session (client) -> zenohd tcp/127.0.0.1:7447
    |
 browser (zenoh-ts), other services, Recorder
```

**Node by node:**

1. **Session** (`core/libs/adapters/comms/src/lib.rs:140`) wraps a `Backend` enum
   (`core/libs/adapters/comms/src/lib.rs:55`). The contract is the `CommsBackend` trait
   (`core/libs/adapters/comms/driver/src/lib.rs:145`).
2. **The four primitives the Kernel uses:**
   - `publish(key, payload, encoding)` at `core/libs/adapters/comms/src/lib.rs:177` calls Zenoh `put` at
     `core/libs/adapters/comms/zenoh/src/lib.rs:46`. It is used for State, Event, and log.
   - `declare_queryable(key)` is at `core/libs/adapters/comms/src/lib.rs:193`. A task at
     `core/libs/adapters/comms/zenoh/src/lib.rs:94` converts each Zenoh query into an `IncomingQuery`
     (`core/libs/adapters/comms/driver/src/lib.rs:71`), which has a responder. The reply always uses the declared
     key, so wildcard `get` calls can tell replies apart (`core/libs/adapters/comms/zenoh/src/query_responder.rs:23`).
   - `declare_state(key)` at `core/libs/adapters/comms/src/lib.rs:224` returns a `StateHandle`
     (`core/libs/adapters/comms/src/state.rs:21`). It combines a queryable and a publisher with a cached last value.
   - `declare_liveliness(key)`: the token stays alive while the process is alive.
3. **Wire format.** The payload is the CDR bytes of a Message. The encoding string is
   `application/cdr;<package>/msg/<Name>` (`core/libs/api/src/lib.rs:67`), built by `message_to_payload`
   (`core/libs/app/service/src/builder.rs:390`). There is no framing (`doc/architecture/decisions.md:254`).
4. **Device IO.** `Effect::Io` calls the `.io` closure (`core/services/example/app/src/lib.rs:110`). That closure
   calls `io_to_command` (`core/services/example/app/src/lib.rs:192`), which calls `run_step`
   (`core/services/example/adapters/simulated_pump/src/lib.rs:16`).

**All keys of the example service:**

| Kind | Key | Message | Registered at |
|---|---|---|---|
| liveliness | `blueos/v1/services/example` | none | `core/libs/app/service/src/runtime.rs:356` |
| Query | `blueos/v1/example/query/info` | `ServiceInfo` | `core/libs/app/service/src/runtime.rs:412` |
| State | `blueos/v1/example/state/status` | `ServiceStatus` | `core/services/example/app/src/lib.rs:66` |
| State | `blueos/v1/example/jobs` | `JobList` | `core/services/example/app/src/lib.rs:67` |
| State | `blueos/v1/example/settings` | `SettingsEnvelope` | `core/services/example/app/src/lib.rs:113` |
| State | `blueos/v1/example/state/pump` | `PumpState` | `core/services/example/app/src/lib.rs:84` |
| Command | `blueos/v1/example/command/SetLevel` | `SetLevelRequest` | `core/services/example/app/src/lib.rs:68` |
| Command | `blueos/v1/example/command/StartSelfTest` | `EmptyRequest` | `core/services/example/app/src/lib.rs:73` |
| Command | `blueos/v1/example/command/CancelSelfTest` | `EmptyRequest` | `core/services/example/app/src/lib.rs:74` |
| Command | `blueos/v1/example/command/UpdateSettings` | `SettingsEnvelope` | `core/libs/app/service/src/runtime.rs:388` |
| Query | `blueos/v1/example/query/Level` | `LevelQueryResponse` | `core/services/example/app/src/lib.rs:77` |
| Event | `blueos/v1/example/event/SelfTestCompleted` | `SelfTestCompleted` | `core/services/example/app/src/lib.rs:85` |
| Event | `blueos/v1/example/event/RestartRequired` | `RestartRequired` | `core/services/example/app/src/lib.rs:98` |
| log | `blueos/v1/example/log` | `foxglove_msgs/Log` | `core/libs/app/service/src/runtime.rs:360` |

**The client side (frontend):**

| Function | Defined at |
|---|---|
| `sendCommand` | `core/frontend/src/libs/blueos-api/command.ts:9` |
| `query` | `core/frontend/src/libs/blueos-api/query.ts:7` |
| `watchState` | `core/frontend/src/libs/blueos-api/state.ts:18` |
| `watchJobs` | `core/frontend/src/libs/blueos-api/jobs.ts:19` |

## 2.3 The Command pipeline

```text
client: get(command key, CDR bytes)
 [1] Zenoh queryable task              -> IncomingQuery
 [2] Command Adapter task: decode      -> Command
       decode fails -> CommandAck{accepted:false}   (Domain never runs)
 [3] Inbox <- InboxMessage::Command{Command, reply channel}
 [4] Inbox loop -> dispatch_command
 [5] App::handle -> Domain::handle_command -> Decision
       rejected -> CommandAck{accepted:false}, nothing is published
 [6] App::handle adds one Effect::Io per new runnable job leaf
 [7] Kernel runs Effects (Io, Schedule, CancelSchedule, Persist)
 [8] CommandAck{accepted:true, job_id} -> Adapter task -> Zenoh reply
 [9] publish all States, then publish Events
```

**Node by node:**

1. **Zenoh queryable task.** `core/libs/adapters/comms/zenoh/src/lib.rs:94`. It reads each Zenoh query, copies the
   payload, and pushes an `IncomingQuery` into a stream.
2. **Command Adapter task.** `core/libs/app/service/src/runtime.rs:782`. There is one task per Command key. It
   declares the queryable at `core/libs/app/service/src/runtime.rs:791`.
   - It decodes at `core/libs/app/service/src/runtime.rs:796`, using the Registration closure from
     `core/libs/app/service/src/builder.rs:174`: first `decode_strict`
     (`core/libs/app/service/src/request.rs:14`), then your mapping (for example
     `core/services/example/app/src/lib.rs:68`, which creates `PumpCommand::SetLevel`).
   - If decoding fails, it replies with a rejected ack right away (`core/libs/app/service/src/runtime.rs:797`).
3. **Into the Inbox.** `core/libs/app/service/src/runtime.rs:812`. The task creates a one-shot reply channel and
   sends `InboxMessage::Command` (`core/libs/app/service/src/runtime.rs:296`). Then it waits for the ack
   (`core/libs/app/service/src/runtime.rs:826`).
4. **Inbox loop.** `core/libs/app/service/src/runtime.rs:556`:
   - It waits on either the Inbox or the shutdown signal (`core/libs/app/service/src/runtime.rs:577`).
   - The Command branch is at `core/libs/app/service/src/runtime.rs:598`. During shutdown, it rejects the Command.
   - Otherwise it calls `dispatch_command` (`core/libs/app/service/src/runtime.rs:609`).
5. **Domain.** `dispatch_command` (`core/libs/app/service/src/runtime.rs:945`) calls `App::handle` at
   `core/libs/app/service/src/runtime.rs:958`. The call chain is:
   - `core/libs/logic/cqrs/src/lib.rs:181`
   - `core/services/example/logic/pump/src/domain.rs:21`
   - `core/services/example/logic/pump/src/command.rs:27`

   For example, `handle_start_self_test` (`core/services/example/logic/pump/src/command.rs:60`) changes the Snapshot,
   adds the job graph with `jobs.enqueue` (`core/services/example/logic/pump/src/command.rs:70`), and returns an
   `Effect::Schedule` for the timeout (`core/services/example/logic/pump/src/command.rs:74`).

   A rejection looks like `Decision::reject` (`core/services/example/logic/pump/src/command.rs:88`). The Kernel then
   returns early (`core/libs/app/service/src/runtime.rs:959`) and sends a rejected ack
   (`core/libs/app/service/src/runtime.rs:626`).
6. **Jobs become IO.** At `core/libs/logic/cqrs/src/lib.rs:185`, after the handler, `poll_runnable`
   (`core/libs/logic/jobs/src/lib.rs:180`) marks the ready leaves as Running. Each ready leaf becomes an `Effect::Io`
   through `io_from_job` (`core/services/example/logic/pump/src/domain.rs:42`).
7. **Effects.** The domain events are saved first (`core/libs/app/service/src/runtime.rs:964`). Then each Effect runs:
   - `Io` (`core/libs/app/service/src/runtime.rs:967`): the Kernel clones the App, spawns a task
     (`core/libs/app/service/src/runtime.rs:976`), and sends the result back as `InboxMessage::IoComplete`
     (`core/libs/app/service/src/runtime.rs:981`). It never blocks the loop.
   - `Schedule` (`core/libs/app/service/src/runtime.rs:987`): the Kernel replaces any timer with the same id and
     starts a sleep task (`core/libs/app/service/src/runtime.rs:998`). When the sleep ends, it sends the Command
     with no reply channel (`core/libs/app/service/src/runtime.rs:1003`).
   - `CancelSchedule` (`core/libs/app/service/src/runtime.rs:1017`): the Kernel sets a cancel flag and aborts the
     task. A timer that already fired can still deliver its Command (review M1).
   - `Persist` (`core/libs/app/service/src/runtime.rs:1023`): the Kernel sends the job to the persist worker and
     **waits** for it (`core/libs/app/service/src/runtime.rs:1038`). So an ack means the settings are on disk. The
     worker writes on a blocking thread (`core/libs/app/service/src/runtime.rs:145`). For what happens when the write
     fails, see review H2.
8. **Ack.** The ack is built at `core/libs/app/service/src/runtime.rs:619` by `accepted_ack`
   (`core/libs/app/service/src/runtime.rs:1142`). The Adapter task encodes it and replies to Zenoh
   (`core/libs/app/service/src/runtime.rs:829`).
9. **Publish.** States are published at `core/libs/app/service/src/runtime.rs:623` and Events at
   `core/libs/app/service/src/runtime.rs:624` (see section 2.5).

**The loop back (how a job graph moves forward):**

```text
Effect::Io -> io task -> Command(JobIoFinished) -> Inbox (IoComplete) -> [5]
                                                              |
                          jobs.complete -> poll_runnable -> next Effect::Io
```

1. The IoComplete branch is at `core/libs/app/service/src/runtime.rs:648`. It calls the same `dispatch_command`.
2. The Domain handles the result in `handle_job_io_finished` (`core/services/example/logic/pump/src/command.rs:124`),
   which calls `jobs.complete` (`core/services/example/logic/pump/src/command.rs:134`).
3. Back in `App::handle`, `poll_runnable` finds the next leaf of the `Sequence`
   (`core/services/example/logic/pump/src/jobs.rs:33`).

**Example timeline for `StartSelfTest`:**

```text
t0  request -> Snapshot.phase=RUNNING, enqueue [VerifyOff, RampUp, VerifyOn],
               Schedule(timeout) + Io(VerifyOff); ack; publish States
t1  IoComplete(VerifyOff ok) -> Io(RampUp)
t2  IoComplete(RampUp ok)    -> Io(VerifyOn)
t3  IoComplete(VerifyOn ok)  -> root job Succeeded -> SelfTestCompleted, CancelSchedule
    (if the timer fires first: SelfTestTimedOut,
     core/services/example/logic/pump/src/command.rs:150)
```

## 2.4 The Query pipeline

```text
client: get(query key, CDR bytes or empty)
 [1] Zenoh queryable task           -> IncomingQuery
 [2] Query Adapter task -> Inbox <- InboxMessage::Query{index, bytes, reply channel}
 [3] Inbox loop: handler[index](bytes, &App)
       decode -> your closure -> App::query -> Domain::handle_query -> View
       -> response Message -> CDR
 [4] reply channel -> Adapter task -> Zenoh reply (or reply_error)
```

A Query only reads `&App`. It produces no Decision, no Effects, and no publish.

**Node by node:**

1. **Zenoh queryable task.** This is the same as step 1 of the Command pipeline.
2. **Query Adapter task.** `core/libs/app/service/src/runtime.rs:892`. It does not decode anything. It sends the raw
   bytes and the handler index (`core/libs/app/service/src/runtime.rs:905`). The index is the registration order
   (`core/libs/app/service/src/runtime.rs:498`).
3. **Inside the Inbox loop.** `core/libs/app/service/src/runtime.rs:670`. It calls the handler at
   `core/libs/app/service/src/runtime.rs:682`. The handler was built at `core/libs/app/service/src/builder.rs:225`
   and runs these steps:
   - `decode_allow_empty_body` (`core/libs/app/service/src/request.rs:4`)
   - your closure (`core/services/example/app/src/lib.rs:77`)
   - `App::query` (`core/libs/logic/cqrs/src/lib.rs:195`)
   - `handle_query` (`core/services/example/logic/pump/src/domain.rs:29`), which returns a `PumpQueryView`
   - your closure turns the View into a `LevelQueryResponse`
4. **Reply.** `core/libs/app/service/src/runtime.rs:918`.

Queries go through the Inbox together with Commands. So a Query always sees the App after all earlier Commands.

**The four kinds of reads:**

| Read | Path | Reads the App? | Uses the Inbox? | Defined at |
|---|---|---|---|---|
| Query | Inbox | yes | yes (a short turn) | `core/libs/app/service/src/runtime.rs:892` |
| IO query | its own task, one request at a time | no | no | `core/libs/app/service/src/runtime.rs:849` |
| State `get` | StateHandle cache | only the last published copy | no | `core/libs/adapters/comms/src/state.rs:27` |
| `info` | its own task | no | no | `core/libs/app/service/src/runtime.rs:412` |

## 2.5 Event emission

```text
Domain::handle_command -> Decision.events       (domain events)
 [1] collect: pending_events += events
 [2] Command applied -> ack sent -> States published
 [3] for each domain event:
       for each Event Registration:
         filter(event)? no -> skip
         encode(event)     -> Message -> CDR
         Session.publish(blueos/v1/<service>/event/<Name>)
```

**Node by node:**

1. **Created.** The Domain puts domain events in the Decision. For example, `SelfTestCompleted` at
   `core/services/example/logic/pump/src/command.rs:176` and `RestartRequired` at
   `core/services/example/logic/pump/src/command.rs:113`.
2. **Collected.** `core/libs/app/service/src/runtime.rs:964`. The events wait in `pending_events`.
3. **Triggered.** Only when a Command is applied:
   - from a client or a timer: `core/libs/app/service/src/runtime.rs:624`
   - from IoComplete: `core/libs/app/service/src/runtime.rs:659`
   - from the shutdown Command: `core/libs/app/service/src/runtime.rs:769`
4. **Filtered and encoded.** `core/libs/app/service/src/runtime.rs:1074`. It uses the Registration closures from
   `core/libs/app/service/src/builder.rs:278` (example: `core/services/example/app/src/lib.rs:85`).
5. **Published.** `core/libs/app/service/src/runtime.rs:1086`, then `core/libs/adapters/comms/src/lib.rs:177`, then
   Zenoh `put`.

**Important properties:**

- **Order.** The client sees the ack first, then the new States, then the Events
  (`core/libs/app/service/src/runtime.rs:619` to `core/libs/app/service/src/runtime.rs:624`). A client that reads a
  State right after the ack can still get the old value (review M7).
- **Unregistered events stay private.** A domain event with no Registration never leaves the process. In the example,
  `LevelChanged` and `SelfTestStarted` are private.
- **Events have no memory.** An Event has no queryable. A client that is not subscribed at that moment misses it. For
  "what is true now", use a State.

**Compared with State publishing** (same moment, different rules): `publish_all_states`
(`core/libs/app/service/src/runtime.rs:1052`) runs every State selector after every applied Command.
`StateHandle::publish` skips the publish when the bytes did not change (`core/libs/adapters/comms/src/state.rs:57`).
It also caches the value for late `get` calls (`core/libs/adapters/comms/src/state.rs:27`).

**Logs** are a third output stream. Every `tracing` record goes to
`core/libs/adapters/logging/src/zenoh_layer.rs:54`. It is encoded as a Foxglove `Log`
(`core/libs/app/service/src/runtime.rs:1104`) and put on a bounded channel; when the channel is full, the record is
dropped (`core/libs/adapters/logging/src/zenoh_layer.rs:60`). A task then publishes it to
`blueos/v1/<service>/log` (`core/libs/adapters/logging/src/zenoh_layer.rs:97`).

---

# Suggested reading order

1. `core/services/example/logic/pump/src/domain.rs`
2. `core/services/example/logic/pump/src/command.rs`
3. `core/services/example/app/src/lib.rs`
4. `core/libs/app/service/src/builder.rs`
5. `core/libs/app/service/src/runtime.rs`, from `run` (line 331) and `dispatch_command` (line 945)
