# Discovery Improve — false-positive audit of `feature_traces.json`

Audits `catalog/feature_traces.json` (30 `intro_clusters`) against
`catalog/src/tools/feature_trace_enrich.rs` (`discover_follow_up_prs`,
`discover_backport_prs`, `discovery_paths`, `is_repo_wide_sweep`).

## 1. Cluster size distribution

`follow_up_prs` sizes across 30 clusters: max 61, mean ~28, **20/30 clusters
have >15 follow-ups**. `backport_prs`: max 6, mean ~1.8 (much healthier —
the `backport_title_re` + release-line gate already keeps this tight).

The 20 outliers are *all* old bootstrap-era intro commits (landing PRs #5,
#7, #62, #95, #233, #480, #511, #517, #529, #806, #871, #878, #1160,
#2339–#2806) whose intro diff spans many top-level service files at once, so
`discovery_paths` ends up wide (e.g. `core/services/ardupilot_manager`,
`cable_guy`, `commander`, `wifi`, `versionchooser`, `helper` all under one
cluster). Every unrelated single-file fix to *any* of those hot files over
years then counts as a "follow-up," even though most individual hits are
real commits to the right file — the false positive is topical (wrong
feature attribution), not fabricated data.

## 2. Worst-5 clusters — sampled titles/files

| Cluster (landing) | follow_ups | Confirmed FPs (title, files, in-scope) |
|---|---|---|
| ApplyParameterFile (#806) | 61 | #1889 "Fix random typos" (13f/1) |
| AccessBlueosWebInterface… (#871) | 60 | **#3686** "Core: isort fixes" (87f/12); #3930 "Add BlueOS cutomization" (19f/1) |
| DeleteLocalBlueosVersion… (#62) | 57 | **#871** "Rework project to use blueos name" (49f/6); #1889 typos; #3396 "Sanitize Libs" (13f/1); #3535 "Fix mypy" (15f/2) |
| BrowseAvailableWebServices… (#95) | 56 | **#2269** "Update kraken" (14f/1); #3357 "Add sentry SDK…" (17f/1); #2104 "Add limit_ram_usage" (15f/1); #3427 "Disable package in pyproject" (14f/1) |
| ChangeBoard… (#7) | 54 | #1889 typos; #379/#1029/#1532/#3257 pylint/lint sweeps |

Root cause confirmed: the existing `is_repo_wide_sweep` gate
(`>20 files && <10% in-scope`, `feature_trace_enrich.rs:846-862`) **never
fires on any surviving follow-up in the dataset** — most sweep PRs above sit
at 13–19 files (just under the `>20` trigger), so the threshold has a blind
spot exactly where the real sweeps cluster.

## 3. Golden re-check — all still hold

- **InspectZenohNetwork**: 13 follow-ups incl. full 3381–3393 family, no
  sweeps. ✓
- **ChangeUiThemeColor**: `follow_up_prs=[]`, `backport_prs=[]`. ✓
- **InspectDiskUsage**: exactly `[3681, 3691, 3743]`. ✓
- **RunInternetSpeedTest**: `[3758]`, no `3686`; issue `[2146]`; landing
  `[3602]`. ✓
- **LevelHorizon**: `backport_prs=[3867]`; follow-ups `[3874, 3962]`, no
  `3930`. ✓

## 4. Proposed fixes (ranked)

1. **Title denylist for sweep PRs** (highest impact/lowest risk). Add a
   regex in `feature_trace_enrich.rs` gating `discover_follow_up_prs`
   (isort, pylint/ruff/lint, mypy, "random typos", "rework project to use
   *name*", "sanitize libs", "update kraken", "100% mobile"/"mobile
   friendly", "sentry sdk", "limit_ram_usage", "disable package in
   pyproject", base-image/Dockerfile bumps, UV/venv migrations). Verified:
   removes #3686, #871, #1889, #2269, #3357, #3396, #3427, #3535 and 16 more
   unique PRs (102 of 839 total follow-up occurrences) with zero golden
   regressions.
2. **Overlap-≥2 requirement when `files_changed` is large**, not a blanket
   fraction. A flat `<25%` fraction threshold would wrongly reject golden
   `#3953` ("Update zenoh and enable shared memory," 14 files/2 in-scope,
   frac 14%) and any legit backport-style multi-file PR. Requiring
   `covered_files ≥ 2` when `files_changed.len() > ~12` keeps `#3953`
   (covered=2) while dropping `#2269`/`#1889`/`#3396` (covered=1) — catches
   sweeps the current `>20`-file trigger misses without new denylist
   entries.
3. **Lower/complement the existing breadth trigger**: keep `>20 files &&
   <10%` as a coarse safety net (it's cheap and never over-fires), but treat
   it as a fallback behind (1) and (2), since alone it catches 0 of the
   sampled FPs (all sit at 13–19 files).
   Combined impact (title denylist ∨ overlap<2-when->12files ∨ existing
   `>20&&<10%`): **133/839 (16%) of follow-up occurrences removed, all 5
   goldens unaffected** (`reject=False` re-verified for every golden PR).
4. *(Lower priority, structural, not a quick patch)* For the 20 wide-cluster
   outliers, the deeper fix is that one bootstrap intro commit spans many
   unrelated services — `discovery_paths` module-preference logic
   (`:390-447`) can't split it further without the journey's own module tag
   narrowing it first. Worth a follow-up investigation, not proposed here as
   a concrete patch.

**Not proposed**: path-prefix "majority" dominance and per-PR discovery-path
majority voting were considered but rejected — `#3867` (LevelHorizon's
*required* backport) is 67 files/10 in-scope (15%) and would fail any
per-PR-majority rule, so that heuristic must stay scoped away from
`backport_prs` if ever added, and doesn't add coverage beyond items 1–3
above for `follow_up_prs`.

## 5. Post-implement QA (title denylist + covered≥2 when >12 files)

- Unit tests: `incidental_follow_up_tests` (title / medium graze / zenoh keep / small keep).
- Full enrich regenerated `feature_traces.json`.
- Known FP PR set `{3686,871,1889,2269,3357,3396,3427,3535}`: **absent** from all clusters.
- Follow-up totals: **839 → 697** (−17%); clusters with >15 follow-ups: 20 → 19.
- Goldens unchanged: Zenoh (13, incl. 3381–3393 + 3953), Customization empty,
  Disk `[3681,3691,3743]`, Speed `[3758]` + issue 2146, LevelHorizon bp `[3867]` fu `[3874,3962]`.
- Remaining large clusters are bootstrap-era multi-service intros (structural),
  not sweep-title FPs — deferred to a later intro-commit split.

