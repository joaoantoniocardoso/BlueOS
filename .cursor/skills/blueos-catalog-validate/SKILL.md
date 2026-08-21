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
cargo run --bin provenance_lint  # every citation resolves AND anchor matches cited line; Relocated is fatal -- tree needing --fix does not pass
# validate() runs inside tests; confirm it exercises the new service
```

- **`validate()`** — invariants + coverage threshold (authority uniqueness, edge targets exist, journey/runtime cross-refs, frontend `FRONTEND_API_ENDPOINTS` / `API_URL` composition via `FrontendRoutePrefixDropped`, `Unknown` count ≤ threshold).
- **`extract`** — the hand-authored observed layer (startup_tier/memory/cpu) matches `core/start-blueos-core`.
- **`drift`** — no asserted field contradicts the observed layer; runtime StateContracts reference declared states.
- **`provenance_lint`** — every `Evidence`/`Provenance::Doc` citation resolves and its `anchor` still matches the cited line; `Relocated`/`AmbiguousAnchor`/`AnchorLost` are fatal (a tree needing `--fix` does not pass).
- **`journey_http` smoke (required when Pi reachable)** — blocking gate for journey/route/runtime work:

```bash
BLUEOS_BASE=http://192.168.0.177 bash catalog/gate.sh
# or at minimum:
cargo run -q --bin journey_http -- --base http://192.168.0.177 --smoke --fixtures internet,pirate,advanced
```

Require **`failed=0`**. (`--smoke` = safe GETs only; requires live Pi.) If Pi unreachable: ACCEPT only with explicit per-step `UnverifiedLive` in the verdict — never ACCEPT `Known` `expected_status` on an unverified `RouteRef`.

## FROZEN JOURNEY ROUTE RUBRIC v1.0

These rules make journey `RouteRef` QA deterministic. They were frozen after inter-rater
calibration on 3 representative journeys spanning query params, API version prefixes, and
nested router prefixes. Apply them exactly; deviating requires a rationale. **A provenance
line alone is NEVER sufficient for `RouteRef` acceptance.**

### Hard fields (must agree)

1. **Resolved URL** — `resolve_http_path(RouteRef)` is documented in the QA report for every
   Known routed step sampled.
2. **Live proof** — either smoke `failed=0` summary for the PR, OR per-step `UnverifiedLive`
   with reason (Pi down / hardware fixture).
3. **Capture key** — when the outcome is Runtime, the capture key equals the full nginx path of
   the resolved URL.
4. **Query params / API version / nested router prefix** — present when frontend `API_URL`
   requires them.

### Soft fields

Step description wording; doc line choice among equivalents.

### Calibration set (must pass smoke when Pi up)

- `probe_interface_internet_connectivity` (query param)
- `browse_extension_store` (kraken v2.0)
- `browse_video_recordings` (nested `/recorder` prefix)

**QA ACCEPT** requires: smoke summary including these three as Pass (or Skip only for fixture
reasons, never Fail) when `BLUEOS_BASE` reachable.

### Calibration evidence (2026-07-18)

Base `http://192.168.0.177`, fixtures `internet,pirate,advanced`:

```
probe_interface_internet_connectivity: passed=1 failed=0 skipped=0 — Pass (1 step)
browse_extension_store:                passed=1 failed=0 skipped=0 — Pass (1 step)
browse_video_recordings:               passed=2 failed=0 skipped=0 — Pass (2 steps)
```

Aggregate: **4 routed steps, 4 Pass, 0 Fail, 0 Skip.**

## Provenance spot-check (manual, sample K fields)

`provenance_lint` automates the mechanical half: a fabricated or stale `file:line` or mismatched anchor cannot survive the gate. The spot-check's job is harder -- an intact anchor proves the cited **line** survived, not that surrounding behaviour still supports the claim.

For K randomly chosen observed fields on the card:

1. Open the cited `file:line` and read the anchored line in context.
2. Ask: does this line actually support the value asserted, given what surrounds it?
3. A single citation whose anchor matches but whose context no longer supports the value **fails the card**.

Use `cargo run --bin impact -- --path <file>` to list every catalog entity citing a file -- the way to check whether an extraction missed an entity.

Prioritize sampling: ports, nginx routes, MAVLink connect strings, outbound edges — the fields most prone to drift.

## Inter-rater rubric (FROZEN v1.0 — calibration complete)

**Service cards:** calibration done (ardupilot_manager + kraken, 13/14 hard-field agreement).
Frozen in `blueos-service-card/SKILL.md` → **Decision rules — FROZEN RUBRIC v1.0**. When
spot-checking judgment fields, apply those rules and BOUNCE deviations lacking a rationale:

**Journey routes:** frozen in this skill → **FROZEN JOURNEY ROUTE RUBRIC v1.0** above. Provenance
line alone is insufficient; require resolved URL + live proof per the hard fields.

Service-card rules:
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
- [ ] provenance_lint green (no Relocated; tree needs no --fix)
- [ ] coverage threshold met
- [ ] provenance spot-check: K/K citations verified in context (anchor match alone insufficient)
- [ ] any new check demonstrated non-vacuous (break it, watch fail, restore)
- [ ] judgment fields obey FROZEN RUBRIC v1.0
- [ ] authorities/edges trace to observed capabilities
- [ ] journey routes obey FROZEN JOURNEY ROUTE RUBRIC v1.0 (resolved URL + live proof; not Source line alone)
- [ ] Pi reachable → smoke **`failed=0`** (paste summary in verdict) OR explicit per-step `UnverifiedLive`; do NOT ACCEPT without one of these
```

## Verdict format

Return one of:
- **ACCEPT** — all gates green; for journey/route/runtime work when Pi reachable, include pasted smoke summary (`failed=0`) or explicit per-step `UnverifiedLive`; note anything to revisit later.
- **BOUNCE** — list each failing gate with the exact command output or the bad `file:line`, and the specific fix required. Smoke `failed>0` when Pi reachable is an automatic BOUNCE.
