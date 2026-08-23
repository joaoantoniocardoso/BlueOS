# Requirements baseline delta: 1.5.0-beta.40

Dry-run P5 re-baseline loop. Catalog tree stayed on HEAD (`16a8faa533`); tag was not checked out.

## Target

| Field | Value |
|-------|-------|
| Tag | `1.5.0-beta.40` |
| SHA | `c1327ee917e202cbf97277f2e2c5bfd7846e8b1a` |
| Commit | `core: frontend: src: Show unknown MAVLink endpoint types` |
| Previous tag (`--since`) | `1.5.0-beta.39` (`7adad3f63`) |
| Previous snapshot (`--diff` stem) | `catalog/requirements-baselines/1.4.4-beta.21.json` |

Git `--since` (`1.5.0-beta.39`) and `--diff` stem (`1.4.4-beta.21`) differ by design.

## Impact work order (`--since 1.5.0-beta.39`)

| Range end | Repo diff paths | Impacted indexed paths | Module keys | Work-order entries |
|-----------|-----------------|------------------------|-------------|-------------------|
| `--until 1.5.0-beta.40` (`c1327ee9`) | 139 | 43 | 54 | 631 |
| implicit `HEAD` (`16a8faa5`) | 608 | 41 | 74 | 729 |

Measured work-order ratio (HEAD / until): 729 / 631 = 1.15.

Named `--until` JSON: `catalog/extras/requirements-baseline/1.5.0-beta.40/impact.json`.
HEAD contrast JSON: `/tmp/impact-head.json` (runner TMPDIR; forbidden location).

54 module keys under `impact.json` `modules` (journeys, pages, services clusters). Full list in JSON.

## Requirements snapshot

Written: `catalog/requirements-baselines/1.5.0-beta.40.json`

| Metric | 1.4.4-beta.21 | 1.5.0-beta.40 |
|--------|---------------|---------------|
| Total | 381 | 473 |
| Unknown statement | 19 | 20 |
| Unknown criteria | 64 | 85 |
| Contamination findings | 6 | 6 |

`by_kind` (1.5.0-beta.40): constraint 42, functional 100, interface 87, performance 80, robustness 63, system 101.

## Diff vs `1.4.4-beta.21` snapshot

Command: `requirements --version 1.5.0-beta.40 --diff 1.4.4-beta.21 --json`

| Category | Count |
|----------|-------|
| Added | 92 |
| Removed | 0 |
| Changed statement | 0 |
| Changed criteria | 0 |
| Changed availability | 0 |

This is not a two-tree loss check. Both sides filter the same HEAD catalog; `1.4.4-beta.21` and `1.5.0-beta.40` admit different requirement IDs via availability / version filters. Added 92 reflects that filter gap, not demonstrated loss or preservation across git trees. `removed=0` does not constitute a passed loss check. Identity trap (0/0/0 on same-tree filter) does not apply here; changed availability is still zero.

Full JSON: `catalog/extras/requirements-baseline/1.5.0-beta.40/diff.json`

## Extract / lint (current HEAD tree)

| Check | Result |
|-------|--------|
| `cargo run -q --bin extract` | exit 0 |
| `cargo run -q --bin provenance_lint` (read-only) | ran |
| `provenance_lint --fix` | skipped (dry run) |
| `generate_feature_presence` | skipped |
| `enrich_feature_traces` | skipped |

## Gate (`CARGO_TARGET_DIR` under `$HOME`)

| Step | Result |
|------|--------|
| `bash gate.sh` | ALL GATES GREEN (`GATE_EXIT=0`) |
| `harness_ratchet` | PASS |
| feature-traces local gate | PASS |

## Holes (expected for this wave)

- No DUT / runtime re-ground
- Step 10 per-module re-ground not dispatched (54 keys in until-range impact; no Extractor workers spawned)
- `generate_feature_presence` / `enrich_feature_traces` not run (would rewrite generated files on HEAD)
- `provenance_lint --fix` not run
- Doc citations (`../BlueOS-docs`) not diffed by `impact`
- No two-tree `--diff` (tag not checked out; snapshot diff is same-HEAD filter contrast only)
- HashMap `by_kind` byte-stability not verified on this write (serde HashMap key order can flip snapshot bytes without semantic change)

## Refused

- `git checkout 1.5.0-beta.40`
- `provenance_lint --fix`
- `generate_feature_presence`, `enrich_feature_traces`
- Step 10 re-ground dispatch
- DUT / runtime capture
- Commit, push, PR, GitHub comments
