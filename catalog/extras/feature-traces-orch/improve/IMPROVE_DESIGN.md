# Phase 1 — Improvement design: T1–T7 acceptance criteria

Freezes scope before Phase 2 implementation. Precision gate baseline:
`precision/PRECISION_QA.md` (5/5 goldens PASS, camera ratio 0.075).

## T1 — Measurable precision
**Goal:** Replace ad-hoc ratio checks with a repeatable sibling-ratio matrix + optional Pickaxe relevance score.
- AC: `cargo run -p blueos-catalog --bin sibling_matrix` (or `--features sibling-matrix` flag on an
  existing bin) prints `|∩|/|∪|` for every pair below; exits non-zero if any pair ≥ 0.40.
- AC: hub path-intersection is never used as a gate (already true per `PRECISION_QA.md` §2 — encode
  as a doc comment + test, not a new check).
- AC: cargo test asserts the camera pair and ≥3 other pairs from the matrix below.
- Files: `catalog/src/tools/sibling_matrix.rs` (new), `catalog/src/bin/sibling_matrix.rs` (new),
  `catalog/src/feature_trace.rs` (tests).
- Non-goals: no new discovery heuristics, no Pickaxe scoring UI.
- Deps: none (reads existing `feature_traces.json`).

## T2 — Enricher harden
**Goal:** Make `gh` calls resilient to transient failures and make golden misses fail loudly.
- AC: `gh_pr_detail`/`gh_issue` retry on 504/5xx with exponential backoff (max 3 attempts); non-5xx
  errors do not retry.
- AC: `--strict-goldens` flag on `enrich_feature_traces` exits non-zero if any of the 5 golden PRs
  (3300, 3930/3602 landing set, 3681/3691/3743, 2146, 3867) is missing from `pull_requests`/`issues`.
- AC: every successful `gh` response is written to cache before the calling function returns (already
  true for `cached_json`/`cached_json_try` — add a regression test using a fake fetcher).
- Files: `catalog/src/tools/feature_trace_enrich.rs` (`gh_pr_detail`, `gh_issue`, `run`), `catalog/src/tools/shell.rs`.
- Non-goals: no new gh subcommands, no rate-limit dashboard.
- Deps: none.

## T3 — Shared-legit semantics
**Goal:** Attempt non-path separation for shared-legit groups; document impossibility otherwise.
- AC: for each of the 4 groups below, either (a) a Pickaxe route/method or title-token override drops
  the pair's sibling ratio measurably (recorded in T1 matrix), or (b) `IMPROVE_AUDIT.md` documents why
  with file:line evidence.
- AC: no group loses its existing (file-level) shared status as a regression — goldens still pass.
- Files: `catalog/src/tools/feature_trace_enrich.rs` (`Override::Pickaxe` additions),
  `catalog/extras/feature-traces-orch/improve/IMPROVE_AUDIT.md` (new).
- Non-goals: do not force-split a group with no route/method/title signal (over-fragmentation is worse
  than an honest shared-legit label).
- Deps: T1 matrix (to measure the ratio drop).

## T4 — MODULE_S / noisy hubs
**Goal:** Fix the 8 `MODULE_S_and_proxied` journeys with empty/wrong hints and audit `fu > 30` hubs.
- AC: `AccessWebTerminal`, `ViewSystemInformation`, `InspectMavlinkMessagesInBrowser`,
  `ViewCameraStreams`, `ConfigureCameraStream`, `RemoveCameraStream`, `ConfigureUvcDeviceControls`
  each get an `Override::Path` per `PRECISION_INVENTORY.md` (MODULE_S section); all have
  `discovery_paths.len() > 0` and non-empty `follow_up_prs` when history exists.
- AC: no camera journey's `discovery_paths` contains `store/mavlink.ts` (regression test).
- AC: every module cluster with `fu > 30` (`frontend_calibration`, `ardupilot_manager`, `wifi`, etc.)
  is either narrowed or has a one-line justification in `IMPROVE_AUDIT.md` (churn is genuine, not a hint bug).
- Files: `catalog/src/tools/feature_presence.rs` (`MODULE_DEFAULT_PATH`, `OVERRIDES`, `MODULE_S`),
  `catalog/src/feature_trace.rs` (tests), `IMPROVE_AUDIT.md`.
- Non-goals: no new module types beyond the existing `MODULE_S` set.
- Deps: none; feeds T7 (presence regen must follow).

## T5 — Product surface
**Goal:** Surface intro→landing→follow-ups→backports→issues per journey without a new frontend.
- AC: new bin emits JSON (array of `{journey_id, intro_commit, landing_prs, follow_up_prs,
  backport_prs, issue_numbers}`) for all 94 journeys, reusing `ReportTrace::for_journey` +
  `presence_for_journey`.
- AC: same bin supports `--format md` producing one table row per journey.
- AC: sample output for the 5 golden journeys matches `PRECISION_QA.md` §1 values exactly.
- Files: `catalog/src/bin/feature_trace_report.rs` (new), `catalog/src/report.rs` (reuse `ReportTrace`,
  no schema change).
- Non-goals: no Vue component; no HTTP endpoint (that's `journey_http`'s job, out of scope here).
- Deps: T4 (fixed hints should be reflected in the export), T7 (presence parity).

## T6 — Local hygiene
**Goal:** Add a drift-detection script and expand cargo coverage; stop tracking orchestrator process noise.
- AC: `catalog/extras/feature-traces-orch/drift_check.sh` runs `generate_feature_presence` +
  `enrich_feature_traces` in `--check`/dry mode and diffs against committed JSON; non-zero exit on drift.
- AC: `catalog/src/feature_trace.rs` gains tests for all 5 goldens (3 exist today — add
  `InspectDiskUsage` exact-set and `RunInternetSpeedTest`/`LevelHorizon` checks) plus ≥3 sibling-ratio
  assertions from the T1 matrix.
- AC: `improve/memory.xml`, `improve/memory_archive.md`, `precision/memory.xml`,
  `precision/memory_archive.md` are gitignored (see decision below), not deleted.
- Files: `catalog/extras/feature-traces-orch/drift_check.sh` (new), `catalog/src/feature_trace.rs`,
  `.gitignore`.
- Non-goals: no CI wiring (local-only per orchestrator constraints).
- Deps: T2 (enricher `--check` mode), T4 (tests reference fixed hints).

## T7 — Presence ↔ traces
**Goal:** Keep `feature_presence_map.json` and `feature_traces.json` intro-commit-consistent after OVERRIDES changes.
- AC: after any T4 `OVERRIDES`/`MODULE_DEFAULT_PATH` edit, re-run `generate_feature_presence` before
  `enrich_feature_traces` (documented order in `drift_check.sh`).
- AC: `presence_and_traces_share_intro_commits` test (existing, `feature_trace.rs:266`) still passes and
  gains the 2 remaining MODULE_S journeys (`ViewCameraStreams`, `AccessWebTerminal`) to its coverage set.
- Files: `catalog/src/feature_trace.rs` (test), `catalog/src/tools/feature_presence.rs`.
- Non-goals: no changes to the intro-commit detection algorithm itself.
- Deps: T4 (must run after hint fixes).

## Ordered implementation sequence (Phase 2–5)
1. **Phase 2 (Foundation):** T2 (retry/strict-goldens) → T6 drift script skeleton → T1 scoring harness
   (needs stable cache from T2) → T7 regen-order doc.
2. **Phase 3 (Audit+fix):** T4 (MODULE_S hint fixes, run presence regen per T7) → T3 (shared-legit
   experiments, scored via T1 matrix) → `IMPROVE_AUDIT.md`.
3. **Phase 4 (Surface):** T5 export bin, validated against goldens.
4. **Phase 5 (Final QA):** re-run T6 cargo tests + drift script; `IMPROVE_QA.md`; confirm T1 matrix,
   goldens, camera ratio all still green.

## Precision regression invariants (must hold through every phase)
- `InspectZenohNetwork`: landing 3300; follow-ups ⊇ {3313, 3381–3393 family (6/6), 3953}; no sweeps.
- `ChangeUiThemeColor`: `follow_up_prs == []` and `backport_prs == []`.
- `InspectDiskUsage`: `follow_up_prs == {3681, 3691, 3743}` exactly.
- `RunInternetSpeedTest`: landing 3602; issue 2146 present; 3686 absent.
- `LevelHorizon`: `backport_prs ∋ 3867`; 3930 absent from follow/backport.
- `ConfigureCameraStream` vs `ViewCameraStreams` sibling ratio **< 0.40** (currently 0.075).

## T1 sibling-pair matrix (must appear, derived from `PRECISION_INVENTORY.md` separable pairs)
| # | Pair | Cluster | Why it's a risk (proximity) |
|---|---|---|---|
| 1 | `ConfigureCameraStream` / `ViewCameraStreams` | mavlink_camera_manager | primary gate, < 0.40 |
| 2 | `ConnectToWifiNetwork` / `ForgetSavedWifiNetwork` | wifi | same file `ConnectionDialog.vue`, distinct route |
| 3 | `ConfigureHotspotCredentials` / `ToggleSmartHotspot` | wifi | same file `WifiSettingsDialog.vue`, distinct route |
| 4 | `InspectRaspberryEepromBootloader` / `UpdateRaspberryEepromBootloader` | commander | same file `Firmware.vue`, distinct Pickaxe methods |
| 5 | `AcquireDynamicIpAddress` / `DisableOnboardDhcpServer` | cable_guy | same file `InterfaceCard.vue`, distinct block |
| 6 | `RenameVehicle` / `ChangeMdnsHostname` | beacon | same file `VehicleBanner.vue`, distinct route |
| 7 | `BrowseAvailableWebServices` / `MonitorInternetConnectivity` | helper | existing gate (0.053), regression watch |
| 8 | `VerifyInternetConnectivity` / `ProbeInterfaceInternetConnectivity` | helper | distinct files, same cluster |
| 9 | `UpdateBootstrapImage` / `DeleteLocalBlueosVersion` | versionchooser | distinct router files, same hub |
| 10 | `StartAutopilot` / `UpdateFirmwareOnline` | ardupilot_manager | existing gate (0.044), cross-concern regression watch |

## T3 shared-legit groups (all 4, from `PRECISION_INVENTORY.md`)
1. `RebootOnboardComputer` / `ShutdownOnboardComputer` (commander) — one `shutdown()` handler, body enum differs.
2. `UpdateBlueosVersion` / `SwitchLocalBlueosVersion` / `PullBlueosVersionWithoutSwitch` (versionchooser) — overlapping `VersionChooser.vue` regions.
3. `InstallExtension` / `UninstallExtension`, and `EditExtensionDevVersion` / `ConfigureInstalledExtension` (kraken) — shared version-dropdown / edit-tag controls.
4. `StartAutopilot` / `StopAutopilot` / `RestartAutopilot` (ardupilot_manager) — shared `AutopilotManagerUpdater.ts` + router lifecycle, only route path differs.

## Decision: gitignore vs delete orchestrator memory/archive
**Gitignore, don't delete.** `precision/memory.xml` and `precision/memory_archive.md` are already
committed (tracked); `improve/memory.xml`/`memory_archive.md` are untracked. Deleting would destroy
the only handoff record if this session is interrupted, and the orchestrator prompt's `<handoff>`
contract depends on the file existing on disk. Add both dirs' `memory.xml`/`memory_archive.md` to
`.gitignore` going forward (T6); untrack the already-committed `precision/` pair with
`git rm --cached` in the same commit that adds the ignore rule — but only when the user explicitly
asks to commit (per orchestrator constraint "do not commit unless user asks").
