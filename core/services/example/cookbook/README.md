# Example cookbook (D-20)

Numbered entries under `tests/` answer every "how do I do X?" question from the draft 1 ergonomics review
(P4.3). Each entry is an integration test on the in-process channel backend through a Service's real `build()`.
`example-minimal` stays the small copy target; this cookbook covers everything else.

An entry file name starts with the number of the **first** question it answers (`01-command.rs` for questions 1–3).

Run:

```bash
export PATH=$HOME/.cargo/bin:$PATH RUSTC_WRAPPER=sccache CARGO_TARGET_DIR=core/target
cd core && cargo test -p blueos-example-cookbook
```

## Question index

| # | Question | Entry | Status |
|---|----------|-------|--------|
| 1 | Add a Command with a body | [`tests/01-command.rs`](tests/01-command.rs) | written |
| 2 | Add a Command with no body | [`tests/01-command.rs`](tests/01-command.rs) | written |
| 3 | Reject a Command | [`tests/01-command.rs`](tests/01-command.rs) | written |
| 4 | Add a Query | [`tests/04-query.rs`](tests/04-query.rs) | written |
| 5 | Publish a State | [`tests/05-state.rs`](tests/05-state.rs) | written |
| 6 | Emit an Event | [`tests/06-event.rs`](tests/06-event.rs) | written |
| 7 | Keep a domain event private | [`tests/07-private-domain-event.rs`](tests/07-private-domain-event.rs) | written |
| 8 | Hold state | [`tests/08-hold-state.rs`](tests/08-hold-state.rs) | written |
| 9 | Define settings and load them at start | [`tests/09-settings.rs`](tests/09-settings.rs) | written |
| 10 | Persist settings | [`tests/09-settings.rs`](tests/09-settings.rs) | written |
| 11 | Mark a restart-required field | [`tests/09-settings.rs`](tests/09-settings.rs) | written |
| 12 | Arm a timer | [`tests/12-timers.rs`](tests/12-timers.rs) | written |
| 13 | Cancel a timer | [`tests/12-timers.rs`](tests/12-timers.rs) | written |
| 14 | Do device IO | [`tests/14-io-effects.rs`](tests/14-io-effects.rs) | written |
| 15 | Run multi-step work (job graph) | `15-jobs.rs` | not written yet (#62) |
| 16 | Show job status in the UI | `16-job-status.rs` | not written yet (#62) |
| 17 | Report partial progress of one job | `17-job-progress.rs` | not written yet (#62) |
| 18 | Unit-test the Domain | [`../logic/domain/src/lib.rs`](../logic/domain/src/lib.rs) (`#[cfg(test)]`) | written |
| 19 | Integration-test the service | [`../app/tests/endpoints.rs`](../app/tests/endpoints.rs) | written |
| 20 | Use the service from the frontend | [`../README.md`](../README.md) (Frontend section) | written |
| 21 | Evolve a message safely | `21-evolve-message.rs` | not written yet (#62) |
| 22 | Add an IO query | `22-io-query.rs` | not written yet (#62) |
| 23 | Accept a non-IDL Command body | `23-non-idl-command.rs` | not written yet (#62) |
| 24 | Send a Command from in-process code | `24-command-sender.rs` | not written yet (#62) |
| 25 | Subscribe to another service's topic | `25-cross-subscribe.rs` | not written yet (#62) |
| 26 | Call another service's Command | `26-cross-command.rs` | not written yet (#62) |
| 27 | Run a long-lived background task | `27-tasks.rs` | not written yet (#62) |
| 28 | Compose two Domains | `28-compose-domains.rs` | not written yet (#62) |
| 29 | Extend the CLI | [`tests/29-extend-cli.rs`](tests/29-extend-cli.rs) | written |
| 30 | Run a Command at startup | [`tests/30-startup-command.rs`](tests/30-startup-command.rs) | written |
| 31 | Clean up at shutdown | [`tests/31-shutdown-cleanup.rs`](tests/31-shutdown-cleanup.rs) | written |
| 32 | Define and return typed errors | [`tests/32-typed-errors.rs`](tests/32-typed-errors.rs) | written |
| 33 | Log with structure | [`tests/33-structured-logging.rs`](tests/33-structured-logging.rs) | written |
| 34 | Register a new service | [`tests/34-register-new-service.rs`](tests/34-register-new-service.rs) | written |

The [`tests/question_index.rs`](tests/question_index.rs) test parses this table: every question maps to exactly
one entry, pending rows must match the explicit `#62` list, written cookbook files must exist under `tests/`,
and `cargo test` compiles every entry file there.
