# Discovery QA — `catalog/feature_traces.json` (schema v2)

Spot-check of `intro_clusters` against known-good goldens, verified against live
GitHub state (`gh pr/issue view`). Code refs: `catalog/src/tools/feature_trace_enrich.rs`.

## Golden journeys

### 1. InspectZenohNetwork — **PARTIAL FAIL** (completeness)
- Landing #3300 ✓. `backport_prs: []` ✓. `follow_up_prs` = `[3313, 3368, 3376,
  3386, 3407, 3635, 3820, 3953]` — all genuinely zenoh-related, **no FPs**.
- **But incomplete**: PRs #3381, #3387, #3391, #3392, #3393 are real zenoh
  follow-ups (verified via `gh pr view`, all touch
  `core/frontend/src/components/zenoh-inspector/{ZenohNetwork,RawVideoPlayer}.vue`)
  and are missing from the cluster.
- Root cause: `discovery_paths` = exactly `['.../ZenohInspector.vue',
  '.../ZenohInspectorView.vue']` — the two literal files touched by the intro
  commit/landing PR. `ZenohNetwork.vue`/`RawVideoPlayer.vue` didn't exist yet
  at intro time, so `discover_follow_up_prs`'s path-scoped `git log` never
  sees them (`feature_trace_enrich.rs:394-450`, `:804-833`).
- **Fix**: in `discovery_paths()`, when paths share a common `.../<dir>/`
  prefix (e.g. `zenoh-inspector/`, `nmea-injector/`, `customization/`), widen
  to that directory glob instead of (or in addition to) the literal file list,
  so sibling files added later are covered.

### 2. ChangeUiThemeColor — **PASS** (backport/follow-up), **FAIL** (issue harvest)
- Landing #3930 ✓. `backport_prs: []` ✓. `follow_up_prs: []` ✓.
- `issues: [{number: 3426, sources: [{kind: "search"}]}]` is a **false
  positive** — issue #3426 title is "detect if there is a board available
  before running wizard **customization step**" (setup-wizard step naming
  collision), unrelated to the theme/branding `customization` service.
- Root cause: `cluster_sources` empty → falls back to
  `gh_issue_search(module)` (`:723-760`, called at `:958-964`) with keyword =
  module id `"customization"`, which is generic and collides with the wizard's
  unrelated "customization step" terminology. This is the exact risk the
  Probe already flagged for title-keyword search.
- **Fix**: drop the bare-module-name search fallback (`:958-964`) or require
  it to be confirmed by a `timeline` cross-reference before being kept, same
  as the `search` kind's stated "lowest confidence" — currently it's kept
  unconditionally.

### 3. InspectDiskUsage — **PASS**
- Landing #3669 ✓, `backport_prs: []` ✓, `follow_up_prs: [3681, 3691, 3743]`
  — includes required #3691 plus both preferred #3681/#3743, all disk-related.
  Issue #2572 (disk bloat) confirmed relevant.

### 4. RunInternetSpeedTest — **FAIL**
- Landing #3602 ✓, issue #2146 present (`closing`+`body`, high confidence) ✓,
  `backport_prs: []` ✓.
- `follow_up_prs: [3686]` is a **false positive**: PR #3686 "Core: isort
  fixes" touches 80+ files repo-wide (ardupilot_manager, kraken, wifi, ping,
  …) and only incidentally includes `core/services/pardal/main.py`. Not
  pardal-related.
- Root cause (same class as #1's inverse): `discover_follow_up_prs` accepts
  any PR that touches *any* file in `discovery_paths`, with no check on how
  large/unrelated the rest of that PR's diff is (`:804-833`).

### 5. LevelHorizon — **PASS**
- Landing #3826 ✓, `backport_prs: [3867]` = "[1.4 backports] Bring master's
  vehicle-setup updates to 1.4" base `1.4` — genuine backport. `follow_up_prs:
  []` ✓.

## Random sample (3 other clusters)

- **nmea-injector** (intro `0384553acf47`, landing #529): `follow_up_prs` has
  38 entries; a large fraction are repo-wide sweeps that only incidentally
  touch `core/services/nmea_injector/{main,setup}.py`: #607 "Refactor stores
  syntax", #871 "Rework project to use blueos name", #969 "chaining operator
  on all error.response.data", #1002 "100% mobile friendly", #1370/#1388/#1403
  lint sweeps, #2269 **"Update kraken"** (verified: 14 files, only
  `nmea_injector/setup.py` touched — completely unrelated service), #3278 "UV
  package manager migration", #3686 same isort PR as finding #4. This is the
  #4 root cause at much larger scale — confirms it's systemic, not a one-off.
- **filebrowser** (intro `c38dc6db3ae2`, landing #359): single discovery path
  `core/tools/filebrowser/bootstrap.sh`; follow-ups #377/#667/#2686 read as
  plausible (bootstrap-script-specific titles) but weren't independently
  verified — lower confidence, no verified FP found here.
- **zenohd** (= finding #1, re-sampled): see above.

## Fix summary (code-level, no implementation done)

1. **`discovery_paths()`** (`feature_trace_enrich.rs:394`): widen literal
   sibling files under a shared feature directory to a directory-level glob,
   to fix false negatives (finding #1).
2. **`discover_follow_up_prs()`** (`:804`): add a breadth/dominance filter —
   reject or down-rank candidate PRs where the feature's `discovery_paths`
   are a small minority of that PR's total `files` (e.g. PR touches >20-30
   files and <10% are in-scope). This is the single highest-value fix; it
   directly resolves finding #4 and the bulk of the nmea-injector sample.
3. **`gh_issue_search` fallback** (`:723`, wired at `:958`): either remove it
   or gate it behind a `timeline` confirmation before inclusion — currently
   any bare module-name text match is kept as-is (finding #2).
