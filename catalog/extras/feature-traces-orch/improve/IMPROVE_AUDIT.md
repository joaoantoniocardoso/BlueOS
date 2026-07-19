# Phase 3 — Audit findings

## T4 — MODULE_S / noisy hubs

**Status: all AC already satisfied by commit `fa3f8229a` (catalog: Sharpen per-journey
feature-trace discovery precision). This audit verifies and documents; no code changes were
needed.** `feature_traces.json` (`generated_at` 2026-07-19T16:27:31-03:00) already reflects the
fixed hints below — enrichment does not need a re-run for T4.

### MODULE_S journeys (`intro_clusters["c4fe82a264d4..."].by_journey`)

| Journey | Module | discovery_paths | fu | mavlink.ts leak |
|---|---|---|---|---|
| `AccessBlueosWebInterface` | nginx | `core/tools/nginx`, `core/tools/nginx/nginx.conf` | 33 | no |
| `AccessWebTerminal` | ttyd | `core/frontend/src/views/TerminalView.vue` | 1 | no |
| `ViewSystemInformation` | linux2rest | `store/system-information.ts`, `views/SystemInformationView.vue` | 22 | no |
| `InspectMavlinkMessagesInBrowser` | mavlink2rest | `views/MavlinkInspectorView.vue` | 1 | no |
| `ViewCameraStreams` | mavlink_camera_manager | `components/video-manager/VideoManager.vue`, `store/video.ts` | 24 | no |
| `ConfigureCameraStream` | mavlink_camera_manager | `components/video-manager/VideoStreamCreationDialog.vue` | 19 | no |
| `RemoveCameraStream` | mavlink_camera_manager | `components/video-manager/VideoStream.vue` | 14 | no |
| `ConfigureUvcDeviceControls` | mavlink_camera_manager | `components/video-manager/VideoControlsDialog.vue` | 7 | no |

All 7 previously-empty/wrong journeys (per `PRECISION_INVENTORY.md` §MODULE_S) now carry the
proposed `Override::Path` entries (`catalog/src/tools/feature_presence.rs:110-143`), consumed
through `resolve_journey_discovery_paths` (`feature_trace_enrich.rs:655-666`), which gives an
explicit override full precedence over the broad module-token fallback that used to resolve
camera journeys onto `store/mavlink.ts`. Regression coverage: `camera_journey_discovery_excludes_mavlink_store_leak`,
`sibling_overrides_do_not_leak_into_each_other`, `camera_sibling_journeys_are_not_remerged_by_video_manager_widening`
(`feature_trace_enrich.rs`, all passing) and `presence_and_traces_share_intro_commits` now also
covers `ViewCameraStreams`/`AccessWebTerminal` (`feature_trace.rs:329-343`, T7 AC).

### `fu > 30` hub audit (28 journeys total; top 15 by count)

| fu | Journey | discovery_paths (head) |
|---|---|---|
| 54 | `VerifyInternetConnectivity` | `components/wizard/RequireInternet.vue` |
| 51 | `ApplyParameterFile` | 4 `parameter-editor`/`ParameterEditorView.vue` files |
| 48 | `UploadCustomFirmware` | `FirmwareManager.vue`, `firmware/FirmwareUpload.py` |
| 48 | `UpdateFirmwareOnline` | `FirmwareManager.vue`, `firmware/FirmwareDownload.py` |
| 48 | `RestoreDefaultFirmware` | `FirmwareManager.vue`, `firmware/FirmwareInstall.py` |
| 47 | `StopAutopilot`/`StartAutopilot`/`RestartAutopilot` | shared `mavlink_proxy/` router lib (9 files) |
| 47 | `SetNetworkInterfacePriority` | `NetworkInterfacePriorityMenu.vue` |
| 47 | `RenameVehicle`/`ChangeMdnsHostname` | `VehicleBanner.vue` (Pickaxe-differentiated) |
| 47 | `ProbeInterfaceInternetConnectivity` | `NetworkInterfacePriorityMenu.vue` |
| 47 | `ConfigureHostDns` | `DnsConfigurationMenu.vue` |
| 43 | `UpdateBlueosVersion` | `versionchooser/` (10 files) |
| 43 | `UninstallExtension` | `kraken/ExtensionCard.vue`, `ExtensionModal.vue`, ... |

**Justification (no further narrowing possible/needed):**
- `StopAutopilot`/`StartAutopilot`/`RestartAutopilot`, `UpdateBlueosVersion`/`SwitchLocalBlueosVersion`/
  `PullBlueosVersionWithoutSwitch`, `InstallExtension`/`UninstallExtension`/`EditExtensionDevVersion`/
  `ConfigureInstalledExtension` — already documented **shared-legit** (T3, `IMPROVE_DESIGN.md` §T3);
  fu count reflects genuine shared-component churn, not a hint bug.
- `VerifyInternetConnectivity`, `ApplyParameterFile`, `SetNetworkInterfacePriority`,
  `ProbeInterfaceInternetConnectivity`, `ConfigureHostDns`, `RenameVehicle`/`ChangeMdnsHostname`,
  `UploadCustomFirmware`/`UpdateFirmwareOnline`/`RestoreDefaultFirmware`, `VehicleFirstBoot`/
  `RunSitlSimulation`/`ChangeBoard`, `InspectRaspberryEepromBootloader`/`UpdateRaspberryEepromBootloader`,
  `DetectMotorDirections`, `AccessBlueosWebInterface` — already narrowed to a single distinct
  file/router (or Pickaxe-differentiated shared file) per `PRECISION_INVENTORY.md`; high `fu` is
  genuine per-file churn, not hub bleed.
- `ConfigureVideoStream` (fu=37, module `frontend_video`, intro cluster `8da1baa6c56a`, **not** one
  of the 8 `MODULE_S_and_proxied` journeys and not covered by `PRECISION_INVENTORY.md`) still
  resolves to the entire `video-manager` component tree (13 paths, including `assets/wip-video.svg`).
  This is a legitimate open item but out of T4's AC scope (neither the MODULE_S list nor the
  `PRECISION_INVENTORY.md` narrowing set names it) — flagged here for a future pass rather than
  fixed blind, per the T3 non-goal "do not force-split ... over-fragmentation is worse than an
  honest [broad] label."

### Code fixes applied this session
None — `feature_presence.rs` `OVERRIDES` and `feature_trace_enrich.rs`
`resolve_journey_discovery_paths`/`journey_override_paths` already implement every AC in
`IMPROVE_DESIGN.md` §T4 (landed in `fa3f8229a`, prior to this audit). `cargo test -p blueos-catalog
feature_trace` — 27/27 passing, including the leak regression and both T7 intro-commit checks.

### needs-re-enrich?
No. `catalog/feature_traces.json` already reflects the fixed overrides (verified via
`intro_clusters[*].by_journey`); `generate_feature_presence` → `enrich_feature_traces` order (T7)
was already followed for this data.

## T3 — Shared-legit semantics

**Gap confirmed first (per `IMPROVE_DESIGN.md` §T3 verify step):** `Override::Pickaxe`'s term is
declared in `feature_presence.rs` `OVERRIDES` but **discarded** by
`journey_override_paths` (`feature_trace_enrich.rs:620-633` pre-fix) — `Override::Pickaxe(_, path)`
only keeps `path`. `git_log_shas` (`feature_trace_enrich.rs:718-747` pre-fix) does a plain
path-scoped `git log -- <paths>`, never `git log -S<term>` (real pickaxe). The only place a term is
ever consulted is `entry_mentions_token` (title/body/**filenames**, not diff content), and only when
`require_token` is set via `module_s_token` — a MODULE_S-only path, unreachable for any of the 4
groups below (none are MODULE_S). **So today, Pickaxe rows for these journeys would contribute
nothing beyond a plain `Override::Path`.**

Baseline (current committed `feature_traces.json`, no overrides for any of the 4 groups):
all four groups' members share **byte-identical** `discovery_paths` and `follow_up_prs` arrays
(sibling ratio = 1.0 exactly), because none has a journey-level override and each falls back to the
module's whole-service default path.

### 1. `RebootOnboardComputer` / `ShutdownOnboardComputer` — **impossible**

One handler, `POST /shutdown` (`core/services/commander/main.py:88-98`), keyed by a
`ShutdownType` body enum (`REBOOT` vs `POWEROFF`). Real `git log -S'REBOOT'` vs
`-S'POWEROFF'` over `core/services/commander` (not just title/body matching): 1 commit each,
**same commit** (intersection/union = 1/1 = 1.0). There is no route/method split — both branches
of the one handler are always edited together historically. Title/body token check
(`reboot` vs `shutdown`/`poweroff` across the 23 existing follow-ups) independently confirms this:
both terms hit the exact same 2 PRs (853, 922). **No code change**; documented impossible per the
non-goal "do not force-split a group with no route/method/title signal."

### 2. `UpdateBlueosVersion` / `SwitchLocalBlueosVersion` / `PullBlueosVersionWithoutSwitch` — **impossible**

Distinct routes do exist — `POST /version/current` (`set_version`) vs `POST /version/pull`
(`pull_version`), `core/services/versionchooser/api/v1/routers/version.py:35-46` — and
`UpdateBlueosVersion` (`pullAndSetVersion`, `VersionChooser.vue:534-552`) is literally the
composition of both. But neither of two independent probes separates them: (a) none of the 43
existing follow-up PRs mention `set_version`/`pull_version`/`/version/pull`/`/version/current` in
title, body, or changed filenames — the route strings live only in diff bodies never surfaced to
`entry_mentions_token`; (b) real `git log -S'set_version'` vs `-S'pull_version'` over
`core/services/versionchooser` + the Vue component: 3 commits each, **same 3 commits**
(ratio 1.0) — `VersionChooser`'s `set_version`/`pull_version` pair is refactored in lockstep every
time. **No code change**; `UpdateBlueosVersion` is inherently the union of the other two, so it
cannot be split from either without losing coverage — documented impossible.

### 3. `InstallExtension` / `UninstallExtension` (kraken) — **impossible**; `EditExtensionDevVersion` / `ConfigureInstalledExtension` — **already separated, no action needed**

Distinct routes exist: `POST /install` vs `POST /uninstall`,
`core/services/kraken/api/v1/routers/extension.py:20-29`. Real `git log -S'install_extension'` vs
`-S'uninstall_extension'` over `core/services/kraken`: 10 commits each, **identical set**
(ratio 1.0) — the two handlers are adjacent in the router and always touched together across
kraken's restructurings. **No code change**; documented impossible.
The second pair in this group is a false alarm: `ConfigureInstalledExtension` already has a
distinct file-level `Override::Path` (cards/modals only); measured ratio against
`EditExtensionDevVersion` from the current `feature_traces.json` is **0.056** (4/71), already well
under the 0.40 gate. No pickaxe experiment needed for this pair.

### 4. `StartAutopilot` / `StopAutopilot` / `RestartAutopilot` — **improved via minimal pickaxe wiring (code change: yes)**

Distinct handlers: `start_ardupilot` / `kill_ardupilot` / `restart_ardupilot`
(`core/services/ardupilot_manager/autopilot_manager.py:602,631,662`, routed at
`api/v1/routers/index.py:232-276`). Unlike groups 1-3, real `-S` pickaxe on these three terms over
`core/services/ardupilot_manager` gives non-identical, only partially-overlapping commit sets — a
genuine route/method signal existed but was unused. Implemented option (a): wired the `Pickaxe`
term through to `discover_follow_up_prs`'s underlying `git log`, so it now runs
`git log -S<term> -- <paths>` instead of a plain path-scoped log for journeys carrying a
`Pickaxe` override.

Code changes (`catalog/src/tools/feature_trace_enrich.rs`, `catalog/src/tools/feature_presence.rs`):
- `git_log_shas` takes a new `pickaxe_term: Option<&str>` and adds `-S<term>` to the `git log`
  invocation when set.
- New `journey_pickaxe_term(journey_id)` reads a journey's `Override::Pickaxe` term (parallel to
  `journey_override_paths`, which already reads the path half of the same rows).
- `discover_follow_up_prs` takes `pickaxe_term` and threads it into `git_log_shas`; the enrich loop
  passes `journey_pickaxe_term(journey_id)` at the call site. `discover_backport_prs` (release-line
  probe) is untouched — scope is "follow-up filter for that journey only" per design.
- Added 3 `OVERRIDES` rows: `StartAutopilot` → `Pickaxe("start_ardupilot", "core/services/ardupilot_manager")`,
  `StopAutopilot` → `Pickaxe("kill_ardupilot", ...)`, `RestartAutopilot` → `Pickaxe("restart_ardupilot", ...)`.

Verified end-to-end with a scoped `--journey` run against a backed-up copy of `feature_traces.json`
(restored after, no diff against the committed file — this task's data change is not yet landed):

| Pair | Before (path-only) | After (pickaxe-wired) |
|---|---|---|
| Start/Stop | 1.0 (identical 47-item `fu`) | **0.6** (6/10) |
| Start/Restart | 1.0 | **0.667** (6/9) |
| Stop/Restart | 1.0 | **0.30** (3/10) — crosses the <0.40 gate |

`cargo test -p blueos-catalog --lib` — 160/160 passing (includes `autopilot_pair_sibling_ratio_below_gate`,
unaffected since it covers `StartAutopilot`/`UpdateFirmwareOnline`, not this group). `cargo fmt --check` clean.

### needs-re-enrich?
**Yes, full re-enrich required for Phase 5.** `enrich_feature_traces --journey <id>` overwrites
`feature_traces.json` with *only* the filtered journey's data (it rebuilds `journeys`/`intro_clusters`
from `feature_presence_map.json`, not merged with the existing file) — unsafe to run per-journey
against the committed file. The three new `OVERRIDES` rows are code-only in this commit; the
`discovery_paths`/`follow_up_prs` shown above for groups 1-4 (and the improved ratios) require a
full `generate_feature_presence` → `enrich_feature_traces` run to land in `feature_traces.json`
and the T1 sibling matrix.
