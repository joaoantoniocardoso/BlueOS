# BlueOS Rust style guide

How Rust code in `core/` is written. It applies to people and to their agents alike. The reasons behind the
architecture are in `docs/adr/decisions.md`; the gates that enforce this guide are in D-30 there; the words are
defined in `GLOSSARY.md`.

The teaching example (`core/services/example/`) passes every rule with zero `#[allow]`. When this guide and the
example disagree, fix one of them in the same change.

## Checklist

This block is copied verbatim into `AGENTS.md` and `.cursor/rules/rust-blueos.mdc`. `.hooks/lib/rust_checks.sh`
fails when the copies differ, so edit it here and copy it.

<!-- rust-style:begin -->
Rust checklist. Full text with examples: `docs/architecture/rust-style.md`. Gates: `docs/adr/decisions.md` D-30.

- Write the test first. Keep it simple: no abstraction, helper, or pattern the task does not need, and no helper
  used once that is small enough to read inline. A new architectural pattern needs a decision entry first.
- Document every public item. Open every crate root with `//!` saying what the crate owns, in user terms.
  Private items need a doc comment only when they are not obvious.
- Add a dependency to `[workspace.dependencies]` with `default-features = false` and only the features needed;
  members use `workspace = true`.
- Never abbreviate a name. Name values by meaning, not by type. `Err(error)`, never `Err(e)` or `Err(err)`.
- Return typed errors, never strings or magic payloads.
- Log with structured fields and a constant message: `warn!(%error, path = %path.display(), "Failed to open")`.
- Group imports in five blocks separated by a blank line: std, third-party crates, `blueos` crates, owned modules
  (`crate::`), relative paths (`self::`, `super::`). Chain each crate in one `use`.
- Order declarations top-down: constants and type aliases, then types (a type before the types it uses), then
  `impl` blocks in the same order (trait `impl`s before the inherent one), then free functions (a caller before
  its callees), then `#[cfg(test)] mod tests`.
- No renaming re-export, and no re-export of another crate's domain types. A facade needs
  `#![expect(clippy::pub_use, reason = "...")]`.
- Bind a value cloned for a `move` closure or an `async move` block inside a block attached to the spawn, never in
  the enclosing scope.
- Make illegal states unrepresentable: newtypes for ids, units (`Duration`) and validated input parsed once at the
  boundary; an enum for state that is stored; type-state for builders and resource handles.
- Borrow before cloning. Clone a handle with `Arc::clone(&handle)`; a data copy needs a reason.
- No `unsafe`. No `#[allow]`: use `#[expect(lint, reason = "...")]`. Never `.expect` a lock.
- Fix a `rustqual` finding by simplifying the code. Code that is already simplest (one struct literal, one linear
  sequence) stays whole under a `// qual:allow(...)` with a reason; carrier structs and forwarders are worse.
- Run `./.hooks/pre-push --fix`, then `./.hooks/pre-push`, before finishing.
<!-- rust-style:end -->

## Simplicity and patterns

Write idiomatic Rust and the least code that is correct. Before writing a mechanism, check whether the codebase,
the standard library or an adopted dependency (D-32) already has it.

Do not extract a function, variable or type that is used once and small enough to read inline. Extract a helper
only when it is used more than once or hides real complexity.

```rust
// Bad: a one-line helper called once
fn recorder_directory(arguments: &Arguments) -> PathBuf {
    PathBuf::from(&arguments.recorder_path)
}

// Good: the argument is already a PathBuf
let directory = &arguments.recorder_path;
```

The architectural patterns are the ones in `docs/adr/decisions.md` (sans-IO Domain and Blocks, Tasks, Projections,
the reconcile pattern, the endpoint manifest, durable state). A new pattern gets a decision entry before the first
line that uses it. Draft 1 grew four undocumented patterns, one of which contradicted D-03.

## Tests

Write the failing test first (D-20). Each test lives in the lowest layer that can see the behavior (D-30): a Domain
rule is an L1 unit test with no runtime, never a Kernel test. Tests use a paused clock and the injected `Clock`.
Async test code never waits on wall time: no `std::thread::sleep` in async code, and no real-clock
`tokio::time::sleep`. On a paused runtime, `tokio::time::sleep` is allowed only as a drain of outstanding
`spawn_blocking` work (see `drain_blocking_io` in the Recorder harness). A blocking test double may block its own
thread to stand in for slow IO.

A test that needs a State waits for it with a predicate (the `wait_for_*` helpers, which subscribe and
check the current value), rather than reading it once and branching on it, because the IO behind the State
may not have landed yet.

A test that needs private items goes in an inline `#[cfg(test)] mod tests { ... }` at the end of its file. A test
that uses only the public API goes in the crate's `tests/` folder. The syn checker rejects an out-of-line
`#[cfg(test)] mod tests;`, which would sit between the two.

## Documentation

- Every public item has a doc comment (`missing_docs` is denied). Private items need one only when they are not
  obvious.
- Every crate root opens with `//!` and one sentence about what the crate owns, written for a user rather than for
  an architecture reviewer. Constraints such as `#![no_std]` come right after that sentence, before longer prose.
- A comment says why, never what. Do not repeat the code.

```rust
// Bad
//! Recording policy, see D-03, D-09 and D-15.

// Good
//! Decides when the Recorder records, and which file it records to.
```

## Dependencies

- Every dependency is declared once in `core/Cargo.toml` under `[workspace.dependencies]`, with
  `default-features = false` and only the features the workspace needs. A member cannot turn off a default feature
  of an inherited dependency, so the workspace line must be minimal and members add features.
- Members use `workspace = true`, never a version or a relative `path`.
- Logic crates are `no_std` + `alloc`; any dependency they take must build for `thumbv7em-none-eabihf` and
  `wasm32-unknown-unknown`.

```toml
# core/Cargo.toml
chrono = { version = "=0.4.41", default-features = false, features = ["alloc"] }

# an app crate
chrono = { workspace = true, features = ["std", "clock"] }
```

## Names

- Never abbreviate: `service`, `command`, `configuration`, `sender`, `receiver`, `error`, not `svc`, `cmd`, `cfg`,
  `tx`, `rx`, `e`. Established acronyms (`IO`, `CLI`, `CDR`, `MCAP`) and generic parameters (`T`, `D`) are fine.
- Name a value by what it means, not by its type. Two different byte buffers in one scope must not both be
  `bytes`.
- One word, one meaning, as defined in `GLOSSARY.md`. "Session" is the Zenoh connection and nothing else.

```rust
// Bad
let bytes = sample.payload().to_bytes();
let bytes = encode(&message)?;

// Good
let received_payload = sample.payload().to_bytes();
let encoded_reply = encode(&message)?;
```

## Errors

Return typed errors (`thiserror`), never `String` errors or magic payloads such as `b"FAILED"`. Never `.expect` a
lock: a poisoned lock must not turn one panic into many (D-29).

## Logging

Log with structured fields and a constant message, so logs can be filtered and the message reads the same every
time.

```rust
// Bad
error!("recorder: failed to open {path:?}: {error}");

// Good
error!(%error, path = %path.display(), "Failed to open recording");
```

## Imports

Five blocks separated by one blank line, in this order: std (`std`, `core`, `alloc`), third-party crates, `blueos`
crates, owned modules (`crate::`), relative paths (`self::`, `super::`). Each crate appears in one chained `use`.
Stable `rustfmt` keeps the blocks and sorts inside them.

```rust
// Bad: one alphabetical block mixes the groups, and one crate is split over two lines
use blueos_api::keys;
use bytes::Bytes;
use tokio::sync::mpsc;
use tokio::time::Duration;

// Good
use bytes::Bytes;
use tokio::{sync::mpsc, time::Duration};

use blueos_api::keys;
```

## Declaration order

A reader understands a file in one pass from top to bottom, without jumping to the end and back.

1. Constants and type aliases, right after the imports.
2. Types. A type comes before the types it uses.
3. `impl` blocks, in the same order as their types. For one type, trait `impl`s come before the inherent `impl`.
4. Free functions. A caller comes before the functions it calls.
5. `#[cfg(test)] mod tests`, always last.

When uses form a cycle, follow the main direction of use as far as possible.

```rust
const INBOX_CAPACITY: usize = 256;

struct Recorder {
    library: Library,
}

struct Library {
    files: Vec<RecordingFile>,
}

struct RecordingFile;

impl Default for Recorder { /* ... */ }

impl Recorder { /* ... */ }

impl Library { /* ... */ }

fn run() { /* calls start() */ }

fn start() { /* ... */ }

#[cfg(test)]
mod tests { /* ... */ }
```

Clippy orders item kinds; it is configured never to sort struct fields or enum variants, because field order is the
CDR wire order (D-05, D-06). The in-repo checker enforces the use order.

## Re-exports

A re-export hides where a type comes from. Forbidden: a renaming re-export (`pub use library::OperationKind as
RecordingOperationKind`), and re-exporting another crate's domain types to spare a caller a dependency. A facade
(a crate root that exposes its own modules, or macros that must be in scope at the use site) is allowed only behind
an `expect` with a reason, so each one argues for itself in review.

```rust
#![expect(clippy::pub_use, reason = "tracing macros must be in scope where services log")]
pub use tracing::{debug, error, info, trace, warn};
```

## Clones for spawned work

A value cloned only so that a `move` closure or an `async move` block can own it is bound inside a block attached to
the spawn, so the enclosing scope gains no name it does not use.

```rust
// Bad
let session = Arc::clone(&session);
tokio::spawn(async move { session.run().await });

// Good
tokio::spawn({
    let session = Arc::clone(&session);
    async move { session.run().await }
});
```

## Types that make illegal states unrepresentable

- **Newtypes** for ids, units and validated input. Use `core::time::Duration` for durations, never a bare number
  with a unit in its name. Parse input once at the boundary into a type that cannot hold an invalid value ("parse,
  don't validate"); code past the boundary never validates again.
- **An enum for state that is stored.** The Snapshot is one `Clone` value the Kernel owns, so its state machines are
  enums with data, never several fields that must agree.
- **Type-state for state that moves**: builders where a missing required part is a compile error, and resource
  handles (an open file versus a finished one).
- **An outcome is an enum**, never a set of flags.
- **An uninhabited type for what cannot happen.** A Domain or Block with no IO requests, no domain events or no
  timers sets that associated type to `core::convert::Infallible` or an empty enum, never a placeholder variant or
  `()`, so the compiler proves the case never occurs and no `match` needs an arm for it.

```rust
// Bad: a placeholder every match must handle
enum PumpIoRequest {
    None,
}

// Good
type IoRequest = core::convert::Infallible;
```

```rust
// Bad: eight combinations, three legal
struct OperationFinished {
    succeeded: bool,
    cancelled: bool,
    error: String,
}

// Good
enum OperationOutcome {
    Succeeded,
    Cancelled,
    Failed { reason: String },
}
```

```rust
// Bad: "active" with no recording is representable
struct Capture {
    session_active: bool,
    session: Option<ActiveRecording>,
}

// Good
enum Capture {
    Idle,
    Recording(ActiveRecording),
}
```

## Borrowing before cloning

Prefer references, `Cow` and well-scoped containers over copies. A handle clone (an `Arc`, a Session, `Bytes`) only
bumps a reference count and is free; a data clone allocates and needs a reason. The reader must tell them apart at
the call site, so handles are cloned with `Arc::clone(&handle)`, never `handle.clone()`.

```rust
// Bad: copies the whole policy for every sample
let policy = gate.borrow().clone();
if policy.allows(&sample) { /* ... */ }

// Good
if gate.borrow().allows(&sample) { /* ... */ }
```

Clippy's clone lints are on, but no lint finds a copy on a hot path; review does.

## Lints and suppressions

No `unsafe` (`unsafe_code` is forbidden). No `#[allow]`: a suppression is an `#[expect(lint, reason = "...")]`, so
it fails when it is no longer needed and always says why.

`rustqual` and thai-lint fail the build on any finding (D-30). Fix a finding by making the code simpler: a cohesive
step with its own name, a helper the code already repeats, a smaller type. Some code is already simplest and still
measures high: a function that is one struct literal per field, or one linear sequence of steps. Keep it whole under
a `// qual:allow(<dimension>, <metric>=<value>) reason: "..."` marker, as `declare_boot` in
`core/libs/app/service/src/kernel/boot/run/declare.rs` does. `max_suppression_ratio` in `core/rustqual.toml` caps how
many markers the workspace holds.

Splitting such code to pass the gate makes it worse, and review rejects it:

- a **carrier struct** is built once only to hand fields to the next function, which destructures it;
- a **forwarder** is a function whose body only calls another with the same arguments.
