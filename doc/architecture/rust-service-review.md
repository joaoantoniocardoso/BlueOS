# BlueOS Rust service: review findings

Companion to `doc/architecture/rust-service-overview.md`, which uses the same vocabulary (App, Inbox, Kernel,
Adapter task, Effect, State, Event, ...).

**Scope.** The following code was reviewed:

- the Kernel: `core/libs/app/service`
- the logic libraries: `core/libs/logic/cqrs`, `core/libs/logic/jobs`
- the adapters: `core/libs/adapters/comms*`, `core/libs/adapters/settings`, `core/libs/adapters/logging`
- the IDL codec and its gates: `core/libs/idl`
- the teaching example: `core/services/example`
- the frontend client library: `core/frontend/src/libs/blueos-api`

The Recorder was checked only for how it uses the Kernel.

**Method.** Code reading, plus checks against the dependency sources (zenoh 1.9.0, zenoh-ts type definitions). H1 was
reproduced with a 16-byte payload. The other findings were not run on a device. References are `path:line` from the
repository root, as of commit `17a3f639d`.

**Severity.**

- **High:** a client can crash a service, or the service silently diverges from what it reports.
- **Medium:** wrong behavior in a realistic situation, or a decision in `decisions.md` that the code does not enforce.
- **Low:** rare, performance-only, debug-only, or test-only.

## Summary

| ID | Severity | Area | Finding |
|---|---|---|---|
| H1 | High | IDL codec | A 16-byte request aborts any service that has settings |
| H2 | High | Kernel | A failed Effect reports "rejected" but keeps the change and leaks its events |
| M1 | Medium | Kernel | A cancelled timer can still deliver its Command |
| M2 | Medium | Kernel | One publish or encode error stops the service without a clean shutdown |
| M3 | Medium | Settings | A `VERSION` sent by a client resets all settings at the next start |
| M4 | Medium | Settings | Without `--config`, the Snapshot and the settings file disagree |
| M5 | Medium | Frontend | `watchSettings` never delivers updates |
| M6 | Medium | Frontend, D-10 | State watchers can miss an update and stay stale |
| M7 | Medium | Kernel | The ack is sent before States are published |
| M8 | Medium | IDL, D-06 | The schema-evolution gates are not enforced |
| M9 | Medium | Jobs | The job graph grows forever and is republished on every Command |
| M10 | Medium | Kernel | A panicking IO task leaves its job Running forever |
| M11 | Medium | Logging | At trace level, published logs generate more logs forever |
| L1 | Low | Kernel | Liveliness is announced before the endpoints exist |
| L2 | Low | Kernel | `on_start` Commands can run after client Commands |
| L3 | Low | Kernel | Endpoint declaration failures are silent |
| L4 | Low | Kernel | Duplicate registrations run a Command twice |
| L5 | Low | Kernel | `CommandAck.job_id` is often wrong or ambiguous |
| L6 | Low | Kernel | `io_query` has no timeout and serves one request at a time |
| L7 | Low | Kernel | Timer registrations are never removed |
| L8 | Low | Comms, D-09 | Payloads are copied on the way through |
| L9 | Low | Settings | The settings write is not durable across power loss |
| L10 | Low | Example | Early errors are silent and the process exits with code 0 |
| L11 | Low | Example | `SetLevel` during a self-test is accepted but ignored |
| L12 | Low | Example, D-11 | `RestartRequired` is only an Event |
| L13 | Low | Tests | The channel test backend hides fan-out and wildcard behavior |

None of H1, H2, M1, M2, or M10 is covered by `core/libs/app/service/tests/kernel.rs` or `core/libs/idl/tests/`.

---

## High

### H1. A 16-byte request aborts any service that has settings

**Where**

- `core/libs/idl/codegen/src/lib.rs:523` reads a sequence length from the payload.
- `core/libs/idl/codegen/src/lib.rs:524` calls `Vec::with_capacity(length)` before any element is read.
- `core/libs/idl/src/cdr.rs:142` checks `self.position + count` with no overflow protection.
- Command requests are decoded in the Command Adapter task (`core/libs/app/service/src/runtime.rs:796`). Query
  requests are decoded inside the Inbox loop (`core/libs/app/service/src/runtime.rs:682`).

**What happens**

- The decoder trusts the sequence length. `UpdateSettings` takes a `SettingsEnvelope`, which has a `SettingField[]`
  sequence (`core/libs/idl/interfaces/blueos_msgs/msg/SettingsEnvelope.msg:5`).
- A length of `0xFFFFFFFF` asks for 4294967295 x 32 bytes, which is about 137 GB.
- **Reproduced:** decoding `00 01 00 00  01 00 00 00  00 00 00 00  FF FF FF FF` as `SettingsEnvelope` prints
  `memory allocation of 137438953440 bytes failed`, and the process aborts. An abort ignores `panic = "unwind"`, so
  the whole service dies.
- On 32-bit targets (`armv7-unknown-linux-musleabihf` is shipped), the failure is a panic instead:
  - `with_capacity` panics with "capacity overflow".
  - A string length near `u32::MAX` makes `position + count` wrap around, because release builds have no overflow
    checks. The bounds check then passes, and the slice at `core/libs/idl/src/cdr.rs:145` panics.
  - A panic in a Command Adapter task kills that endpoint until the next restart. A panic in the Inbox loop kills the
    whole service.

**Who can send it:** any Zenoh client, including any browser tab on the vehicle network through zenoh-ts. Every
service that calls `.settings(...)` exposes `UpdateSettings`.

**Fix direction**

- Pre-allocate at most `min(length, remaining bytes)`, and let the element loop fail with `UnexpectedEnd`.
- Use `checked_add` in `read_exact`.
- Add the 16-byte payload above as a regression test in `core/libs/idl/tests/`.

### H2. A failed Effect reports "rejected" but keeps the change and leaks its events

**Where**

- `core/libs/app/service/src/runtime.rs:958`: the App is changed by `handle`.
- `core/libs/app/service/src/runtime.rs:964`: the domain events are queued in `pending_events`.
- `core/libs/app/service/src/runtime.rs:1040`: a `Persist` error returns early.
- `core/libs/app/service/src/runtime.rs:638`: the loop replies with a rejected ack.

**What happens**

1. `App::handle` has already changed the Snapshot.
2. The domain events go into `pending_events` before any Effect runs.
3. An Effect fails. Examples: the settings write fails (disk full, SD card remounted read-only), or a programming error
   such as `Effect::Io` without `.io`. `dispatch_command` returns `Err`. The Effects before the failing one have
   already run (IO tasks spawned, timers armed). The Effects after it are skipped.
4. The loop sends `accepted: false` and publishes nothing. It does not undo the Snapshot, and it does not clear
   `pending_events`.

**Result:** the client is told that nothing happened, but the App kept the change. The next successful Command then
publishes:

- the new States; for settings, these are values that are not on disk;
- the stale events, for example a `RestartRequired` for a change that the client thinks failed.

**Fix direction**

- Run `Persist` before the other Effects.
- On error, restore a copy of the App taken before `handle`, and clear `pending_events`.
- Add a Kernel test with a read-only settings folder.

---

## Medium

### M1. A cancelled timer can still deliver its Command

**Where**

- `core/libs/app/service/src/runtime.rs:998`: the timer task.
- `core/libs/app/service/src/runtime.rs:1000`: the cancel check before the send.
- `core/libs/app/service/src/runtime.rs:1017`: `CancelSchedule`.
- The contract promises "Safe if the timer already fired" (`core/libs/logic/cqrs/src/lib.rs:115`).

**What happens:** when a timer fires, its task puts the Command in the Inbox. The Domain may then cancel or re-arm
the same `TimerId` while that Command still waits in the queue. The cancel flag and `abort()` come too late, so the
stale Command is still handled.

**Example**

1. The self-test timer fires while `JobIoFinished(VerifyOn)` and a new `StartSelfTest` are already queued.
2. The old test finishes, and the new test starts and re-arms `TimerId(1)`.
3. The stale `SelfTestTimedOut` then arrives and fails the new test at once
   (`core/services/example/logic/pump/src/command.rs:150`).

The Recorder capture-status timer (`core/services/recorder/logic/policy/src/lib.rs:228`) is exposed in the same way.

**Fix direction:** give each arm a generation number. The timer task sends `(TimerId, generation, Command)`. The Inbox
loop drops the message when the timer was cancelled or re-armed since then. Domains do not change.

### M2. One publish or encode error stops the service without a clean shutdown

**Where**

- `core/libs/app/service/src/runtime.rs:623`, `:624`, `:658`, `:659`: `?` inside the Inbox loop.
- `core/libs/app/service/src/runtime.rs:587`: `begin_shutdown`.

**What happens:** any error returns from `run`. It can come from a State selector, from an Event encode closure, or
from `Session::publish`. Then:

- The ack was already sent as accepted.
- `on_shutdown` is not dispatched. For the Recorder, `StopRecording` does not run
  (`core/services/recorder/app/src/lib.rs:214`).
- In-flight IO is not drained.

Also, `StateHandle::publish` stores the new value before it sends it (`core/libs/adapters/comms/src/state.rs:63`). If
the send fails, deduplication never retries it.

**Triggers:** an `.event` encode closure that returns `Err` (the example has two:
`core/services/example/app/src/lib.rs:90` and `core/services/example/app/src/lib.rs:103`), or a Zenoh put error.

**Fix direction:** log publish errors and continue. Keep `?` only for startup. Store the State value only after a
successful send.

### M3. A `VERSION` sent by a client resets all settings at the next start

**Where**

- `core/services/example/app/src/settings_schema.rs:65`: `VERSION` is copied from the request.
- `core/services/example/app/src/settings_schema.rs:35`: `VERSION` is written back to disk.
- `core/libs/adapters/settings/src/schema.rs:41`: a higher version is refused.
- `core/libs/adapters/settings/src/schema.rs:108`: 0 is refused.
- `core/libs/adapters/settings/src/manager.rs:128` to `core/libs/adapters/settings/src/manager.rs:147`: the manager
  falls back to defaults and saves them.

**What happens**

1. `UpdateSettings` takes `VERSION` from the client JSON and saves it into `settings-1.json`.
2. With `VERSION: 2` or `VERSION: 0`, the save succeeds.
3. At the next start, loading that file fails ("from the future", or bad attributes). The manager skips it and finds no
   other file.
4. The manager overwrites `settings-1.json` with defaults. The user's settings are lost, and no error is shown.

**Fix direction:** the Kernel's `update_settings_decode` rejects a request whose `VERSION` differs from
`S::VERSION`. The example always writes `S::VERSION`.

### M4. Without `--config`, the Snapshot and the settings file disagree

**Where**

- `core/services/example/app/src/lib.rs:130`: without `--config`, the example uses defaults and does not read the disk.
- `core/libs/app/service/src/runtime.rs:120`, then `core/libs/adapters/settings/src/manager.rs:22`: the Kernel's
  manager still loads `~/.config/example/`.
- `core/libs/app/service/src/runtime.rs:145` to `core/libs/app/service/src/runtime.rs:153`: persisting writes the
  Snapshot.

**What happens**

- The published `settings` State shows the defaults, not the file.
- The first `UpdateSettings` overwrites the file with defaults plus the change.
- The README run command has no `--config` (`core/services/example/README.md:74`).
- The help for `--config` says "JSON5 config file" (`core/libs/adapters/cli/src/lib.rs:16`), but the code uses it as
  the parent folder for settings (`core/libs/adapters/settings/src/manager.rs:21`).

**Fix direction:** load the settings once, in the Kernel, and hand the loaded value to the app (for example, a
builder closure `Fn(S) -> Snapshot`). Fix the help text.

### M5. `watchSettings` never delivers updates

**Where**

- `core/frontend/src/libs/blueos-api/settings.ts:50`: `sample.kind` is compared without calling it. zenoh-ts defines
  `kind(): SampleKind`.
- `core/frontend/src/libs/blueos-api/settings.ts:54`: the arguments of `decodeSample` are swapped. The signature is at
  `core/frontend/src/libs/blueos-api/zenoh-helpers.ts:27`.
- `core/frontend/src/libs/blueos-api/settings.ts:61`: `session.declareLinkListener` does not exist in zenoh-ts. The
  API is `linkEventsListener`, as used in `core/frontend/src/libs/blueos-api/state.ts:60`.

**What happens**

- Every settings sample returns early at line 50.
- Line 61 throws a `TypeError` inside a `void` async function. The result is an unhandled promise rejection, and the
  re-query after a reconnect never runs.
- The settings panels in `core/frontend/src/views/ExampleServiceView.vue:181` and
  `core/frontend/src/views/RecorderView.vue:133` only show the value that was read when the view opened.
- Neither check caught this: lint skips `.ts` files (D-22), and `type-check` goes through `tsc-baseline`
  (`core/frontend/package.json:15`).

**Fix direction:** reuse one watcher helper for `state.ts`, `jobs.ts`, and `settings.ts`. The three files are near
copies.

### M6. State watchers can miss an update and stay stale

**Where**

- `core/frontend/src/libs/blueos-api/state.ts:44`, then `core/frontend/src/libs/blueos-api/state.ts:50`.
- The same in `core/frontend/src/libs/blueos-api/jobs.ts:43`, then `core/frontend/src/libs/blueos-api/jobs.ts:49`.
- The order comes from D-10: "Clients query first, then subscribe" (`doc/architecture/decisions.md:264`).

**What happens:** the watcher waits for the query reply, and only then declares the subscriber. A State published
between these two moments is lost. States publish only on change (`core/libs/adapters/comms/src/state.rs:57`), so the
UI can stay wrong until the next change, which may never come.

**Fix direction:** subscribe first, then query. Ignore the query reply if a sample has already arrived. Update the
D-10 wording.

### M7. The ack is sent before States are published

**Where**

- `core/libs/app/service/src/runtime.rs:619`: the ack.
- `core/libs/app/service/src/runtime.rs:623`: the publish.
- A State `get` is answered from the StateHandle cache (`core/libs/adapters/comms/src/state.rs:27`).

**What happens:** a client that reads a State right after the ack can still get the old value. "Send a Command, then
read" is a normal client pattern, and it is racy here.

**Fix direction:** publish the States before sending the ack. Events can stay after the ack.

### M8. The schema-evolution gates are not enforced (D-06)

D-06 promises two things that the code does not deliver.

**1. The lock gate cannot see what kind of change was made.**

- `core/interfaces/api.lock` stores only a hash per message.
- `core/libs/idl/tests/api_lock.rs:36` to `core/libs/idl/tests/api_lock.rs:51` rewrites the lock with any new hash
  and keeps the old major version.
- So the test cannot tell an append from a breaking change, and it never asks for a major bump.
- `is_append_only_evolution` (`core/libs/idl/codegen/src/lib.rs:864`) is used only by its own unit tests
  (`core/libs/idl/tests/api_lock.rs:74` to `core/libs/idl/tests/api_lock.rs:101`).
- D-06 requires this gate (`doc/architecture/decisions.md:200`).

**2. The type hash is never sent.**

- Each message has `TYPE_HASH` (`core/libs/idl/src/message.rs:14`).
- The Kernel publishes with `attachment: None` (`core/libs/app/service/src/runtime.rs:1091`).
- Nothing reads `TYPE_HASH_ATTACHMENT_KEY` (`core/libs/api/src/lib.rs:12`).
- D-06 says that a mismatch must "fail loudly" (`doc/architecture/decisions.md:196`).

**Also: append-only is safe only for the outer message.** The decoder fills in a missing field only when the whole
buffer is exhausted (`core/libs/idl/codegen/src/lib.rs:520`, `:538`, `:555`). Some messages are used inside a
sequence or as a nested field (`JobStatus`, `SettingField`, `EndpointInfo`, `RecordingFile`, and others). A field
appended to one of them shifts every byte that follows, so old and new readers both decode garbage. D-06 does not
forbid this.

**And:** `cargo semver-checks` is skipped for crates that are not on crates.io yet
(`.hooks/lib/rust_checks.sh:123`). Today that is all of them.

**Fix direction**

- Store the field signature, not only its hash, in `api.lock`.
- Check it with `is_append_only_evolution`, and require a major bump for anything else.
- Flag any change to a message that is used as a nested field.
- Send `TYPE_HASH` as an attachment, and check it in the decoders.

### M9. The job graph grows forever and is republished on every Command

**Where**

- `core/libs/logic/jobs/src/lib.rs:82`: nodes are never removed (there is a `ponytail` note).
- `core/libs/logic/jobs/src/lib.rs:180`: `poll_runnable` scans every node on every `App::handle`
  (`core/libs/logic/cqrs/src/lib.rs:185`).
- `core/services/example/app/src/lib.rs:162`: the `jobs` State lists every node ever created.
- `core/libs/app/service/src/runtime.rs:1163`: every ack scans every node.
- `core/libs/app/service/src/runtime.rs:973` and `core/libs/app/service/src/runtime.rs:1029`: every `Io` and
  `Persist` Effect clones the whole App.

**What happens:** each self-test adds 4 nodes. Memory, CPU per Command, and the size of the `jobs` State all grow
without limit. The Recorder records the backbone (D-01), so every new `jobs` State is written to MCAP in full.

Today only the example uses jobs; the Recorder publishes an empty list
(`core/services/recorder/app/src/lib.rs:224`). The setup wizard and calibration services planned in D-01 will use them.

**Fix direction:** drop finished root graphs after a retention count or time. Publish only the live graphs plus a
bounded recent history.

### M10. A panicking IO task leaves its job Running forever

**Where:** `core/libs/app/service/src/runtime.rs:976` to `core/libs/app/service/src/runtime.rs:985`. The `JoinHandle`
is dropped. If the executor panics, no `IoComplete` is sent.

**What happens:** the job leaf stays `Running`, its graph never ends, and the Domain never hears about it. The example
recovers only because it has a watchdog timer; a Domain without one hangs. D-23 fixed this inside the Recorder
adapters, not in the Kernel.

**Fix direction:** the Kernel awaits the task (or uses `catch_unwind`). It then delivers a failure Command that the
Domain provides, for example through a `Domain::io_failed(request) -> Command` hook.

### M11. At trace level, published logs generate more logs forever

**Where**

- `core/libs/adapters/logging/src/lib.rs:26`: `-vv` sets one global `trace` filter, which also covers zenoh.
- `core/libs/adapters/logging/src/lib.rs:37`: the Zenoh log layer sees every record that passes the filter.
- `core/libs/adapters/logging/src/zenoh_layer.rs:54` and `core/libs/adapters/logging/src/zenoh_layer.rs:97`: each
  record is published with `Session::publish`.
- Zenoh 1.9.0 emits `trace!` on every put: `zenoh-1.9.0/src/api/session.rs:2474` (`write(...)`) and
  `zenoh-transport-1.9.0/src/unicast/universal/tx.rs:150` (`Scheduled ...`). These paths are in the cargo registry.

**What happens:** each published log line produces at least two new trace records, which are published too. The
1024-entry channel stays full, the publisher task runs forever, and the log stream fills with Zenoh's own write traces.
This happens only with `-vv` or `RUST_LOG=trace`. It was not run on a device.

**Fix direction:** make the Zenoh layer ignore `zenoh*` targets, or give it a filter separate from the console filter.

---

## Low

### L1. Liveliness is announced before the endpoints exist

**Where:** the token is declared first (`core/libs/app/service/src/runtime.rs:356`). Then come the States
(`core/libs/app/service/src/runtime.rs:362` to `:411`) and `info` (`:413`). The Command and Query endpoints are
declared even later, inside spawned tasks (`:791`, `:899`).

**What happens:** a client that reacts to "service alive" by reading `info` or by sending a Command can get no reply.
The Zenoh inspector (D-24) discovers services this way.

**Fix direction:** first declare all queryables and await them, then publish the initial States, and only then
declare the token.

### L2. `on_start` Commands can run after client Commands

**Where:** the Command endpoints are spawned at `core/libs/app/service/src/runtime.rs:517`. The `on_start` Commands
enter the Inbox later, at `core/libs/app/service/src/runtime.rs:538`.

**What happens:** a client Command that arrives in between is handled before the startup Commands. For the Recorder,
a `DeleteRecording` could run before `InitializeLibrary` (`core/services/recorder/app/src/lib.rs:213`). The window
is short.

**Fix direction:** send the startup Commands before spawning the adapters. The Inbox has room for 256 messages.

### L3. Endpoint declaration failures are silent

**Where:** `core/libs/app/service/src/runtime.rs:791`, `:857`, `:899` (`let Ok(...) else { return; }` with no log).

**What happens:** the endpoint does not exist and nothing is logged, but `ServiceInfo.endpoints` still lists it.

**Fix direction:** fail startup. The L1 fix, which awaits the declarations in `run`, gives this for free.

### L4. Duplicate registrations run a Command twice

**Where**

- `core/libs/app/service/src/builder.rs:165`: there is no check for duplicate names.
- `core/libs/app/service/src/runtime.rs:388`: `UpdateSettings` is added even when the service registered its own.
- `core/libs/app/service/src/runtime.rs:240`: the duplicate is only hidden from `ServiceInfo`.

**What happens:** two queryables end up on one key. The Kernel never declares a queryable as "complete", so Zenoh's
default query target sends the query to every matching queryable, and the Command runs twice.
`.state("status", ...)` next to `.status(...)` collides in the same way.

**Fix direction:** reject duplicate names and reserved names in the builder.

### L5. `CommandAck.job_id` is often wrong or ambiguous

**Where**

- `core/libs/app/service/src/runtime.rs:1163` to `core/libs/app/service/src/runtime.rs:1173`: `job_id` is the newest
  root job of the whole graph.
- `core/libs/app/service/src/runtime.rs:1154` and `core/libs/app/service/src/runtime.rs:1172`: 0 means "no job".
- `core/libs/logic/jobs/src/lib.rs:31`: `JobId(0)` is also the first real job.

**What happens:** `SetLevel` after a self-test returns the id of the self-test job. The first self-test gets id 0,
which is the same value as "no job".

**Fix direction:** return the root job that this Command created (compare the job count before and after `handle`).
Start job ids at 1.

### L6. `io_query` has no timeout and serves one request at a time

**Where:** `core/libs/app/service/src/runtime.rs:855` (a `ponytail` note) and `core/libs/app/service/src/runtime.rs:868`.

**What happens:** one request that never finishes blocks that endpoint forever. Later requests wait until their
clients time out.

**Fix direction:** wrap the handler in `tokio::time::timeout`.

### L7. Timer registrations are never removed

**Where:** registrations are inserted at `core/libs/app/service/src/runtime.rs:1013`. They are removed only by a
re-arm or a cancel (`core/libs/app/service/src/runtime.rs:992`, `:1018`).

**What happens:** a Domain that uses a new `TimerId` for each operation makes the map grow forever. The current
Domains use fixed ids, so today the map stays small.

**Fix direction:** remove the entry when the timer fires. This fits the M1 generation fix.

### L8. Payloads are copied on the way through (D-09)

**Where**

- `core/libs/adapters/comms/driver/src/payload.rs:63` to `core/libs/adapters/comms/driver/src/payload.rs:68`:
  `into_bytes` copies even when the storage is already `Bytes`. Every publish uses it
  (`core/libs/adapters/comms/zenoh/src/payload.rs:25`).
- `core/libs/adapters/comms/driver/src/payload.rs:52`: `as_slice` always allocates. `StateHandle` deduplication calls
  it twice per State on every Command (`core/libs/adapters/comms/src/state.rs:59`).
- Incoming query payloads are copied (`core/libs/adapters/comms/zenoh/src/lib.rs:103`), then copied again by the CDR
  reader (`core/libs/idl/src/cdr.rs:112`).

**What happens:** D-09 says that comms never copies payloads, but the publish path always copies. D-22 lists only the
Recorder tap copy as known.

**Fix direction:** return the inner `Bytes` without a copy when the storage is `BytesStorage`. Add a borrowed accessor.
Let the reader borrow the input.

### L9. The settings write is not durable across power loss

**Where:** `core/libs/adapters/settings/src/schema.rs:92` to `core/libs/adapters/settings/src/schema.rs:94`. The file
is synced and then renamed, but the folder is never synced.

**What happens:** after a sudden power cut, the rename can be lost, and the old settings come back. Vehicles lose power
often. Python behaves the same way.

**Fix direction:** after the rename, open the parent folder and call `sync_all` on it.

### L10. Early errors are silent and the process exits with code 0

**Where**

- `core/services/example/app/src/lib.rs:124`: `info!` runs before logging is initialized.
- `core/services/example/app/src/lib.rs:43`: `error!` is used for every error.
- `core/libs/app/service/src/builder.rs:334`: `Session::open` runs before logging init at
  `core/libs/app/service/src/builder.rs:365`.
- `core/app/blueos/src/main.rs:34` to `core/app/blueos/src/main.rs:35`: the example always returns success.

**What happens:** a settings load error or a Zenoh connection error prints nothing, and the process exits with code 0,
so a supervisor sees a clean exit.

**Fix direction:** initialize logging first. Return `ExitCode` from `example::run`, like the Recorder does.

### L11. `SetLevel` during a self-test is accepted but ignored

**Where:** `core/services/example/logic/pump/src/command.rs:45` to
`core/services/example/logic/pump/src/command.rs:47`. The test `core/services/example/logic/pump/src/tests.rs:32`
locks this behavior in.

**What happens:** the client gets `accepted: true`, and nothing changes. The example README says that a Command that
is invalid for the current Snapshot must be rejected (`core/services/example/README.md:49`). This is the teaching
example, so the pattern will be copied.

**Fix direction:** return `Decision::reject("self-test running")`.

### L12. `RestartRequired` is only an Event

**Where**

- `core/services/example/logic/pump/src/command.rs:112` to `core/services/example/logic/pump/src/command.rs:116`:
  the event is emitted.
- `core/services/example/logic/pump/src/command.rs:191`: the diff compares against the last saved value, not the
  running value.

**What happens:** a UI that opens after the change never learns that a restart is pending. Changing `device_model`
back to the running value still reports a restart.

**Fix direction:** keep the running value and the pending restart fields in the Snapshot, and publish them as a State.
Update the D-11 wording to match.

### L13. The channel test backend hides fan-out and wildcard behavior

**Where**

- `core/libs/adapters/comms/channel/src/broker.rs:131` to `core/libs/adapters/comms/channel/src/broker.rs:135`: a
  query goes to the first matching queryable only. Zenoh sends it to all of them.
- `core/libs/adapters/comms/channel/src/broker.rs:79` to `core/libs/adapters/comms/channel/src/broker.rs:85`: closed
  subscribers are never removed.

**What happens:** Kernel tests cannot catch L4 or the behavior of wildcard `get` calls. D-23 already records three
bugs that the channel tests missed.

**Fix direction:** send each query to all matching queryables and collect the replies, like Zenoh does.

---

## Checked and found correct

- **Jobs:** the ordering of `Sequence` and `Parallel`, cancel propagation, and the `Cancelling` handshake
  (`core/libs/logic/jobs/src/lib.rs:137`, `:330`, `:410`).
- **No self-deadlock:** the Inbox loop never sends to its own Inbox. IO tasks and timers send from spawned tasks.
- **Persist during shutdown:** `Persist` is awaited inside dispatch, so the persist queue is empty when the loop exits.
  The shutdown time is bounded by the 5-second IO drain.
- **Strings in the CDR decoder:** they are bounds-checked before allocating (on 64-bit targets; see H1 for 32-bit).
- **Settings writes:** they are atomic (temp file + rename), and stale `.tmp` files are removed at load.
