# Requirements baseline delta: 1.4.4-beta.21

Dry-run P4 re-baseline loop. Catalog tree stayed on HEAD (`ce8a8cf3`); tag was not checked out.

## Target

| Field | Value |
|-------|-------|
| Tag | `1.4.4-beta.21` |
| SHA | `2853762783633f82b7ab242acb9822807f73bcd3` |
| Commit | `core: services: helper: Return null when no previous internet speedtest` |
| Previous tag | `1.4.4-beta.20` (`357f87b39`) |
| Previous snapshot | `catalog/requirements-baselines/1.4-dev.json` |

## Impact work order (`--since 1.4.4-beta.20`)

| Range end | Repo diff paths | Impacted indexed paths | Modules | Work-order entries |
|-----------|-----------------|------------------------|---------|-------------------|
| `--until 1.4.4-beta.21` | 1 | 1 | 2 | 28 |
| implicit `HEAD` (`ce8a8cf3`) | 944 | 168 | 77 | 1480 |

`--until 1.4.4-beta.21` narrows the diff to one file: `core/services/helper/main.py`.

Modules (beta.21 range):

- `journeys/helper.rs` -- 4 entries (journeys: monitor_internet_connectivity, verify_internet_connectivity, browse_available_web_services, probe_interface_internet_connectivity)
- `services/helper.rs` -- 24 entries (service helper observed citations)

Full JSON: `catalog/extras/requirements-baseline/1.4.4-beta.21/impact.json`

## Requirements snapshot

Written: `catalog/requirements-baselines/1.4.4-beta.21.json`

| Metric | 1.4-dev | 1.4.4-beta.21 |
|--------|---------|---------------|
| Total | 381 | 381 |
| Unknown statement | 19 | 19 |
| Unknown criteria | 64 | 64 |
| Contamination findings | 6 | 6 |

## Diff vs `1.4-dev` snapshot

Command: `requirements --version 1.4.4-beta.21 --diff 1.4-dev --json`

| Category | Count |
|----------|-------|
| Added | 0 |
| Removed | 0 |
| Changed statement | 0 |
| Changed criteria | 0 |
| Changed availability | 0 |

No lost requirements, and this run could not have found one. Both `--version 1.4-dev` and `--version 1.4.4-beta.21` filter the same HEAD catalog: `1.4-dev` uses `present_on_1_4_dev`, and `1.4.4-beta.21` is newer than every presence-table entry so `is_stale_table_present` admits the same 381 IDs. Combined with never checking the tag out, both sides of `--diff` are the same derivation. The 0/0/0 result is that identity, not a demonstration that nothing was lost. `--diff` itself still reports added/removed/changed when a snapshot is actually different.

## Extract / lint (current HEAD tree)

| Check | Result |
|-------|--------|
| `cargo run -q --bin extract` | exit 0 |
| `cargo run -q --bin provenance_lint` (read-only) | 0 unresolved |
| `provenance_lint --fix` | skipped (would rewrite citations against HEAD, not tag) |

## Gate (`CARGO_TARGET_DIR` under `$HOME`, `../BlueOS-docs` present)

| Step | Result |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy -D warnings` | PASS |
| `cargo test` | PASS |
| `bash gate.sh` | ALL GATES GREEN |
| `harness_ratchet` | PASS |
| feature-traces local gate | PASS |

## Holes (expected for this wave)

- No DUT / runtime re-ground
- No per-cluster module re-examination (impact lists helper only for the tag range; the other 75 modules appear only under HEAD-anchored `--until`)
- `generate_feature_presence` / `enrich_feature_traces` not run (would rewrite generated files on HEAD)
- `provenance_lint --fix` not run
- Doc citations (`../BlueOS-docs`) not diffed by `impact`
- Step 7 (`requirements --diff`) was not exercised against two different trees. See Diff vs `1.4-dev` snapshot.

## Refused

- `git checkout 1.4.4-beta.21`
- `provenance_lint --fix`
- `generate_feature_presence`, `enrich_feature_traces`
- Commit, push, PR, GitHub comments
