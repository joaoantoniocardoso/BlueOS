# Phase 1 — NEXT11 design (N1–N11)

Frozen decisions: **N8/N9 → HTML (+JSON) surface** (`feature_trace_report --format html` is
primary; Vue is non-goal/deferred — no catalog HTTP API to fetch from, and bundling a static JSON
snapshot into `SystemInformationView.vue` was rejected for this local catalog crate); **N11 goldens →
`AccessWebTerminal`, `InspectMavlinkMessagesInBrowser`, `CalibrateGyroscope`** (rationale below).

## N1 — ConfigureVideoStream hub narrowing
**Goal:** stop `ConfigureVideoStream` resolving to the whole `video-manager/` tree (13 paths,
fu=37); scope it to files no camera sibling already claims.
**AC:** add `Override::Path` for `VideoDiagnosticHelper.vue` + `VideoThumbnail.vue` (the only
`frontend_video.rs`-cited files not already an override target of `ViewCameraStreams`
(`video.ts`/`VideoManager.vue`), `ConfigureCameraStream` (`VideoStreamCreationDialog.vue`), or
`RemoveCameraStream` (`VideoStream.vue`)); `sibling_ratio("ConfigureVideoStream",
"ViewCameraStreams") < 0.40` and same vs `ConfigureCameraStream`, both asserted in
`feature_trace.rs` tests; `fu` drops from 37.
**Files:** `catalog/src/tools/feature_presence.rs` (OVERRIDES), `catalog/src/feature_trace.rs`
(2 new tests), `catalog/src/tools/sibling_matrix.rs` (PAIRS +2).
**Non-goals:** no capability/step edits to `journeys/frontend_video.rs`; don't touch the
already-correct MODULE_S camera overrides.
**Deps:** none; unblocks N11 (ConfigureVideoStream excluded from goldens due to fu volatility).

## N2 — `--journey` scoped merge
**Goal:** `enrich_feature_traces --journey X` merges X's cluster into the existing
`feature_traces.json` instead of overwriting `journeys`/`intro_clusters` with only X.
**AC:** after `--journey CalibrateGyroscope`, all other journeys' entries are byte-identical to
before the run; X's entry is refreshed; new test `journey_filter_merges_not_overwrites` (build
two fake outputs, run merge fn, assert union). Root cause: `run()` (`feature_trace_enrich.rs:2081-2144`)
sources `journeys` from `feature_presence_map.json` (already pre-filtered) then blind-writes
`out_journeys`/`clusters` — never reads the prior `feature_traces.json`.
**Files:** `catalog/src/tools/feature_trace_enrich.rs` (`run`: read existing `out_path` first when
`journey_filter.is_some()`, union `journeys` by id, union `intro_clusters` by sha, union
`commits`/`pull_requests`/`issues` maps).
**Non-goals:** no change to full (`--journey`-less) run semantics; no dedup of `by_commit` grouping.
**Deps:** shares merge helper with N10.

## N3 — Local `check-traces.sh`
**Goal:** one entrypoint for the 3 local gates already in use ad hoc.
**AC:** `catalog/extras/feature-traces-orch/next11/check-traces.sh` runs, in order:
`improve/drift_check.sh` → `cargo run --bin sibling_matrix` → `cargo run --bin feature_trace_report
-- --check-goldens`; exits non-zero on first failure; documents `enrich_feature_traces --strict-goldens`
as a separate opt-in step (not run by default — it hits `gh`).
**Files:** new `check-traces.sh` (executable, `set -euo pipefail`, mirrors `catalog/gate.sh` style).
**Non-goals:** not wired into `catalog/gate.sh` (feature-traces are extras, not core catalog gate).
**Deps:** N6/N11 additions must land first so the script's golden/sibling list is current.

## N4 — Shared-legit policy
**Goal:** close the "classifier vs documented-model" fork left open by `IMPROVE_AUDIT.md` T3.
**AC:** no classifier (title/body signal already proven absent for 3/4 groups by real `-S`
pickaxe in T3); instead add `NEXT11_AUDIT.md` §N4 formally modeling `RebootOnboardComputer`/
`ShutdownOnboardComputer`, the 3 versionchooser journeys, and `InstallExtension`/
`UninstallExtension` as `SharedCluster` groups, each with a test asserting `sibling_ratio(...) ==
1.0` (documents the pin, catches accidental future separation) — inverse of the N1/N6 `<0.40` gate.
**Files:** `catalog/src/feature_trace.rs` (3 new `_is_shared_cluster` tests), new `NEXT11_AUDIT.md`.
**Non-goals:** no classifier code; `StartAutopilot`/`Stop`/`Restart` excluded (already separated
via Pickaxe in T3, ratio no longer 1.0 — belongs to N6 instead).
**Deps:** none.

## N5 — Pickaxe relevance score
**Goal:** surface, per follow-up PR, whether its `Pickaxe` term hit the title or a changed
filename (today `entry_mentions_token` decides `require_token` filtering but the boolean itself
isn't exposed).
**AC:** `JourneyTimeline`/report JSON gains `follow_up_prs[].term_hit: Option<bool>` (`Some` only
when the journey carries a `Pickaxe` override, else `None`); `feature_trace_report --format md`
prints `(term match)` suffix when true; unit test with a synthetic PR title.
**Files:** `catalog/src/tools/feature_trace_report.rs` (`PrRef` +field, `pr_ref`/`pr_line`),
`catalog/src/tools/feature_trace_enrich.rs` (expose `journey_pickaxe_term` as `pub(crate)`).
**Non-goals:** no new scoring UI in `sibling_matrix`; score is boolean hit, not a weighted number.
**Deps:** reuses N4/T3's `journey_pickaxe_term`.

## N6 — Expand sibling cargo gates
**Goal:** ≥5 hard sibling-ratio assertions (today 3: camera, autopilot, helper).
**AC:** add `wifi_pair_sibling_ratio_below_gate` (`ConnectToWifiNetwork`/`ForgetSavedWifiNetwork`,
`<0.40`) and `eeprom_pair_sibling_ratio_below_gate`
(`InspectRaspberryEepromBootloader`/`UpdateRaspberryEepromBootloader`, `<0.40`); both pairs
already in `sibling_matrix::PAIRS`, just unasserted in cargo.
**Files:** `catalog/src/feature_trace.rs` only.
**Non-goals:** no new pairs beyond `PAIRS` (N1 adds 2 there; N6 asserts 2 of the pre-existing 8).
**Deps:** independent; can land before or after N1.

## N7 — Report polish (links, merge method, HTML)
**Goal:** timeline output actually surfaces the GitHub link and merge method already stored.
**AC:** `pr_line`/`issue_line` markdown render `[#N](url)` instead of bare `#N` when `url.is_some()`;
`PrRef` gains `merge_method: Option<String>` (from `feature_trace::pull_request().merge_method`),
rendered as `(squash)`/`(merge)`/`(rebase)` suffix; `Format::Html` variant added, `--format html`
writes a minimal styled table (reuse `timeline_events`, no new deps); `check_goldens()` unaffected
(asserts PR number sets, not string formatting).
**Files:** `catalog/src/tools/feature_trace_report.rs`.
**Non-goals:** no CSS framework, no JS; HTML is server-rendered `String`.
**Deps:** none.

## N8 — HTML/JSON provenance surface (DECISION: HTML primary, Vue non-goal)
**Goal:** a no-build, no-fetch surface for a journey's timeline: N7's `--format html` output.
**Why HTML/JSON over Vue:** catalog output isn't served by a running BlueOS API — there's no
endpoint for a Vue tab to fetch from. Bundling a static JSON snapshot into
`SystemInformationView.vue` (the alternative explored) was rejected: it couples a local
dev/CI-only catalog-crate artifact to the shipped frontend build for no runtime benefit and no
live data. N7 already emits a styled, self-contained HTML table plus the underlying JSON — that
pair is the primary surface; no frontend changes.
**AC:** `feature_trace_report --format html` (N7) is the primary provenance surface — opening the
generated HTML file (or reading its JSON) shows the full timeline; no Vue component/route/tab
exists or is planned.
**Files:** `catalog/src/tools/feature_trace_report.rs` (N7's existing HTML/JSON paths; no new file
for N8 itself).
**Non-goals:** no Vue component, no `SystemInformationView.vue` tab, no bundled JSON snapshot in
`core/frontend`, no catalog HTTP API/live fetch.
**Deps:** N7 (HTML/JSON format is the vehicle).

## N9 — Presence chips in the HTML timeline (same surface as N8)
**Goal:** show `present_in_tags`/`present_on_1_4_dev` next to the HTML timeline from N8.
**AC:** `feature_trace_report --format html` renders a small tag/channel chip row (inline-styled
`<span>`s, no CSS framework/JS) above each journey's timeline, sourced from
`present_in_tags`/`present_on_1_4_dev` (already in `feature_presence_map.json`, threaded into
`JourneyTimeline`); same fields appear in the JSON output.
**Files:** `catalog/src/tools/feature_trace_report.rs` (`JourneyTimeline` +2 fields sourced from
`feature_presence_map.json`; HTML render fn).
**Non-goals:** no separate presence-only page; no Vue.
**Deps:** N8 (same HTML/JSON render path).

## N10 — Enricher resume after interrupt
**Goal:** a killed multi-commit enrich run doesn't redo already-built clusters.
**AC:** after each `build_intro_cluster` in the `by_commit` loop, write the accumulated
`clusters`/`ctx.{commits,pull_requests,issues}` to `out_path` (same merge path as N2, so partial
progress is valid `feature_traces.json` at any point); re-running the same command after a
simulated kill (`kill -9` mid-loop in a smoke test) skips shas already present with an identical
group signature and only processes the remainder; offline unit test constructs a partial
`clusters` map and asserts the loop's "already done" filter skips it.
**Files:** `catalog/src/tools/feature_trace_enrich.rs` (`run`: checkpoint write inside the
`for (i, (sha, group))` loop; skip-if-present check before `build_intro_cluster`).
**Non-goals:** no separate progress-file format — `feature_traces.json` is the checkpoint (gh
per-key cache under `.cache/gh_traces` already gives sub-cluster resume for free).
**Deps:** N2 (merge-not-overwrite is the same code path).

## N11 — Golden corpus growth (frozen)
**New goldens:** `AccessWebTerminal` (fu=[659,2279], backport=[]), `InspectMavlinkMessagesInBrowser`
(fu=[3310], backport=[]), `CalibrateGyroscope` (fu=[3443], backport=[3867]) — all read directly
from the current committed `catalog/feature_traces.json`, chosen for minimal/stable follow-up sets
(1-2 items) so future re-enrich churn is low; `CalibrateGyroscope` also exercises the
calibration-family backport (3867, shared with `LevelHorizon`), regression-testing N4's shared
`3867` backport fan-out doesn't silently grow.
**AC:** 3 new `match build_timeline(...)` blocks in `feature_trace_report::check_goldens()`
asserting the exact sets above; 3 new IDs appended to both `GOLDEN_JOURNEY_IDS` consts
(`feature_trace_report.rs:23`, `feature_trace_enrich.rs:1605` — kept in sync, not merged into one
const, per minimal-diff); `--strict-goldens` and `--check-goldens` both pass for all 8 goldens.
**Files:** `catalog/src/tools/feature_trace_report.rs`, `catalog/src/tools/feature_trace_enrich.rs`.
**Non-goals:** `ConfigureVideoStream` excluded (fu=37, pre-N1 volatile; revisit post-N1, not now).
**Deps:** none — reads current data; no re-enrich required to freeze the numbers.

## Implementation order (Phase 2–5)
1. **Phase 2 (core engine):** N2 (merge) → N10 (resume, same code path) → N1 (video override) → N5 (pickaxe score).
2. **Phase 3 (gates & goldens):** N6 (cargo asserts) → N11 (goldens, needs N1 landed so
   `ConfigureVideoStream` isn't a moving target) → N3 (check-traces.sh, needs final golden/pair lists).
3. **Phase 4 (shared-legit):** N4 (audit doc + pinning tests) — independent, can run parallel to Phase 3.
4. **Phase 5 (surfaces):** N7 (report fields) → N8 (HTML/JSON surface) → N9 (presence chips, same HTML render).

## Regression invariants (every phase, via N3's `check-traces.sh`)
- Prior 5 goldens intact: `InspectZenohNetwork`, `ChangeUiThemeColor`, `InspectDiskUsage`,
  `RunInternetSpeedTest`, `LevelHorizon` (`feature_trace_report --check-goldens`).
- Camera sibling ratio (`ConfigureCameraStream`/`ViewCameraStreams`) stays `< 0.40` (`sibling_matrix`, at 0.100 today).
- `cargo test -p blueos-catalog --lib` all green (160 baseline + new tests per track).
- `drift_check.sh` reports 0 OVERRIDES drift between `feature_presence_map.json`/`feature_traces.json`.
