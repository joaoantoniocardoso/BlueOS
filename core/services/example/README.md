# example-minimal (D-20)

The teaching Service: one Command (`SetLevel`), one Query (`Level`), one State (`pump`). Copy this tree when you add a
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
| `app/src/service.rs` | `impl Service` calling `endpoints::register` |
| `app/src/cli.rs` | Service-specific `clap::Args` (common flags come from the Kernel) |
| `app/src/tasks.rs` | Optional supervised Tasks (see cookbook Q27) |
| `app/src/handlers.rs` | Optional custom Command/Query handlers when the manifest marks an endpoint `custom` |

There is no `adapters/` crate until you have real IO.

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

1. **Messages** — Add or reuse `.msg` files under `core/libs/idl/interfaces/`. Regenerate IDL:
   `cargo run -p blueos-idl-codegen -- --write` in `core/`.
2. **Crates** — `services/<name>/logic/domain`, `logic/api`, `app` (same shape as this directory). Register the three
   members in `core/Cargo.toml` and `[workspace.dependencies]`.
3. **Manifest** — `app/endpoints.toml` with `service = "<name>"` and tables for each endpoint kind. Regenerate:
   `cargo run -p blueos-idl-codegen -- --write` (updates `logic/api/src/endpoints.rs`, `app/src/endpoints.rs`,
   `frontend/src/libs/blueos-api/services/<name>.ts`, and `core/libs/idl/api.lock`).
4. **Domain** — Implement `Domain` / `DomainQueries` in `logic/domain`. Unit-test `handle` and `query` with no tokio.
5. **Conversions** — `impl Conversions for ...` in `logic/api/src/lib.rs`.
6. **Service** — `impl Service` in `app/src/service.rs`: `const NAME = endpoints::NAME`, `build` returns
   `endpoints::register(ServiceBuilder::new(...))` plus `.service_metadata(...)` when you need `info` fields.
7. **Multicall** — Add `"<name>"` to `KNOWN` in `core/app/blueos/src/main.rs` (always, even when the feature is off).
   Add a `#[cfg(feature = "<name>")]` match arm calling `blueos_service::entry::run::<YourService>`.
8. **Cargo feature** — On `core/app/blueos/Cargo.toml`, an optional dependency on your `app` crate and a feature that
   enables it (for example `example = ["dep:blueos-example-app"]`). Wire the match arm with `#[cfg(feature = "example")]`.
   Do **not** add that feature to `build_cross.sh`, CI cross-build, `core/Dockerfile`, or `core/start-blueos-core`
   unless the Service ships.
9. **Integration tests** — `app/tests/` using `blueos_service::testing::Harness` and `Harness::start` (real `build`,
   no hand-wired registration).
10. **nginx** — When the Service exposes HTTP, add a `location` in `core/tools/nginx/nginx.conf` (see other services).
11. **Startup** — When the Service should run on the vehicle, add a line in `core/start-blueos-core` in dependency order.
12. **Frontend** — Import the generated client from `@/libs/blueos-api/services/<name>`, use `sendCommand` /
    `watchState` or `blueosApiMixin`. Vitest uses `tests/blueos-api/fake-transport.ts`.

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

## Relation to `tank`

`core/services/tank` is a test Service with every endpoint shape (custom Command/Query, IO query, Events). It is not in
the multicall binary. Much of `tank` duplicates concepts shown here: `SetLevel`, `Level`, and a level State overlap
`example-minimal`; `tank` adds `Drain`, custom validation handlers, `LevelAfterFill`, `Probe`, and Events for codegen
tests.
