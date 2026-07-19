# Phase 5d — Precision QA gate

**Overall: PASS**

Source: `catalog/feature_traces.json` (107124 lines, regenerated 2026-07-19
16:27), scored via `score_precision.sh` + ad-hoc `jq`/`python3` against
`PRECISION_DESIGN.md` §5 and the goldens in the orchestrator prompt. This
supersedes the Phase 4b run recorded in `memory_archive.md` (3/5 goldens,
camera ratio 0.79).

## 1. Goldens (5/5 PASS)

| Journey | Check | Result | Detail |
|---|---|---|---|
| `InspectZenohNetwork` | landing 3300; follow-ups ⊇ 3313 + 3381–3393 family + 3953; no sweep PRs | **PASS** | landing=3300 ✓. `discovery_paths` now includes the `zenoh-inspector` dir (not just `ZenohInspectorView.vue`), so follow-ups = `{3313,3368,3376,3381,3386,3387,3391,3392,3393,3407,3635,3820,3953}` — 3313 ✓, full 3381–3393 family (6/6) ✓, 3953 ✓. Checked files_changed for the two largest (3368: 12 files, 3953: 14 files) — both are zenoh-inspector feature commits plus routine lockfile/deps churn, not repo-wide sweeps. |
| `ChangeUiThemeColor` | empty `follow_up_prs` AND `backport_prs` | **PASS** | both `[]`, landing=3930 (unchanged from Phase 4b) |
| `InspectDiskUsage` | `follow_up_prs` == `{3681, 3691, 3743}` | **PASS** | exact match, no extras. `discovery_paths` now pairs backend (`core/services/disk_usage`) with frontend (`views/Disk.vue`, `store/disk.ts`); the 3681 harvest gap noted in Phase 4b is resolved. |
| `RunInternetSpeedTest` | landing 3602; issue 2146 present; no 3686 | **PASS** | landing=3602, issue 2146 (closing+body via 3602) present, `follow_up_prs=[3758]`, 3686 absent |
| `LevelHorizon` | `backport_prs` ∋ 3867; no 3930 in follow/backport | **PASS** | `backport_prs=[3867]`, `follow_up_prs=[3874,3962]`, 3930 absent from both |

## 2. Sibling separation (primary gate, per task note #4)

Hub path-intersection precision is not scored here as a gate: `discover_follow_up_prs`
runs `git log -- <paths>` on the exact `discovery_paths` slice stored per journey, so
every discovered follow-up trivially "intersects" its own hint set by construction —
that check only validates internal consistency, not whether the hint itself is
correctly scoped. Sibling ratio (shared follow-ups / union) is the real signal for
over-broad hints, so it is treated as primary here alongside the goldens.

| Pair | `\|∩\|` | `\|∪\|` | ratio | Verdict |
|---|---|---|---|---|
| `StartAutopilot` vs `UpdateFirmwareOnline` | 4 | 91 | **0.044** | good — well separated |
| `BrowseAvailableWebServices` vs `MonitorInternetConnectivity` | 1 | 19 | **0.053** | good — well separated |
| `ConfigureCameraStream` vs `ViewCameraStreams` | 3 | 40 | **0.075** | **PASS — target < 0.40** |

Camera pair fixed: `ConfigureCameraStream.discovery_paths` is now just
`video-manager/VideoStreamCreationDialog.vue`, and `ViewCameraStreams.discovery_paths`
is `video-manager/VideoManager.vue` + `store/video.ts` — the shared parent
`video-manager/` directory is no longer re-added by directory widening, so the two
journeys no longer re-merge into one shared scope (down from ratio 0.79 → 0.075).

## 3. MODULE_S checks (PASS)

| Check | Result |
|---|---|
| `AccessWebTerminal.follow_up_prs.len() > 0` | **PASS** — `len() == 1` (`[2279]`) |
| `ViewCameraStreams.discovery_paths` excludes `store/mavlink.ts` | **PASS** — paths are `video-manager/VideoManager.vue`, `store/video.ts` |

## Remediations closed since Phase 4b

1. PR 3681 harvest gap fixed — now present in `pull_requests` and included in
   `InspectDiskUsage.follow_up_prs`.
2. `InspectZenohNetwork` hint widened to cover `zenoh-inspector/` dir, not just
   the single view file — recovers 3953 and the full 3381–3393 family.
3. `InspectDiskUsage` hint now pairs backend + frontend (`Disk.vue`, `disk.ts`).
4. `widen_with_feature_dirs` / `OVERRIDES` no longer re-broaden `video-manager/`
   into a shared hub for both camera journeys — sibling ratio 0.79 → 0.075.
