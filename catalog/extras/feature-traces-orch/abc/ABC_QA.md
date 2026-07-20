# ABC Phase 5 — Final QA (tracks A + B + C)

**Date:** 2026-07-19  
**Baseline:** next15 PASS (218 tests → now 225); 8 goldens; camera sibling < 0.40

## Overall: **PASS**

A2 and C2 fixed (TS skip helper + README pointer). All automated gates green.

---

## Track results

| ID | Result | Evidence |
|---|---|---|
| **A1** | **PASS** | `export_feature_provenance` bin (`catalog/src/bin/export_feature_provenance.rs`, `Cargo.toml`); `core/frontend/public/assets/feature-provenance.json` present (8 golden journeys, `schema_version` 1); `cargo test` — `build_snapshot_has_nonempty_journeys`, `inspect_disk_usage_golden_fields`, `skip_reasons_shape_matches_reference_tags` |
| **A2** | **PASS** | `core/frontend/src/utils/feature_provenance.ts`: `featurePresentOn`, `formatAvailabilitySkipReason`, `availabilitySkip`, `lookupPrecomputedSkipReason` (precomputed tags → skip string\|null; other tags → precomputed-only message); `FeatureProvenanceView.vue` wired; `yarn eslint --fix` clean on helper + view |
| **A3** | **PASS** | `FeatureProvenanceView.vue`; route `/tools/feature-provenance` in `router/index.ts`; `menus.ts` advanced entry; timeline + presence chips + skip panel (reference tags); `yarn eslint src/views/FeatureProvenanceView.vue` clean |
| **B1** | **PASS** | Prior live smoke (iter 3): `enrich_feature_traces --jobs 2 --resume` on filtered set, 1 cluster processed, exit 0 (`memory.xml`, `memory_archive.md`). `--jobs` flag + `Using parallel enrich with {jobs} workers…` in `feature_trace_enrich.rs:3028` |
| **B2** | **PASS** | `shell.rs`: `GH_RATE_LIMIT` mutex, `is_github_rate_limited`, `wait_on_shared_rate_limit`; tests `run_with_retry_retries_rate_limit_with_shared_backoff`, `parallel_workers_retry_rate_limit_without_panic`; `cargo test -p blueos-catalog --lib` 225/225 |
| **C1** | **PASS** | `release_traces_workflow.sh --old /tmp/abc-old.json` exit 0; HTML at `reports/feature_trace_report.html`; diff stdout (none vs self-copy); `snapshots/.gitignore` present |
| **C2** | **PASS** | `feature-traces-orch/README.md` §Release & QA links `abc/RELEASE_CHECKLIST.md`, `abc/ABC_QA.md`, and `abc/release_traces_workflow.sh` usage |

---

## Regression invariants

| Gate | Result | Evidence |
|---|---|---|
| `cargo test -p blueos-catalog --lib` | **PASS** | 225 passed, 0 failed (2026-07-19) |
| `check-traces.sh` | **PASS** | drift 66 OVERRIDES agree; `--check-goldens: all 8 golden journeys match`; `ALL FEATURE-TRACES GATES GREEN` |
| 8 goldens | **PASS** | `feature_trace_report --check-goldens` in check-traces |
| Camera sibling < 0.40 | **PASS** | `ConfigureCameraStream` ↔ `ViewCameraStreams` ratio **0.100** (sibling_matrix) |
| `release_traces_workflow.sh` | **PASS** | exit 0 demo run |

---

## Open follow-ups

1. **Full parallel gh soak** — B1 validated filtered subset only; default remains `--jobs 1` for production enrich; full N-worker soak not CI-gated.
2. **Full provenance JSON in UI** — export writes 8 goldens to bundled asset (~42KB); full 94-journey `feature-provenance-full.json` gitignored; UI does not load full snapshot.
3. **Microservice deferred** — no `catalog_provenance_serve` in `start-blueos-core` / nginx (by design; static snapshot only).

---

## Operator release checklist (summary)

See `abc/RELEASE_CHECKLIST.md` for full flow:

1. `cargo run -p blueos-catalog --bin enrich_feature_traces -- --strict-goldens` (optional `--jobs 2 --resume` smoke)
2. Snapshot: `cp feature_traces.json extras/feature-traces-orch/snapshots/traces-<tag>.json`
3. `bash extras/feature-traces-orch/abc/release_traces_workflow.sh --old snapshots/traces-<old>.json`
4. `bash extras/feature-traces-orch/next11/check-traces.sh`
5. `bash gate.sh`
6. Regenerate UI asset: `cargo run -p blueos-catalog --bin export_feature_provenance`

---

## Follow-on F1–F2

| ID | Result | Evidence |
|---|---|---|
| **F1** multi-cluster soak | **PASS** | `abc/B_SMOKE.md`: 2 clusters processed with `--jobs 2 --resume --strict-goldens`, exit 0, 8/8 goldens |
| **F2** full UI | **PASS** | `FeatureProvenanceView.vue`: `v-switch` "All journeys" toggles `useFullSet`; `?full=1` query sync; loads `feature-provenance-full.json` |
