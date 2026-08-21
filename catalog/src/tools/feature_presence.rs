//! Regenerates `catalog/src/journey_presence.rs` and `catalog/feature_presence_map.json`
//! from git tag membership. Invoked by:
//! `cargo run -p blueos-catalog --bin generate_feature_presence`

use std::collections::BTreeMap;

use regex::Regex;
use serde::Serialize;
use sha1::{Digest, Sha1};

use crate::tools::shell::{repo_root, run as run_git, run_ok};

pub(crate) enum Override {
    Path(&'static str),
    Pickaxe(&'static str, &'static str),
}

// Copied from generate logic — keep in sync with journey overrides.
pub(crate) const OVERRIDES: &[(&str, Override)] = &[
    (
        "InspectZenohNetwork",
        Override::Path("core/frontend/src/views/ZenohInspectorView.vue"),
    ),
    (
        "InspectZenohNetwork",
        Override::Path("core/frontend/src/components/zenoh-inspector"),
    ),
    (
        "RunInternetSpeedTest",
        Override::Pickaxe("internet_best_server", "core/services/pardal"),
    ),
    ("RunLanSpeedTest", Override::Path("core/services/pardal")),
    (
        "LevelHorizon",
        Override::Path(
            "core/frontend/src/components/vehiclesetup/configuration/compass/LevelHorizonCalibration.vue",
        ),
    ),
    ("InspectDiskUsage", Override::Path("core/services/disk_usage")),
    (
        "InspectDiskUsage",
        Override::Path("core/frontend/src/views/Disk.vue"),
    ),
    (
        "InspectDiskUsage",
        Override::Path("core/frontend/src/store/disk.ts"),
    ),
    ("FreeDiskSpace", Override::Path("core/services/disk_usage")),
    (
        "RunSingleDiskSpeedTest",
        Override::Pickaxe("disktest", "core/services/disk_usage"),
    ),
    (
        "RunMultiSizeDiskSpeedTest",
        Override::Pickaxe("disktest", "core/services/disk_usage"),
    ),
    (
        "BrowseVideoRecordings",
        Override::Path("core/services/recorder_extractor"),
    ),
    (
        "DownloadVideoRecording",
        Override::Path("core/services/recorder_extractor"),
    ),
    (
        "DeleteVideoRecording",
        Override::Path("core/services/recorder_extractor"),
    ),
    ("ChangeUiThemeColor", Override::Path("core/services/customization")),
    ("ResetUiThemeColor", Override::Path("core/services/customization")),
    ("UploadCustomLogo", Override::Path("core/services/customization")),
    ("RemoveCustomLogo", Override::Path("core/services/customization")),
    (
        "UploadCustomVehicleImage",
        Override::Path("core/services/customization"),
    ),
    (
        "RemoveCustomVehicleImage",
        Override::Path("core/services/customization"),
    ),
    ("Upload3dModelOverride", Override::Path("core/services/customization")),
    ("Delete3dModelOverride", Override::Path("core/services/customization")),
    (
        "DockerRegistryLogin",
        Override::Pickaxe("docker/login", "core/services/versionchooser"),
    ),
    (
        "CalibrateGyroscope",
        Override::Path("core/frontend/src/components/vehiclesetup/overview/GyroCalib.vue"),
    ),
    (
        "CalibrateBarometer",
        Override::Path("core/frontend/src/components/vehiclesetup/overview/BaroCalib.vue"),
    ),
    (
        "CalibrateAccelerometer",
        Override::Path(
            "core/frontend/src/components/vehiclesetup/configuration/accelerometer/FullAccelerometerCalibration.vue",
        ),
    ),
    (
        "CalibrateCompass",
        Override::Path(
            "core/frontend/src/components/vehiclesetup/configuration/compass/FullCompassCalibrator.vue",
        ),
    ),
    (
        "DetectMotorDirections",
        Override::Path("core/frontend/src/components/vehiclesetup/MotorDetection.vue"),
    ),
    (
        "ViewCameraStreams",
        Override::Path("core/frontend/src/store/video.ts"),
    ),
    (
        "ViewCameraStreams",
        Override::Path("core/frontend/src/components/video-manager/VideoManager.vue"),
    ),
    ("AccessWebTerminal", Override::Path("core/frontend/src/views/TerminalView.vue")),
    (
        "ViewSystemInformation",
        Override::Path("core/frontend/src/views/SystemInformationView.vue"),
    ),
    (
        "ViewSystemInformation",
        Override::Path("core/frontend/src/store/system-information.ts"),
    ),
    (
        "InspectMavlinkMessagesInBrowser",
        Override::Path("core/frontend/src/views/MavlinkInspectorView.vue"),
    ),
    (
        "ConfigureCameraStream",
        Override::Path("core/frontend/src/components/video-manager/VideoStreamCreationDialog.vue"),
    ),
    (
        "RemoveCameraStream",
        Override::Path("core/frontend/src/components/video-manager/VideoStream.vue"),
    ),
    (
        "ConfigureUvcDeviceControls",
        Override::Path("core/frontend/src/components/video-manager/VideoControlsDialog.vue"),
    ),
    (
        "ConfigureVideoStream",
        Override::Path("core/frontend/src/components/video-manager/VideoDiagnosticHelper.vue"),
    ),
    (
        "ConfigureVideoStream",
        Override::Path("core/frontend/src/components/video-manager/VideoThumbnail.vue"),
    ),
    (
        "ApplyParameterFile",
        Override::Path("core/frontend/src/components/parameter-editor/ParameterEditor.vue"),
    ),
    (
        "ApplyParameterFile",
        Override::Path("core/frontend/src/components/parameter-editor/ParameterLoader.vue"),
    ),
    (
        "ApplyParameterFile",
        Override::Path("core/frontend/src/components/parameter-editor/ParameterEditorDialog.vue"),
    ),
    (
        "ApplyParameterFile",
        Override::Path("core/frontend/src/views/ParameterEditorView.vue"),
    ),
    (
        "MonitorInternetConnectivity",
        Override::Path("core/frontend/src/store/helper.ts"),
    ),
    (
        "VerifyInternetConnectivity",
        Override::Path("core/frontend/src/components/wizard/RequireInternet.vue"),
    ),
    (
        "BrowseAvailableWebServices",
        Override::Path("core/frontend/src/views/AvailableServicesView.vue"),
    ),
    (
        "BrowseAvailableWebServices",
        Override::Path("core/frontend/src/components/scanner/availableServicesTable.vue"),
    ),
    (
        "ProbeInterfaceInternetConnectivity",
        Override::Path("core/frontend/src/components/app/NetworkInterfacePriorityMenu.vue"),
    ),
    ("SyncSystemTime", Override::Path("core/frontend/src/utils/update_time.ts")),
    (
        "EnableLegacyCameraSupport",
        Override::Path("core/frontend/src/components/video-manager/VideoManager.vue"),
    ),
    ("ResetBlueosSettings", Override::Path("core/frontend/src/views/SettingsView.vue")),
    ("RunHostCommand", Override::Path("core/frontend/src/store/commander.ts")),
    (
        "InspectRaspberryEepromBootloader",
        Override::Pickaxe("getVcgencmd", "core/frontend/src/components/system-information/Firmware.vue"),
    ),
    (
        "InspectRaspberryEepromBootloader",
        Override::Pickaxe("getRaspiEEPROM", "core/frontend/src/components/system-information/Firmware.vue"),
    ),
    (
        "UpdateRaspberryEepromBootloader",
        Override::Pickaxe("doRaspiEEPROMUpdate", "core/frontend/src/components/system-information/Firmware.vue"),
    ),
    (
        "DisconnectFromWifiNetwork",
        Override::Path("core/frontend/src/components/wifi/DisconnectionDialog.vue"),
    ),
    (
        "ForgetSavedWifiNetwork",
        Override::Pickaxe("/remove", "core/frontend/src/components/wifi/ConnectionDialog.vue"),
    ),
    (
        "ConnectToWifiNetwork",
        Override::Pickaxe("/connect", "core/frontend/src/components/wifi/ConnectionDialog.vue"),
    ),
    (
        "ConnectToHiddenWifiNetwork",
        Override::Pickaxe("/connect", "core/frontend/src/components/wifi/ConnectionDialog.vue"),
    ),
    (
        "ForceWifiNetworkPassword",
        Override::Pickaxe("/connect", "core/frontend/src/components/wifi/ConnectionDialog.vue"),
    ),
    (
        "ReconnectToSavedWifiNetwork",
        Override::Pickaxe("/connect", "core/frontend/src/components/wifi/ConnectionDialog.vue"),
    ),
    (
        "RejectInvalidWifiCredentials",
        Override::Pickaxe("/connect", "core/frontend/src/components/wifi/ConnectionDialog.vue"),
    ),
    (
        "DetectWifiApLoss",
        Override::Path("core/frontend/src/components/wifi/WifiUpdater.vue"),
    ),
    (
        "AutoconnectToSavedWifiNetwork",
        Override::Pickaxe("/connect", "core/frontend/src/components/wifi/ConnectionDialog.vue"),
    ),
    (
        "ConfigureHotspotCredentials",
        Override::Pickaxe("hotspot_credentials", "core/frontend/src/components/wifi/WifiSettingsDialog.vue"),
    ),
    (
        "ToggleSmartHotspot",
        Override::Pickaxe("smart_hotspot", "core/frontend/src/components/wifi/WifiSettingsDialog.vue"),
    ),
    ("ToggleHotspot", Override::Path("core/frontend/src/components/wifi/WifiManager.vue")),
    (
        "UpdateBootstrapImage",
        Override::Path("core/services/versionchooser/api/v1/routers/bootstrap.py"),
    ),
    (
        "DeleteLocalBlueosVersion",
        Override::Pickaxe("/delete", "core/services/versionchooser/api/v1/routers/version.py"),
    ),
    (
        "AddCustomManifest",
        Override::Path("core/frontend/src/components/kraken/BackAlleyTab.vue"),
    ),
    (
        "BrowseExtensionStore",
        Override::Path("core/frontend/src/components/kraken/BazaarTab.vue"),
    ),
    (
        "InstallCustomExtension",
        Override::Path("core/frontend/src/components/kraken/modals/ExtensionCreationModal.vue"),
    ),
    (
        "ConfigureInstalledExtension",
        Override::Path("core/frontend/src/components/kraken/cards/InstalledExtensionCard.vue"),
    ),
    (
        "ConfigureInstalledExtension",
        Override::Path("core/frontend/src/components/kraken/modals/ExtensionSettingsModal.vue"),
    ),
    (
        "ConfigureInstalledExtension",
        Override::Path("core/frontend/src/components/kraken/modals/ExtensionLogsModal.vue"),
    ),
    (
        "AssignStaticIpAddress",
        Override::Path("core/frontend/src/components/ethernet/AddressCreationDialog.vue"),
    ),
    (
        "EnableOnboardDhcpServer",
        Override::Path("core/frontend/src/components/ethernet/DHCPServerDialog.vue"),
    ),
    (
        "SetNetworkInterfacePriority",
        Override::Path("core/frontend/src/components/app/NetworkInterfacePriorityMenu.vue"),
    ),
    (
        "ConfigureHostDns",
        Override::Path("core/frontend/src/components/app/DnsConfigurationMenu.vue"),
    ),
    (
        "AcquireDynamicIpAddress",
        Override::Pickaxe("triggerForDynamicIP", "core/frontend/src/components/ethernet/InterfaceCard.vue"),
    ),
    (
        "DisableOnboardDhcpServer",
        Override::Pickaxe("removeDHCPServer", "core/frontend/src/components/ethernet/InterfaceCard.vue"),
    ),
    (
        "RenameVehicle",
        Override::Pickaxe("/vehicle_name", "core/frontend/src/components/app/VehicleBanner.vue"),
    ),
    (
        "ChangeMdnsHostname",
        Override::Pickaxe("/hostname", "core/frontend/src/components/app/VehicleBanner.vue"),
    ),
    (
        "ViewConfiguredSerialBridges",
        Override::Path("core/frontend/src/store/bridget.ts"),
    ),
    (
        "CreateSerialToUdpBridge",
        Override::Path("core/frontend/src/components/bridges/BridgeCreationDialog.vue"),
    ),
    ("CreateSerialToUdpBridge", Override::Path("core/services/bridget/bridget.py")),
    (
        "RemoveSerialBridge",
        Override::Pickaxe("removeBridge", "core/frontend/src/components/bridges/BridgeCard.vue"),
    ),
    (
        "AddExternalNmeaGpsSocket",
        Override::Path("core/frontend/src/components/nmea-injector/NMEASocketCreationDialog.vue"),
    ),
    (
        "RemoveConfiguredNmeaSocket",
        Override::Path("core/frontend/src/components/nmea-injector/NMEASocketCard.vue"),
    ),
    (
        "ViewConfiguredNmeaSockets",
        Override::Path("core/frontend/src/components/nmea-injector/NMEAInjector.vue"),
    ),
    ("VehicleFirstBoot", Override::Path("core/services/ardupilot_manager/firmware")),
    (
        "VehicleFirstBoot",
        Override::Path("core/frontend/src/components/autopilot/FirmwareManager.vue"),
    ),
    (
        "VehicleFirstBoot",
        Override::Pickaxe(
            "install_firmware_from_url",
            "core/services/ardupilot_manager/api/v1/routers/index.py",
        ),
    ),
    (
        "ChangeBoard",
        Override::Path("core/services/ardupilot_manager/flight_controller_detector"),
    ),
    (
        "ChangeBoard",
        Override::Path("core/frontend/src/components/autopilot/BoardChangeDialog.vue"),
    ),
    (
        "StartAutopilot",
        Override::Pickaxe("start_ardupilot", "core/services/ardupilot_manager"),
    ),
    (
        "StopAutopilot",
        Override::Pickaxe("kill_ardupilot", "core/services/ardupilot_manager"),
    ),
    (
        "RestartAutopilot",
        Override::Pickaxe("restart_ardupilot", "core/services/ardupilot_manager"),
    ),
    ("RunSitlSimulation", Override::Path("core/services/ardupilot_manager/typedefs.py")),
    (
        "RunSitlSimulation",
        Override::Path("core/frontend/src/components/autopilot/SitlConfiguration.vue"),
    ),
    (
        "UpdateFirmwareOnline",
        Override::Path("core/services/ardupilot_manager/firmware/FirmwareDownload.py"),
    ),
    (
        "UpdateFirmwareOnline",
        Override::Path("core/frontend/src/components/autopilot/FirmwareManager.vue"),
    ),
    (
        "UploadCustomFirmware",
        Override::Path("core/services/ardupilot_manager/firmware/FirmwareUpload.py"),
    ),
    (
        "UploadCustomFirmware",
        Override::Path("core/frontend/src/components/autopilot/FirmwareManager.vue"),
    ),
    (
        "RestoreDefaultFirmware",
        Override::Path("core/services/ardupilot_manager/firmware/FirmwareInstall.py"),
    ),
    (
        "RestoreDefaultFirmware",
        Override::Path("core/frontend/src/components/autopilot/FirmwareManager.vue"),
    ),
];

pub(crate) const MODULE_DEFAULT_PATH: &[(&str, &str)] = &[
    ("ardupilot_manager", "core/services/ardupilot_manager"),
    ("bag_of_holding", "core/services/bag_of_holding"),
    ("beacon", "core/services/beacon"),
    ("bridget", "core/services/bridget"),
    ("cable_guy", "core/services/cable_guy"),
    ("commander", "core/services/commander"),
    ("customization", "core/services/customization"),
    ("disk_usage", "core/services/disk_usage"),
    ("filebrowser", "core/tools/filebrowser"),
    ("helper", "core/services/helper"),
    ("kraken", "core/services/kraken"),
    ("nmea_injector", "core/services/nmea_injector"),
    ("pardal", "core/services/pardal"),
    ("ping", "core/services/ping"),
    ("recorder_extractor", "core/services/recorder_extractor"),
    ("versionchooser", "core/services/versionchooser"),
    ("wifi", "core/services/wifi"),
    ("zenohd", "core/frontend/src/views/ZenohInspectorView.vue"),
    (
        "frontend_video",
        "core/frontend/src/components/video-manager/VideoManager.vue",
    ),
    (
        "frontend_parameters",
        "core/frontend/src/views/ParameterEditorView.vue",
    ),
    (
        "frontend_calibration",
        "core/frontend/src/components/vehiclesetup/calibration.ts",
    ),
    ("user_terminal", "core/frontend/src/views/TerminalView.vue"),
];

pub(crate) const MODULE_S: &[(&str, &str, &str)] = &[
    ("linux2rest", "linux2rest", "core/start-blueos-core"),
    ("mavlink2rest", "mavlink2rest", "core/start-blueos-core"),
    (
        "mavlink_camera_manager",
        "mavlink-camera-manager",
        "core/start-blueos-core",
    ),
    ("ttyd", "ttyd", "core/start-blueos-core"),
    ("nginx", "nginx", "core/start-blueos-core"),
];

#[derive(Serialize)]
struct JourneyPresence {
    journey: String,
    module: String,
    intro_commit: String,
    intro_commit_short: String,
    method: String,
    present_in_tags: Vec<String>,
    present_on_master: bool,
    present_on_1_4_dev: bool,
    first_tag: Option<String>,
    tag_count: usize,
    #[serde(skip_serializing)]
    tags_const: String,
    #[serde(skip_serializing)]
    const_name: String,
}

#[derive(Serialize)]
struct FeaturePresenceMap {
    schema_version: u32,
    note: &'static str,
    journeys: Vec<JourneyPresence>,
    version_index: BTreeMap<String, Vec<String>>,
}

fn tag_sort_key(tag: &str) -> (u32, u32, u32, u32, u32, String) {
    let dot_beta = Regex::new(r"\.beta(\d+)$").expect("static regex");
    let dash_beta = Regex::new(r"-beta(\d+)$").expect("static regex");
    let normalized = dot_beta.replace(tag, "-beta.$1");
    let normalized = dash_beta.replace(&normalized, "-beta.$1");

    let full = Regex::new(r"^(\d+)\.(\d+)\.(\d+)(?:-beta\.(\d+))?$").expect("static regex");
    let Some(caps) = full.captures(&normalized) else {
        return (999, 999, 999, 999, 999, tag.to_string());
    };
    let maj: u32 = caps[1].parse().expect("digit group");
    let min: u32 = caps[2].parse().expect("digit group");
    let patch: u32 = caps[3].parse().expect("digit group");
    match caps.get(4) {
        Some(beta) => (
            maj,
            min,
            patch,
            0,
            beta.as_str().parse().expect("digit group"),
            tag.to_string(),
        ),
        None => (maj, min, patch, 1, 0, tag.to_string()),
    }
}

fn camel_to_snake(s: &str) -> String {
    let re = Regex::new(r"([a-z0-9])([A-Z])").expect("static regex");
    re.replace_all(s, "${1}_$2").to_uppercase()
}

fn first_add(root: &std::path::Path, path: &str) -> Option<String> {
    let out = run_ok(
        &[
            "git",
            "log",
            "--diff-filter=A",
            "--follow",
            "--format=%H",
            "--reverse",
            "--",
            path,
        ],
        root,
    )
    .or_else(|| {
        run_ok(
            &[
                "git",
                "log",
                "--follow",
                "--format=%H",
                "--reverse",
                "--",
                path,
            ],
            root,
        )
    });
    out.and_then(|s| s.lines().next().map(str::to_string))
}

fn first_pickaxe(root: &std::path::Path, term: &str, path: &str) -> Option<String> {
    let out = run_ok(
        &[
            "git",
            "log",
            "-S",
            term,
            "--format=%H",
            "--reverse",
            "--",
            path,
        ],
        root,
    );
    out.and_then(|s| s.lines().next().map(str::to_string))
}

fn resolve_commit(
    root: &std::path::Path,
    jid: &str,
    module: &str,
) -> (Option<String>, Option<String>) {
    if let Some((_, ov)) = OVERRIDES.iter().find(|(k, _)| *k == jid) {
        return match ov {
            Override::Path(path) => (first_add(root, path), Some(format!("path:{path}"))),
            Override::Pickaxe(term, path) => {
                (first_pickaxe(root, term, path), Some(format!("S:{term}")))
            }
        };
    }
    if let Some((_, term, path)) = MODULE_S.iter().find(|(m, _, _)| *m == module) {
        return (first_pickaxe(root, term, path), Some(format!("S:{term}")));
    }
    if let Some((_, path)) = MODULE_DEFAULT_PATH.iter().find(|(m, _)| *m == module) {
        return (first_add(root, path), Some(format!("path:{path}")));
    }
    (None, None)
}

fn source_path_for(jid: &str, module: &str) -> Option<&'static str> {
    if let Some((_, ov)) = OVERRIDES.iter().find(|(k, _)| *k == jid) {
        return match ov {
            Override::Path(path) => Some(*path),
            Override::Pickaxe(_, path) => Some(*path),
        };
    }
    if let Some((_, _, path)) = MODULE_S.iter().find(|(m, _, _)| *m == module) {
        return Some(*path);
    }
    MODULE_DEFAULT_PATH
        .iter()
        .find(|(m, _)| *m == module)
        .map(|(_, path)| *path)
}

fn path_exists_on(root: &std::path::Path, git_ref: &str, path: &str) -> bool {
    run_git(
        &["git", "cat-file", "-e", &format!("{git_ref}:{path}")],
        root,
    )
    .is_ok()
}

fn first_existing_ref(root: &std::path::Path, names: &[&str]) -> String {
    for name in names {
        if run_git(&["git", "rev-parse", "--verify", name], root).is_ok() {
            return (*name).to_string();
        }
    }
    names[0].to_string()
}

/// Parse `git ls-remote --tags` stdout into (tag, commit sha). Prefers peeled `^{}` commits.
pub(crate) fn parse_ls_remote_tags(stdout: &str) -> Vec<(String, String)> {
    let version = Regex::new(r"^\d+\.\d+").expect("static regex");
    let mut commits: BTreeMap<String, String> = BTreeMap::new();
    let mut peeled: BTreeMap<String, String> = BTreeMap::new();
    for line in stdout.lines() {
        let Some((sha, rest)) = line.split_once('\t') else {
            continue;
        };
        let Some(name) = rest.strip_prefix("refs/tags/") else {
            continue;
        };
        if let Some(tag) = name.strip_suffix("^{}") {
            if version.is_match(tag) {
                peeled.insert(tag.to_string(), sha.to_string());
            }
            continue;
        }
        if version.is_match(name) {
            commits.insert(name.to_string(), sha.to_string());
        }
    }
    for (tag, sha) in peeled {
        commits.insert(tag, sha);
    }
    let mut tags: Vec<(String, String)> = commits.into_iter().collect();
    tags.sort_by_key(|a| tag_sort_key(&a.0));
    tags
}

fn origin_version_tags(root: &std::path::Path) -> Result<Vec<(String, String)>, String> {
    let out = run_ok(&["git", "ls-remote", "--tags", "origin"], root).ok_or_else(|| {
        "git ls-remote --tags origin failed (need the bluerobotics origin remote)".to_string()
    })?;
    let tags = parse_ls_remote_tags(&out);
    if tags.is_empty() {
        return Err("git ls-remote --tags origin returned no version tags".into());
    }
    Ok(tags)
}

fn commit_in_ref(root: &std::path::Path, commit: &str, git_ref: &str) -> bool {
    run_git(
        &["git", "merge-base", "--is-ancestor", commit, git_ref],
        root,
    )
    .is_ok()
}

pub fn run() -> Result<(), String> {
    let root = repo_root();
    let out_rs = root.join("catalog/src/journey_presence.rs");
    let out_json = root.join("catalog/feature_presence_map.json");

    let _ = run_git(
        &[
            "git",
            "fetch",
            "--no-tags",
            "origin",
            "refs/tags/*:refs/catalog-presence/*",
            "refs/heads/1.4-dev:refs/catalog-presence-heads/1.4-dev",
            "refs/heads/master:refs/catalog-presence-heads/master",
        ],
        &root,
    );
    let origin_tags = origin_version_tags(&root)?;
    let master_tip = first_existing_ref(
        &root,
        &[
            "refs/catalog-presence-heads/master",
            "origin/master",
            "master",
        ],
    );
    let dev_14_tip = first_existing_ref(
        &root,
        &[
            "refs/catalog-presence-heads/1.4-dev",
            "origin/1.4-dev",
            "1.4-dev",
        ],
    );

    let mut journey_files: Vec<std::path::PathBuf> =
        std::fs::read_dir(root.join("catalog/src/journeys"))
            .map_err(|err| format!("failed to read journeys dir: {err}"))?
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|p| p.extension().is_some_and(|ext| ext == "rs"))
            .collect();
    journey_files.sort();

    let id_re = Regex::new(r"id:\s*JourneyId::(\w+)").expect("static regex");
    let mut journeys = Vec::new();
    for file in &journey_files {
        let text = std::fs::read_to_string(file)
            .map_err(|err| format!("failed to read {file:?}: {err}"))?;
        let module = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        for caps in id_re.captures_iter(&text) {
            let jid = caps[1].to_string();
            let (commit, method) = resolve_commit(&root, &jid, &module);
            let Some(commit) = commit else {
                return Err(format!("no commit for {jid}"));
            };
            let method = method.expect("method set alongside commit");
            let path = source_path_for(&jid, &module);
            let mut tags: Vec<String> = origin_tags
                .iter()
                .filter(|(_, sha)| {
                    commit_in_ref(&root, &commit, sha)
                        || path.is_some_and(|p| path_exists_on(&root, sha, p))
                })
                .map(|(name, _)| name.clone())
                .collect();
            tags.sort_by_key(|t| tag_sort_key(t));
            tags.dedup();
            let present_on_master = commit_in_ref(&root, &commit, &master_tip)
                || commit_in_ref(&root, &commit, "HEAD")
                || path.is_some_and(|p| path_exists_on(&root, &master_tip, p));
            let present_on_1_4_dev = commit_in_ref(&root, &commit, &dev_14_tip)
                || path.is_some_and(|p| path_exists_on(&root, &dev_14_tip, p));
            journeys.push(JourneyPresence {
                journey: jid,
                module: module.clone(),
                intro_commit: commit.clone(),
                intro_commit_short: commit.chars().take(12).collect(),
                method,
                present_on_master,
                present_on_1_4_dev,
                first_tag: tags.first().cloned(),
                tag_count: tags.len(),
                present_in_tags: tags,
                tags_const: String::new(),
                const_name: String::new(),
            });
        }
    }

    let mut tag_list_ids: BTreeMap<String, String> = BTreeMap::new();
    let mut shared: Vec<(String, Vec<String>)> = Vec::new();
    for journey in &mut journeys {
        let mut hasher = Sha1::new();
        hasher.update(journey.present_in_tags.join("\n").as_bytes());
        let digest = hasher.finalize();
        let fp: String = digest
            .iter()
            .take(5)
            .map(|byte| format!("{byte:02X}"))
            .collect();
        let name = tag_list_ids.entry(fp.clone()).or_insert_with(|| {
            let name = format!("TAGS_{fp}");
            shared.push((name.clone(), journey.present_in_tags.clone()));
            name
        });
        journey.tags_const = name.clone();
        journey.const_name = format!("PRESENCE_{}", camel_to_snake(&journey.journey));
    }

    let mut version_index: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for journey in &journeys {
        for tag in &journey.present_in_tags {
            version_index
                .entry(tag.clone())
                .or_default()
                .push(journey.journey.clone());
        }
        if journey.present_on_master {
            version_index
                .entry("master".to_string())
                .or_default()
                .push(journey.journey.clone());
        }
        if journey.present_on_1_4_dev {
            version_index
                .entry("1.4-dev".to_string())
                .or_default()
                .push(journey.journey.clone());
        }
    }
    for journeys in version_index.values_mut() {
        journeys.sort();
        journeys.dedup();
    }

    journeys.sort_by(|a, b| a.journey.cmp(&b.journey));

    let map = FeaturePresenceMap {
        schema_version: 1,
        note: "git tag --contains intro_commit; channel tips via merge-base --is-ancestor",
        journeys,
        version_index,
    };
    let json = serde_json::to_string_pretty(&map)
        .map_err(|err| format!("failed to serialize json: {err}"))?;
    let json_len = json.len();
    std::fs::write(&out_json, json + "\n")
        .map_err(|err| format!("failed to write {out_json:?}: {err}"))?;

    let journeys = map.journeys;
    let mut lines = vec![
        "//! Auto-generated journey feature presence (git tag membership).".to_string(),
        "//!".to_string(),
        "//! Regenerated by `cargo run -p blueos-catalog --bin generate_feature_presence`."
            .to_string(),
        "//! Regenerated from origin tag SHAs (`git ls-remote --tags origin`) plus path"
            .to_string(),
        "//! existence on `origin/1.4-dev` / `1.4-dev`. Local-only tags are ignored.".to_string(),
        "//!".to_string(),
        "//! Each journey records:".to_string(),
        "//! - `intro_commit`: landing commit on master (features land on master first)"
            .to_string(),
        "//! - `present_in_tags`: origin version tags whose SHA contains the intro commit"
            .to_string(),
        "//!   or still has the tracked path (backports / cherry-picks).".to_string(),
        "//! - `present_on_master` / `present_on_1_4_dev`: floating channel tips".to_string(),
        String::new(),
        "use crate::version::FeatureAvailability;".to_string(),
        String::new(),
        format!(
            "// {} shared tag-list constants, {} journeys",
            shared.len(),
            journeys.len()
        ),
        String::new(),
    ];
    for (name, tags) in &shared {
        lines.push(format!("const {name}: &[&str] = &["));
        for tag in tags {
            lines.push(format!("    \"{tag}\","));
        }
        lines.push("];".to_string());
        lines.push(String::new());
    }

    lines.push("// --- per-journey presence ---".to_string());
    lines.push(String::new());
    for journey in &journeys {
        lines.push(format!(
            "/// {}: intro `{}` via {}; {} tags; master={}; 1.4-dev={}",
            journey.journey,
            journey.intro_commit_short,
            journey.method,
            journey.tag_count,
            journey.present_on_master,
            journey.present_on_1_4_dev
        ));
        lines.push(format!(
            "pub const {}: FeatureAvailability = FeatureAvailability {{",
            journey.const_name
        ));
        lines.push(format!("    intro_commit: \"{}\",", journey.intro_commit));
        lines.push(format!("    present_in_tags: {},", journey.tags_const));
        lines.push(format!(
            "    present_on_master: {},",
            journey.present_on_master
        ));
        lines.push(format!(
            "    present_on_1_4_dev: {},",
            journey.present_on_1_4_dev
        ));
        lines.push("};".to_string());
        lines.push(String::new());
    }

    lines.push("/// All journey presence records (for version→feature maps).".to_string());
    lines.push("pub const ALL_JOURNEY_PRESENCE: &[(&str, FeatureAvailability)] = &[".to_string());
    for journey in &journeys {
        lines.push(format!(
            "    (\"{}\", {}),",
            journey.journey, journey.const_name
        ));
    }
    lines.push("];".to_string());
    lines.push(String::new());

    let rs = lines.join("\n") + "\n";
    let rs_len = rs.len();
    std::fs::write(&out_rs, rs).map_err(|err| format!("failed to write {out_rs:?}: {err}"))?;

    println!(
        "Wrote {} ({rs_len} bytes)",
        out_rs.strip_prefix(&root).unwrap_or(&out_rs).display()
    );
    println!(
        "Wrote {} ({json_len} bytes)",
        out_json.strip_prefix(&root).unwrap_or(&out_json).display()
    );
    println!(
        "journeys={} shared_tag_lists={}",
        journeys.len(),
        shared.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ls_remote_tags_prefers_peeled_commit() {
        let stdout = "\
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\trefs/tags/1.4.4-beta.10
bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\trefs/tags/1.4.4-beta.15
cccccccccccccccccccccccccccccccccccccccc\trefs/tags/1.4.4-beta.15^{}
dddddddddddddddddddddddddddddddddddddddd\trefs/tags/not-a-version
";
        let tags = parse_ls_remote_tags(stdout);
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].0, "1.4.4-beta.10");
        assert_eq!(tags[1].0, "1.4.4-beta.15");
        assert_eq!(tags[1].1, "cccccccccccccccccccccccccccccccccccccccc");
        assert!(!tags.iter().any(|(name, _)| name.contains("beta.100")));
    }
}
