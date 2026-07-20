# NEXT15 final QA (P1–P15)

**Overall: PASS**

`cargo test -p blueos-catalog --lib`: 218/218 passing. `bash
catalog/extras/feature-traces-orch/next11/check-traces.sh`: `ALL FEATURE-TRACES GATES GREEN`
(includes `--check-goldens`, all 8 pass). `bash catalog/gate.sh` (offline, no `BLUEOS_BASE`):
`ALL GATES GREEN` (fmt, clippy, test, extract, drift, export, check-traces all green).

> Note: an in-flight concurrent edit to `feature_trace_enrich.rs` (P6 `--jobs` work) was caught
> mid-write during this QA pass — a transient `cargo test` failure (missing `check_strict_goldens`
> arg) self-resolved ~30s later once the file settled (217→218 tests). Final state above is from
> a stable re-run after the file's mtime stopped advancing.

## Per-track

| Track | Status | Evidence |
|---|---|---|
| P1 — Stale cluster cleanup | PASS | `prune_stale_clusters` in `feature_trace_enrich.rs:2074` + 4 unit tests (`...keeps_cluster_with_a_live_journey`, `...drops_orphan_after_repoint`, `...drops_empty_by_journey`, `...keeps_shared_cluster_if_any_sibling_still_live`); wired into merge/checkpoint path (`:3064`). |
| P2 — Scratch cleanup | PASS | `catalog/examples/` no longer exists; `cargo build --examples` has nothing to build (no examples dir). |
| P3 — Gate wiring | PASS | `catalog/gate.sh:23-27` calls `next11/check-traces.sh` guarded by `[ -f "$check_traces" ]`; `bash catalog/gate.sh` exits 0 offline. |
| P4 — Resume UX | PASS | `"Resumed: {skipped} skipped (already enriched, ...), {processed} processed ..."` summary in `feature_trace_enrich.rs:2605`; 3 unit tests on the counter (`:2618,2626,2634`). |
| P5 — Cache schema version | PASS | `CACHE_VERSION: u64 = 1` (`:56`); envelope read/write treats missing/mismatched version as a cache miss (`:342-358`); tests `cached_json_treats_missing_cache_version_as_miss`, `...treats_mismatched_cache_version_as_miss`. |
| P6 — Parallel enrich (`--jobs`, default 1) | PASS | `--jobs` flag parsed (`:2876`, default `1`, `:2859`); `Mutex`-guarded queue/state/cache-write-lock (`CACHE_WRITE_LOCK`, `:59`); `concurrent_cache_writes_do_not_corrupt_the_file` test (`:507`). No live `N>1` `gh` smoke run asserted in CI (by design, non-goal). |
| P7 — Shared-legit (KEEP accept-shared) | PASS | `NEXT15_AUDIT.md` §P7: re-checked live `feature_traces.json`, no new `sibling_ratio ≥ 0.99` pairs beyond the 3 pinned `SharedCluster` groups; no classifier added. |
| P8 — Live Pickaxe `term_hit` | PASS | `NEXT15_AUDIT.md` §P8: 6 journeys re-enriched live, all `follow_up_prs.term_hit` are `Some(bool)`, never `None`; pre-existing `--strict-goldens` scoping bug documented (out of scope, not a P8 regression). |
| P9 — Backport audit | PASS | `NEXT15_AUDIT.md` §P9: all 27 `backport_prs` sampled/cross-checked against `base_ref` + `backport_title_re`; 0 FPs/FNs; no regex change, documented per design non-goal. |
| P10 — HTML artifact bundle | PASS | `--output`/`--out-dir` flag in `feature_trace_report.rs:722`; `catalog/extras/feature-traces-orch/reports/feature_trace_report.html` present on disk (143 KB). |
| P11 — Static HTML, no Vue | PASS | `reports/.gitignore` (`*` + `!.gitignore`) present; `README.md:14-15` has the 1-line regen pointer; no Vue component/route/HTTP endpoint added (grep of `core/frontend` for this feature: none). |
| P12 — Tag cross-links | PASS | `tag_release_url()` (`feature_trace_report.rs:408`) wraps numbered tags in `<a href="https://github.com/.../releases/tag/{tag}">`, returns `None` for `master`/`*-dev`; unit tests `tag_release_url_links_numbered_release_tag`, `...none_for_master_and_dev_channels`, plus an HTML-render assertion (`:1073`). |
| P13 — Runner skip by presence | PASS | Grep-audited independently: `journey_availability_skip`/`availability_skip` used only in `src/bin/journey_http.rs` (2 hits) across all 18 `src/bin/*.rs`; all other bins are static-analysis-only (no DUT-tag execution loop) — confirms design's frozen "no further wiring needed" call. |
| P14 — Skip explanation | PASS | Direct unit tests on `journey_availability_skip` itself in `runner.rs`: `journey_availability_skip_reports_reason_when_absent`, `journey_availability_skip_none_when_present` (`:1533,1552`), on top of existing `version.rs::availability_skip_when_absent`. |
| P15 — Traces diff | PASS | `catalog/src/bin/feature_trace_diff.rs` + `Cargo.toml:79-80` bin entry; `--help` prints usage; dry run against a synthetically-trimmed old snapshot correctly reports `+ ViewSystemInformation` under `## Journeys`. |

## Regression invariants

- **8 goldens intact**: `check-traces.sh --check-goldens` → "all 8 golden journeys match
  PRECISION_QA.md §1 + NEXT11_DESIGN.md N11". PASS.
- **Camera sibling ratio < 0.40**: `sibling_matrix` → `ConfigureCameraStream`/`ViewCameraStreams`
  = **0.100** (also `ConfigureVideoStream` pairs: 0.111, 0.065 — all well under threshold). PASS.
- **`cargo test -p blueos-catalog --lib`**: 218/218 passing (baseline 193 + new tracks' tests).
  PASS.
- **`check-traces.sh` / `bash gate.sh`**: both exit 0. PASS.

## Open follow-ups (non-blocking)

- **P6**: `--jobs N>1` has no live parallel `gh`-call smoke test in CI (only the `Mutex`
  cache-write-safety unit test) — matches the frozen design non-goal, but real rate-limit behavior
  under concurrency is still unverified against live GitHub.
- **P8**: pre-existing `check_strict_goldens` bug (hardcodes all 8 goldens regardless of the
  caller's `--journey`-scoped subset) causes false "not found" noise on scoped runs — documented,
  not fixed (out of P7/P9 scope).
- **P11 / Vue**: static-HTML-only decision remains deferred pending a real catalog HTTP API; no
  Vue tab exists to consume `feature_traces.json` live.
- **P13**: the grep-audit conclusion (only `journey_http.rs` is DUT-aware) lives in this QA doc and
  the design doc, not as a standalone dated audit entry in `NEXT15_AUDIT.md` — minor documentation
  gap, no functional impact.
