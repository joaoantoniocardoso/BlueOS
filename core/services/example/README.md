# example-minimal (D-20)

The teaching Service: one Job type (`SetLevel`), one Query (`Level`), one State (`pump`). Copy this tree when you add a
real Service. Vocabulary and rules live in [GLOSSARY.md](../../../GLOSSARY.md) at the repo root; this file only walks
the steps.

The full Draft 1 example under `git show 17a3f639d:core/services/example/` is reference only. `example-minimal` is
the small wiring path; the numbered [`cookbook/`](cookbook/README.md) answers every other "how do I do X?" question.

## Layout (D-02)

| Path | Role |
|------|------|
| `logic/domain/` | `#![no_std]` Domain: `Snapshot`, `Request`, `Query`, `handle`, `query` |
| `logic/api/` | `impl` of the generated `Conversions` trait (Message ↔ Domain) |
| `app/endpoints.toml` | Public endpoint manifest (D-26) |
| `app/src/endpoints.rs` | Generated `register` (do not edit) |
| `app/src/lib.rs` | Crate root: `cli`, `endpoints`, `service` only |
| `app/src/service.rs` | `impl Service`: `context` builds the Context (here `()`), `build` calls `endpoints::register` |
| `app/src/context.rs` | The Context and its Ports, once the Service has IO or Tasks (example-minimal has none) |
| `app/src/cli.rs` | Service-specific `clap::Args` (common flags come from the Kernel) |
| `cookbook/tests/27-tasks.rs` | Supervised Tasks (Q27); example-minimal ships none |
| `cookbook/tests/35-metrics.rs` | A metric recorded from a Task, and a count a Domain exposes (Q35, Q36) |
| `<block>/handlers.rs` under `app/src/` | Optional handlers for the `custom` Job types and the `io` Queries of the manifest (D-23) |

There is no `adapters/` crate until you have real IO.

## Endpoint kinds (D-26)

`app/endpoints.toml` declares four kinds of endpoint, each with its interface type in `type`:

```toml
[job]
SetLevel = { type = "blueos_example_msgs/action/SetLevel", nature = { lasting = true } }

[query]
Level = { type = "blueos_example_msgs/srv/Level" }

[state]
pump = { type = "blueos_example_msgs/msg/PumpState" }
```

- **`job`**: a Job type, a `.action`. A client submits its Goal; `Conversions` maps the Goal to a Request, or returns
  its typed error, which the ack carries as the rejection. `Conversions` also gives the Job type's Feedback while a
  Job runs and its Job result when it ends, which the Kernel publishes on `jobs/<JobType>/feedback` and
  `jobs/<JobType>/result`. `nature` says what the Job type allows: without it, a Job is instant. `custom = true`
  moves the Goal mapping to a `Handlers` method, for a Goal that needs the Context.
- **`query`**: a Query, a `.srv`. `Conversions` maps its request to a Domain Query and the Domain's response to its
  response. `io = true` answers it with a `Handlers` method outside the Inbox, for a read that needs IO.
- **`state`** and **`event`**: a `.msg`, which `Conversions` makes of the Snapshot or of a domain event.

The Kernel declares the Job controls (`CancelJob`, `PauseJob`, `ResumeJob`, `AnswerPermission`), `UpdateSettings`,
`info`, `status`, `settings`, `jobs` and `log` for every Service: a manifest never lists them. `info` lists each
endpoint with its kind, key, interface type and schema text, and `core/libs/idl/api.lock` records each key with its
interface type.

`SetLevel` moves the pump one level per second: its Feedback is the level the pump is at, and its Job result the level
it reached. A level above `MAX_LEVEL` is rejected by its Goal conversion.

## Run locally (not shipped)

```bash
# Router on 7447 if needed
export PATH=$HOME/.cargo/bin:$PATH RUSTC_WRAPPER=sccache CARGO_TARGET_DIR=core/target
cd core
cargo run -p blueos --features example -- example -v
```

Shipped images and CI build `blueos` with `--features recorder` only (`core/build_cross.sh`). The `example` feature is
optional on `core/app/blueos`; without it, `blueos example` prints that the name is not compiled in.

## Add a new Service (checklist)

1. **Interfaces** — Add or reuse a `.action` per Job type, a `.srv` per Query, and a `.msg` per State or Event under
   `core/libs/idl/interfaces/`. Regenerate IDL: `cargo run -p blueos-idl-codegen -- --write` in `core/`.
2. **Crates** — `services/<name>/logic/domain`, `logic/api`, `app` (same shape as this directory). Register the three
   members in `core/Cargo.toml` and `[workspace.dependencies]`.
3. **Manifest** — `app/endpoints.toml` with `service = "<name>"` and a table for each endpoint kind (see
   [Endpoint kinds](#endpoint-kinds-d-26)). Regenerate:
   `cargo run -p blueos-idl-codegen -- --write` (updates `logic/api/src/endpoints.rs`, `app/src/endpoints.rs`,
   `frontend/src/libs/blueos-api/services/<name>.ts`, and `core/libs/idl/api.lock`).
4. **Domain** — Implement `Domain` / `DomainQueries` in `logic/domain`, and `DomainJobs` once a Job type has a
   `nature`. Unit-test `handle` and `query` with no tokio.
5. **Conversions** — `impl Conversions for ...` in `logic/api/src/lib.rs`: a Goal conversion with its error type per
   Job type that is not `custom`, Feedback and Job result per Job type, and one function per Query, State and Event.
6. **Service** — `impl Service` in `app/src/service.rs`: `const NAME = endpoints::NAME`, then two steps that the
   Kernel runs in order. `context` builds the Context: it may open what the arguments name and fills every Port with
   its real adapter (`Ok(())` when there is nothing). `build` is pure, with no IO and no spawning: it returns
   `endpoints::register(ServiceBuilder::new(...))`; the Kernel takes the `info` fields and the settings folder from
   the Service itself. A Task
   that follows a Projection captures the handle in `build`; the Context never holds a Projection.
7. **Multicall** — Add `"<name>"` to `KNOWN` in `core/app/blueos/src/main.rs` (always, even when the feature is off).
   Add a `#[cfg(feature = "<name>")]` match arm calling `blueos_service::entry::run::<YourService>`.
8. **Cargo feature** — On `core/app/blueos/Cargo.toml`, an optional dependency on your `app` crate and a feature that
   enables it (for example `example = ["dep:blueos-example-app"]`). Wire the match arm with `#[cfg(feature = "example")]`.
   Do **not** add that feature to `build_cross.sh`, CI cross-build, `core/Dockerfile`, or `core/start-blueos-core`
   unless the Service ships.
9. **Integration tests** — `app/tests/` using `blueos_service::testing::Harness`: `Harness::start` runs the real
   `context` and `build`; `Harness::start_with(arguments, |context| ...)` replaces a Port or a tunable in between.
   No second wiring function, and no test-only `pub` item in the app crate.
10. **nginx** — When the Service exposes HTTP, add a `location` in `core/tools/nginx/nginx.conf` (see other services).
11. **Startup** — When the Service should run on the vehicle, add a line in `core/start-blueos-core` in dependency order.
12. **Frontend** — Import the generated client from `@/libs/blueos-api/services/<name>`, use `sendCommand` /
    `watchState` or `blueosApiMixin`. Vitest uses `frontend/tests/blueos-api/fake-transport.ts`.

## Frontend for this example

`core/frontend/src/components/example/ExampleMinimalPanel.vue` sends `SetLevel` and binds `pump` through the generated
client. It is **not** registered in the Vue router (no dev-only route exists yet); import it in a dev build or mount it
from a parent you control. Browser check against a live Service is manual.

## Tests

```bash
cd core
cargo test -p blueos-example-domain -p blueos-example-app
cargo test -p blueos-example-cookbook
TRYBUILD=overwrite cargo test -p blueos-example-app --test compile_errors
bun --cwd core/frontend test tests/example/ExampleMinimalPanel.test.ts
```
