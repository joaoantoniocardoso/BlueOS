# Phase 1 — ABC design (tracks A, B, C)

Frozen baseline: next15 QA PASS (218 tests, 8 goldens, camera sibling < 0.40). P11 deferred Vue;
ABC track A **supersedes P11 for UX only** — still no production microservice.

## Frozen architecture decisions

### Track A — primary: static snapshot + Vue tools page (option 1)

**Choice:** generate a compact provenance JSON snapshot into `core/frontend/public/assets/feature-provenance.json`, serve it via existing nginx static frontend bundle, mount `FeatureProvenanceView.vue` at `/tools/feature-provenance`.

**Not primary:** `catalog_provenance_serve` bin — optional **local-dev-only** helper under `catalog/scripts/` for engineers who want to browse against an uncommitted `feature_traces.json` without rebuilding frontend; it does **not** ship on-vehicle and is **not** registered in `start-blueos-core`.

**Justification:** `nginx.conf` has no catalog/reports static location; every `/tools/*` route is a Vue page backed by existing services or bundled assets (`public/assets/vehicles/…`). A new microservice would require `pyproject.toml`, `start-blueos-core`, and `nginx.conf` wiring for data that is **build-time catalog output**, not runtime vehicle state. Bundled JSON matches offline UX, avoids CORS, and reuses the `SystemInformationView` tools pattern.

**DUT context:** default live tag from `GET /version-chooser/v1.0/version/current` (`tag` field). Override via `?dut=<tag>` query param (fixture/dev). Skip reasons resolved **client-side** from snapshot `availability` fields using a minimal TS port of `version.rs::{feature_present_on, format_availability_skip_reason}` — same strings as `journey_availability_skip` in `runner.rs`.

**Skip-reason source of truth:** Rust export embeds per-journey `FeatureAvailability` (from `journey_presence` / `feature_traces.json` journey row). Export bin may also precompute `skip_reasons: { "<tag>": "<reason>|null" }` for `REFERENCE_DUT_TAGS` (master, 1.4-dev, all `present_in_tags` union) at generation time so UI can lookup without duplicating logic for common tags; live/override tags use TS fallback mirroring `availability_skip`.

**Snapshot shape (compact):** top-level `schema_version`, `generated_at`, `reference_dut_tags[]`, `journeys[]` each with `id`, `intro_commit`, `availability`, optional `skip_reasons`, `presence_tags[]`, `timeline` (reuse `feature_trace_report::build_timeline` fields — PR lists, tags, pickaxe `term_hit`).

**Export command:** `cargo run -p blueos-catalog --bin export_feature_provenance` writes golden journeys (8) to `core/frontend/public/assets/feature-provenance.json` (~42KB UI default) and all 94 journeys to `feature-provenance-full.json` (gitignored, ~735KB; regenerate locally).

---

### Track B — parallel enrich target

**B1 smoke:** `enrich_feature_traces --jobs 2 --resume` on a **filtered** set first (`--journey` × 6 Pickaxe journeys from P8, or `--journey` golden subset); if green, optional full run with `--resume`. Requires `gh` auth + network; not CI-gated.

**B2 harden:** today `run_gh_dyn` → `shell::run_with_retry` retries per-worker independently (`RETRY_BACKOFFS_SECS`); `403`/`rate limit` are **not** transient. Add **process-wide** `GH_RATE_LIMIT` mutex: on 403/rate-limit stderr, holder sleeps with shared exponential backoff before any worker retries; extend `is_transient` for `403` + `rate limit` when secondary limiter engaged. Keep `CACHE_WRITE_LOCK`; no new deps.

**Success criteria:** smoke exits 0; `feature_traces.json` valid schema v2; resume summary matches skipped/processed counts; `concurrent_cache_writes_do_not_corrupt_the_file` + full `cargo test -p blueos-catalog --lib` green; `check-traces.sh --check-goldens` unchanged; document observed 403/backoff in `ABC_QA.md`.

**Safe fallback:** if live smoke still flakes, default remains `--jobs 1`, document max safe N, ship B2 shared limiter anyway.

---

### Track C — release workflow script

**Script:** `catalog/extras/feature-traces-orch/abc/release_traces_workflow.sh`

**Steps:** (1) snapshot `catalog/feature_traces.json` → `catalog/extras/feature-traces-orch/snapshots/feature_traces_<label>.json` (label = arg or `git describe`/timestamp); (2) `cargo run -p blueos-catalog --bin feature_trace_report -- --format html --output catalog/extras/feature-traces-orch/reports`; (3) if `--old <path>` given, `cargo run -p blueos-catalog --bin feature_trace_diff -- <old> <new>`; (4) print release checklist (gate.sh, check-traces, diff summary, HTML path).

**Tag usage:** `./release_traces_workflow.sh --label 1.4.0 --old snapshots/feature_traces_1.3.2.json` after enrich; store snapshots beside reports (gitignored dir + `.gitignore`).

---

## Per-track specs

### A1 — Provenance snapshot export
| | |
|---|---|
| **Goal** | Build-time JSON consumable by frontend without a catalog HTTP API. |
| **AC** | `export_feature_provenance` writes valid JSON to `public/assets/`; includes all journeys' timelines + `availability`; `cargo test` covers export shape. |
| **Files** | `catalog/src/bin/export_feature_provenance.rs`, `catalog/src/tools/feature_trace_report.rs` (shared timeline), `catalog/Cargo.toml`, `core/frontend/public/assets/feature-provenance.json` (generated). |
| **Non-goals** | No FastAPI service, no nginx location, no runtime `gh`. |
| **Deps** | P10/P12 report helpers; `feature_traces.json` current. |

### A2 — Skip-reason surface in snapshot
| | |
|---|---|
| **Goal** | UI shows `journey_availability_skip`-equivalent strings without calling Rust at runtime. |
| **AC** | Each journey has serializable `availability`; optional `skip_reasons` for reference tags; TS helper matches `version.rs` unit-test cases (`1.4-dev` absent, `master` present). |
| **Files** | export bin, `core/frontend/src/utils/feature_provenance.ts` (skip helper). |
| **Non-goals** | No duplicate `AvailabilitySkip` enum in TS beyond string formatting. |
| **Deps** | A1; `runner.rs`/`version.rs` strings frozen. |

### A3 — Vue provenance + skip panel
| | |
|---|---|
| **Goal** | Operators browse journey provenance and skip reasons on-vehicle. |
| **AC** | Route `/tools/feature-provenance`; timeline + presence chips; skip chip when DUT tag unavailable; `yarn lint` clean; advanced menu entry. |
| **Files** | `core/frontend/src/views/FeatureProvenanceView.vue`, `router/index.ts`, `menus.ts`, optional `catalog/scripts/catalog_provenance_serve.sh` (dev only). |
| **Non-goals** | No edit/enrich from UI; no Graphify. |
| **Deps** | A1, A2; version-chooser current API. |

### B1 — Live `--jobs 2` smoke
| | |
|---|---|
| **Goal** | Prove parallel enrich against real `gh` under concurrency. |
| **AC** | Documented command completes on filtered set; resume line printed; no cache/version corruption. |
| **Files** | `ABC_QA.md` (evidence), optional `abc/smoke_parallel.sh`. |
| **Non-goals** | Not added to `gate.sh` (network). |
| **Deps** | P6 mutex baseline. |

### B2 — Parallel harden
| | |
|---|---|
| **Goal** | Shared backoff so N workers don't amplify 403s. |
| **AC** | Shared limiter wired through `run_gh_dyn`; unit test simulates staggered 403; goldens intact. |
| **Files** | `catalog/src/tools/shell.rs`, `feature_trace_enrich.rs`. |
| **Non-goals** | No thread-pool crate. |
| **Deps** | B1 findings. |

### C1 — Release diff workflow
| | |
|---|---|
| **Goal** | One script snapshots, reports, diffs. |
| **AC** | Script exits 0 on demo old/new pair; HTML under `reports/`; diff stdout captured. |
| **Files** | `abc/release_traces_workflow.sh`, `snapshots/.gitignore`. |
| **Non-goals** | No HTML diff view (text diff only, P15). |
| **Deps** | `feature_trace_diff`, `feature_trace_report` bins. |

### C2 — QA checklist
| | |
|---|---|
| **Goal** | Operator-facing release steps for catalog maintainers. |
| **AC** | `ABC_QA.md` checklist: enrich → snapshot → workflow script → gate.sh → check-traces; tag example documented. |
| **Files** | `abc/ABC_QA.md`, pointer in `feature-traces-orch/README.md`. |
| **Non-goals** | No CI workflow YAML. |
| **Deps** | C1. |

---

## Phase order (unchanged)

1. Design (this doc) → 2. B → 3. C → 4. A → 5. `ABC_QA.md` final.

## Regression invariants

8 goldens, camera sibling < 0.40, `cargo test -p blueos-catalog --lib`, `check-traces.sh`, `gate.sh` — all must remain green after each track.
