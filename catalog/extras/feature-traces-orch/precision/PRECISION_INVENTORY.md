# Phase 1 — Precision inventory

Method: for each hub, read `catalog/src/journeys/<module>.rs` for the
per-journey `const` file citations already in `Provenance::source(...)`, then
cross-check the exact backend route handler (`main.py`/router) each journey
hits. A hint is "distinct" only if it isolates a journey to files/pickaxe
terms no sibling in the same intro cluster also cites. Current baseline for
all modules below (unless noted) is a single `MODULE_DEFAULT_PATH` shared by
every journey in the module (see `feature_presence.rs`).

## frontend_parameters (cluster `40c0bac44abd`, 1 journey, fup=60, paths=13)
No siblings (only `ApplyParameterFile`) — problem is hint width, not
disambiguation. Current discovery pulls in hub files shared by every
autopilot-adjacent journey: `store/autopilot.ts`, `libs/MAVLink2Rest/index.ts`,
`ArduPilot-Parameter-Repository` (submodule pointer).
Proposed hints (replace `MODULE_DEFAULT_PATH` with explicit list):
1. `core/frontend/src/components/parameter-editor/ParameterEditor.vue`
2. `core/frontend/src/components/parameter-editor/ParameterLoader.vue`
3. `core/frontend/src/components/parameter-editor/ParameterEditorDialog.vue`
4. `core/frontend/src/views/ParameterEditorView.vue`
5. Drop `store/autopilot.ts` and `libs/MAVLink2Rest` — too broad, shared by every mavlink2rest-consuming journey.

## helper (cluster `f72511146a46`, 4 journeys, fup=46 each, paths=4)
All 4 share `core/services/helper` only. Distinct per-journey hints exist:
- `MonitorInternetConnectivity` → `core/frontend/src/store/helper.ts` (20s poll)
- `VerifyInternetConnectivity` → `core/frontend/src/components/wizard/RequireInternet.vue`
- `BrowseAvailableWebServices` → `core/frontend/src/views/AvailableServicesView.vue`, `core/frontend/src/components/scanner/availableServicesTable.vue`
- `ProbeInterfaceInternetConnectivity` → `core/frontend/src/components/app/NetworkInterfacePriorityMenu.vue`
Backend `/check_internet_access` handler stays shared-legit between Monitor/Verify (same route, no frontend overlap otherwise).

## commander (cluster `d1c0ee626e36`, 8 journeys, fup=23 each, paths=3)
Distinct file already citable for 5/8:
- `SyncSystemTime` → `core/frontend/src/utils/update_time.ts`
- `EnableLegacyCameraSupport` → `core/frontend/src/components/video-manager/VideoManager.vue`
- `ResetBlueosSettings` → `core/frontend/src/views/SettingsView.vue`
- `RunHostCommand` → `core/frontend/src/store/commander.ts`
- `InspectRaspberryEepromBootloader` → Pickaxe(`getVcgencmd`/`getRaspiEEPROM`, `Firmware.vue`) — same file as Update but distinct methods (line 232-233 vs 213)
- `UpdateRaspberryEepromBootloader` → Pickaxe(`doRaspiEEPROMUpdate`, `Firmware.vue`)
- **shared-legit**: `RebootOnboardComputer` + `ShutdownOnboardComputer` both call the single `shutdown()` handler in `commander/main.py:90` differing only by request body enum (`ShutdownType.REBOOT`/`POWEROFF`); no source-file separation possible.

## wifi (cluster `899812c65007`, 6 journeys, fup=31 each, paths=5)
- `DisconnectFromWifiNetwork` → `core/frontend/src/components/wifi/DisconnectionDialog.vue` (already distinct)
- `ForgetSavedWifiNetwork` → Pickaxe(`/remove`, `ConnectionDialog.vue`) (line 230)
- `ConnectToWifiNetwork` → Pickaxe(`/connect`, `ConnectionDialog.vue`) (line 208, same file as Forget, distinct route)
- `ConfigureHotspotCredentials` → Pickaxe(`hotspot_credentials`, `WifiSettingsDialog.vue`)
- `ToggleSmartHotspot` → Pickaxe(`smart_hotspot`, `WifiSettingsDialog.vue`) (shares file with Credentials, distinct route)
- `ToggleHotspot` → `core/frontend/src/components/wifi/WifiManager.vue` (line 262, distinct file from the other two)

## versionchooser (2 clusters)
`DockerRegistryLogin` cluster (`a7e47b06d4d8`, fup=14) already isolated via existing `Pickaxe("docker/login", ...)` override — no change needed.
Main hub cluster `7c62889397a3` (5 journeys, fup=43, paths=11) — all funnel through the single `VersionChooser.vue`:
- `UpdateBootstrapImage` → `core/services/versionchooser/api/v1/routers/bootstrap.py` (distinct router file, already citable)
- `DeleteLocalBlueosVersion` → Pickaxe(`/version/delete`, `versionchooser/api/v1/routers/version.py`) (unique route)
- **shared-legit**: `UpdateBlueosVersion`, `SwitchLocalBlueosVersion`, `PullBlueosVersionWithoutSwitch` all read/write overlapping `VersionChooser.vue` regions (`/version/pull` + `/version/current`) — no clean file/route split without over-fragmenting a single component.

## kraken (cluster `ce9c65e18f0b`, 7 journeys, fup=44, paths=10)
Journey defs cite doc lines only (no source consts) — hints come from grepping `core/frontend/src/components/kraken/`:
- `AddCustomManifest` → `core/frontend/src/components/kraken/BackAlleyTab.vue`
- `BrowseExtensionStore` → `core/frontend/src/components/kraken/BazaarTab.vue`
- `InstallCustomExtension` → `core/frontend/src/components/kraken/modals/ExtensionCreationModal.vue`
- `ConfigureInstalledExtension` → `core/frontend/src/components/kraken/cards/InstalledExtensionCard.vue`, `modals/ExtensionSettingsModal.vue`, `modals/ExtensionLogsModal.vue`
- **shared-legit**: `InstallExtension` + `UninstallExtension` both live in the same version-dropdown code path of `StoreExtensionCard.vue`; `EditExtensionDevVersion` also shares `InstalledExtensionCard.vue` with `ConfigureInstalledExtension` (same edit-tag control).

## cable_guy (cluster `635536507e69`, 6 journeys, fup=24, paths=12)
4/6 already distinct: `AssignStaticIpAddress`→`AddressCreationDialog.vue`, `EnableOnboardDhcpServer`→`DHCPServerDialog.vue`, `SetNetworkInterfacePriority`→`NetworkInterfacePriorityMenu.vue`, `ConfigureHostDns`→`DnsConfigurationMenu.vue`.
- `AcquireDynamicIpAddress` → Pickaxe(`dynamic_ip`, `InterfaceCard.vue`)
- `DisableOnboardDhcpServer` → Pickaxe(`dhcp_server`, `InterfaceCard.vue`) (same file as above, distinct feature block)

## beacon (cluster `f5958d484bf1`, 3 journeys, fup=22, paths=5)
- `RenameVehicle` → Pickaxe(`/vehicle_name`, `VehicleBanner.vue`)
- `ChangeMdnsHostname` → Pickaxe(`/hostname`, `VehicleBanner.vue`) (same file, distinct route)
- `DiscoverBlueosOnNetwork` → keep `core/services/beacon/main.py` (service-wide mDNS advertisement; no frontend component)

## bridget (cluster `295763cb8ff9`, 3 journeys, fup=25, paths=8)
- `ViewConfiguredSerialBridges` → `core/frontend/src/store/bridget.ts`
- `CreateSerialToUdpBridge` → `core/frontend/src/components/bridges/BridgeCreationDialog.vue`, `core/services/bridget/bridget.py`
- `RemoveSerialBridge` → Pickaxe(`remove`, `BridgeCard.vue`) (view step also touches this file; remove button is the distinguishing region)

## nmea_injector (cluster `0384553acf47`, 3 journeys, fup=26, paths=15)
- `AddExternalNmeaGpsSocket` → `core/frontend/src/components/nmea-injector/NMEASocketCreationDialog.vue`
- `RemoveConfiguredNmeaSocket` → `core/frontend/src/components/nmea-injector/NMEASocketCard.vue`
- `ViewConfiguredNmeaSockets` → `core/frontend/src/components/nmea-injector/NMEAInjector.vue` (list rendering only; already narrower than the current 15-path set which drags in `README.md`/`setup.py`/`exceptions.py`)

## frontend_calibration
Not a sibling-sharing hub: each journey (`CalibrateGyroscope`, `CalibrateBarometer`, `CalibrateAccelerometer`, `CalibrateCompass`, `DetectMotorDirections`, `LevelHorizon`) is its own intro cluster and already has a per-journey `Override::Path` to a distinct Vue file. High fup counts (33-34) reflect genuine calibration-code churn, not hint imprecision — no change proposed.

## ardupilot_manager (cluster `c0185c6686cf12ccb065a39299b943dec75b7962`, 9 journeys, fup=47 each, paths=9)
Current discovery is entirely `mavlink_proxy/`-heavy (`AbstractRouter.py`, `Endpoint.py`, `MAVLinkRouter.py`, `MAVProxy.py`, `main.py`) — shared by every journey regardless of intent (router churn, not per-journey signal). Distinct per-journey hints:
- `VehicleFirstBoot` → `core/services/ardupilot_manager/firmware/`, `core/frontend/src/components/autopilot/FirmwareManager.vue`, `core/services/ardupilot_manager/api/v1/routers/index.py` (`/install_firmware_from_url`)
- `ChangeBoard` → `core/services/ardupilot_manager/flight_controller_detector/`, `core/frontend/src/components/autopilot/BoardChangeDialog.vue`
- `RunSitlSimulation` → `core/services/ardupilot_manager/typedefs.py` (SITL frame types), `core/frontend/src/components/autopilot/SitlConfiguration.vue`
- `UpdateFirmwareOnline` → `core/services/ardupilot_manager/firmware/FirmwareDownload.py`, `FirmwareManager.vue`
- `UploadCustomFirmware` → `core/services/ardupilot_manager/firmware/FirmwareUpload.py`, `FirmwareManager.vue`
- `RestoreDefaultFirmware` → `core/services/ardupilot_manager/firmware/FirmwareInstall.py`, `FirmwareManager.vue`
- **shared-legit**: `StartAutopilot` / `StopAutopilot` / `RestartAutopilot` all funnel through `core/frontend/src/components/autopilot/AutopilotManagerUpdater.ts` (start/stop/restart buttons on the same page) plus the shared `mavlink_proxy/` router lifecycle — no source-file separation possible beyond the already-distinct `/start`, `/stop`, `/restart` routes in `api/v1/routers/index.py`.

## MODULE_S_and_proxied (cluster `c4fe82a264d4`, 8 journeys, fup varies 0-33, paths=1 each)
Root cause: `module_hint_paths()` (`catalog/src/tools/feature_trace_enrich.rs:564-577`) only
special-cases `module == "nginx"` when appending a path for a `MODULE_S` entry; the other four
`MODULE_S` services (`ttyd`, `linux2rest`, `mavlink2rest`, `mavlink_camera_manager`) have no
`MODULE_DEFAULT_PATH` entry and no `OVERRIDES`, so hints come back empty and discovery falls back
to the single generic `core/start-blueos-core` path (or, for the camera journeys, incidentally
lands on the wrong file `store/mavlink.ts`).
- `AccessBlueosWebInterface` (nginx) — already good: `core/tools/nginx/nginx.conf` (fu=33)
- `AccessWebTerminal` (ttyd) → add `Override::Path("core/frontend/src/views/TerminalView.vue")` (fu=0, no `MODULE_DEFAULT_PATH` today)
- `ViewSystemInformation` (linux2rest) → add `Override::Path`s for `core/frontend/src/views/SystemInformationView.vue` and `core/frontend/src/store/system-information.ts` (fu=3, weak `core/start-blueos-core` fallback)
- `InspectMavlinkMessagesInBrowser` (mavlink2rest) → add `Override::Path("core/frontend/src/views/MavlinkInspectorView.vue")`, optional `Override::Pickaxe` token for mavlink2rest requests (fu=2, weak fallback)
- `ViewCameraStreams` (mavlink_camera_manager) → add `Override::Path`s for `core/frontend/src/store/video.ts` and `core/frontend/src/components/video-manager/VideoManager.vue` (fu=2, currently wrong: resolves to `store/mavlink.ts`, not `video.ts`)
- `ConfigureCameraStream` (mavlink_camera_manager) → add `Override::Path("core/frontend/src/components/video-manager/VideoStreamCreationDialog.vue")` (fu=2, same wrong `store/mavlink.ts` fallback)
- `RemoveCameraStream` (mavlink_camera_manager) → add `Override::Path("core/frontend/src/components/video-manager/VideoStream.vue")` (fu=2, same wrong fallback)
- `ConfigureUvcDeviceControls` (mavlink_camera_manager) → add `Override::Path("core/frontend/src/components/video-manager/VideoControlsDialog.vue")` (fu=2, same wrong fallback)

## Shared-legit summary (cannot be separated by file/pickaxe)
- commander: `RebootOnboardComputer` / `ShutdownOnboardComputer`
- versionchooser: `UpdateBlueosVersion` / `SwitchLocalBlueosVersion` / `PullBlueosVersionWithoutSwitch`
- kraken: `InstallExtension` / `UninstallExtension`; `EditExtensionDevVersion` / `ConfigureInstalledExtension`
- ardupilot_manager: `StartAutopilot` / `StopAutopilot` / `RestartAutopilot`
