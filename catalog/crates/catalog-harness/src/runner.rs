use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::report::{ConflictKind, ReportConflict};
use catalog_core::catalog::Catalog;
use catalog_core::http::resolve_http_path;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{Grounded, GroundedSet};
use catalog_kernel::version::{availability_skip, format_availability_skip_reason};
use catalog_model::http::http_method_label;
use catalog_model::journey::{
    http_automatable, Actor, BlastRadius, BodyKind, HttpMethod, RouteRef, UseCase,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail(String),
    Inconclusive(String),
    Error,
}

impl Verdict {
    pub fn from_report_str(s: &str) -> Option<Self> {
        match s {
            "pass" => Some(Self::Pass),
            "fail" => Some(Self::Fail(String::new())),
            "skip" | "inconclusive" => Some(Self::Inconclusive(String::new())),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    fn report_str(&self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail(_) => "fail",
            Self::Inconclusive(_) => "inconclusive",
            Self::Error => "error",
        }
    }

    pub fn matrix_rank(&self, reason: Option<&str>) -> u8 {
        match self {
            Self::Error => 0,
            Self::Inconclusive(_) => 1,
            Self::Pass => {
                if reason.is_some_and(|r| !r.is_empty()) {
                    4
                } else {
                    3
                }
            }
            Self::Fail(_) => 5,
        }
    }
}

impl Serialize for Verdict {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.report_str())
    }
}

impl<'de> Deserialize<'de> for Verdict {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Self::from_report_str(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown verdict: {s}")))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JourneyResult {
    Pass,
    Fail,
    Skip,
    Partial,
}

impl Verdict {
    pub fn from_journey_result(outcome: JourneyResult) -> Self {
        match outcome {
            JourneyResult::Pass | JourneyResult::Partial => Self::Pass,
            JourneyResult::Fail => Self::Fail(String::new()),
            JourneyResult::Skip => Self::Inconclusive(String::new()),
        }
    }
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
    pub body_kind: BodyKind,
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
    pub fn record(&mut self, result: &Verdict) {
        match result {
            Verdict::Pass => self.passed += 1,
            Verdict::Fail(_) => self.failed += 1,
            Verdict::Inconclusive(_) => self.skipped += 1,
            Verdict::Error => self.unasserted += 1,
        }
    }
}

pub fn http_journeys(catalog: &Catalog) -> Vec<&UseCase> {
    catalog
        .journeys()
        .iter()
        .filter(|journey| http_automatable(journey))
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

pub const MUTATING_SMOKE_DEFAULT_FIXTURES: &str = "internet,pirate,advanced,confirm-dangerous,board:any,wifi-radio,hotspot,nmea-socket,serial-bridge,dhcp-active,extension-installed,recording,local-version,wifi-saved,wifi-connected";

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

pub fn journey_http_mode_conflict(
    smoke: bool,
    mutating_smoke: bool,
    negative: bool,
    ui: bool,
) -> Result<(), &'static str> {
    let modes = [smoke, mutating_smoke, negative, ui]
        .into_iter()
        .filter(|enabled| *enabled)
        .count();
    if modes > 1 {
        return Err("--smoke, --mutating-smoke, --negative, and --ui are mutually exclusive");
    }
    Ok(())
}

pub fn journey_http_requires_base(
    dry_run: bool,
    smoke: bool,
    mutating_smoke: bool,
    negative: bool,
    ui: bool,
    base: Option<&str>,
) -> Result<(), &'static str> {
    if (smoke || mutating_smoke || ui) && base.is_none() {
        return Err("--base is required for --smoke, --mutating-smoke, and --ui");
    }
    if negative && !dry_run && base.is_none() {
        return Err("--base is required for --negative");
    }
    if !dry_run && base.is_none() {
        return Err("--base is required unless --dry-run is set");
    }
    Ok(())
}

pub fn run_negative_probe(
    base: &str,
    probe: &crate::negative_probes::NegativeProbe,
    allow_mutating: bool,
    current_tag: Option<&str>,
) -> Verdict {
    if probe.id == "NP-38" {
        return run_negative_probe_np38_concurrent_scan(base, allow_mutating, probe);
    }

    let body = negative_probe_body(probe, current_tag);
    if probe.id == "NP-62" && body.is_none() {
        return fail_negative_probe(
            probe,
            "NP-62 requires running tag from GET /version/current".into(),
        );
    }

    let url = crate::negative_probes::negative_probe_url(base, probe);
    let (status_code, body_text) =
        match execute_curl(&probe.method, &url, allow_mutating, body.as_deref(), None) {
            Ok(response) => response,
            Err(err) => return fail_negative_probe(probe, err),
        };

    evaluate_negative_probe_response(probe, status_code, &body_text)
}

fn negative_probe_body(
    probe: &crate::negative_probes::NegativeProbe,
    current_tag: Option<&str>,
) -> Option<String> {
    if probe.id == "NP-62" {
        let tag = current_tag?;
        return Some(format!(
            r#"{{"repository":"bluerobotics/blueos-core","tag":"{tag}"}}"#
        ));
    }
    probe.body.map(|body| body.to_string())
}

fn evaluate_negative_probe_response(
    probe: &crate::negative_probes::NegativeProbe,
    status_code: u16,
    body: &str,
) -> Verdict {
    let result = evaluate_http_response(status_code, body, probe.expected_status, None);
    match &result {
        Verdict::Pass => eprintln!("PASS {} HTTP {status_code}", probe.id),
        Verdict::Fail(msg) => eprintln!("FAIL {} HTTP {status_code} — {msg}", probe.id),
        Verdict::Error => eprintln!("UNASSERTED {} HTTP {status_code}", probe.id),
        _ => {}
    }
    result
}

/// Report a probe that failed before any status code came back. `evaluate_negative_probe_response`
/// is the only other place that prints, so a `Fail` returned around it is counted in the summary
/// but never shown.
fn fail_negative_probe(probe: &crate::negative_probes::NegativeProbe, message: String) -> Verdict {
    eprintln!("FAIL {} — {message}", probe.id);
    Verdict::Fail(message)
}

fn run_negative_probe_np38_concurrent_scan(
    base: &str,
    allow_mutating: bool,
    probe: &crate::negative_probes::NegativeProbe,
) -> Verdict {
    use std::sync::mpsc;
    use std::thread;

    let url = crate::negative_probes::negative_probe_url(base, probe);
    let (tx, rx) = mpsc::channel();
    for _ in 0..2 {
        let tx = tx.clone();
        let url = url.clone();
        thread::spawn(move || {
            let result = execute_curl(&HttpMethod::Get, &url, allow_mutating, None, None);
            let _ = tx.send(result);
        });
    }
    drop(tx);

    let mut statuses = Vec::new();
    for _ in 0..2 {
        match rx.recv() {
            Ok(Ok((status, _))) => statuses.push(status),
            Ok(Err(err)) => return fail_negative_probe(probe, err),
            Err(_) => {
                return fail_negative_probe(probe, "NP-38 concurrent scan thread failed".into())
            }
        }
    }

    if statuses.contains(&425) {
        return evaluate_negative_probe_response(probe, 425, "");
    }

    fail_negative_probe(
        probe,
        format!(
            "NP-38 expected 425 on at least one concurrent scan, got {:?}",
            statuses
        ),
    )
}

pub fn http_steps(journey: &UseCase) -> Vec<RunnableStep> {
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
        let (expected_status, body_predicate, body_kind) = match &step.value.outcome {
            None => (None, None, BodyKind::Unknown),
            Some(Grounded::Unknown { .. }) => (None, None, BodyKind::Unknown),
            Some(Grounded::Known { value: outcome, .. }) => (
                outcome.expected_status,
                outcome.body_predicate,
                outcome.body_kind,
            ),
        };
        runnable.push(RunnableStep {
            journey_id: journey.id,
            step_index,
            route: route.clone(),
            expected_status,
            body_predicate,
            body_kind,
            body: None,
            query: None,
            form_file: None,
        });
    }
    runnable
}

pub fn fixture_path(relative: &str) -> PathBuf {
    catalog_paths::catalog_dir().join("fixtures").join(relative)
}

pub fn http_smoke_steps(journey: &UseCase) -> Vec<RunnableStep> {
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
        (JourneyId::ForgetSavedWifiNetwork, "/remove", Post) => Some("ssid=BlueOS-Hotspot"),
        (JourneyId::ConnectToWifiNetwork, "/connect", Post)
        | (JourneyId::ForceWifiNetworkPassword, "/connect", Post)
        | (JourneyId::ReconnectToSavedWifiNetwork, "/connect", Post)
        | (JourneyId::RejectInvalidWifiCredentials, "/connect", Post) => Some("hidden=false"),
        (JourneyId::ConnectToHiddenWifiNetwork, "/connect", Post) => Some("hidden=true"),
        (JourneyId::ToggleHotspot, "/hotspot", Post) => Some("enable=true"),
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
            Some(crate::wifi_rf::CONNECT_SMOKE_BODY)
        }
        (JourneyId::ConnectToHiddenWifiNetwork, "/connect", Post)
        | (JourneyId::ForceWifiNetworkPassword, "/connect", Post) => {
            Some(crate::wifi_rf::CONNECT_SMOKE_BODY)
        }
        (JourneyId::ReconnectToSavedWifiNetwork, "/connect", Post) => {
            Some(crate::wifi_rf::EMPTY_PASSWORD_SMOKE_BODY)
        }
        (JourneyId::RejectInvalidWifiCredentials, "/connect", Post) => {
            Some(crate::wifi_rf::WRONG_PASSWORD_SMOKE_BODY)
        }
        (JourneyId::ConfigureHotspotCredentials, "/hotspot_credentials", Post) => {
            Some(crate::wifi_rf::HOTSPOT_CREDENTIALS_SMOKE_BODY)
        }
        (JourneyId::AddExternalNmeaGpsSocket, "/socks", Post)
        | (JourneyId::RemoveConfiguredNmeaSocket, "/socks", Delete) => Some(SMOKE_NMEA_SOCK_JSON),
        (JourneyId::RemoveSerialBridge, "/bridges", Delete) => Some(SMOKE_BRIDGE_JSON),
        (JourneyId::DockerRegistryLogin, "/docker/login", Post) => {
            Some(r##"{"username":"","password":"","registry":"","root":true}"##)
        }
        // Tag `master` is the DUT restore/pull name, not an image identity pin (see capture_env digests).
        (JourneyId::UpdateBlueosVersion, "/version/pull", Post)
        | (JourneyId::PullBlueosVersionWithoutSwitch, "/version/pull", Post) => {
            Some(r##"{"repository":"bluerobotics/blueos-core","tag":"master"}"##)
        }
        (JourneyId::UpdateBootstrapImage, "/version/pull", Post) => {
            Some(r##"{"repository":"bluerobotics/blueos-bootstrap","tag":"master"}"##)
        }
        (JourneyId::UpdateBlueosVersion, "/version/current", Post) => {
            Some(r##"{"repository":"bluerobotics/blueos-core","tag":"master"}"##)
        }
        (JourneyId::SwitchLocalBlueosVersion, "/version/current", Post) => {
            Some(SMOKE_CORE_SWITCH_JSON)
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
        (JourneyId::ConnectToWifiNetwork, "/connect", Post)
        | (JourneyId::ConnectToHiddenWifiNetwork, "/connect", Post)
        | (JourneyId::ForceWifiNetworkPassword, "/connect", Post)
        | (JourneyId::ReconnectToSavedWifiNetwork, "/connect", Post) => Some(200),
        (JourneyId::RejectInvalidWifiCredentials, "/connect", Post) => Some(500),
        (JourneyId::AddExternalNmeaGpsSocket, "/socks", Post) => Some(201),
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
        _ => None,
    }
}

pub fn mutating_smoke_skip_reason(
    journey_id: JourneyId,
    _fixtures: &crate::fixture::FixtureInventory,
) -> Option<&'static str> {
    if (crate::wifi_rf::needs_host_ap(journey_id) || crate::wifi_rf::needs_host_station(journey_id))
        && !crate::wifi_rf::host_rf_available()
    {
        return Some("host WiFi RF unavailable (need nmcli + HOST_WIFI_IFACE on the runner)");
    }
    None
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
const SMOKE_ROUTE_FLUSH_QUERY: &str =
    "command=ip%20route%20flush%20proto%20static&i_know_what_i_am_doing=true";
const SMOKE_ADDR_DEL_1_QUERY: &str = "command=bash%20-lc%20%27curl%20-s%20-m%2010%20-o%20%2Fdev%2Fnull%20-X%20DELETE%20%22http%3A%2F%2F127.0.0.1%2Fcable-guy%2Fv1.0%2Faddress%3Finterface_name%3Deth0%26ip_address%3D192.168.0.1%22%20%7C%7C%20true%3B%20ip%20addr%20del%20192.168.0.1%2F24%20dev%20eth0%202%3E%2Fdev%2Fnull%20%7C%7C%20true%27&i_know_what_i_am_doing=true";
const SMOKE_ADDR_DEL_178_QUERY: &str = "command=bash%20-lc%20%27curl%20-s%20-m%2010%20-o%20%2Fdev%2Fnull%20-X%20DELETE%20%22http%3A%2F%2F127.0.0.1%2Fcable-guy%2Fv1.0%2Faddress%3Finterface_name%3Deth0%26ip_address%3D192.168.0.178%22%20%7C%7C%20true%3B%20ip%20addr%20del%20192.168.0.178%2F24%20dev%20eth0%202%3E%2Fdev%2Fnull%20%7C%7C%20true%27&i_know_what_i_am_doing=true";
const SMOKE_RESOLV_FIX_QUERY: &str = "command=docker%20exec%20blueos-core%20sh%20-c%20%27printf%20%22nameserver%208.8.8.8%5Cnnameserver%201.1.1.1%5Cn%22%20%3E%20%2Fetc%2Fresolv.conf.host%27&i_know_what_i_am_doing=true";
// Retag the running `blueos-core` image (same digest, any channel) to a local alias.
const SMOKE_LOCAL_VERSION_TAG_QUERY: &str = "command=docker%20tag%20%24%28docker%20inspect%20-f%20%27%7B%7B.Image%7D%7D%27%20blueos-core%29%20bluerobotics%2Fblueos-core%3Asmoke-catalog-deleteme&i_know_what_i_am_doing=true";
const SMOKE_CORE_SWITCH_TAG_QUERY: &str = "command=docker%20tag%20%24%28docker%20inspect%20-f%20%27%7B%7B.Image%7D%7D%27%20blueos-core%29%20bluerobotics%2Fblueos-core%3Asmoke-catalog-switch&i_know_what_i_am_doing=true";
pub const SMOKE_CORE_SWITCH_JSON: &str =
    r##"{"repository":"bluerobotics/blueos-core","tag":"smoke-catalog-switch"}"##;
pub const SMOKE_CORE_SWITCH_TAG: &str = "smoke-catalog-switch";

pub fn dut_version_current_json(dut: &DutVersion) -> String {
    format!(
        r#"{{"repository":"{}","tag":"{}"}}"#,
        dut.repository, dut.tag
    )
}
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
        JourneyId::ForgetSavedWifiNetwork
        | JourneyId::ForceWifiNetworkPassword
        | JourneyId::DetectWifiApLoss
        | JourneyId::AutoconnectToSavedWifiNetwork
        | JourneyId::DisconnectFromWifiNetwork => &[
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Wifi,
                    method: Post,
                    path: "/hotspot",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some("enable=false"),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Wifi,
                    method: Post,
                    path: "/connect",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: Some(crate::wifi_rf::CONNECT_SMOKE_BODY),
                query: Some("hidden=false"),
                form_file: None,
            },
        ],
        JourneyId::ReconnectToSavedWifiNetwork => &[
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Wifi,
                    method: Post,
                    path: "/hotspot",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some("enable=false"),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Wifi,
                    method: Post,
                    path: "/connect",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: Some(crate::wifi_rf::CONNECT_SMOKE_BODY),
                query: Some("hidden=false"),
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Wifi,
                    method: Get,
                    path: "/disconnect",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: None,
                form_file: None,
            },
        ],
        JourneyId::ConnectToWifiNetwork
        | JourneyId::ConnectToHiddenWifiNetwork
        | JourneyId::RejectInvalidWifiCredentials => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Wifi,
                method: Post,
                path: "/hotspot",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: Some("enable=false"),
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
        JourneyId::ToggleHotspot => &[
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Wifi,
                    method: Get,
                    path: "/disconnect",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: None,
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Wifi,
                    method: Post,
                    path: "/hotspot_credentials",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: Some(crate::wifi_rf::HOTSPOT_CREDENTIALS_SMOKE_BODY),
                query: None,
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Wifi,
                    method: Post,
                    path: "/hotspot",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some("enable=false"),
                form_file: None,
            },
        ],
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
        JourneyId::SwitchLocalBlueosVersion => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Commander,
                method: Post,
                path: "/command/host",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: Some(SMOKE_CORE_SWITCH_TAG_QUERY),
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
        JourneyId::ChangeMdnsHostname => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Beacon,
                method: Post,
                path: "/hostname",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: Some("hostname=blueos"),
            form_file: None,
        }],
        // Core restore (POST /version/current → original DUT tag) runs in journey_http before these calls.
        JourneyId::SwitchLocalBlueosVersion => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Versionchooser,
                method: Delete,
                path: "/version/delete",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: Some(SMOKE_CORE_SWITCH_JSON),
            query: None,
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
            query: Some("enable=false"),
            form_file: None,
        }],
        JourneyId::ConnectToWifiNetwork
        | JourneyId::ConnectToHiddenWifiNetwork
        | JourneyId::ForceWifiNetworkPassword
        | JourneyId::ReconnectToSavedWifiNetwork
        | JourneyId::AutoconnectToSavedWifiNetwork => &[
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Wifi,
                    method: Get,
                    path: "/disconnect",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: None,
                form_file: None,
            },
            SmokeHttpCall {
                route: RouteRef {
                    service: ServiceId::Wifi,
                    method: Post,
                    path: "/remove",
                    version: Some("v1.0"),
                },
                expected_status: 200,
                body: None,
                query: Some("ssid=BlueOS-Hotspot"),
                form_file: None,
            },
        ],
        // Wrong-password never associates; AP-loss / disconnect leave idle — GET /disconnect is 500.
        JourneyId::RejectInvalidWifiCredentials
        | JourneyId::DetectWifiApLoss
        | JourneyId::DisconnectFromWifiNetwork => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::Wifi,
                method: Post,
                path: "/remove",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: None,
            query: Some("ssid=BlueOS-Hotspot"),
            form_file: None,
        }],
        // POST /remove already drops the association; GET /disconnect is 500 when idle.
        JourneyId::ForgetSavedWifiNetwork => &[],
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
        JourneyId::AddExternalNmeaGpsSocket => &[SmokeHttpCall {
            route: RouteRef {
                service: ServiceId::NmeaInjector,
                method: Delete,
                path: "/socks",
                version: Some("v1.0"),
            },
            expected_status: 200,
            body: Some(SMOKE_NMEA_SOCK_JSON),
            query: None,
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

pub fn http_mutating_smoke_steps(journey: &UseCase) -> Vec<RunnableStep> {
    http_steps(journey)
        .into_iter()
        .filter(|step| {
            let side_effecting_get = journey.id == JourneyId::DisconnectFromWifiNetwork
                && matches!(step.route.method, HttpMethod::Get)
                && step.route.path == "/disconnect";
            (!matches!(step.route.method, HttpMethod::Get) || side_effecting_get)
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

const DUT_VERSION_PATH: &str = "/version-chooser/v1.0/version/current";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DutVersion {
    pub repository: String,
    pub tag: String,
    pub digest: Option<String>,
}

#[derive(serde::Deserialize)]
struct VersionCurrentJson {
    repository: String,
    tag: String,
    #[serde(default)]
    sha: Option<String>,
}

pub fn fetch_dut_version(base: &str) -> Result<DutVersion, String> {
    let url = join_url(base, DUT_VERSION_PATH);
    let (status, body) = execute_curl(&HttpMethod::Get, &url, false, None, None)?;
    if status != 200 {
        return Err(format!("GET {DUT_VERSION_PATH} returned HTTP {status}"));
    }
    parse_dut_version_json(&body)
}

fn parse_dut_version_json(body: &str) -> Result<DutVersion, String> {
    let parsed: VersionCurrentJson =
        serde_json::from_str(body).map_err(|err| format!("parse version/current JSON: {err}"))?;
    Ok(DutVersion {
        repository: parsed.repository,
        tag: parsed.tag,
        digest: parsed.sha,
    })
}

pub fn journey_availability_skip(journey: &UseCase, dut: &DutVersion) -> Option<String> {
    availability_skip(&dut.tag, &journey.availability)
        .map(|skip| format_availability_skip_reason(&skip, &dut.tag))
}

/// Hosts whose state may be destroyed: image switches, firmware flashes, settings resets. Both are
/// lab devices whose owner has signed off on losing their configuration.
const PLAY_SACRIFICIAL_HOSTS: [&str; 2] = ["192.168.0.177", "192.168.0.88"];

/// Hosts whose management link is the only way back in. `192.168.0.88` is the same physical vehicle
/// as `192.168.2.2`, reached through the lab router instead of the direct tether, so the
/// reconfiguration journeys that could strand it are exactly as dangerous there.
const NEVER_STRAND_MGMT_HOSTS: [&str; 2] = ["192.168.2.2", "192.168.0.88"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DutProfile {
    pub sacrificial: bool,
    pub never_strand_mgmt: bool,
    pub rf_serial: bool,
}

pub fn dut_profile_for_host(host: &str) -> DutProfile {
    let host = host.trim();
    DutProfile {
        sacrificial: PLAY_SACRIFICIAL_HOSTS
            .iter()
            .any(|play| host.contains(play))
            || std::env::var("BLUEOS_DUT_SACRIFICIAL").ok().as_deref() == Some("1"),
        never_strand_mgmt: NEVER_STRAND_MGMT_HOSTS
            .iter()
            .any(|mgmt| host.contains(mgmt)),
        rf_serial: true,
    }
}

pub fn journey_profile_skip(journey: &UseCase, profile: &DutProfile) -> Option<String> {
    if profile.never_strand_mgmt {
        match journey.id {
            JourneyId::AssignStaticIpAddress
            | JourneyId::AcquireDynamicIpAddress
            | JourneyId::EnableOnboardDhcpServer
            | JourneyId::DisableOnboardDhcpServer
            | JourneyId::SetNetworkInterfacePriority
            | JourneyId::ChangeMdnsHostname => {
                return Some(format!(
                    "strand-risk journey {journey_id} refused on never_strand_mgmt DUT",
                    journey_id = journey.id
                ));
            }
            _ => {}
        }
    }
    if !profile.sacrificial
        && matches!(
            journey.blast_radius,
            Grounded::Known {
                value: BlastRadius::Destructive,
                ..
            }
        )
    {
        return Some(format!(
            "destructive journey {journey_id} refused on non-sacrificial DUT",
            journey_id = journey.id
        ));
    }
    None
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

/// POST `/version/current` kills `blueos-core` before the HTTP response is written, so curl
/// often sees HTTP 0 / transport failure. Treat that as expected, wait for recovery, then
/// assert the running tag.
pub fn run_core_image_switch(
    catalog: &Catalog,
    base: &str,
    body: &str,
    expected_tag: &str,
    allow_mutating: bool,
) -> Verdict {
    use HttpMethod::*;
    let post = RouteRef {
        service: ServiceId::Versionchooser,
        method: Post,
        path: "/version/current",
        version: Some("v1.0"),
    };
    let Some(path) = resolve_http_path(catalog, &post) else {
        return Verdict::Inconclusive("unresolved /version/current".into());
    };
    let url = join_url(base, &path);
    match execute_curl(&Post, &url, allow_mutating, Some(body), None) {
        Ok((200, _)) | Ok((0, _)) => {}
        Ok((code, _)) => {
            return Verdict::Fail(format!(
                "core switch: expected HTTP 200 or connection drop, got {code}"
            ));
        }
        Err(_) => {}
    }
    if let Err(err) = wait_for_blueos(base, 600) {
        return Verdict::Fail(format!("core switch recovery: {err}"));
    }
    let get = RouteRef {
        service: ServiceId::Versionchooser,
        method: Get,
        path: "/version/current",
        version: Some("v1.0"),
    };
    let Some(get_path) = resolve_http_path(catalog, &get) else {
        return Verdict::Fail("unresolved GET /version/current after switch".into());
    };
    let get_url = join_url(base, &get_path);
    match execute_curl(&Get, &get_url, false, None, None) {
        Ok((200, body))
            if body.contains(&format!("\"tag\":\"{expected_tag}\""))
                || body.contains(&format!("\"tag\": \"{expected_tag}\"")) =>
        {
            Verdict::Pass
        }
        Ok((200, body)) => Verdict::Fail(format!(
            "core switch: expected tag {expected_tag}, body={body}"
        )),
        Ok((code, _)) => Verdict::Fail(format!(
            "core switch: GET /version/current expected 200, got {code}"
        )),
        Err(err) => Verdict::Fail(format!("core switch: GET /version/current: {err}")),
    }
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
        || url.contains("/board")
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

/// Commonwealth stream endpoints always return HTTP 200; failures are encoded in JSON fragments.
pub fn streamed_fragment_error(body: &str) -> Option<String> {
    let chunks: Vec<&str> = body
        .split("|\n\n|")
        .map(str::trim)
        .filter(|chunk| !chunk.is_empty())
        .collect();

    for chunk in chunks {
        let value: serde_json::Value = match serde_json::from_str(chunk) {
            Ok(value) => value,
            Err(_) => return None,
        };
        let fragment = value.get("fragment").and_then(serde_json::Value::as_i64)?;
        let status = value
            .get("status")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let error = value.get("error").and_then(|value| match value {
            serde_json::Value::Null => None,
            serde_json::Value::String(text) if text.is_empty() => None,
            serde_json::Value::String(text) => Some(text.as_str()),
            _ => None,
        });

        if status >= 400 || error.is_some() {
            let error_text = error.unwrap_or("");
            return Some(format!(
                "fragment {fragment}: status {status}: {error_text}"
            ));
        }
    }

    None
}

pub fn evaluate_http_response(
    status_code: u16,
    body: &str,
    expected_status: Option<u16>,
    body_predicate: Option<&str>,
) -> Verdict {
    if let Some(expected) = expected_status {
        if status_code != expected {
            return Verdict::Fail(format!("expected HTTP {expected}, got {status_code}"));
        }
        if let Some(predicate) = body_predicate {
            if let Some(needle) = predicate.strip_prefix("contains:") {
                if !body.contains(needle) {
                    return Verdict::Fail(format!("body missing expected substring: {needle}"));
                }
            }
        }
        if let Some(message) = streamed_fragment_error(body) {
            return Verdict::Fail(message);
        }
        Verdict::Pass
    } else {
        Verdict::Error
    }
}

pub fn run_smoke_http_call(
    catalog: &Catalog,
    base: &str,
    call: &SmokeHttpCall,
    allow_mutating: bool,
) -> Verdict {
    let step = RunnableStep {
        journey_id: JourneyId::ConnectToWifiNetwork,
        step_index: 0,
        route: call.route.clone(),
        expected_status: Some(call.expected_status),
        body_predicate: None,
        body_kind: BodyKind::Unknown,
        body: call.body,
        query: call.query,
        form_file: call.form_file.clone(),
    };
    let result = run_http_step(catalog, base, &step, allow_mutating);
    // wifi-manager returns 500 when already idle; treat as success for setup/teardown.
    if call.route.path == "/disconnect"
        && matches!(&result, Verdict::Fail(msg) if msg.contains("got 500"))
    {
        return Verdict::Pass;
    }
    result
}

pub fn effect_read_enabled(effect_step_index: Option<usize>, skip_rf: bool) -> bool {
    effect_step_index.is_some() && !skip_rf
}

pub fn mutating_effect_read_phases(effect_read_active: bool) -> Vec<&'static str> {
    let mut phases = Vec::new();
    if effect_read_active {
        phases.push("before");
    }
    phases.push("mutate");
    if effect_read_active {
        phases.push("after");
    }
    phases.push("restore");
    phases
}

pub fn runnable_http_step_at(journey: &UseCase, step_index: usize) -> Option<RunnableStep> {
    http_steps(journey)
        .into_iter()
        .find(|step| step.step_index == step_index)
}

pub fn effect_observation_changed(before: &str, after: &str, body_predicate: Option<&str>) -> bool {
    match body_predicate {
        Some(predicate) if let Some(needle) = predicate.strip_prefix("contains:") => {
            before.contains(needle) != after.contains(needle)
        }
        _ => before != after,
    }
}

pub struct EffectReadBefore {
    pub probe: RunnableStep,
    pub before_body: String,
}

pub enum EffectReadBeforeResult {
    Skipped,
    Ready(EffectReadBefore),
    Failed(Verdict),
}

pub struct EffectReadAfter {
    pub conflict: Option<ReportConflict>,
    pub result: Verdict,
}

pub fn effect_read_conflict(
    journey_id: JourneyId,
    probe: &RunnableStep,
    body_predicate: Option<&str>,
) -> ReportConflict {
    ReportConflict {
        kind: ConflictKind::EffectNotApplied,
        context: format!(
            "{journey_id} effect_read step {} GET {} unchanged after mutate (predicate={body_predicate:?})",
            probe.step_index, probe.route.path
        ),
    }
}

fn fetch_http_step_body(
    catalog: &Catalog,
    base: &str,
    step: &RunnableStep,
    allow_mutating: bool,
) -> Result<String, Verdict> {
    let Some(path) = resolve_http_path(catalog, &step.route) else {
        return Err(Verdict::Inconclusive(
            "unresolved or templated route path".into(),
        ));
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
        Err(err) => return Err(Verdict::Fail(err)),
    };
    match evaluate_http_response(status_code, &body, step.expected_status, None) {
        Verdict::Pass | Verdict::Error => Ok(body),
        other => Err(other),
    }
}

pub fn effect_read_before(
    catalog: &Catalog,
    base: &str,
    journey: &UseCase,
    effect_step_index: usize,
) -> EffectReadBeforeResult {
    let Some(probe) = runnable_http_step_at(journey, effect_step_index) else {
        return EffectReadBeforeResult::Failed(Verdict::Fail(format!(
            "effect_read step_index {effect_step_index} not runnable for {}",
            journey.id
        )));
    };
    if !matches!(probe.route.method, HttpMethod::Get) {
        return EffectReadBeforeResult::Failed(Verdict::Fail(format!(
            "effect_read step {} {:?} {} is not GET",
            probe.step_index, probe.route.method, probe.route.path
        )));
    }
    match fetch_http_step_body(catalog, base, &probe, false) {
        Ok(body) => EffectReadBeforeResult::Ready(EffectReadBefore {
            probe,
            before_body: body,
        }),
        Err(result) => EffectReadBeforeResult::Failed(result),
    }
}

pub fn effect_read_after(
    catalog: &Catalog,
    base: &str,
    journey_id: JourneyId,
    before: &EffectReadBefore,
) -> EffectReadAfter {
    let after_body = match fetch_http_step_body(catalog, base, &before.probe, false) {
        Ok(body) => body,
        Err(result) => {
            return EffectReadAfter {
                conflict: None,
                result,
            };
        }
    };
    effect_read_after_observed(journey_id, before, &after_body)
}

pub fn effect_read_after_observed(
    journey_id: JourneyId,
    before: &EffectReadBefore,
    after_body: &str,
) -> EffectReadAfter {
    if effect_observation_changed(&before.before_body, after_body, before.probe.body_predicate) {
        return EffectReadAfter {
            conflict: None,
            result: Verdict::Pass,
        };
    }
    let conflict = effect_read_conflict(journey_id, &before.probe, before.probe.body_predicate);
    EffectReadAfter {
        result: Verdict::Fail(conflict.context.clone()),
        conflict: Some(conflict),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpStepRun {
    pub result: Verdict,
    pub conflict: Option<ReportConflict>,
}

pub fn product_missing_reject_conflict(
    step: &RunnableStep,
    status_code: u16,
) -> Option<ReportConflict> {
    if step.body_kind != BodyKind::ErrorEnvelope || !(200..300).contains(&status_code) {
        return None;
    }
    Some(ReportConflict {
        kind: ConflictKind::ProductMissingReject,
        context: format!(
            "{} step {} {:?} {} HTTP {status_code} with catalog ErrorEnvelope",
            step.journey_id, step.step_index, step.route.method, step.route.path
        ),
    })
}

pub fn run_http_step_detailed(
    catalog: &Catalog,
    base: &str,
    step: &RunnableStep,
    allow_mutating: bool,
) -> HttpStepRun {
    if !matches!(step.route.method, HttpMethod::Get) && !allow_mutating {
        return HttpStepRun {
            result: Verdict::Inconclusive(format!(
                "{:?} requires --allow-mutating",
                step.route.method
            )),
            conflict: None,
        };
    }

    let Some(path) = resolve_http_path(catalog, &step.route) else {
        return HttpStepRun {
            result: Verdict::Inconclusive("unresolved or templated route path".into()),
            conflict: None,
        };
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
        Err(err) => {
            return HttpStepRun {
                result: Verdict::Fail(err),
                conflict: None,
            }
        }
    };

    let result = evaluate_http_response(
        status_code,
        &body,
        step.expected_status,
        step.body_predicate,
    );
    let conflict = product_missing_reject_conflict(step, status_code);
    HttpStepRun { result, conflict }
}

pub fn run_http_step(
    catalog: &Catalog,
    base: &str,
    step: &RunnableStep,
    allow_mutating: bool,
) -> Verdict {
    run_http_step_detailed(catalog, base, step, allow_mutating).result
}

pub fn summarize_journey(step_results: &[Verdict]) -> JourneyResult {
    if step_results.is_empty() {
        return JourneyResult::Skip;
    }
    let mut has_fail = false;
    let mut has_pass = false;
    let mut has_skip = false;
    let mut has_unasserted = false;

    for result in step_results {
        match result {
            Verdict::Fail(_) => has_fail = true,
            Verdict::Pass => has_pass = true,
            Verdict::Inconclusive(_) => has_skip = true,
            Verdict::Error => has_unasserted = true,
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

#[cfg(test)]
mod tests {
    use super::*;
    use catalog_model::journey::{BodyKind, BLAST_RADIUS_UNKNOWN};

    const TEST_PRESENCE: catalog_kernel::version::Availability =
        catalog_kernel::version::Availability {
            intro_commit: "0000000000000000000000000000000000000001",
            present_in_tags: &["1.0.0"],
            present_on_master: true,
            present_on_1_4_dev: true,
        };

    const TEST_ABSENCE: catalog_kernel::version::Availability =
        catalog_kernel::version::Availability {
            intro_commit: "0000000000000000000000000000000000000002",
            present_in_tags: &["1.5.0"],
            present_on_master: true,
            present_on_1_4_dev: false,
        };

    use catalog_kernel::id::service::ServiceId;
    use catalog_kernel::provenance::{GroundedItem, Provenance};
    use catalog_model::journey::{JourneyStep, Visibility};

    const DOC: Provenance = Provenance::doc("test.md", 1, "");

    fn test_journey(availability: catalog_kernel::version::Availability) -> UseCase {
        UseCase {
            id: JourneyId::ConnectToWifiNetwork,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions: GroundedSet::known(&[]),
            steps: GroundedSet::known(&[]),
            availability,
            blast_radius: BLAST_RADIUS_UNKNOWN,
            chains_from: None,
        }
    }

    fn test_dut(tag: &str) -> DutVersion {
        DutVersion {
            repository: "bluerobotics/blueos-core".into(),
            tag: tag.into(),
            digest: None,
        }
    }

    #[test]
    fn journey_availability_skip_reports_reason_when_absent() {
        let journey = test_journey(TEST_ABSENCE);
        let reason =
            journey_availability_skip(&journey, &test_dut("1.4-dev")).expect("expected skip");
        assert!(
            reason.contains("intro 000000000000"),
            "reason should cite intro_commit: {reason}"
        );
        assert!(
            reason.contains("first tag 1.5.0"),
            "reason should cite first_tag/present_in_tags: {reason}"
        );
        assert_eq!(
            reason,
            "not present on 1.4-dev (intro 000000000000; first tag 1.5.0)"
        );
    }

    #[test]
    fn journey_availability_skip_none_when_present() {
        let journey = test_journey(TEST_PRESENCE);
        assert_eq!(
            journey_availability_skip(&journey, &test_dut("1.0.0")),
            None
        );
        assert_eq!(
            journey_availability_skip(&journey, &test_dut("master")),
            None
        );
    }

    #[test]
    fn http_journeys_are_http_automatable() {
        let catalog = Catalog::bootstrap();
        for journey in http_journeys(&catalog) {
            assert!(http_automatable(journey));
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
                        catalog_model::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            body_kind: BodyKind::Unknown,
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
        let journey = UseCase {
            id: JourneyId::ConnectToWifiNetwork,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions: GroundedSet::known(&[]),
            steps: GroundedSet::known(STEPS),
            availability: TEST_PRESENCE,
            blast_radius: BLAST_RADIUS_UNKNOWN,
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
    fn parse_dut_version_json_extracts_fields() {
        let body = r#"{"repository":"bluerobotics/blueos-core","tag":"master","sha":"sha256:abc"}"#;
        let dut = parse_dut_version_json(body).expect("parse");
        assert_eq!(dut.repository, "bluerobotics/blueos-core");
        assert_eq!(dut.tag, "master");
        assert_eq!(dut.digest.as_deref(), Some("sha256:abc"));
        let json = dut_version_current_json(&dut);
        assert_eq!(
            json,
            r#"{"repository":"bluerobotics/blueos-core","tag":"master"}"#
        );
    }

    #[test]
    fn smoke_core_alias_tags_running_image_not_master() {
        assert!(
            !SMOKE_CORE_SWITCH_TAG_QUERY.contains("%3Amaster"),
            "switch alias must not docker-tag from :master"
        );
        assert!(
            !SMOKE_LOCAL_VERSION_TAG_QUERY.contains("%3Amaster"),
            "delete alias must not docker-tag from :master"
        );
        assert!(SMOKE_CORE_SWITCH_TAG_QUERY.contains("smoke-catalog-switch"));
        assert!(SMOKE_CORE_SWITCH_JSON.contains(SMOKE_CORE_SWITCH_TAG));
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
            Verdict::Error
        );
    }

    #[test]
    fn evaluate_http_response_passes_known_status_and_contains_predicate() {
        assert_eq!(
            evaluate_http_response(200, r#"{"online": true}"#, Some(200), Some("contains:true")),
            Verdict::Pass
        );
        assert!(matches!(
            evaluate_http_response(404, "{}", Some(200), None),
            Verdict::Fail(_)
        ));
    }

    #[test]
    fn evaluate_http_response_fails_on_streamed_fragment_error() {
        let body = concat!(
            r#"{"fragment": 0, "status": 500, "data": null, "error": "Extension williangalvani.example1 not found"}"#,
            "|\n\n|",
        );
        let result = evaluate_http_response(200, body, Some(200), None);
        assert!(
            matches!(&result, Verdict::Fail(message) if message.contains("williangalvani.example1")),
            "expected Fail with extension name, got {result:?}"
        );
    }

    #[test]
    fn evaluate_http_response_passes_successful_fragment_stream() {
        let body = concat!(
            r#"{"fragment": 0, "status": 200, "data": "bG9n", "error": null}"#,
            "|\n\n|",
            r#"{"fragment": 1, "status": 200, "data": "ZG9uZQ==", "error": null}"#,
            "|\n\n|",
        );
        assert_eq!(
            evaluate_http_response(200, body, Some(200), None),
            Verdict::Pass
        );
    }

    #[test]
    fn evaluate_http_response_passes_fragment_stream_with_heartbeats() {
        let body = concat!(
            r#"{"fragment": -1, "status": 200, "data": "aGVhcnRiZWF0", "error": null}"#,
            "|\n\n|",
            r#"{"fragment": 0, "status": 200, "data": "bG9n", "error": null}"#,
            "|\n\n|",
        );
        assert_eq!(
            evaluate_http_response(200, body, Some(200), None),
            Verdict::Pass
        );
    }

    #[test]
    fn evaluate_http_response_passes_non_stream_json_negative_probe() {
        let body = r#"{"detail":"Extension np.no.such.extension not found"}"#;
        assert_eq!(
            evaluate_http_response(404, body, Some(404), None),
            Verdict::Pass
        );
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
            body_kind: BodyKind::Unknown,
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
            body_kind: BodyKind::Unknown,
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
        assert!(journey_http_requires_base(false, true, false, false, false, None).is_err());
        assert!(journey_http_requires_base(false, false, true, false, false, None).is_err());
        assert!(journey_http_requires_base(false, false, false, true, false, None).is_err());
        assert!(journey_http_requires_base(false, false, false, false, true, None).is_err());
        assert!(journey_http_requires_base(true, false, false, true, false, None).is_ok());
        assert!(journey_http_requires_base(true, false, false, false, false, None).is_ok());
        assert!(journey_http_requires_base(false, false, false, false, false, None).is_err());
        assert!(
            journey_http_requires_base(false, true, false, false, false, Some("http://pi")).is_ok()
        );
        assert!(
            journey_http_requires_base(false, false, true, false, false, Some("http://pi")).is_ok()
        );
        assert!(
            journey_http_requires_base(false, false, false, true, false, Some("http://pi")).is_ok()
        );
        assert!(
            journey_http_requires_base(false, false, false, false, true, Some("http://pi")).is_ok()
        );
    }

    #[test]
    fn journey_http_smoke_modes_are_mutually_exclusive() {
        assert!(journey_http_mode_conflict(true, true, false, false).is_err());
        assert!(journey_http_mode_conflict(true, false, true, false).is_err());
        assert!(journey_http_mode_conflict(false, true, true, false).is_err());
        assert!(journey_http_mode_conflict(true, false, false, true).is_err());
        assert!(journey_http_mode_conflict(true, false, false, false).is_ok());
        assert!(journey_http_mode_conflict(false, true, false, false).is_ok());
        assert!(journey_http_mode_conflict(false, false, true, false).is_ok());
        assert!(journey_http_mode_conflict(false, false, false, true).is_ok());
        assert!(journey_http_mode_conflict(false, false, false, false).is_ok());
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
                        catalog_model::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            body_kind: BodyKind::Unknown,
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
                        catalog_model::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            body_kind: BodyKind::Unknown,
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
                        catalog_model::journey::StepOutcome {
                            expected_status: Some(204),
                            body_predicate: None,
                            body_kind: BodyKind::Unknown,
                            transition: None,
                        },
                        DOC,
                    )),
                },
                DOC,
            ),
        ];
        let journey = UseCase {
            id: JourneyId::ChangeUiThemeColor,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions: GroundedSet::known(&[]),
            steps: GroundedSet::known(STEPS),
            availability: TEST_PRESENCE,
            blast_radius: BLAST_RADIUS_UNKNOWN,
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
                        catalog_model::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            body_kind: BodyKind::Unknown,
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
                        catalog_model::journey::StepOutcome {
                            expected_status: Some(200),
                            body_predicate: None,
                            body_kind: BodyKind::Unknown,
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
        let journey = UseCase {
            id: JourneyId::ConnectToWifiNetwork,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions: GroundedSet::known(&[]),
            steps: GroundedSet::known(STEPS),
            availability: TEST_PRESENCE,
            blast_radius: BLAST_RADIUS_UNKNOWN,
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

    #[test]
    fn dut_profile_for_host_maps_lab_ips() {
        let play = dut_profile_for_host("http://192.168.0.177");
        assert!(play.sacrificial);
        assert!(!play.never_strand_mgmt);

        let vehicle = dut_profile_for_host("192.168.2.2");
        assert!(!vehicle.sacrificial);
        assert!(vehicle.never_strand_mgmt);

        let field87 = dut_profile_for_host("192.168.0.87");
        assert!(!field87.sacrificial);
        assert!(!field87.never_strand_mgmt);

        let field124 = dut_profile_for_host("http://192.168.0.124/");
        assert!(!field124.sacrificial);
        assert!(!field124.never_strand_mgmt);

        // Same vehicle as 192.168.2.2, routed instead of tethered: its state is expendable, but
        // the management link it is reached over is not.
        let routed_vehicle = dut_profile_for_host("http://192.168.0.88");
        assert!(routed_vehicle.sacrificial);
        assert!(routed_vehicle.never_strand_mgmt);
    }

    #[test]
    fn journey_profile_skip_refuses_strand_and_destructive_by_host() {
        use catalog_core::catalog::Catalog;
        use catalog_kernel::provenance::Grounded;

        let catalog = Catalog::bootstrap();
        let strand = catalog
            .journey_by_id(&JourneyId::AssignStaticIpAddress)
            .expect("assign_static_ip_address");
        let destructive = catalog
            .journeys()
            .iter()
            .find(|journey| {
                matches!(
                    journey.blast_radius,
                    Grounded::Known {
                        value: BlastRadius::Destructive,
                        ..
                    }
                )
            })
            .expect("destructive journey");
        let safe = catalog
            .journey_by_id(&JourneyId::ConnectToWifiNetwork)
            .expect("connect_to_wifi_network");

        let vehicle = dut_profile_for_host("192.168.2.2");
        assert!(journey_profile_skip(strand, &vehicle).is_some());
        assert!(journey_profile_skip(destructive, &vehicle).is_some());
        assert!(journey_profile_skip(safe, &vehicle).is_none());

        let play = dut_profile_for_host("192.168.0.177");
        assert!(journey_profile_skip(destructive, &play).is_none());

        let field87 = dut_profile_for_host("192.168.0.87");
        assert!(journey_profile_skip(destructive, &field87).is_some());
        assert!(journey_profile_skip(safe, &field87).is_none());
    }

    #[test]
    fn effect_read_enabled_skips_none_and_wifi_rf() {
        assert!(!effect_read_enabled(None, false));
        assert!(effect_read_enabled(Some(0), false));
        assert!(!effect_read_enabled(Some(0), true));
    }

    #[test]
    fn effect_observation_changed_uses_raw_body_without_predicate() {
        assert!(!effect_observation_changed("same", "same", None));
        assert!(effect_observation_changed("before", "after", None));
    }

    #[test]
    fn effect_observation_changed_uses_contains_predicate() {
        let pred = Some(r#"contains:"online":true"#);
        assert!(!effect_observation_changed(
            r#"{"online":true}"#,
            r#"{"online":true}"#,
            pred
        ));
        assert!(effect_observation_changed(
            r#"{"online":false}"#,
            r#"{"online":true}"#,
            pred
        ));
        assert!(effect_observation_changed(
            r#"{"x":1}"#,
            r#"{"online":true}"#,
            pred
        ));
    }

    struct EffectReadWiredTestFixture {
        journey_id: JourneyId,
        step_index: usize,
        probe: RunnableStep,
        before_body: String,
        changed_after_body: String,
    }

    fn effect_read_wired_test_fixture() -> EffectReadWiredTestFixture {
        let journey_id = JourneyId::ChangeUiThemeColor;
        let probe = RunnableStep {
            journey_id,
            step_index: 1,
            route: RouteRef {
                service: ServiceId::Customization,
                method: HttpMethod::Get,
                path: "/theme",
                version: Some("v1.0"),
            },
            expected_status: Some(200),
            body_predicate: Some(r#"contains:"primary":"white""#),
            body_kind: BodyKind::Unknown,
            body: None,
            query: None,
            form_file: None,
        };
        EffectReadWiredTestFixture {
            journey_id,
            step_index: probe.step_index,
            probe,
            before_body: r#"{"primary":"black"}"#.into(),
            changed_after_body: r#"{"primary":"white"}"#.into(),
        }
    }

    #[test]
    fn effect_read_wired_fixture_drives_before_mutate_after_restore() {
        let fixture = effect_read_wired_test_fixture();
        assert!(effect_read_enabled(Some(fixture.step_index), false));
        assert_eq!(
            mutating_effect_read_phases(effect_read_enabled(Some(fixture.step_index), false)),
            &["before", "mutate", "after", "restore"]
        );
        let before = EffectReadBefore {
            probe: fixture.probe.clone(),
            before_body: fixture.before_body.clone(),
        };
        let changed =
            effect_read_after_observed(fixture.journey_id, &before, &fixture.changed_after_body);
        assert!(changed.conflict.is_none());
        assert_eq!(changed.result, Verdict::Pass);
    }

    #[test]
    fn effect_read_stale_identical_body_yields_effect_not_applied() {
        let fixture = effect_read_wired_test_fixture();
        let before = EffectReadBefore {
            probe: fixture.probe,
            before_body: fixture.before_body.clone(),
        };
        let after = effect_read_after_observed(fixture.journey_id, &before, &fixture.before_body);
        assert_eq!(
            after.conflict.as_ref().map(|conflict| conflict.kind),
            Some(ConflictKind::EffectNotApplied)
        );
        assert!(matches!(after.result, Verdict::Fail(_)));
    }

    #[test]
    fn effect_read_phase_order_none_skips_before_after() {
        let phases = mutating_effect_read_phases(effect_read_enabled(None, false));
        assert_eq!(phases, &["mutate", "restore"]);
    }

    #[test]
    fn effect_read_phase_order_includes_before_after_when_enabled() {
        let phases = mutating_effect_read_phases(effect_read_enabled(Some(0), false));
        assert_eq!(phases, &["before", "mutate", "after", "restore"]);
    }

    #[test]
    fn product_missing_reject_on_2xx_with_error_envelope_catalog() {
        let step = RunnableStep {
            journey_id: JourneyId::InspectDiskUsage,
            step_index: 1,
            route: RouteRef {
                service: ServiceId::DiskUsage,
                method: HttpMethod::Get,
                path: "/disk/usage",
                version: Some("v1.0"),
            },
            expected_status: Some(200),
            body_predicate: Some("\"detail\""),
            body_kind: BodyKind::ErrorEnvelope,
            body: None,
            query: None,
            form_file: None,
        };
        let conflict = product_missing_reject_conflict(&step, 200).expect("conflict");
        assert_eq!(conflict.kind, ConflictKind::ProductMissingReject);
        assert!(conflict.context.contains("inspect_disk_usage"));
        assert!(conflict.context.contains("ErrorEnvelope"));
        assert!(product_missing_reject_conflict(&step, 404).is_none());
        let mut payload_step = step;
        payload_step.body_kind = BodyKind::Payload;
        assert!(product_missing_reject_conflict(&payload_step, 200).is_none());
    }
}
