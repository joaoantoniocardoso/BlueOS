//! Host WiFi RF control for catalog Tier-2 wifi/hotspot smoke.
//!
//! Drives NetworkManager on the runner machine (AP for client journeys, station
//! for hotspot journeys). Scripts live under `catalog/harness/wifi/`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use crate::id::JourneyId;

/// Must match `catalog/harness/wifi/config.example.env` WPA2 defaults.
pub const SMOKE_CLIENT_SSID: &str = "BlueOS-Hotspot";
pub const SMOKE_CLIENT_PSK: &str = "changeme1234";
/// Must match `E2E_HOTSPOT_*` defaults (credentials we set on the DUT soft-AP).
pub const SMOKE_HOTSPOT_SSID: &str = "BlueOS-E2E-Hotspot";
pub const SMOKE_HOTSPOT_PSK: &str = "changeme1234";

pub const CONNECT_SMOKE_BODY: &str = r#"{"ssid":"BlueOS-Hotspot","password":"changeme1234"}"#;
pub const HOTSPOT_CREDENTIALS_SMOKE_BODY: &str =
    r#"{"ssid":"BlueOS-E2E-Hotspot","password":"changeme1234"}"#;
pub const WRONG_PASSWORD_SMOKE_BODY: &str =
    r#"{"ssid":"BlueOS-Hotspot","password":"definitely-wrong-password-xyz"}"#;
pub const EMPTY_PASSWORD_SMOKE_BODY: &str = r#"{"ssid":"BlueOS-Hotspot","password":""}"#;

static HARNESS_DIR: OnceLock<PathBuf> = OnceLock::new();

fn harness_dir() -> &'static Path {
    HARNESS_DIR.get_or_init(|| {
        if let Ok(override_dir) = std::env::var("BLUEOS_WIFI_HARNESS_DIR") {
            return PathBuf::from(override_dir);
        }
        // catalog/src/wifi_rf.rs → ../../harness/wifi from CARGO_MANIFEST_DIR
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("harness/wifi")
    })
}

/// True when this runner can drive host RF (nmcli + configured wifi iface).
pub fn host_rf_available() -> bool {
    if Command::new("nmcli").arg("-v").output().is_err() {
        return false;
    }
    let iface = host_wifi_iface();
    Path::new("/sys/class/net").join(&iface).exists()
}

pub fn host_wifi_iface() -> String {
    std::env::var("HOST_WIFI_IFACE").unwrap_or_else(|_| {
        // Prefer config.env / example; fall back to common laptop name.
        read_config_value("HOST_WIFI_IFACE").unwrap_or_else(|| "wlp8s0".to_string())
    })
}

fn read_config_value(key: &str) -> Option<String> {
    let dir = harness_dir();
    for name in ["config.env", "config.example.env"] {
        let path = dir.join(name);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                if k.trim() == key {
                    return Some(v.trim().trim_matches('"').to_string());
                }
            }
        }
    }
    None
}

fn run_script(script: &str, args: &[&str]) -> Result<(), String> {
    let path = harness_dir().join(script);
    if !path.is_file() {
        return Err(format!("wifi harness script missing: {}", path.display()));
    }
    let output = Command::new(&path)
        .args(args)
        .current_dir(harness_dir())
        .env("HOST_WIFI_IFACE", host_wifi_iface())
        .output()
        .map_err(|err| format!("spawn {}: {err}", path.display()))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    Err(format!(
        "{} {:?} failed (status {:?}): {stderr}{stdout}",
        script,
        args,
        output.status.code()
    ))
}

pub fn host_ap_ensure() -> Result<(), String> {
    run_script("host-ap.sh", &["ensure"])
}

pub fn host_ap_up(mode: &str) -> Result<(), String> {
    run_script("host-ap.sh", &["up", mode])
}

pub fn host_ap_down() -> Result<(), String> {
    run_script("host-ap.sh", &["down"])
}

pub fn host_station_up(ssid: &str, psk: &str) -> Result<(), String> {
    run_script("host-station.sh", &["up", ssid, psk])
}

pub fn host_station_down() -> Result<(), String> {
    run_script("host-station.sh", &["down"])
}

pub fn host_station_wait_lease(timeout_sec: &str) -> Result<String, String> {
    let path = harness_dir().join("host-station.sh");
    let output = Command::new(&path)
        .args(["wait-lease", timeout_sec])
        .current_dir(harness_dir())
        .env("HOST_WIFI_IFACE", host_wifi_iface())
        .output()
        .map_err(|err| format!("spawn host-station wait-lease: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("host-station wait-lease failed: {stderr}"));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn host_scan_has_ssid(ssid: &str) -> Result<(), String> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(45);
    let mut last = String::new();
    while std::time::Instant::now() < deadline {
        match run_script("host-station.sh", &["scan-has", ssid]) {
            Ok(()) => return Ok(()),
            Err(err) => last = err,
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    Err(format!("SSID `{ssid}` not seen within 45s ({last})"))
}

/// Journeys that need the runner host to act as AP (BlueOS is the WiFi client).
pub fn needs_host_ap(journey_id: JourneyId) -> bool {
    matches!(
        journey_id,
        JourneyId::ConnectToWifiNetwork
            | JourneyId::ConnectToHiddenWifiNetwork
            | JourneyId::DisconnectFromWifiNetwork
            | JourneyId::ForgetSavedWifiNetwork
            | JourneyId::ForceWifiNetworkPassword
            | JourneyId::ReconnectToSavedWifiNetwork
            | JourneyId::RejectInvalidWifiCredentials
            | JourneyId::DetectWifiApLoss
            | JourneyId::AutoconnectToSavedWifiNetwork
    )
}

/// Journeys that need the runner host to join the BlueOS soft-AP.
pub fn needs_host_station(journey_id: JourneyId) -> bool {
    matches!(journey_id, JourneyId::ToggleHotspot)
}

pub fn connect_smoke_body() -> &'static str {
    CONNECT_SMOKE_BODY
}

pub fn hotspot_credentials_smoke_body() -> &'static str {
    HOTSPOT_CREDENTIALS_SMOKE_BODY
}

/// Bring up host RF required before a journey's HTTP mutate steps.
pub fn rf_setup(journey_id: JourneyId) -> Result<(), String> {
    if needs_host_ap(journey_id) {
        host_ap_ensure()?;
        host_ap_up("wpa2")?;
        return Ok(());
    }
    if needs_host_station(journey_id) {
        // Free host radio so it can join the DUT AP.
        let _ = host_ap_down();
        let _ = host_station_down();
        return Ok(());
    }
    Ok(())
}

/// Tear down host RF after a journey (best-effort after HTTP teardown).
pub fn rf_teardown(journey_id: JourneyId) -> Result<(), String> {
    if needs_host_ap(journey_id) {
        host_ap_down()?;
        return Ok(());
    }
    if needs_host_station(journey_id) {
        let _ = host_station_down();
        return Ok(());
    }
    Ok(())
}

/// After ToggleHotspot enables the DUT AP: host scans, associates, waits for lease.
pub fn rf_verify_hotspot_join() -> Result<String, String> {
    host_scan_has_ssid(SMOKE_HOTSPOT_SSID)?;
    host_station_up(SMOKE_HOTSPOT_SSID, SMOKE_HOTSPOT_PSK)?;
    host_station_wait_lease("45")
}

/// GET /wifi-manager/v1.0/status → associated SSID (None if idle / missing).
pub fn dut_status_ssid(blueos_base: &str) -> Result<Option<String>, String> {
    let base = blueos_base.trim_end_matches('/');
    let url = format!("{base}/wifi-manager/v1.0/status");
    let output = Command::new("curl")
        .args(["-sS", "-m", "15", "-w", "\n%{http_code}", url.as_str()])
        .output()
        .map_err(|err| format!("curl status: {err}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let (body, code) = text.rsplit_once('\n').unwrap_or((text.as_ref(), "000"));
    if code.trim() != "200" {
        return Err(format!("GET /status HTTP {code}: {body}"));
    }
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|err| format!("parse /status: {err}"))?;
    Ok(value
        .get("ssid")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string))
}

fn wait_dut_ssid(
    blueos_base: &str,
    want: Option<&str>,
    timeout_sec: u64,
) -> Result<Option<String>, String> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_sec);
    let mut last = None;
    while std::time::Instant::now() < deadline {
        last = dut_status_ssid(blueos_base)?;
        match (want, last.as_deref()) {
            (None, None) => return Ok(None),
            (Some(expected), Some(got)) if got == expected => return Ok(last),
            (None, Some(_)) | (Some(_), None) | (Some(_), Some(_)) => {}
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    Err(format!(
        "timeout waiting for status ssid={want:?} (last={last:?})"
    ))
}

/// Setup already associated the DUT to [`SMOKE_CLIENT_SSID`]. Drop the host AP and
/// wait until BlueOS notices (status SSID clears).
pub fn run_detect_ap_loss(blueos_base: &str) -> Result<(), String> {
    let associated = dut_status_ssid(blueos_base)?;
    if associated.as_deref() != Some(SMOKE_CLIENT_SSID) {
        return Err(format!(
            "expected associated to {SMOKE_CLIENT_SSID} before AP drop, got {associated:?}"
        ));
    }
    host_ap_down()?;
    wait_dut_ssid(blueos_base, None, 90)?;
    Ok(())
}

/// Setup already associated + saved. Drop AP, wait disconnect, restore AP, wait
/// autoconnect without a new POST /connect.
pub fn run_autoconnect(blueos_base: &str) -> Result<(), String> {
    let associated = dut_status_ssid(blueos_base)?;
    if associated.as_deref() != Some(SMOKE_CLIENT_SSID) {
        return Err(format!(
            "expected associated to {SMOKE_CLIENT_SSID} before AP drop, got {associated:?}"
        ));
    }
    host_ap_down()?;
    wait_dut_ssid(blueos_base, None, 90)?;
    host_ap_up("wpa2")?;
    wait_dut_ssid(blueos_base, Some(SMOKE_CLIENT_SSID), 120)?;
    Ok(())
}

pub fn is_rf_status_journey(journey_id: JourneyId) -> bool {
    matches!(
        journey_id,
        JourneyId::DetectWifiApLoss | JourneyId::AutoconnectToSavedWifiNetwork
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harness_dir_contains_host_ap() {
        assert!(
            harness_dir().join("host-ap.sh").is_file(),
            "expected host-ap.sh under {}",
            harness_dir().display()
        );
    }

    #[test]
    fn connect_body_matches_default_ssid() {
        assert!(connect_smoke_body().contains(SMOKE_CLIENT_SSID));
        assert!(connect_smoke_body().contains(SMOKE_CLIENT_PSK));
    }
}
