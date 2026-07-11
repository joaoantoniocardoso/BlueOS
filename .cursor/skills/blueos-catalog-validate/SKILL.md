---
name: blueos-catalog-validate
description: >-
  Independent QA gate for BlueOS catalog worker output — run validate(), the
  drift diff, the coverage threshold, and a provenance spot-check, then return an
  ACCEPT/BOUNCE verdict. Use as the QA Reviewer agent (fresh context, not the
  author) verifying an extractor, card-author, or crate output. Bounces failing
  work back with exact evidence; never edits the artifact.
disable-model-invocation: true
---

# BlueOS Catalog Validate (QA Reviewer)

You are the **QA Reviewer**: an independent verifier with fresh context. Review exactly one worker output and return a verdict. You did not author this artifact and you must not edit it — a failure is a **BOUNCE**, not a fix. You are not given the author's reasoning; verify the artifact on its own evidence.

## Ordering rule

For a Card Author output, confirm its matching observed artifact was already ACCEPTed. If it was not, BOUNCE — ground truth comes first. (The orchestrator enforces dispatch order; you enforce it at the gate.)

## Automated gates (all must be green)

```bash
bash gate.sh   # runs everything below, fails fast; prefer this single command
# equivalently, individually:
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --bin extract      # observed layer vs core/start-blueos-core (source of truth)
cargo run --bin drift        # asserted vs observed + asserted vs runtime reconciliation
# validate() runs inside tests; confirm it exercises the new service
```

- **`validate()`** — invariants + coverage threshold (authority uniqueness, edge targets exist, journey/runtime cross-refs, `Unknown` count ≤ threshold).
- **`extract`** — the hand-authored observed layer (startup_tier/memory/cpu) matches `core/start-blueos-core`.
- **`drift`** — no asserted field contradicts the observed layer; runtime StateContracts reference declared states.

## Provenance spot-check (manual, sample K fields)

For K randomly chosen observed fields on the card:

1. Open the cited `file:line`.
2. Confirm the value actually appears there.
3. A single fabricated or stale citation **fails the card**.

Prioritize sampling: ports, nginx routes, MAVLink connect strings, outbound edges — the fields most prone to drift.

## Inter-rater rubric (FROZEN v1.0 — calibration complete)

Calibration is done (ardupilot_manager + kraken, 13/14 hard-field agreement). The rubric is
frozen in `blueos-service-card/SKILL.md` → **Decision rules — FROZEN RUBRIC v1.0**. When spot-checking
judgment fields, apply those rules and BOUNCE deviations lacking a rationale:
- `user_confirmation` must be `Required` iff `dangerous_operations` is non-empty.
- `dangerous_operations` include only irreversible / untrusted-code / unsafe-physical-state ops (not reversible lifecycle).
- `authorities` are exclusive rights other services depend on (a private-only settings dir is a Resource, not an authority).
- `bounded_context` judged by semantic equivalence, not exact string.

## QA checklist

```
QA for <id>:
- [ ] Observed artifact passed gate first (for card work)
- [ ] cargo fmt --check clean
- [ ] cargo clippy -D warnings clean
- [ ] cargo test green (validate exercised)
- [ ] extract green (observed matches source)
- [ ] drift green
- [ ] coverage threshold met
- [ ] provenance spot-check: K/K citations verified
- [ ] judgment fields obey FROZEN RUBRIC v1.0
- [ ] authorities/edges trace to observed capabilities
```

## Verdict format

Return one of:
- **ACCEPT** — all gates green; note anything to revisit later.
- **BOUNCE** — list each failing gate with the exact command output or the bad `file:line`, and the specific fix required.
