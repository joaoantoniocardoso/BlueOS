# Phase 5b — Final QA (T1–T7)

**Overall: PASS**

Source: `catalog/feature_traces.json` regenerated per T7 order (`generate_feature_presence` →
`enrich_feature_traces`), verified against `IMPROVE_DESIGN.md` ACs and `IMPROVE_AUDIT.md`.

## Per-track results

**T1 — Measurable precision: PASS.** `cargo run --bin sibling_matrix` prints all 10 designed
pairs; camera ratio 0.100 (5/50) < 0.40. `sibling_matrix::tests::matrix_has_all_ten_designed_pairs`
and `camera_pair_passes_gate_on_real_data` pass; `feature_trace.rs` has ≥3 sibling-ratio
assertions (`camera_pair_sibling_ratio_below_gate`, `autopilot_pair_sibling_ratio_below_gate`,
`helper_pair_sibling_ratio_below_gate`).

**T2 — Enricher harden: PASS.** `shell::run_with_retry` used by all `gh` fetch paths
(`feature_trace_enrich.rs:290-294`); `check_strict_goldens` + `strict_goldens_tests` module
(6 unit tests, `feature_trace_enrich.rs:1649,1758`); `--strict-goldens` flag wired in `run`
(`:2038-2208`) and exercised successfully via `feature_trace_report --check-goldens` ("all 5
golden journeys match PRECISION_QA.md §1").

**T3 — Shared-legit semantics: PASS.** 3/4 groups documented impossible with `git log -S`
evidence (commander reboot/shutdown 1.0; versionchooser set/pull 1.0; kraken install/uninstall
1.0) in `IMPROVE_AUDIT.md`. 4th group (ardupilot start/stop/restart) improved via real Pickaxe
wiring: Start/Stop = **0.556** (5/9, confirmed by direct computation), Start/Restart 0.625,
Stop/Restart 0.222 — all down from 1.0. No existing shared-status regression (goldens still pass).

**T4 — MODULE_S / noisy hubs: PASS.** All 7 previously-empty/wrong MODULE_S journeys carry
`Override::Path` entries (`feature_presence.rs:110-143`); none resolve to `store/mavlink.ts`
(`camera_journey_discovery_excludes_mavlink_store_leak` passes). `fu > 30` hubs all justified in
`IMPROVE_AUDIT.md` (shared-legit or genuine per-file churn) except `ConfigureVideoStream`, flagged
as an explicit out-of-scope follow-up (not in the MODULE_S/`PRECISION_INVENTORY.md` sets).

**T5 — Product surface: PASS.** `src/bin/feature_trace_report.rs` exists; `--check-goldens`
confirms sample output matches `PRECISION_QA.md` §1 exactly for all 5 golden journeys.

**T6 — Local hygiene: PASS.** `improve/drift_check.sh` runs and reports "65 OVERRIDES journeys
agree between feature_presence_map.json and feature_traces.json" (0 drift). `cargo test -p
blueos-catalog --lib` → 160/160 passing (goldens + ≥3 sibling-ratio tests present). All 4 memory
files (`improve/`, `precision/` × `memory.xml`/`memory_archive.md`) match
`catalog/extras/feature-traces-orch/.gitignore` rules `**/memory.xml`/`**/memory_archive.md`;
already-tracked `precision/` pair staged for untrack (`git rm --cached`, not yet committed per
orchestrator constraint).

**T7 — Presence ↔ traces: PASS.** `presence_and_traces_share_intro_commits`
(`feature_trace.rs:329-343`) covers all 5 base journeys plus both required MODULE_S additions
(`ViewCameraStreams`, `AccessWebTerminal`); `drift_check.sh` confirms 0 mismatches repo-wide.

## Precision regression (baseline: `precision/PRECISION_QA.md`)

5/5 goldens still PASS (`InspectZenohNetwork`, `ChangeUiThemeColor`, `InspectDiskUsage`,
`RunInternetSpeedTest`, `LevelHorizon` — verified via `--check-goldens`); camera sibling ratio
0.100, still well under the 0.40 gate (was 0.075 pre-re-enrich; both pass, delta from the T4
re-enrich landing, not a regression).

## Open follow-ups

1. `ConfigureVideoStream` (fu=37) still resolves to the full `video-manager/` tree (13 paths,
   incl. `assets/wip-video.svg`) — out of T4 scope, needs its own narrowing pass.
2. `RebootOnboardComputer`/`ShutdownOnboardComputer` (commander), `UpdateBlueosVersion`/
   `SwitchLocalBlueosVersion`/`PullBlueosVersionWithoutSwitch` (versionchooser), and
   `InstallExtension`/`UninstallExtension` (kraken) remain path-shared by design — no
   route/method/title signal separates them (documented impossible, T3).
3. Pickaxe relevance score UI (Phase 1 T1 non-goal) still not built — optional, deferred.
