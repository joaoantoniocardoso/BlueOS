# Phase 1 — NEXT15 design (P1–P15)

Frozen decisions: **P7 → KEEP accept-shared, no classifier** (document only — N4's audit already
proved title/body signal absent for 3/4 shared groups; no new pairs surfaced this round to justify
revisiting); **P11 → static HTML artifact, no Vue microservice** (no catalog HTTP API exists for a
Vue tab to fetch from — same rationale as N8); **P6 → `--jobs N` bounded concurrency, default `1`**
(mutex-guarded cache writes; ships real if the `gh`-call graph proves safely parallelizable,
otherwise flag lands with default-1 semantics + a concurrency-safety unit test only, no live
parallel run); **P13/P14 → reuse existing `journey_availability_skip`/`format_availability_skip_reason`
API** (already in `runner.rs`/`version.rs`, already wired into `journey_http.rs` — no new API
needed; the gap is coverage, not existence).

## P1 — Stale cluster cleanup
**Goal:** drop/orphan `intro_clusters` entries whose sha no longer matches any journey's current
`intro_commit` (NEXT11_QA open follow-up: old `8da1baa6c56a` cluster still lists
`ConfigureVideoStream` post-N1 re-point).
**AC:** `feature_trace_enrich` (or a small standalone pass) removes/flags clusters with empty
live `by_journey` (no journey currently points at that sha); `intro_clusters.len()` test count
drops accordingly; existing 8 goldens + camera gate unaffected.
**Files:** `catalog/src/tools/feature_trace_enrich.rs` (merge/checkpoint path), `catalog/src/feature_trace.rs` (count test).
**Non-goals:** no rewrite of `by_journey` grouping semantics.
**Deps:** none.

## P2 — Scratch cleanup
**Goal:** remove `catalog/examples/n4_check.rs` (untracked scratch from N4).
**AC:** file deleted; `cargo build --examples` still succeeds.
**Files:** delete `catalog/examples/n4_check.rs`.
**Non-goals:** no promotion to a real example. **Deps:** none.

## P3 — Gate wiring
**Goal:** `catalog/gate.sh` invokes the feature-traces local check.
**AC:** `gate.sh` gains an optional/final step calling `check-traces.sh` (guarded so it doesn't
require `gh` network access by default — mirrors N3's `--strict-goldens` opt-out); `bash
catalog/gate.sh` still exits 0 with no network.
**Files:** `catalog/gate.sh`.
**Non-goals:** no change to check-traces.sh's own gate order.
**Deps:** none.

## P4 — Resume UX
**Goal:** enrich prints a skip/resume summary (X skipped, Y processed) after the run, not just
per-cluster lines.
**AC:** after the `for (i, (sha, group))` loop, one summary line: `Resumed: N skipped (already
enriched), M processed`; unit test on the counter, not stdout scraping.
**Files:** `catalog/src/tools/feature_trace_enrich.rs` (`run`, near existing `println!("Wrote
{out_rel}...")`).
**Non-goals:** no new progress-bar dependency.
**Deps:** builds on existing N10 `--resume`/checkpoint code (already present).

## P5 — Cache schema version
**Goal:** version-stamp `.cache/gh_traces` entries; invalidate stale cache on mismatch (today only
`feature_traces.json` has `schema_version: 2`; the per-key gh cache has none).
**AC:** each cache file gains a `cache_version` field; on read, mismatch → treat as miss (re-fetch)
instead of using stale shape; unit test with a hand-written old-version cache file.
**Files:** `catalog/src/tools/feature_trace_enrich.rs` (cache read/write helpers).
**Non-goals:** no cache migration tool; mismatched entries are just re-fetched, not upgraded in place.
**Deps:** none.

## P6 — Parallel enrich (frozen: `--jobs N`, default 1)
**Goal:** optional bounded concurrency for the cluster enrich loop without breaking `gh` rate
limits or the `.cache/gh_traces` cache.
**AC:** `--jobs N` flag (default `1` = today's sequential behavior, unchanged golden output);
cache writes behind a `Mutex`/single-writer discipline; if a live `N>1` smoke run risks rate-limit
flakiness, ship the flag + a `Mutex`-safety unit test (concurrent writers don't corrupt a cache
file) without asserting a live parallel `gh` run in CI.
**Files:** `catalog/src/tools/feature_trace_enrich.rs`.
**Non-goals:** no thread pool framework dependency (use `std::thread`/`std::sync::Mutex` only).
**Deps:** P5 (cache version check must be race-safe under concurrent writers).

## P7 — Shared-legit classifiers (frozen: KEEP accept-shared)
**Goal:** close the reopened "classifier?" question from PLAN.md.
**AC:** `NEXT15_DESIGN.md` (this doc) + a short note in `NEXT15_AUDIT.md` (P9) recording: no new
shared-cluster groups surfaced since N4's audit; default remains accept-shared (pinned
`sibling_ratio == 1.0` tests from N4), no title/body classifier added.
**Files:** doc only (`NEXT15_AUDIT.md`).
**Non-goals:** no classifier code, no Pickaxe-based re-splitting beyond what N5/N6 already do.
**Deps:** N4 (baseline).

## P8 — Live Pickaxe term_hit
**Goal:** journeys carrying a `Pickaxe` override actually show scored `term_hit` in committed
`feature_traces.json` (N5 added the field; today's data may still show `None` if those journeys
weren't re-enriched live).
**AC:** run `enrich_feature_traces --journey <pickaxe-journeys>` for every journey with a
`Pickaxe` override; commit refresh shows `term_hit: Some(bool)` for their `follow_up_prs`; 8
goldens unaffected (scoped merge, N2).
**Files:** data refresh only (`catalog/feature_traces.json`); no code change unless a bug surfaces.
**Non-goals:** no new Pickaxe terms.
**Deps:** N2/N5 (scoped merge, term_hit field) — both already land.

## P9 — Backport audit
**Goal:** sample/audit `discover_backport_prs`'s `backport_title_re` for FPs/FNs.
**AC:** `NEXT15_AUDIT.md` §P9 lists a manual sample (≥5 PRs) checked against the regex; any
confirmed miscategorization gets a regex fix + regression test, else documented as acceptable
with rationale (mirrors N4's audit style).
**Files:** `catalog/src/tools/feature_trace_enrich.rs` (regex, only if a real bug found), new
`NEXT15_AUDIT.md`.
**Non-goals:** no full re-architecture of backport detection.
**Deps:** none.

## P10 — HTML artifact bundle
**Goal:** a script/enrich step writes the `--format html` report to disk instead of only stdout.
**AC:** new small helper (in `feature_trace_report.rs` or a thin wrapper script) writes
`catalog/extras/feature-traces-orch/reports/<journey>.html` for each golden (or all journeys);
same HTML content as existing `--format html`, just persisted.
**Files:** `catalog/src/tools/feature_trace_report.rs` (add `--out-dir` or reuse stdout redirect
in a wrapper script), new `reports/` dir (gitignored, see P11).
**Non-goals:** no new templating engine.
**Deps:** existing N7/N8 HTML render path.

## P11 — Static HTML artifact (frozen: no Vue microservice)
**Goal:** freeze the surface decision explicitly for P10's output location.
**AC:** `catalog/extras/feature-traces-orch/reports/` holds the generated HTML (gitignored via a
new `reports/.gitignore` or entry, since it's regenerable); optional one-line link/mention added
to an existing extras README pointing at how to regenerate it; **no** Vue component, route, or
catalog HTTP endpoint is added.
**Files:** `catalog/extras/feature-traces-orch/reports/.gitignore` (new), `catalog/extras/feature-traces-orch/README.md` (1-line pointer, optional).
**Non-goals:** no `SystemInformationView.vue` tab, no live-fetch API — same rationale as N8.
**Deps:** P10 (this doc only freezes the decision; P10 does the writing).

## P12 — Tag cross-links
**Goal:** report links `present_in_tags` chips (N9) to actual GitHub tag/release URLs where derivable.
**AC:** HTML chip render (`feature_trace_report.rs`) wraps each tag chip in an `<a
href="https://github.com/bluerobotics/BlueOS/releases/tag/{tag}">` when the tag looks like a real
release tag (matches existing `parse_release_tag` shapes); untagged/`master`/`1.4-dev` chips stay
plain `<span>`; unit test on the href-generation helper only (no network fetch).
**Files:** `catalog/src/tools/feature_trace_report.rs`.
**Non-goals:** no live GitHub API call to verify tag existence.
**Deps:** N9 (chip render path).

## P13 — Runner skip by presence (frozen: reuse existing API)
**Goal:** every journey-execution entrypoint respects `journey_availability_skip`, not just
`journey_http.rs`'s `--dut-tag` path (today `frontend_smoke.rs`/coverage bins don't check
availability at all — real gap, not missing API).
**AC:** any additional runner path that iterates journeys against a real/simulated DUT tag calls
`journey_availability_skip`/`availability_skip` before executing steps; if no other path actually
targets a DUT tag (most coverage bins are static-analysis-only), document that `journey_http.rs`
remains the sole DUT-aware runner and no further wiring is needed — verified by grep audit, not
assumed.
**Files:** audit only, possibly `catalog/src/bin/journey_http.rs` if a second DUT-aware path is found.
**Non-goals:** no new skip-reason enum variant (existing `AvailabilitySkip::NotPresentOnDut` covers it).
**Deps:** none — API already exists (`runner.rs:1112`, `version.rs:145,175`).

## P14 — Skip explanation (frozen: reuse existing API)
**Goal:** confirm/extend the human-readable reason string already produced by
`format_availability_skip_reason`.
**AC:** add a direct unit test in `runner.rs` for `journey_availability_skip` itself (today only
`version.rs::availability_skip_when_absent` covers the underlying logic, and coverage is indirect
via `journey_http.rs`); reason format stays `"not present on {tag} (intro {sha12}; first tag
{tag})"` — no new fields unless P13's audit finds a runner needing extra context.
**Files:** `catalog/src/runner.rs` (new `#[test]`).
**Non-goals:** no i18n/formatting rework.
**Deps:** P13 (audit informs whether more context is needed).

## P15 — Traces diff
**Goal:** a bin/script diffing two `feature_traces.json` snapshots for changelog provenance.
**AC:** new `catalog/src/bin/feature_trace_diff.rs` takes two file paths, prints added/removed
journeys, added/removed `follow_up_prs`/`backport_prs` per journey, and new/orphaned `intro_clusters`;
unit test with two small synthetic JSON fixtures.
**Files:** new `catalog/src/bin/feature_trace_diff.rs`, `catalog/Cargo.toml` (bin entry if needed).
**Non-goals:** no HTML diff view (text/markdown only); no git-blame integration.
**Deps:** none — reads committed JSON only.

## Implementation order (Phase 2–5)
1. **Phase 2 (hygiene+engine, P1–P6):** P2 (delete scratch) → P1 (stale clusters) → P5 (cache
   version) → P6 (jobs flag, needs P5) → P4 (resume summary) → P3 (gate wiring, last so the
   script list is final).
2. **Phase 3 (semantics+audit, P7–P9):** P7 (doc-only) → P9 (backport audit) → P8 (live enrich,
   run last so it captures any P9 regex fix).
3. **Phase 4 (surfaces+diff, P10–P12, P15):** P12 (chip links, independent) → P10 (HTML bundle) →
   P11 (freeze location, same PR as P10) → P15 (diff bin, independent).
4. **Phase 5 (runner, P13–P14):** P13 (audit) → P14 (test, informed by P13).
5. **Phase 6:** NEXT15_QA.md.

## Regression invariants (every phase)
- 8 goldens intact: `InspectZenohNetwork`, `ChangeUiThemeColor`, `InspectDiskUsage`,
  `RunInternetSpeedTest`, `LevelHorizon`, `AccessWebTerminal`, `InspectMavlinkMessagesInBrowser`,
  `CalibrateGyroscope` (`feature_trace_report --check-goldens`).
- Camera sibling ratio (`ConfigureCameraStream`/`ViewCameraStreams`) stays `< 0.40` (currently
  ~0.100, `sibling_matrix`/cargo gate).
- `cargo test -p blueos-catalog --lib` all green (193 baseline + new tracks' tests).
- `drift_check.sh` / `check-traces.sh` exit 0.
