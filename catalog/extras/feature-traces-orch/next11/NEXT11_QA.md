# NEXT11 QA — Phase 6 (final)

## Overall: PASS

All 11 tracks (N1–N11) meet their `NEXT11_DESIGN.md` ACs; `cargo test -p blueos-catalog --lib`
is 193/193 green; `check-traces.sh` exits 0; prior 5 goldens + camera ratio + 3 N11 goldens all
intact.

## Phase 6 actions taken this session

1. **Drift fix.** `ConfigureVideoStream`'s N1 `Override::Path` entries (2 files) were already in
   `feature_presence.rs`, but `feature_presence_map.json`/`feature_traces.json` were stale
   (still keyed on the old whole-tree intro commit `8da1baa6c56a`). Re-ran
   `generate_feature_presence` (intro commit → `11b310ea575a`, `VideoDiagnosticHelper.vue`) then
   `enrich_feature_traces --journey ConfigureVideoStream` (N2 scoped merge) to bring
   `feature_traces.json` in sync. `drift_check.sh`: FAIL → PASS (66 OVERRIDES journeys agree).
2. **Test drift.** `feature_trace::tests::loads_schema_v2` hardcoded `intro_clusters.len() == 55`;
   the new `11b310ea575a` cluster from the above re-enrich makes this legitimately 56 (old
   `8da1baa6c56a` cluster stays, still shared by `ConfigureCameraStream`/`EnableLegacyCameraSupport`/
   `RemoveCameraStream`/`ConfigureUvcDeviceControls`). Updated assertion to `56`.

## Per-track results

| Track | Status | Evidence |
|---|---|---|
| N1 — ConfigureVideoStream hub narrowing | PASS | `discovery_paths` now exactly `[VideoDiagnosticHelper.vue, VideoThumbnail.vue]` (was 13-path whole `video-manager/`, `fu`=37). `sibling_matrix`: vs `ViewCameraStreams` 4/36=**0.111**, vs `ConfigureCameraStream` 2/31=**0.065** (both `<0.40`, both cargo-asserted). |
| N2 — `--journey` scoped merge | PASS | Ran `enrich_feature_traces --journey ConfigureVideoStream`: all 93 other journeys byte-identical pre/post (incl. `InspectDiskUsage`); only `ConfigureVideoStream`'s entry + 1 new intro cluster added; `commits`/`pull_requests`/`issues`/`intro_clusters` maps grew (union), not shrank. Unit test `journey_filter_merges_not_overwrites` passes. |
| N3 — `check-traces.sh` | PASS | `bash check-traces.sh` → drift_check, `sibling_matrix`, `feature_trace_report --check-goldens` all green, exit 0, "ALL FEATURE-TRACES GATES GREEN". |
| N4 — Shared-legit policy | PASS | 3 pinned `SharedCluster` tests (`commander_reboot_shutdown_`, `versionchooser_trio_`, `kraken_install_uninstall_is_pinned_shared_cluster`), all `sibling_ratio >= 0.99`, green; `NEXT11_AUDIT.md` documents measured 1.0000 for all 5 pairs and explicitly excludes the autopilot trio (0.5556, belongs to N6). |
| N5 — Pickaxe relevance score | PASS | `follow_up_prs[].term_hit: Option<bool>` present in JSON/HTML; 5 unit tests green (`pr_term_hit_true_when_term_in_title`, `..._in_files_changed_case_insensitive`, `..._false_when_term_absent`, `markdown_omits_pickaxe_summary_when_term_hit_unscored`, `disk_usage_follow_ups_have_no_pickaxe_term_hit`). |
| N6 — Expand sibling cargo gates | PASS | 7 sibling-ratio gates now hard-asserted (≥5 required): camera, autopilot, helper, wifi, eeprom, + 2 new video pairs (N1). All pass. |
| N7 — Report polish | PASS | HTML sample (`CalibrateGyroscope`) shows `<a href="https://github.com/.../pull/2806">#2806</a>` (was bare `#N`) and `(rebase)` merge-method suffix inline. |
| N8 — HTML/JSON provenance surface | PASS | `feature_trace_report --format html` emits a complete, self-contained styled table (verified via sample render); Vue explicitly out of scope per frozen design decision. |
| N9 — Presence chips | PASS | Same HTML render includes tag/channel chip row (`master`, `1.4-dev`, per-tag chips) sourced from `present_in_tags`/`present_on_1_4_dev`; same fields present in JSON. |
| N10 — Enricher resume | PASS | `grep -n '"--resume"' feature_trace_enrich.rs` confirms flag parsing (line 2403) + checkpoint-based skip (`cluster_already_enriched`, line 2542); 6 `merge_and_resume_tests` unit tests green, incl. `cluster_already_enriched_{true,false}_*` and `merge_with_no_existing_file_returns_new_output_unchanged`. No live kill-mid-run smoke test executed this session — unit coverage was judged sufficient (per task instruction to just spot-check + note). |
| N11 — Golden corpus growth | PASS | `GOLDEN_JOURNEY_IDS` (both consts, report.rs:27 + enrich.rs) list all 8: 5 original + `AccessWebTerminal`, `InspectMavlinkMessagesInBrowser`, `CalibrateGyroscope`. `--check-goldens` output: "all 8 golden journeys match". |

## Precision/Improve regression matrix

| Golden | Status |
|---|---|
| InspectZenohNetwork | PASS |
| ChangeUiThemeColor | PASS |
| InspectDiskUsage | PASS |
| RunInternetSpeedTest | PASS |
| LevelHorizon | PASS |
| Camera sibling ratio (`ConfigureCameraStream`/`ViewCameraStreams`) | PASS — 0.100 (`<0.40`, matches design doc baseline) |
| AccessWebTerminal (N11) | PASS |
| InspectMavlinkMessagesInBrowser (N11) | PASS |
| CalibrateGyroscope (N11) | PASS |

## Open follow-ups

- Vue provenance panel deferred (frozen decision, N8/N9 design section) — no catalog HTTP API to
  fetch from; not revisited this phase.
- Stale leftover: the old `8da1baa6c56a` intro cluster still lists `ConfigureVideoStream` in its
  `by_journey` map (pre-N1 artifact); harmless — journey→cluster lookup keys off the journey's
  current `intro_commit` (now `11b310ea575a`), so it's never read, but N2's merge semantics don't
  prune it. Not in scope to fix (no AC requires pruning).
- `catalog/examples/n4_check.rs` is an untracked scratch file from N4 investigation; left as-is,
  not part of any deliverable.
