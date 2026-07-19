use std::path::{Path, PathBuf};
use std::process::Command;

use crate::catalog::Catalog;
use crate::id::{JourneyId, ServiceId};
use crate::journey::{derive_automatable, Actor, Automatable, HttpMethod, RouteRef, UserJourney};
use crate::provenance::{Grounded, GroundedSet, ObservedSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepResult {
    Pass,
    Fail(String),
    Skip(String),
    Unasserted,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JourneyResult {
    Pass,
    Fail,
    Skip,
    Partial,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFilePart {
    pub field: &'static str,
    pub fixture: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmokeHttpCall {
    pub route: RouteRef,
    pub expected_status: u16,
    pub body: Option<&'static str>,
    pub query: Option<&'static str>,
    pub form_file: Option<FormFilePart>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunnableStep {
    pub journey_id: JourneyId,
    pub step_index: usize,
    pub route: RouteRef,
    pub expected_status: Option<u16>,
    pub body_predicate: Option<&'static str>,
    pub body: Option<&'static str>,
    pub query: Option<&'static str>,
    pub form_file: Option<FormFilePart>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RunCounts {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub unasserted: usize,
}

impl RunCounts {
    pub fn record(&mut self, result: &StepResult) {
        match result {
            StepResult::Pass => self.passed += 1,
            StepResult::Fail(_) => self.failed += 1,
            StepResult::Skip(_) => self.skipped += 1,
            StepResult::Unasserted => self.unasserted += 1,
            StepResult::Ignored => self.skipped += 1,
        }
    }
}

pub fn http_journeys(catalog: &Catalog) -> Vec<&UserJourney> {
    catalog
        .journeys()
        .iter()
        .filter(|journey| derive_automatable(journey) == Automatable::Http)
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tier1GetCoverage {
    pub http_journeys: usize,
    pub get_concrete: usize,
    pub get_asserted: usize,
    pub get_unasserted: usize,
    pub get_templated: usize,
    pub unasserted: Vec<(JourneyId, usize, &'static str)>,
}

fn is_templated_http_path(path: &str) -> bool {
    path.contains('{') || path.contains('*')
}

pub fn is_smoke_excluded_get(path: &str) -> bool {
    matches!(
        path,
        "/disk/speed/stream" | "/internet_download_speed" | "/internet_upload_speed"
    )
}

pub fn tier1_get_coverage(catalog: &Catalog) -> Tier1GetCoverage {
    let journeys = http_journeys(catalog);
    let mut coverage = Tier1GetCoverage {
        http_journeys: journeys.len(),
        get_concrete: 0,
        get_asserted: 0,
        get_unasserted: 0,
        get_templated: 0,
        unasserted: Vec::new(),
    };

    for journey in journeys {
        for step in http_steps(journey) {
            if !matches!(step.route.method, HttpMethod::Get) {
                continue;
            }
            if is_templated_http_path(step.route.path) {
                coverage.get_templated += 1;
                continue;
            }
            coverage.get_concrete += 1;
            if step.expected_status.is_some() {
                coverage.get_asserted += 1;
            } else {
                coverage.get_unasserted += 1;
                coverage
                    .unasserted
                    .push((step.journey_id, step.step_index, step.route.path));
            }
        }
    }

    coverage
}

pub const SMOKE_DEFAULT_FIXTURES: &str = "internet,pirate,advanced";

pub const MUTATING_SMOKE_DEFAULT_FIXTURES: &str = "internet,pirate,advanced,confirm-dangerous,board:any,wifi-radio,hotspot,nmea-socket,serial-bridge,dhcp-active,extension-installed,recording,local-version,wifi-saved";

pub fn http_method_label(method: &HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Patch => "PATCH",
    }
}

pub fn format_http_fail(
    journey_id: JourneyId,
    step: &RunnableStep,
    resolved_url: &str,
    reason: &str,
) -> String {
    format!(
        "FAIL {journey_id} step {} {} {} → {resolved_url} — {reason}",
        step.step_index,
        http_method_label(&step.route.method),
        step.route.path
    )
}

pub fn format_dry_run(journey_id: JourneyId, step: &RunnableStep, resolved_path: &str) -> String {
    format!(
        "DRY-RUN {journey_id} step {} {} {} → {resolved_path}",
        step.step_index,
        http_method_label(&step.route.method),
        step.route.path
    )
}

pub fn journey_http_mode_conflict(smoke: bool, mutating_smoke: bool) -> Result<(), &'static str> {
    if smoke && mutating_smoke {
        return Err("--smoke and --mutating-smoke are mutually exclusive");
    }
    Ok(())
}

pub fn journey_http_requires_base(
    dry_run: bool,
    smoke: bool,
    mutating_smoke: bool,
    base: Option<&str>,
) -> Result<(), &'static str> {
    if (smoke || mutating_smoke) && base.is_none() {
        return Err("--base is required for --smoke and --mutating-smoke");
    }
    if !dry_run && base.is_none() {
        return Err("--base is required unless --dry-run is set");
    }
    Ok(())
}

pub fn http_steps(journey: &UserJourney) -> Vec<RunnableStep> {
    let GroundedSet::Known { items: steps } = &journey.steps else {
        return Vec::new();
    };

    let mut runnable = Vec::new();
    for (step_index, step) in steps.iter().enumerate() {
        if matches!(step.value.actor, Actor::Frontend(_)) {
            continue;
        }
        let Some(route) = &step.value.route else {
            continue;
        };
        let Grounded::Known { value: route, .. } = route else {
            continue;
        };
        let (expected_status, body_predicate) = match &step.value.outcome {
            None => (None, None),
            Some(Grounded::Unknown { .. }) => (None, None),
            Some(Grounded::Known { value: outcome, .. }) => {
                (outcome.expected_status, outcome.body_predicate)
            }
        };
        runnable.push(RunnableStep {
            journey_id: journey.id,
            step_index,
            route: route.clone(),
            expected_status,
            body_predicate,
            body: None,
            query: None,
            form_file: None,
        });
    }
    runnable
}

pub fn fixture_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(relative)
}

pub fn http_smoke_steps(journey: &UserJourney) -> Vec<RunnableStep> {
    http_steps(journey)
        .into_iter()
        .filter(|step| {
            matches!(step.route.method, HttpMethod::Get)
                && step.expected_status.is_some()
                && !is_smoke_excluded_get(step.route.path)
        })
        .collect()
}

pub fn mutating_smoke_query(journey_id: JourneyId, route: &RouteRef) -> Option<&'static str> {
    use HttpMethod::*;
    match (journey_id, route.path, &route.method) {
        (JourneyId::RenameVehicle, "/vehicle_name", Post) => Some("name=smoke-catalog"),
        (JourneyId::ChangeMdnsHostname, "/hostname", Post) => Some("hostname=smoke-catalog"),
        (JourneyId::AcquireDynamicIpAddress, "/dynamic_ip", Post) => Some("interface_name=eth0"),
        (JourneyId::AssignStaticIpAddress, "/address", Post) => {
            Some("interface_name=eth0&ip_address=192.168.0.178")
        }
        (JourneyId::DisableOnboardDhcpServer, "/dhcp", Delete) => Some("interface_name=eth0"),
        (JourneyId::EnableOnboardDhcpServer, "/dhcp", Post) => {
            Some("interface_name=eth0&ipv4_gateway=192.168.0.1&is_backup_server=false")
        }
        (JourneyId::ForgetSavedWifiNetwork, "/remove", Post) => Some("ssid=__smoke_catalog__"),
        (JourneyId::ToggleHotspot, "/hotspot", Post) => Some("enable=false"),
        (JourneyId::ToggleSmartHotspot, "/smart_hotspot", Post) => Some("enable=false"),
        (JourneyId::RebootOnboardComputer, "/shutdown", Post) => {
            Some("shutdown_type=reboot&i_know_what_i_am_doing=true")
        }
        (JourneyId::SyncSystemTime, "/set_time", Post) => {
            Some("unix_time_seconds=1700000000&i_know_what_i_am_doing=true")
        }
        (JourneyId::EnableLegacyCameraSupport, "/raspi_config/camera_legacy", Post) => {
            Some("enable=false")
        }
        (JourneyId::UpdateRaspberryEepromBootloader, "/raspi/eeprom_update", Post) => {
            Some("i_know_what_i_am_doing=true")
        }
        (JourneyId::ResetBlueosSettings, "/settings/reset", Post) => {
            Some("i_know_what_i_am_doing=true")
        }
        (JourneyId::RunHostCommand, "/command/host", Post) => {
            Some("command=true&i_know_what_i_am_doing=true")
        }
        (JourneyId::ChangeBoard, "/board", Post) | (JourneyId::RunSitlSimulation, "/board", Post) => {
            None
        }
        (JourneyId::RunSitlSimulation, "/sitl_frame", Post) => Some("frame=vectored"),
        (JourneyId::VehicleFirstBoot, "/install_firmware_from_url", Post)
        | (JourneyId::UpdateFirmwareOnline, "/install_firmware_from_url", Post) => Some(
            "url=https://firmware.ardupilot.org/Sub/stable-4.5.7/navigator/ardusub&board_name=Navigator",
        ),
        (JourneyId::RestoreDefaultFirmware, "/restore_default_firmware", Post) => {
            Some("board_name=Navigator")
        }
        (JourneyId::RemoveCameraStream, "/delete_stream", Delete) => Some("name=__smoke_catalog__"),
        (JourneyId::Upload3dModelOverride, "/models", Post) => Some("name=smoke-catalog.glb"),
        (JourneyId::AddCustomManifest, "/manifest/", Post) => Some("validate_url=false"),
        _ => None,
    }
}

pub fn mutating_smoke_body(journey_id: JourneyId, route: &RouteRef) -> Option<&'static str> {
    use HttpMethod::*;
    match (journey_id, route.path, &route.method) {
        (JourneyId::ChangeUiThemeColor, "/theme", Put) => Some(r##"{"primary":"#1e88e5"}"##),
        (JourneyId::ModifyBagDatabase, "/overwrite", Post) => Some("{}"),
        (JourneyId::ConfigureHostDns, "/host_dns", Post) => {
            Some(r##"{"nameservers":["8.8.8.8"],"lock":false}"##)
        }
        (JourneyId::SetNetworkInterfacePriority, "/set_interfaces_priority", Post) => {
            Some(r##"[{"name":"eth0","priority":100}]"##)
        }
        (JourneyId::ChangeBoard, "/board", Post) => Some(NAVIGATOR_BOARD_JSON),
        (JourneyId::RunSitlSimulation, "/board", Post) => Some(
            r##"{"name":"SITL","manufacturer":"ArduPilot Team","platform":"SITL_arm_linux_gnueabihf","path":null,"flags":[]}"##,
        ),
        (JourneyId::ConnectToWifiNetwork, "/connect", Post) => {
            Some(r##"{"ssid":"__smoke_nonexistent__","password":"invalidpass"}"##)
        }
        (JourneyId::ConfigureHotspotCredentials, "/hotspot_credentials", Post) => {
            Some(r##"{"ssid":"BlueOS","password":"smoke-test"}"##)
        }
        (JourneyId::RemoveConfiguredNmeaSocket, "/socks", Delete) => Some(SMOKE_NMEA_SOCK_JSON),
        (JourneyId::RemoveSerialBridge, "/bridges", Delete) => Some(SMOKE_BRIDGE_JSON),
        (JourneyId::DockerRegistryLogin, "/docker/login", Post) => {
            Some(r##"{"username":"","password":"","registry":"","root":true}"##)
        }
        (JourneyId::UpdateBlueosVersion, "/version/pull", Post)
        | (JourneyId::PullBlueosVersionWithoutSwitch, "/version/pull", Post) => {
            Some(r##"{"repository":"bluerobotics/blueos-core","tag":"master"}"##)
        }
        (JourneyId::UpdateBootstrapImage, "/version/pull", Post) => {
            Some(r##"{"repository":"bluerobotics/blueos-bootstrap","tag":"master"}"##)
        }
        (JourneyId::UpdateBlueosVersion, "/version/current", Post)
        | (JourneyId::SwitchLocalBlueosVersion, "/version/current", Post) => {
            Some(r##"{"repository":"bluerobotics/blueos-core","tag":"master"}"##)
        }
        (JourneyId::DeleteLocalBlueosVersion, "/version/delete", Delete) => {
            Some(r##"{"repository":"bluerobotics/blueos-core","tag":"smoke-catalog-deleteme"}"##)
        }
        (JourneyId::UpdateBootstrapImage, "/bootstrap/current", Post) => {
            Some(r##"{"tag":"master"}"##)
        }
        (JourneyId::AddCustomManifest, "/manifest/", Post) => Some(
            r##"{"name":"smoke-catalog","url":"https://example.com/smoke-catalog.json","enabled":false}"##,
        ),
        (JourneyId::InstallCustomExtension, "/extension/", Post) => Some(SMOKE_CUSTOM_EXT_JSON),
        _ => None,
    }
}

pub fn mutating_smoke_form_file(journey_id: JourneyId, route: &RouteRef) -> Option<FormFilePart> {
    use HttpMethod::*;
    match (journey_id, route.path, &route.method) {
        (JourneyId::UploadCustomLogo, "/branding/logo", Post) => Some(FormFilePart {
            field: "file",
            fixture: "smoke-logo.png",
        }),
        (JourneyId::UploadCustomVehicleImage, "/branding/vehicle-image", Post) => {
            Some(FormFilePart {
                field: "file",
                fixture: "smoke-vehicle.png",
            })
        }
        (JourneyId::Upload3dModelOverride, "/models", Post) => Some(FormFilePart {
            field: "file",
            fixture: "smoke-model.glb",
        }),
        (JourneyId::UploadCustomFirmware, "/install_firmware_from_file", Post) => {
            Some(FormFilePart {
                field: "binary",
                fixture: "smoke-firmware.bin",
            })
        }
        _ => None,
    }
}

pub fn mutating_smoke_expected_status(journey_id: JourneyId, route: &RouteRef) -> Option<u16> {
    use HttpMethod::*;
    match (journey_id, route.path, &route.method) {
        (JourneyId::ConnectToWifiNetwork, "/connect", Post) => Some(500),
        _ => None,
    }
}

pub fn is_mutating_smoke_step_deferred(journey_id: JourneyId, path: &str) -> Option<&'static str> {
    match (journey_id, path) {
        (JourneyId::UpdateBlueosVersion, "/version/current") => {
            Some("deferred: POST /version/current switches running core image")
        }
        (JourneyId::UpdateBootstrapImage, "/bootstrap/current") => {
            Some("deferred: POST /bootstrap/current replaces bootstrap container")
        }
        (JourneyId::SwitchLocalBlueosVersion, "/version/current") => {
            Some("deferred: switching local core image restarts blueos-core mid-suite")
        }
        (JourneyId::ChangeMdnsHostname, "/hostname") => {
            Some("deferred: beacon hostname mutation returns 500 on current test bed")
        }
        _ => None,
    }
}

pub fn mutating_smoke_skip_reason(
    journey_id: JourneyId,
    _fixtures: &crate::fixture::FixtureInventory,
) -> Option<&'static str> {
    match journey_id {
        // POST /connect blocks on association; fake SSID hangs past useful smoke budgets.
        JourneyId::ConnectToWifiNetwork => {
            Some("connect association blocks; needs real SSID or async API")
        }
        _ => None,
    }
}

const NAVIGATOR_BOARD_JSON: &str = r##"{"name":"Navigator","manufacturer":"Blue Robotics","platform":"navigator","path":null,"flags":[]}"##;
const SMOKE_STREAM_JSON: &str = r##"{"name":"__smoke_catalog__","source":"Redirect","stream_information":{"endpoints":["udp://127.0.0.1:5599"],"configuration":{"type":"redirect"},"extended_configuration":{"thermal":false,"disable_lazy":false,"disable_mavlink":false,"disable_thumbnails":true,"disable_zenoh":true}}}"##;
const SMOKE_NMEA_SOCK_JSON: &str = r##"{"kind":"UDP","port":9999,"component_id":220}"##;
const SMOKE_BRIDGE_JSON: &str = r##"{"serial_path":"/dev/ttyAMA3","baud":115200,"ip":"127.0.0.1","udp_target_port":14559,"udp_listen_port":14558}"##;
const SMOKE_HOST_DNS_JSON: &str = r##"{"nameservers":["8.8.8.8","1.1.1.1"],"lock":true}"##;
const SMOKE_MODEL_PATH: &str = "/models/smoke-catalog.glb";
const SMOKE_EXT_INSTALL_PATH: &str = "/extension/williangalvani.example1/v1.0.1/install";
const SMOKE_EXT_UNINSTALL_PATH: &str = "/extension/williangalvani.example1/v1.0.1";
const SMOKE_EXT_EDIT_PATH: &str = "/extension/williangalvani.example1/v1.0.0";
const SMOKE_EXT_UNINSTALL_ALT_PATH: &str = "/extension/williangalvani.example1/v1.0.0";
/// Running extension used for restart/disable (example1 container is flaky right after install).
const SMOKE_EXT_LIFECYCLE_RESTART_PATH: &str = "/extension/blueos.major_tom/restart";
const SMOKE_EXT_LIFECYCLE_DISABLE_PATH: &str = "/extension/blueos.major_tom/disable";
const SMOKE_EXT_LIFECYCLE_ENABLE_PATH: &str = "/extension/blueos.major_tom/2026-02-11/enable";
const SMOKE_CUSTOM_EXT_JSON: &str = r##"{"identifier":"smoke.catalog","tag":"latest","name":"Smoke Catalog","docker":"alpine","enabled":false,"permissions":"{}"}"##;
const SMOKE_CUSTOM_EXT_UNINSTALL_PATH: &str = "/extension/smoke.catalog/latest";
const SMOKE_DISK_DELETE_PATH: &str =
    "/disk/paths/usr%2Fblueos%2Fuserdata%2Fsmoke-catalog%2Fdelete-me.txt";
const SMOKE_RECORDING_DELETE_PATH: &str = "/recorder/files/smoke_catalog%2Fsmoke-catalog.mp4";
const SMOKE_DISK_SEED_QUERY: &str = "command=docker%20exec%20blueos-core%20sh%20-c%20%27mkdir%20-p%20/usr/blueos/userdata/smoke-catalog%20%26%26%20echo%20x%3E/usr/blueos/userdata/smoke-catalog/delete-me.txt%27&i_know_what_i_am_doing=true";
const SMOKE_RECORDING_SEED_QUERY: &str = "command=docker%20exec%20blueos-core%20sh%20-c%20%27mkdir%20-p%20/usr/blueos/userdata/recorder/smoke_catalog%20%26%26%20printf%20mp4%3E/usr/blueos/userdata/recorder/smoke_catalog/smoke-catalog.mp4%27&i_know_what_i_am_doing=true";
const SMOKE_WIFI_SAVE_QUERY: &str = "command=bash%20-lc%20%27id%3D%24%28sudo%20wpa_cli%20-i%20wlan0%20add_network%29%20%26%26%20sudo%20wpa_cli%20-i%20wlan0%20set_network%20%22%24id%22%20ssid%20%22%5C%22__smoke_catalog__%5C%22%22%20%26%26%20sudo%20wpa_cli%20-i%20wlan0%20set_network%20%22%24id%22%20key_mgmt%20NONE%20%26%26%20sudo%20wpa_cli%20-i%20wlan0%20disable_network%20%22%24id%22%20%26%26%20sudo%20wpa_cli%20-i%20wlan0%20save_config%27&i_know_what_i_am_doing=true";
const SMOKE_ROUTE_FLUSH_QUERY: &str =
    "command=ip%20route%20flush%20proto%20static&i_know_what_i_am_doing=true";
const SMOKE_ADDR_DEL_1_QUERY: &str = "command=bash%20-lc%20%27curl%20-s%20-m%2010%20-o%20%2Fdev%2Fnull%20-X%20DELETE%20%22http%3A%2F%2F127.0.0.1%2Fcable-guy%2Fv1.0%2Faddress%3Finterface_name%3Deth0%26ip_address%3D192.168.0.1%22%20%7C%7C%20true%3B%20ip%20addr%20del%20192.168.0.1%2F24%20dev%20eth0%202%3E%2Fdev%2Fnull%20%7C%7C%20true%27&i_know_what_i_am_doing=true";
const SMOKE_ADDR_DEL_178_QUERY: &str = "command=bash%20-lc%20%27curl%20-s%20-m%2010%20-o%20%2Fdev%2Fnull%20-X%20DELETE%20%22http%3A%2F%2F127.0.0.1%2Fcable-guy%2Fv1.0%2Faddress%3Finterface_name%3Deth0%26ip_address%3D192.168.0.178%22%20%7C%7C%20true%3B%20ip%20addr%20del%20192.168.0.178%2F24%20dev%20eth0%202%3E%2Fdev%2Fnull%20%7C%7C%20true%27&i_know_what_i_am_doing=true";
const SMOKE_RESOLV_FIX_QUERY: &str = "command=docker%20exec%20blueos-core%20sh%20-c%20%27printf%20%22nameserver%208.8.8.8%5Cnnameserver%201.1.1.1%5Cn%22%20%3E%20%2Fetc%2Fresolv.conf.host%27&i_know_what_i_am_doing=true";
const SMOKE_LOCAL_VERSION_TAG_QUERY: &str = "command=docker%20tag%20bluerobotics%2Fblueos-core%3Amaster%20bluerobotics%2Fblueos-core%3Asmoke-catalog-deleteme&i_know_what_i_am_doing=true";
const SMOKE_KRAKEN_MANIFEST_CLEAN_QUERY: &str = "command=docker%20exec%20blueos-core%20python3%20-c%20%22import%20json%2Cpathlib%3Bp%3Dpathlib.Path%28%27%2Froot%2F.config%2Fkraken%2Fsettings-2.json%27%29%3Bd%3Djson.loads%28p.read_text%28%29%29%3Bd%5B%27manifests%27%5D%3D%5Bm%20for%20m%20in%20d.get%28%27manifests%27%2C%5B%5D%29%20if%20m.get%28%27name%27%29%21%3D%27smoke-catalog%27%5D%3Bp.write_text%28json.dumps%28d%2Cindent%3D4%29%2Bchr%2810%29%29%22&i_know_what_i_am_doing=true";
const SMOKE_CABLE_GUY_SETTINGS_CLEAN_QUERY: &str = "command=docker%20exec%20blueos-core%20python3%20-c%20%22import%20json%2Cpathlib%3Bp%3Dpathlib.Path%28%27%2Froot%2F.config%2Fcable-guy%2Fsettings-2.json%27%29%3Bd%3Djson.loads%28p.read_text%28%29%29%3B%5Biface.update%28%7B%27addresses%27%3A%5B%7B%27ip%27%3A%270.0.0.0%27%2C%27mode%27%3A%27client%27%7D%5D%2C%27routes%27%3A%5Br%20for%20r%20in%20%28iface.get%28%27routes%27%29%20or%20%5B%5D%29%20if%20r.get%28%27managed%27%29%20and%20str%28r.get%28%27destination%27%2C%27%27%29%29.startswith%28%27224.%27%29%5D%7D%29%20for%20iface%20in%20d.get%28%27content%27%2C%5B%5D%29%20if%20iface.get%28%27name%27%29%3D%3D%27eth0%27%5D%3Bp.write_text%28json.dumps%28d%2Cindent%3D4%29%2Bchr%2810%29%29%22&i_know_what_i_am_doing=true";

/// Bind templated mutating paths to concrete smoke values (Tier-2 only).
pub fn mutating_smoke_path_bind(journey_id: JourneyId, path: &str) -> Option<&'static str> {
    match (journey_id, path) {
        (JourneyId::Delete3dModelOverride, "/models/{name}") => Some(SMOKE_MODEL_PATH),
        (JourneyId::InstallExtension, "/extension/{identifier}/{tag}/install") => {
            Some(SMOKE_EXT_INSTALL_PATH)
        }
        (JourneyId::UninstallExtension, "/extension/{identifier}/{tag}") => {
            Some(SMOKE_EXT_UNINSTALL_PATH)
        }
        (JourneyId::ConfigureInstalledExtension, "/extension/{identifier}/restart") => {
            Some(SMOKE_EXT_LIFECYCLE_RESTART_PATH)
        }
        (JourneyId::ConfigureInstalledExtension, "/extension/{identifier}/disable") => {
            Some(SMOKE_EXT_LIFECYCLE_DISABLE_PATH)
        }
        (JourneyId::EditExtensionDevVersion, "/extension/{identifier}/{tag}") => {
            Some(SMOKE_EXT_EDIT_PATH)
        }
        (JourneyId::FreeDiskSpace, "/disk/paths/{target_path}") => Some(SMOKE_DISK_DELETE_PATH),
        (JourneyId::DeleteVideoRecording, "/recorder/files/{filename}") => {
            Some(SMOKE_RECORDING_DELETE_PATH)
        }
        _ => None,
    }
}

pub fn mutating_smoke_setup_calls(journey_id: JourneyId) -> &'static [SmokeHttpCall] {
    use HttpMethod::*;
    match journey_id {
        JourneyId::ChangeBoard | JourneyId::RunSitlSimulation => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::ArdupilotManager,
                method: Post,
                path: "/stop",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: None,
            form_file: None,
        }],
        JourneyId::VehicleFirstBoot
        | JourneyId::UpdateFirmwareOnline
        | JourneyId::UploadCustomFirmware
        | JourneyId::RestoreDefaultFirmware => &[
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::CableGuy,
                    method: Post,
                    path: "/host_dns",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: Some(SMOKE_HOST_DNS_JSON),
                query: None,
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some(SMOKE_RESOLV_FIX_QUERY),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::ArdupilotManager,
                    method: Post,
                    path: "/start",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: None,
                form_file: None,
            },
        ],
        JourneyId::UninstallExtension | JourneyId::EditExtensionDevVersion => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Kraken,
                method: Post,
                path: SMOKE_EXT_INSTALL_PATH,
                version: Some("v2.0"),
            },
            expected_status: 200,
            body: None,
            query: None,
            form_file: None,
        }],
        JourneyId::ConfigureInstalledExtension => &[
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Kraken,
                    method: Post,
                    path: SMOKE_EXT_LIFECYCLE_ENABLE_PATH,
                    version: Some("v2.0"),
                },
                expected_status: 204,
                body: None,
                query: None,
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some("command=sleep%208&i_know_what_i_am_doing=true"),
                form_file: None,
            },
        ],
        JourneyId::FreeDiskSpace => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Commander,
                method: Post,
                path: "/command/host",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: Some(SMOKE_DISK_SEED_QUERY),
            form_file: None,
        }],
        JourneyId::DeleteVideoRecording => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Commander,
                method: Post,
                path: "/command/host",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: Some(SMOKE_RECORDING_SEED_QUERY),
            form_file: None,
        }],
        JourneyId::ForgetSavedWifiNetwork => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Commander,
                method: Post,
                path: "/command/host",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: Some(SMOKE_WIFI_SAVE_QUERY),
            form_file: None,
        }],
        JourneyId::DeleteLocalBlueosVersion => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Commander,
                method: Post,
                path: "/command/host",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: Some(SMOKE_LOCAL_VERSION_TAG_QUERY),
            form_file: None,
        }],
        JourneyId::RemoveCameraStream => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::MavlinkCameraManager,
                method: Post,
                path: "/streams",
                version: None,
            },
            expected_status: 200,
            body: Some(SMOKE_STREAM_JSON),
            query: None,
            form_file: None,
        }],
        JourneyId::RemoveConfiguredNmeaSocket => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::NmeaInjector,
                method: Post,
                path: "/socks",
                version: Some("v1.0"),
            },
            expected_status: 201,
            body: Some(SMOKE_NMEA_SOCK_JSON),
            query: None,
            form_file: None,
        }],
        JourneyId::RemoveSerialBridge => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Bridget,
                method: Post,
                path: "/bridges",
                version: Some("v1.0"),
            },
            expected_status: 201,
            body: Some(SMOKE_BRIDGE_JSON),
            query: None,
            form_file: None,
        }],
        JourneyId::Delete3dModelOverride => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Customization,
                method: Post,
                path: "/models",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: Some("name=smoke-catalog.glb"),
            form_file: Some(FormFilePart {
                field: "file",
                fixture: "smoke-model.glb",
            }),
        }],
        JourneyId::DisableOnboardDhcpServer => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::CableGuy,
                method: Post,
                path: "/dhcp",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: Some("interface_name=eth0&ipv4_gateway=192.168.0.1&is_backup_server=false"),
            form_file: None,
        }],
        JourneyId::ResetUiThemeColor => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Customization,
                method: Put,
                path: "/theme",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: Some(r##"{"primary":"#1e88e5"}"##),
            query: None,
            form_file: None,
        }],
        _ => &[],
    }
}

pub fn mutating_smoke_teardown_calls(journey_id: JourneyId) -> &'static [SmokeHttpCall] {
    use HttpMethod::*;
    match journey_id {
        JourneyId::ChangeBoard | JourneyId::RunSitlSimulation => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::ArdupilotManager,
                method: Post,
                path: "/board",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: Some(NAVIGATOR_BOARD_JSON),
            query: None,
            form_file: None,
        }],
        JourneyId::AssignStaticIpAddress => &[
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some(SMOKE_ADDR_DEL_178_QUERY),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some(SMOKE_ROUTE_FLUSH_QUERY),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some(SMOKE_CABLE_GUY_SETTINGS_CLEAN_QUERY),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::CableGuy,
                    method: Post,
                    path: "/host_dns",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: Some(SMOKE_HOST_DNS_JSON),
                query: None,
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some(SMOKE_RESOLV_FIX_QUERY),
                form_file: None,
            },
        ],
        JourneyId::EnableOnboardDhcpServer | JourneyId::DisableOnboardDhcpServer => &[
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::CableGuy,
                    method: Delete,
                    path: "/dhcp",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some("interface_name=eth0"),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some(SMOKE_ADDR_DEL_1_QUERY),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some(SMOKE_ROUTE_FLUSH_QUERY),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some(SMOKE_CABLE_GUY_SETTINGS_CLEAN_QUERY),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::CableGuy,
                    method: Post,
                    path: "/host_dns",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: Some(SMOKE_HOST_DNS_JSON),
                query: None,
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some(SMOKE_RESOLV_FIX_QUERY),
                form_file: None,
            },
        ],
        JourneyId::ConfigureHostDns => &[
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::CableGuy,
                    method: Post,
                    path: "/host_dns",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: Some(SMOKE_HOST_DNS_JSON),
                query: None,
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some(SMOKE_ADDR_DEL_1_QUERY),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some(SMOKE_ROUTE_FLUSH_QUERY),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some(SMOKE_CABLE_GUY_SETTINGS_CLEAN_QUERY),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Commander,
                    method: Post,
                    path: "/command/host",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some(SMOKE_RESOLV_FIX_QUERY),
                form_file: None,
            },
        ],
        JourneyId::AddCustomManifest => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Commander,
                method: Post,
                path: "/command/host",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: Some(SMOKE_KRAKEN_MANIFEST_CLEAN_QUERY),
            form_file: None,
        }],
        JourneyId::ToggleHotspot => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Wifi,
                method: Post,
                path: "/hotspot",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: Some("enable=true"),
            form_file: None,
        }],
        JourneyId::ToggleSmartHotspot => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Wifi,
                method: Post,
                path: "/smart_hotspot",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: Some("enable=true"),
            form_file: None,
        }],
        JourneyId::InstallExtension => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Kraken,
                method: Delete,
                path: SMOKE_EXT_UNINSTALL_PATH,
                version: Some("v2.0"),
            },
            expected_status: 202,
            body: None,
            query: None,
            form_file: None,
        }],
        JourneyId::ConfigureInstalledExtension => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Kraken,
                method: Post,
                path: SMOKE_EXT_LIFECYCLE_ENABLE_PATH,
                version: Some("v2.0"),
            },
            expected_status: 204,
            body: None,
            query: None,
            form_file: None,
        }],
        JourneyId::InstallCustomExtension => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Kraken,
                method: Delete,
                path: SMOKE_CUSTOM_EXT_UNINSTALL_PATH,
                version: Some("v2.0"),
            },
            expected_status: 202,
            body: None,
            query: None,
            form_file: None,
        }],
        JourneyId::EditExtensionDevVersion => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Kraken,
                method: Delete,
                path: SMOKE_EXT_UNINSTALL_ALT_PATH,
                version: Some("v2.0"),
            },
            expected_status: 202,
            body: None,
            query: None,
            form_file: None,
        }],
        _ => &[],
    }
}

pub fn http_mutating_smoke_steps(journey: &UserJourney) -> Vec<RunnableStep> {
    http_steps(journey)
        .into_iter()
        .filter(|step| {
            !matches!(step.route.method, HttpMethod::Get)
                && step.expected_status.is_some()
                && (!step.route.path.contains('{')
                    || mutating_smoke_path_bind(step.journey_id, step.route.path).is_some())
                && is_mutating_smoke_step_deferred(step.journey_id, step.route.path).is_none()
        })
        .map(|mut step| {
            if let Some(bound) = mutating_smoke_path_bind(step.journey_id, step.route.path) {
                step.route.path = bound;
            }
            step.body = mutating_smoke_body(step.journey_id, &step.route);
            step.query = mutating_smoke_query(step.journey_id, &step.route);
            step.form_file = mutating_smoke_form_file(step.journey_id, &step.route);
            if let Some(status) = mutating_smoke_expected_status(step.journey_id, &step.route) {
                step.expected_status = Some(status);
            }
            step
        })
        .collect()
}

pub fn join_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    let base = if base.contains("://") {
        base.to_string()
    } else {
        format!("http://{base}")
    };
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    format!("{base}{path}")
}

/// Poll `/status` until BlueOS has rebooted and recovered.
///
/// Requires at least one failed probe (downtime) before accepting HTTP 204, then
/// waits for a short settle window so nginx backends finish coming up.
pub fn wait_for_blueos(base: &str, timeout_secs: u64) -> Result<(), String> {
    let status_url = join_url(base, "/status");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    let mut saw_downtime = false;
    let mut last_err = String::from("no attempts");

    // Brief grace: reboot may not drop HTTP immediately after POST /shutdown returns.
    let downtime_deadline = std::time::Instant::now() + std::time::Duration::from_secs(45);
    while std::time::Instant::now() < downtime_deadline.min(deadline) {
        match execute_curl(&HttpMethod::Get, &status_url, false, None, None) {
            Ok((204, _)) => {
                last_err = "still HTTP 204 (waiting for reboot downtime)".into();
            }
            Ok((code, _)) => {
                saw_downtime = true;
                last_err = format!("HTTP {code}");
                break;
            }
            Err(err) => {
                saw_downtime = true;
                last_err = err;
                break;
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }

    if !saw_downtime {
        // Soft-reboot / no drop: still require sustained 204 after a settle pause.
        std::thread::sleep(std::time::Duration::from_secs(15));
    }

    while std::time::Instant::now() < deadline {
        match execute_curl(&HttpMethod::Get, &status_url, false, None, None) {
            Ok((204, _)) => {
                // Settle so companion services (kraken/wifi/nginx upstreams) finish boot.
                std::thread::sleep(std::time::Duration::from_secs(45));
                match execute_curl(&HttpMethod::Get, &status_url, false, None, None) {
                    Ok((204, _)) => return Ok(()),
                    Ok((code, _)) => last_err = format!("HTTP {code} after settle"),
                    Err(err) => last_err = err,
                }
            }
            Ok((code, _)) => last_err = format!("HTTP {code}"),
            Err(err) => last_err = err,
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
    Err(format!(
        "BlueOS did not recover within {timeout_secs}s (last: {last_err})"
    ))
}

pub fn resolve_http_path(catalog: &Catalog, route: &RouteRef) -> Option<String> {
    let path = route.path;
    if path.contains('{') {
        return None;
    }
    if path_starts_with_service_prefix(catalog, route.service, path) {
        return Some(ensure_leading_slash(path));
    }

    let observed = catalog.observed_by_id(&route.service)?;
    let prefix = first_nginx_prefix(observed)?;

    let mut full = prefix.trim_end_matches('/').to_string();
    if let Some(version) = route.version {
        full.push('/');
        full.push_str(version);
    }
    if path.starts_with('/') {
        full.push_str(path);
    } else {
        full.push('/');
        full.push_str(path);
    }
    Some(full)
}

pub fn execute_curl(
    method: &HttpMethod,
    url: &str,
    allow_mutating: bool,
    body: Option<&str>,
    form_file: Option<&FormFilePart>,
) -> Result<(u16, String), String> {
    if !matches!(method, HttpMethod::Get) && !allow_mutating {
        return Err("mutating HTTP method blocked (pass --allow-mutating)".into());
    }

    let timeout_secs = if url.contains("firmware")
        || url.contains("/extension")
        || url.contains("install_firmware")
        || url.contains("restore_default")
    {
        "600"
    } else {
        "120"
    };

    let mut command = Command::new("curl");
    command.args(["-s", "-m", timeout_secs, "-w", "\n%{http_code}"]);
    match method {
        HttpMethod::Get => {}
        HttpMethod::Post => {
            command.args(["-X", "POST"]);
        }
        HttpMethod::Put => {
            command.args(["-X", "PUT"]);
        }
        HttpMethod::Delete => {
            command.args(["-X", "DELETE"]);
        }
        HttpMethod::Patch => {
            command.args(["-X", "PATCH"]);
        }
    }
    if let Some(form) = form_file {
        let path = fixture_path(form.fixture);
        if !path.is_file() {
            return Err(format!("fixture file missing: {}", path.display()));
        }
        let field = format!("{}=@{}", form.field, path.display());
        command.args(["-F", &field]);
    } else if let Some(body) = body {
        command.args(["-H", "Content-Type: application/json", "-d", body]);
    }
    command.arg(url);

    let output = command
        .output()
        .map_err(|err| format!("curl failed to start: {err}"))?;
    if !output.status.success() && output.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("curl exited with {}: {stderr}", output.status));
    }

    let response = String::from_utf8_lossy(&output.stdout);
    let (body, status) = response
        .rsplit_once('\n')
        .ok_or_else(|| "curl response missing status line".to_string())?;
    let status_code = status
        .trim()
        .parse::<u16>()
        .map_err(|_| format!("invalid HTTP status from curl: {status}"))?;
    Ok((status_code, body.to_string()))
}

pub fn evaluate_http_response(
    status_code: u16,
    body: &str,
    expected_status: Option<u16>,
    body_predicate: Option<&str>,
) -> StepResult {
    if let Some(expected) = expected_status {
        if status_code != expected {
            return StepResult::Fail(format!("expected HTTP {expected}, got {status_code}"));
        }
        if let Some(predicate) = body_predicate {
            if let Some(needle) = predicate.strip_prefix("contains:") {
                if !body.contains(needle) {
                    return StepResult::Fail(format!("body missing expected substring: {needle}"));
                }
            }
        }
        StepResult::Pass
    } else {
        StepResult::Unasserted
    }
}

pub fn run_smoke_http_call(
    catalog: &Catalog,
    base: &str,
    call: &SmokeHttpCall,
    allow_mutating: bool,
) -> StepResult {
    let step = RunnableStep {
        journey_id: JourneyId::ConnectToWifiNetwork,
        step_index: 0,
        route: call.route.clone(),
        expected_status: Some(call.expected_status),
        body_predicate: None,
        body: call.body,
        query: call.query,
        form_file: call.form_file.clone(),
    };
    run_http_step(catalog, base, &step, allow_mutating)
}

pub fn run_http_step(
    catalog: &Catalog,
    base: &str,
    step: &RunnableStep,
    allow_mutating: bool,
) -> StepResult {
    if !matches!(step.route.method, HttpMethod::Get) && !allow_mutating {
        return StepResult::Skip(format!("{:?} requires --allow-mutating", step.route.method));
    }

    let Some(path) = resolve_http_path(catalog, &step.route) else {
        return StepResult::Skip("unresolved or templated route path".into());
    };

    let url = join_url(base, &path);
    let url = if let Some(query) = step.query {
        format!("{url}?{query}")
    } else {
        url
    };
    let (status_code, body) = match execute_curl(
        &step.route.method,
        &url,
        allow_mutating,
        step.body,
        step.form_file.as_ref(),
    ) {
        Ok(response) => response,
        Err(err) => return StepResult::Fail(err),
    };

    evaluate_http_response(
        status_code,
        &body,
        step.expected_status,
        step.body_predicate,
    )
}

pub fn summarize_journey(step_results: &[StepResult]) -> JourneyResult {
    if step_results.is_empty() {
        return JourneyResult::Skip;
    }
    let mut has_fail = false;
    let mut has_pass = false;
    let mut has_skip = false;
    let mut has_unasserted = false;

    for result in step_results {
        match result {
            StepResult::Fail(_) => has_fail = true,
            StepResult::Pass => has_pass = true,
            StepResult::Skip(_) | StepResult::Ignored => has_skip = true,
            StepResult::Unasserted => has_unasserted = true,
        }
    }

    if has_fail {
        JourneyResult::Fail
    } else if has_pass && (has_skip || has_unasserted) {
        JourneyResult::Partial
    } else if has_pass || has_unasserted {
        JourneyResult::Pass
    } else {
        JourneyResult::Skip
    }
}

pub fn path_starts_with_service_prefix(catalog: &Catalog, service: ServiceId, path: &str) -> bool {
    let Some(observed) = catalog.observed_by_id(&service) else {
        return false;
    };
    match &observed.nginx_prefixes {
        ObservedSet::Known { items } => items.iter().any(|prefix| {
            let prefix = prefix.value.0;
            if prefix == "/" {
                return false;
            }
            path.starts_with(prefix) || path.starts_with(prefix.trim_end_matches('/'))
        }),
        ObservedSet::Unknown { .. } => false,
    }
}

fn first_nginx_prefix(observed: &crate::observed::ObservedFacts) -> Option<&'static str> {
    match &observed.nginx_prefixes {
        ObservedSet::Known { items } => items.first().map(|prefix| prefix.value.0),
        ObservedSet::Unknown { .. } => None,
    }
}

fn ensure_leading_slash(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::ServiceId;
    use crate::journey::{JourneyStep, Visibility};
    use crate::provenance::{GroundedItem, Provenance};

    const DOC: Provenance = Provenance::doc("test.md", 1);

    #[test]
    fn http_journeys_are_http_automatable() {
        let catalog = Catalog::bootstrap();
        for journey in http_journeys(&catalog) {
            assert_eq!(derive_automatable(journey), Automatable::Http);
        }
    }

    #[test]
    fn tier1_get_coverage_reports_http_journey_get_steps() {
        let catalog = Catalog::bootstrap();
        let coverage = tier1_get_coverage(&catalog);

        assert!(coverage.http_journeys > 0);
        assert_eq!(
            coverage.get_concrete,
            coverage.get_asserted + coverage.get_unasserted
        );
        assert_eq!(coverage.get_unasserted, coverage.unasserted.len());
        assert!(
            coverage.get_concrete + coverage.get_templated > 0,
            "expected at least one GET step across Http journeys"
        );
    }

    #[test]
    fn tier1_get_coverage_is_complete() {
        let catalog = Catalog::bootstrap();
        let coverage = tier1_get_coverage(&catalog);
        assert_eq!(
            coverage.get_unasserted, 0,
            "unasserted concrete GET steps: {:?}",
            coverage.unasserted
        );
    }

    #[test]
    fn tier1_get_coverage_treats_wildcard_paths_as_templated() {
        let catalog = Catalog::bootstrap();
        let coverage = tier1_get_coverage(&catalog);
        assert!(
            !coverage
                .unasserted
                .iter()
                .any(|(_, _, path)| path.contains('*')),
            "wildcard paths must not count as unasserted concrete GETs"
        );
    }

    #[test]
    fn http_smoke_steps_excludes_streaming_gets() {
        assert!(is_smoke_excluded_get("/disk/speed/stream"));
        assert!(is_smoke_excluded_get("/internet_download_speed"));
        assert!(is_smoke_excluded_get("/internet_upload_speed"));
        assert!(!is_smoke_excluded_get("/internet_best_server"));
        assert!(!is_smoke_excluded_get("/disk/speed"));
    }

    #[test]
    fn http_steps_extracts_known_routes_only() {
        static STEPS: &[GroundedItem<JourneyStep>] = &[
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "open tray",
                    route: None,
                    outcome: None,
                },
                DOC,
            ),
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "scan",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Wifi,
                            method: HttpMethod::Get,
                            path: "/scan",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: Some(Grounded::known(
                        crate::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            transition: None,
                        },
                        DOC,
                    )),
                },
                DOC,
            ),
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "connect",
                    route: Some(Grounded::unknown("not grounded")),
                    outcome: None,
                },
                DOC,
            ),
        ];
        let journey = UserJourney {
            id: JourneyId::ConnectToWifiNetwork,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions: GroundedSet::known(&[]),
            steps: GroundedSet::known(STEPS),
            chains_from: None,
        };

        let steps = http_steps(&journey);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].step_index, 1);
        assert_eq!(steps[0].expected_status, Some(200));
        assert_eq!(steps[0].route.path, "/scan");
    }

    #[test]
    fn dry_run_planning_does_not_require_network() {
        let catalog = Catalog::bootstrap();
        let journeys = http_journeys(&catalog);
        assert!(!journeys.is_empty());

        let mut planned_steps = 0;
        for journey in &journeys {
            planned_steps += http_steps(journey).len();
        }
        assert!(planned_steps > 0);

        for journey in &journeys {
            for step in http_steps(journey) {
                let path = resolve_http_path(&catalog, &step.route);
                if step.route.path.contains('{') {
                    assert!(path.is_none());
                } else {
                    assert!(path.is_some(), "path for {:?}", step.route.path);
                }
            }
        }
    }

    #[test]
    fn join_url_normalizes_base_and_path() {
        assert_eq!(
            join_url("http://example.com/", "/wifi-manager/v1.0/scan"),
            "http://example.com/wifi-manager/v1.0/scan"
        );
        assert_eq!(
            join_url("http://example.com", "wifi-manager/v1.0/scan"),
            "http://example.com/wifi-manager/v1.0/scan"
        );
        assert_eq!(
            join_url("192.168.0.177", "/helper/v1.0/ping?host=1.1.1.1"),
            "http://192.168.0.177/helper/v1.0/ping?host=1.1.1.1"
        );
    }

    #[test]
    fn evaluate_http_response_unasserted_without_expected_status() {
        assert_eq!(
            evaluate_http_response(200, "{}", None, None),
            StepResult::Unasserted
        );
    }

    #[test]
    fn evaluate_http_response_passes_known_status_and_contains_predicate() {
        assert_eq!(
            evaluate_http_response(200, r#"{"online": true}"#, Some(200), Some("contains:true")),
            StepResult::Pass
        );
        assert!(matches!(
            evaluate_http_response(404, "{}", Some(200), None),
            StepResult::Fail(_)
        ));
    }

    #[test]
    fn format_http_fail_includes_resolved_url() {
        let step = RunnableStep {
            journey_id: JourneyId::ConnectToWifiNetwork,
            step_index: 2,
            route: RouteRef {
                service: ServiceId::Helper,
                method: HttpMethod::Get,
                path: "/ping",
                version: Some("v1.0"),
            },
            expected_status: Some(200),
            body_predicate: None,
            body: None,
            query: None,
            form_file: None,
        };
        let line = format_http_fail(
            JourneyId::ConnectToWifiNetwork,
            &step,
            "http://192.168.0.177/helper/v1.0/ping?host=1.1.1.1",
            "expected HTTP 200, got 404",
        );
        assert!(line.contains("→ http://192.168.0.177/helper/v1.0/ping?host=1.1.1.1"));
        assert!(line.contains("FAIL connect_to_wifi_network step 2 GET /ping"));
        assert!(line.contains("expected HTTP 200, got 404"));
    }

    #[test]
    fn format_dry_run_includes_resolved_path() {
        let step = RunnableStep {
            journey_id: JourneyId::ConnectToWifiNetwork,
            step_index: 1,
            route: RouteRef {
                service: ServiceId::Helper,
                method: HttpMethod::Get,
                path: "/ping",
                version: Some("v1.0"),
            },
            expected_status: Some(200),
            body_predicate: None,
            body: None,
            query: None,
            form_file: None,
        };
        let line = format_dry_run(JourneyId::ConnectToWifiNetwork, &step, "/helper/v1.0/ping");
        assert_eq!(
            line,
            "DRY-RUN connect_to_wifi_network step 1 GET /ping → /helper/v1.0/ping"
        );
    }

    #[test]
    fn journey_http_base_required_for_smoke() {
        assert!(journey_http_requires_base(false, true, false, None).is_err());
        assert!(journey_http_requires_base(false, false, true, None).is_err());
        assert!(journey_http_requires_base(true, false, false, None).is_ok());
        assert!(journey_http_requires_base(false, false, false, None).is_err());
        assert!(journey_http_requires_base(false, true, false, Some("http://pi")).is_ok());
        assert!(journey_http_requires_base(false, false, true, Some("http://pi")).is_ok());
    }

    #[test]
    fn journey_http_smoke_modes_are_mutually_exclusive() {
        assert!(journey_http_mode_conflict(true, true).is_err());
        assert!(journey_http_mode_conflict(true, false).is_ok());
        assert!(journey_http_mode_conflict(false, true).is_ok());
        assert!(journey_http_mode_conflict(false, false).is_ok());
    }

    #[test]
    fn http_mutating_smoke_steps_filters_get_out() {
        static STEPS: &[GroundedItem<JourneyStep>] = &[
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "get scan",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Wifi,
                            method: HttpMethod::Get,
                            path: "/scan",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: Some(Grounded::known(
                        crate::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            transition: None,
                        },
                        DOC,
                    )),
                },
                DOC,
            ),
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "put theme",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Customization,
                            method: HttpMethod::Put,
                            path: "/theme",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: Some(Grounded::known(
                        crate::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            transition: None,
                        },
                        DOC,
                    )),
                },
                DOC,
            ),
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "delete templated",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Customization,
                            method: HttpMethod::Delete,
                            path: "/models/{name}",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: Some(Grounded::known(
                        crate::journey::StepOutcome {
                            expected_status: Some(204),
                            body_predicate: None,
                            transition: None,
                        },
                        DOC,
                    )),
                },
                DOC,
            ),
        ];
        let journey = UserJourney {
            id: JourneyId::ChangeUiThemeColor,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions: GroundedSet::known(&[]),
            steps: GroundedSet::known(STEPS),
            chains_from: None,
        };

        let steps = http_mutating_smoke_steps(&journey);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].route.path, "/theme");
        assert_eq!(steps[0].body, Some(r##"{"primary":"#1e88e5"}"##));
        assert!(steps[0].query.is_none());
    }

    #[test]
    fn http_smoke_steps_only_get_with_expected_status() {
        static STEPS: &[GroundedItem<JourneyStep>] = &[
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "get scan",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Wifi,
                            method: HttpMethod::Get,
                            path: "/scan",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: Some(Grounded::known(
                        crate::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            transition: None,
                        },
                        DOC,
                    )),
                },
                DOC,
            ),
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "post connect",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Wifi,
                            method: HttpMethod::Post,
                            path: "/connect",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: Some(Grounded::known(
                        crate::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            transition: None,
                        },
                        DOC,
                    )),
                },
                DOC,
            ),
            GroundedItem::new(
                JourneyStep {
                    actor: Actor::Operator,
                    description: "get unasserted",
                    route: Some(Grounded::known(
                        RouteRef {
                            service: ServiceId::Wifi,
                            method: HttpMethod::Get,
                            path: "/status",
                            version: Some("v1.0"),
                        },
                        DOC,
                    )),
                    outcome: None,
                },
                DOC,
            ),
        ];
        let journey = UserJourney {
            id: JourneyId::ConnectToWifiNetwork,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions: GroundedSet::known(&[]),
            steps: GroundedSet::known(STEPS),
            chains_from: None,
        };

        let steps = http_smoke_steps(&journey);
        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].route.path, "/scan");
    }

    #[test]
    fn resolve_http_path_builds_wifi_scan_url() {
        let catalog = Catalog::bootstrap();
        let route = RouteRef {
            service: ServiceId::Wifi,
            method: HttpMethod::Get,
            path: "/scan",
            version: Some("v1.0"),
        };
        assert_eq!(
            resolve_http_path(&catalog, &route).as_deref(),
            Some("/wifi-manager/v1.0/scan")
        );
    }
}
