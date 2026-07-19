# Bootstrap-era shared intro_commit clusters

Computed from `catalog/feature_traces.json` (`.journeys` grouped by `intro_commit`, cross-referenced with `.intro_clusters[sha].follow_up_prs`).

## 1. Clusters with ≥2 modules or >15 follow_up_prs (19 total)

| intro_commit | follow_ups | modules | journeys |
|---|---|---|---|
| `40c0bac44a` | 60 | frontend_parameters | ApplyParameterFile |
| `c4fe82a264` | 55 | **linux2rest, mavlink2rest, mavlink_camera_manager, nginx, ttyd** | AccessBlueosWebInterface, AccessWebTerminal, ConfigureCameraStream, ConfigureUvcDeviceControls, InspectMavlinkMessagesInBrowser, RemoveCameraStream, ViewCameraStreams, ViewSystemInformation |
| `c0185c6686` | 47 | ardupilot_manager | ChangeBoard, RestartAutopilot, RestoreDefaultFirmware, RunSitlSimulation, StartAutopilot, StopAutopilot, UpdateFirmwareOnline, UploadCustomFirmware, VehicleFirstBoot |
| `f72511146a` | 46 | helper | BrowseAvailableWebServices, MonitorInternetConnectivity, ProbeInterfaceInternetConnectivity, VerifyInternetConnectivity |
| `ce9c65e18f` | 44 | kraken | AddCustomManifest, BrowseExtensionStore, ConfigureInstalledExtension, EditExtensionDevVersion, InstallCustomExtension, InstallExtension, UninstallExtension |
| `7c62889397` | 43 | versionchooser | DeleteLocalBlueosVersion, PullBlueosVersionWithoutSwitch, SwitchLocalBlueosVersion, UpdateBlueosVersion, UpdateBootstrapImage |
| `22025de9a5` | 34 | frontend_calibration | CalibrateGyroscope |
| `11088c7b8a` | 33 | frontend_calibration | DetectMotorDirections |
| `81fddcf87f` | 33 | frontend_calibration | CalibrateBarometer |
| `8da1baa6c5` | 33 | frontend_video | ConfigureVideoStream |
| `dc388cef20` | 33 | frontend_calibration | CalibrateAccelerometer |
| `899812c650` | 31 | wifi | ConfigureHotspotCredentials, ConnectToWifiNetwork, DisconnectFromWifiNetwork, ForgetSavedWifiNetwork, ToggleHotspot, ToggleSmartHotspot |
| `0384553acf` | 26 | nmea_injector | AddExternalNmeaGpsSocket, RemoveConfiguredNmeaSocket, ViewConfiguredNmeaSockets |
| `295763cb8f` | 25 | bridget | CreateSerialToUdpBridge, RemoveSerialBridge, ViewConfiguredSerialBridges |
| `635536507e` | 24 | cable_guy | AcquireDynamicIpAddress, AssignStaticIpAddress, ConfigureHostDns, DisableOnboardDhcpServer, EnableOnboardDhcpServer, SetNetworkInterfacePriority |
| `d1c0ee626e` | 23 | commander | EnableLegacyCameraSupport, InspectRaspberryEepromBootloader, RebootOnboardComputer, ResetBlueosSettings, RunHostCommand, ShutdownOnboardComputer, SyncSystemTime, UpdateRaspberryEepromBootloader |
| `f5958d484b` | 22 | beacon | ChangeMdnsHostname, DiscoverBlueosOnNetwork, RenameVehicle |
| `2f8d657c6c` | 17 | frontend_calibration | CalibrateCompass |
| `5de1feb472` | 16 | ping | ConnectPingViewerToSonar, EnablePing1dRangefinderMavlink, ViewDetectedSonarDevices |

Only **one** cluster (`c4fe82a264`) is truly cross-module: its intro commit is the bootstrap commit that added `main.py` for 8 unrelated services at once. The other 18 are single-module but multi-journey — `discovery_paths` widens to the whole module (e.g. all `frontend_calibration` components) because several distinct calibration journeys (gyro/accel/baro/compass/motor-direction) share one module-level intro commit, so any of that module's PRs count as a follow-up for every journey in the group.

## 2. Current module selection (`build_intro_cluster`)

```1126:1129:catalog/src/tools/feature_trace_enrich.rs
    let module = group
        .first()
        .and_then(|j| j.get("module"))
        .and_then(|v| v.as_str());
```

`group` is the `Vec<Value>` of journeys sharing `intro_commit` (`by_commit` map, unordered by module). One module is picked arbitrarily (`group[0]`); `discovery_paths()` filters by that module's tokens (`/services/<module>/`, `/<module>/`) against the union of intro-commit + landing-PR `files_changed`. For `c4fe82a264`, `group[0].module == "nginx"`, no path matches an `nginx` service dir, so `discovery_paths` falls through to the generic `/services/` fallback (`feature_trace_enrich.rs:430-436`), pulling in `main.py` for every service the bootstrap commit touched — none of which relate to the other 7 modules actually in the group.

## 3. Recommended approach: **A** — per-journey nested discovery

Keep `IntroClusterOut`/`IntroCluster` as the shared-fact record (`intro_commit`, `landing_prs`, `merge_commit_sha`, `merge_method`, `squash_merge`, `intro_sha_in_pr_commits` stay singular — they're properties of the commit/PR, not the journey). Add a `by_journey: BTreeMap<String, JourneyDiscovery>` map nested inside, where `JourneyDiscovery { discovery_paths, follow_up_prs, backport_prs, issues }` is computed per-journey using that journey's own `module` (§4).

- `cluster_for_journey` in `feature_trace.rs` changes only its return path: `cluster.by_journey.get(journey_id)` merged with the shared top-level fields (a couple of struct-field reads, not a signature change) — matches the "minimize churn" requirement.
- **B** (key by journey id) duplicates the shared landing/squash/merge fields per journey and breaks the existing `intro_commit → cluster` map shape and its `assert_eq!(traces.intro_clusters.len(), 30)`-style invariants for no benefit — the shared facts are correctly commit-scoped, only discovery is journey-scoped.
- **C**: none better identified.

## 4. Module-scoped `discovery_paths` algorithm

For each journey in `group`, independently:

1. Take `journey.module` (not `group[0].module`).
2. Filter the same candidate path set (intro-commit `files_changed` ∪ primary-landing-PR `files_changed`) through the *existing* `discovery_paths(paths, Some(module))` token match (`/services/{module}/`, `/{module}/`, module tokens).
3. **New**: if the module-filtered set is empty, do **not** fall back to the global `/services/` sweep (that fallback is what leaks unrelated services). Instead fall back to `focused` (`/components/`,`/views/`,`/store/`) filtered *and* still gated by module token match; if still empty, return the raw per-journey files from `journey`'s own commit/PR data (each journey already carries enough to resolve its own narrow path) rather than the union.
4. Run `discover_follow_up_prs`/`discover_backport_prs` once per resulting per-journey path set instead of once per cluster.

This makes the multi-module case (`c4fe82a264`) resolve 8 independent narrow path sets (one per real module) instead of one wide `/services/*` sweep, and makes same-module multi-journey cases (calibration) resolve per-journey subsets only if the underlying files differ per journey — if they're genuinely identical files (shared calibration component), the follow-up count for that module legitimately stays wide, which is correct, not a bug.

## 5. Expected impact

- `c4fe82a264` (55 → likely single digits per journey): biggest win — it's the only case where paths are objectively *wrong* (unrelated services), so isolating per-journey/module removes 100% of cross-module noise.
- `frontend_calibration` cluster (5 sub-clusters, 34/33/33/33/17): impact depends on whether each calibration journey touches distinct files; if calibration journeys share a common Vue component tree, follow-up counts stay similar per journey (correctly), but per-journey isolation still stops one calibration PR from inflating the *other* four journeys' counts — expect real per-journey reduction since each journey (gyro/accel/baro/compass/motor) has its own dedicated view/component in `frontend/src/components/vehiclesetup/calibration/`.
- `ce9c65e18f` (kraken, 7 journeys), `c0185c6686` (ardupilot_manager, 9 journeys), `d1c0ee626e` (commander, 8 journeys), `899812c650` (wifi, 6 journeys): moderate shrinkage — many journeys per module means today's shared count is split N ways once table-driven per-journey, even though total commit-level activity is unchanged.
- Single-journey clusters (`40c0bac44a`, `ApplyParameterFile`) get **no benefit** from A — the width there isn't cross-journey pollution, it's a genuinely broad `discovery_paths` for one journey; needs separate follow-up (tighter path/noise filtering), out of scope here.

## Post-implement QA

Schema: `intro_clusters[*].by_journey[JourneyId]` holds per-journey
`discovery_paths` / `follow_up_prs` / `backport_prs` / `issues`. Shared merge
fields stay on the cluster.

| journey | follow_ups | notes |
|---|---|---|
| AccessBlueosWebInterface | 33 | nginx.conf-scoped (was 55 shared dump) |
| AccessWebTerminal | 0 | start-blueos-core + `ttyd` token; no matching PRs |
| ViewSystemInformation | 3 | start-blueos-core + `linux2rest` token |
| InspectMavlinkMessagesInBrowser | 2 | mavlink2rest token |
| ConfigureCameraStream | 2 | frontend mavlink store path |
| InspectZenohNetwork | 13 | golden unchanged |
| ChangeBoard | 47 | same-module shared paths; needs journey-specific hints later |

Goldens (zenoh/custom/disk/speed/horizon) still pass. Remaining wide single-module
clusters (ardupilot_manager, parameter editor, …) are genuine module activity,
not cross-service bootstrap leakage.

