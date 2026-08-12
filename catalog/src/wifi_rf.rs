//! Host WiFi RF control for catalog Tier-2 wifi/hotspot smoke.
//!
//! Drives NetworkManager on the runner machine (AP for client journeys, station
//! for hotspot journeys) via `nmcli` / `ip` — logic ported from
//! `catalog/harness/wifi/{lib,host-ap,host-station}.sh`.

use std::net::Ipv4Addr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use crate::id::JourneyId;

/// Must match `catalog/harness/wifi/config.example.env` WPA2 defaults.
pub const SMOKE_CLIENT_SSID: &str = "BlueOS-Hotspot";
pub const SMOKE_CLIENT_PSK: &str = "changeme1234";
/// Must match `E2E_HOTSPOT_*` defaults (credentials we set on the DUT soft-AP).
pub const SMOKE_HOTSPOT_SSID: &str = "BlueOS-E2E-Hotspot";
pub const SMOKE_HOTSPOT_PSK: &str = "changeme1234";
/// Must match `HOTSPOT_GATEWAY` in `catalog/harness/wifi/config.example.env`.
pub const SMOKE_HOTSPOT_GATEWAY: &str = "192.168.42.1";

pub const CONNECT_SMOKE_BODY: &str = r#"{"ssid":"BlueOS-Hotspot","password":"changeme1234"}"#;
pub const HOTSPOT_CREDENTIALS_SMOKE_BODY: &str =
    r#"{"ssid":"BlueOS-E2E-Hotspot","password":"changeme1234"}"#;
pub const WRONG_PASSWORD_SMOKE_BODY: &str =
    r#"{"ssid":"BlueOS-Hotspot","password":"definitely-wrong-password-xyz"}"#;
pub const EMPTY_PASSWORD_SMOKE_BODY: &str = r#"{"ssid":"BlueOS-Hotspot","password":""}"#;

#[derive(Debug, Clone)]
struct HostRfConfig {
    iface: String,
    station_conns: Vec<String>,
    open_conn: String,
    open_ssid: String,
    wpa_conn: String,
    wpa_ssid: String,
    wpa2_conn: String,
    wpa2_ssid: String,
    wpa2_psk: String,
    transition_conn: String,
    transition_ssid: String,
    wpa3_conn: String,
    wpa3_ssid: String,
    wpa3_psk: String,
    ap_channel: String,
    ap_gateway: Ipv4Addr,
    ap_prefix: u8,
    dhcp_range: String,
    hotspot_gateway: Ipv4Addr,
    hotspot_prefix: u8,
    host_station_conn: String,
    e2e_hotspot_ssid: String,
    e2e_hotspot_psk: String,
    ping_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ApMode {
    Open,
    Wpa,
    Wpa2,
    Transition,
    Wpa3,
}

impl ApMode {
    fn parse(mode: &str) -> Result<Self, String> {
        match mode {
            "open" => Ok(Self::Open),
            "wpa" => Ok(Self::Wpa),
            "wpa2" => Ok(Self::Wpa2),
            "transition" => Ok(Self::Transition),
            "wpa3" => Ok(Self::Wpa3),
            other => Err(format!("unknown AP mode: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DutWifiStatus {
    pub ssid: Option<String>,
    pub ip_address: Option<String>,
}

static CONFIG_DIR: OnceLock<PathBuf> = OnceLock::new();
static CONFIG: OnceLock<HostRfConfig> = OnceLock::new();

fn config_dir() -> &'static Path {
    CONFIG_DIR.get_or_init(|| {
        if let Ok(override_dir) = std::env::var("BLUEOS_WIFI_HARNESS_DIR") {
            return PathBuf::from(override_dir);
        }
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("harness/wifi")
    })
}

fn config() -> &'static HostRfConfig {
    CONFIG.get_or_init(HostRfConfig::load)
}

impl HostRfConfig {
    fn load() -> Self {
        let env_or = |key: &str, default: &str| {
            std::env::var(key)
                .ok()
                .filter(|v| !v.is_empty())
                .or_else(|| read_config_file_value(key))
                .unwrap_or_else(|| default.to_string())
        };
        let parse_ip = |key: &str, default: &str| {
            env_or(key, default)
                .parse::<Ipv4Addr>()
                .unwrap_or_else(|_| default.parse().expect("default IPv4"))
        };
        let parse_u8 = |key: &str, default: u8| {
            env_or(key, &default.to_string())
                .parse::<u8>()
                .unwrap_or(default)
        };
        let parse_u32 = |key: &str, default: u32| {
            env_or(key, &default.to_string())
                .parse::<u32>()
                .unwrap_or(default)
        };
        let e2e_psk = env_or("E2E_PSK", SMOKE_CLIENT_PSK);
        let station_conns = env_or("HOST_STATION_CONNS", "")
            .split_whitespace()
            .map(str::to_string)
            .filter(|s| !s.is_empty())
            .collect();
        Self {
            iface: env_or("HOST_WIFI_IFACE", "wlp8s0"),
            station_conns,
            open_conn: env_or("OPEN_CONN", "E2E-Open"),
            open_ssid: env_or("OPEN_SSID", "BlueOS-E2E-Open"),
            wpa_conn: env_or("WPA_CONN", "E2E-WPA"),
            wpa_ssid: env_or("WPA_SSID", "BlueOS-E2E-WPA"),
            wpa2_conn: env_or("WPA2_CONN", "Hotspot-WPA2"),
            wpa2_ssid: env_or("WPA2_SSID", SMOKE_CLIENT_SSID),
            wpa2_psk: env_or("WPA2_PSK", &e2e_psk),
            transition_conn: env_or("TRANSITION_CONN", "E2E-Transition"),
            transition_ssid: env_or("TRANSITION_SSID", "BlueOS-E2E-Transition"),
            wpa3_conn: env_or("WPA3_CONN", "Hotspot"),
            wpa3_ssid: env_or("WPA3_SSID", "BlueOS-Hotspot-WPA3"),
            wpa3_psk: env_or("WPA3_PSK", &e2e_psk),
            ap_channel: env_or("AP_CHANNEL", "6"),
            ap_gateway: parse_ip("AP_GATEWAY", "10.42.0.1"),
            ap_prefix: parse_u8("AP_PREFIX", 24),
            dhcp_range: env_or("DHCP_RANGE", "10.42.0.10,10.42.0.200"),
            hotspot_gateway: parse_ip("HOTSPOT_GATEWAY", SMOKE_HOTSPOT_GATEWAY),
            hotspot_prefix: parse_u8("HOTSPOT_PREFIX", 24),
            host_station_conn: env_or("HOST_STATION_E2E_CONN", "BlueOS-E2E-Station"),
            e2e_hotspot_ssid: env_or("E2E_HOTSPOT_SSID", SMOKE_HOTSPOT_SSID),
            e2e_hotspot_psk: env_or("E2E_HOTSPOT_PSK", SMOKE_HOTSPOT_PSK),
            ping_count: parse_u32("PING_COUNT", 3),
        }
    }

    fn mode_conn(&self, mode: ApMode) -> &str {
        match mode {
            ApMode::Open => &self.open_conn,
            ApMode::Wpa => &self.wpa_conn,
            ApMode::Wpa2 => &self.wpa2_conn,
            ApMode::Transition => &self.transition_conn,
            ApMode::Wpa3 => &self.wpa3_conn,
        }
    }

    fn mode_ssid(&self, mode: ApMode) -> &str {
        match mode {
            ApMode::Open => &self.open_ssid,
            ApMode::Wpa => &self.wpa_ssid,
            ApMode::Wpa2 => &self.wpa2_ssid,
            ApMode::Transition => &self.transition_ssid,
            ApMode::Wpa3 => &self.wpa3_ssid,
        }
    }

    fn all_ap_conns(&self) -> Vec<&str> {
        vec![
            self.open_conn.as_str(),
            self.wpa_conn.as_str(),
            self.wpa2_conn.as_str(),
            self.transition_conn.as_str(),
            self.wpa3_conn.as_str(),
            "Hotspot-1",
            "Hotspot-WPA3",
        ]
    }
}

fn read_config_file_value(key: &str) -> Option<String> {
    let dir = config_dir();
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

fn nmcli(args: &[&str]) -> Result<String, String> {
    let output = Command::new("nmcli")
        .args(args)
        .output()
        .map_err(|err| format!("nmcli {}: {err}", args.join(" ")))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        return Ok(stdout);
    }
    Err(format!(
        "nmcli {} failed (status {:?}): {stderr}{stdout}",
        args.join(" "),
        output.status.code()
    ))
}

fn nmcli_ok(args: &[&str]) -> Result<(), String> {
    nmcli(args).map(|_| ())
}

fn nmcli_ignore(args: &[&str]) {
    let _ = nmcli(args);
}

fn connection_exists(name: &str) -> bool {
    nmcli_ok(&["connection", "show", name]).is_ok()
}

fn ipv4_in_prefix(ip: Ipv4Addr, gateway: Ipv4Addr, prefix: u8) -> bool {
    let prefix = prefix.min(32);
    let mask = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - u32::from(prefix))
    };
    (u32::from(ip) & mask) == (u32::from(gateway) & mask)
}

fn iface_has_gateway(iface: &str, gateway: Ipv4Addr, prefix: u8) -> bool {
    let output = Command::new("ip")
        .args(["-4", "addr", "show", iface])
        .output();
    let Ok(output) = output else {
        return false;
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let needle = format!("{gateway}/{prefix}");
    text.contains(&needle)
}

fn iface_ipv4s(iface: &str) -> Vec<Ipv4Addr> {
    let output = Command::new("ip")
        .args(["-4", "-o", "addr", "show", "dev", iface])
        .output();
    let Ok(output) = output else {
        return Vec::new();
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| {
            // ... inet 192.168.42.10/24 ...
            let mut parts = line.split_whitespace();
            while let Some(tok) = parts.next() {
                if tok == "inet" {
                    let addr = parts.next()?;
                    let ip = addr.split('/').next()?;
                    return ip.parse().ok();
                }
            }
            None
        })
        .collect()
}

fn disable_station_autoconnect(cfg: &HostRfConfig) {
    for conn in &cfg.station_conns {
        if !connection_exists(conn) {
            continue;
        }
        nmcli_ignore(&["connection", "modify", conn, "connection.autoconnect", "no"]);
        nmcli_ignore(&["connection", "down", conn]);
    }
}

fn shared_ipv4(cfg: &HostRfConfig, conn: &str) -> Result<(), String> {
    let addr = format!("{}/{}", cfg.ap_gateway, cfg.ap_prefix);
    nmcli_ok(&[
        "connection",
        "modify",
        conn,
        "ipv4.method",
        "shared",
        "ipv4.addresses",
        &addr,
        "ipv6.method",
        "ignore",
    ])?;
    // Older NetworkManager builds may lack shared-dhcp-range.
    let _ = nmcli(&[
        "connection",
        "modify",
        conn,
        "ipv4.shared-dhcp-range",
        &cfg.dhcp_range,
    ]);
    Ok(())
}

fn ensure_ap_base(cfg: &HostRfConfig, conn: &str, ssid: &str) -> Result<(), String> {
    if !connection_exists(conn) {
        nmcli_ok(&[
            "connection",
            "add",
            "type",
            "wifi",
            "ifname",
            &cfg.iface,
            "con-name",
            conn,
            "autoconnect",
            "no",
            "ssid",
            ssid,
            "802-11-wireless.mode",
            "ap",
            "802-11-wireless.band",
            "bg",
            "ipv4.method",
            "shared",
            "ipv6.method",
            "ignore",
        ])?;
    }
    nmcli_ok(&[
        "connection",
        "modify",
        conn,
        "connection.interface-name",
        &cfg.iface,
        "connection.autoconnect",
        "no",
        "802-11-wireless.ssid",
        ssid,
        "802-11-wireless.mode",
        "ap",
        "802-11-wireless.band",
        "bg",
        "802-11-wireless.channel",
        &cfg.ap_channel,
    ])?;
    shared_ipv4(cfg, conn)
}

fn ensure_open_profile(cfg: &HostRfConfig) -> Result<(), String> {
    ensure_ap_base(cfg, &cfg.open_conn, &cfg.open_ssid)?;
    let _ = nmcli(&[
        "connection",
        "modify",
        &cfg.open_conn,
        "wifi-sec.key-mgmt",
        "none",
    ]);
    nmcli_ignore(&["connection", "modify", &cfg.open_conn, "wifi-sec.psk", ""]);
    Ok(())
}

fn ensure_wpa_profile(cfg: &HostRfConfig) -> Result<(), String> {
    ensure_ap_base(cfg, &cfg.wpa_conn, &cfg.wpa_ssid)?;
    nmcli_ok(&[
        "connection",
        "modify",
        &cfg.wpa_conn,
        "wifi-sec.key-mgmt",
        "wpa-psk",
        "wifi-sec.proto",
        "wpa",
        "wifi-sec.pairwise",
        "tkip",
        "wifi-sec.group",
        "tkip",
        "wifi-sec.pmf",
        "1",
        "wifi-sec.psk",
        &cfg.wpa2_psk,
    ])
}

fn ensure_wpa2_profile(cfg: &HostRfConfig) -> Result<(), String> {
    ensure_ap_base(cfg, &cfg.wpa2_conn, &cfg.wpa2_ssid)?;
    nmcli_ok(&[
        "connection",
        "modify",
        &cfg.wpa2_conn,
        "wifi-sec.key-mgmt",
        "wpa-psk",
        "wifi-sec.proto",
        "rsn",
        "wifi-sec.pairwise",
        "ccmp",
        "wifi-sec.group",
        "ccmp",
        "wifi-sec.pmf",
        "1",
        "wifi-sec.psk",
        &cfg.wpa2_psk,
    ])
}

fn ensure_transition_profile(cfg: &HostRfConfig) -> Result<(), String> {
    ensure_ap_base(cfg, &cfg.transition_conn, &cfg.transition_ssid)?;
    nmcli_ok(&[
        "connection",
        "modify",
        &cfg.transition_conn,
        "wifi-sec.key-mgmt",
        "wpa-psk",
        "wifi-sec.proto",
        "rsn",
        "wifi-sec.pairwise",
        "ccmp",
        "wifi-sec.group",
        "ccmp",
        "wifi-sec.pmf",
        "2",
        "wifi-sec.psk",
        &cfg.wpa2_psk,
    ])
}

fn ensure_wpa3_profile(cfg: &HostRfConfig) -> Result<(), String> {
    ensure_ap_base(cfg, &cfg.wpa3_conn, &cfg.wpa3_ssid)?;
    nmcli_ok(&[
        "connection",
        "modify",
        &cfg.wpa3_conn,
        "wifi-sec.key-mgmt",
        "sae",
        "wifi-sec.proto",
        "rsn",
        "wifi-sec.pairwise",
        "ccmp",
        "wifi-sec.group",
        "ccmp",
        "wifi-sec.pmf",
        "3",
        "wifi-sec.psk",
        &cfg.wpa3_psk,
    ])
}

fn down_ap_conns(cfg: &HostRfConfig) {
    for conn in cfg.all_ap_conns() {
        nmcli_ignore(&["connection", "down", conn]);
    }
}

fn wait_ap(cfg: &HostRfConfig, ssid: &str) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while Instant::now() < deadline {
        if iface_has_gateway(&cfg.iface, cfg.ap_gateway, cfg.ap_prefix) {
            if let Ok(active) =
                nmcli(&["-t", "-f", "DEVICE,STATE", "connection", "show", "--active"])
            {
                let needle = format!("{}:activated", cfg.iface);
                if active.lines().any(|line| line == needle) {
                    eprintln!(
                        "wifi_rf: AP up on {} {}/{} (want SSID={ssid})",
                        cfg.iface, cfg.ap_gateway, cfg.ap_prefix
                    );
                    return Ok(());
                }
            }
            if let Ok(state) = nmcli(&["-g", "GENERAL.STATE", "device", "show", &cfg.iface]) {
                if state.to_lowercase().contains("connected") {
                    eprintln!(
                        "wifi_rf: AP up on {} {}/{} (want SSID={ssid})",
                        cfg.iface, cfg.ap_gateway, cfg.ap_prefix
                    );
                    return Ok(());
                }
            }
        }
        std::thread::sleep(Duration::from_millis(400));
    }
    Err(format!(
        "AP did not become ready for SSID={ssid} on {}",
        cfg.iface
    ))
}

fn ensure_station_profile(cfg: &HostRfConfig, ssid: &str, psk: &str) -> Result<(), String> {
    let conn = &cfg.host_station_conn;
    if !connection_exists(conn) {
        return nmcli_ok(&[
            "connection",
            "add",
            "type",
            "wifi",
            "ifname",
            &cfg.iface,
            "con-name",
            conn,
            "ssid",
            ssid,
            "wifi-sec.key-mgmt",
            "wpa-psk",
            "wifi-sec.psk",
            psk,
            "ipv4.method",
            "auto",
            "ipv6.method",
            "ignore",
        ]);
    }
    nmcli_ok(&[
        "connection",
        "modify",
        conn,
        "802-11-wireless.ssid",
        ssid,
        "wifi-sec.key-mgmt",
        "wpa-psk",
        "wifi-sec.psk",
        psk,
        "ipv4.method",
        "auto",
        "connection.autoconnect",
        "no",
    ])
}

fn wait_hotspot_lease(cfg: &HostRfConfig, timeout_sec: u64) -> Result<String, String> {
    let deadline = Instant::now() + Duration::from_secs(timeout_sec);
    while Instant::now() < deadline {
        for ip in iface_ipv4s(&cfg.iface) {
            if ipv4_in_prefix(ip, cfg.hotspot_gateway, cfg.hotspot_prefix) {
                return Ok(ip.to_string());
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    Err(format!(
        "no IPv4 lease in {}/{} on {} within {timeout_sec}s",
        cfg.hotspot_gateway, cfg.hotspot_prefix, cfg.iface
    ))
}

fn wifi_scan_has_ssid(cfg: &HostRfConfig, ssid: &str) -> Result<(), String> {
    let out = nmcli(&[
        "-t", "-f", "SSID", "device", "wifi", "list", "ifname", &cfg.iface, "--rescan", "yes",
    ])?;
    if out.lines().any(|line| line == ssid) {
        return Ok(());
    }
    Err(format!("SSID `{ssid}` not in scan on {}", cfg.iface))
}

fn http_get(url: &str, timeout_sec: &str) -> Result<(String, String), String> {
    let output = Command::new("curl")
        .args(["-sS", "-m", timeout_sec, "-w", "\n%{http_code}", url])
        .output()
        .map_err(|err| format!("curl GET {url}: {err}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let (body, code) = text.rsplit_once('\n').unwrap_or((text.as_ref(), "000"));
    Ok((body.to_string(), code.trim().to_string()))
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
    config().iface.clone()
}

pub fn host_ap_ensure() -> Result<(), String> {
    let cfg = config();
    ensure_open_profile(cfg)?;
    ensure_wpa_profile(cfg)?;
    ensure_wpa2_profile(cfg)?;
    ensure_transition_profile(cfg)?;
    ensure_wpa3_profile(cfg)?;
    Ok(())
}

pub fn host_ap_up(mode: &str) -> Result<(), String> {
    let cfg = config();
    let mode = ApMode::parse(mode)?;
    disable_station_autoconnect(cfg);
    match mode {
        ApMode::Open => ensure_open_profile(cfg)?,
        ApMode::Wpa => ensure_wpa_profile(cfg)?,
        ApMode::Wpa2 => ensure_wpa2_profile(cfg)?,
        ApMode::Transition => ensure_transition_profile(cfg)?,
        ApMode::Wpa3 => ensure_wpa3_profile(cfg)?,
    }
    down_ap_conns(cfg);
    nmcli_ok(&["connection", "up", cfg.mode_conn(mode)])?;
    wait_ap(cfg, cfg.mode_ssid(mode))?;
    Ok(())
}

pub fn host_ap_down() -> Result<(), String> {
    down_ap_conns(config());
    Ok(())
}

pub fn host_station_up(ssid: &str, psk: &str) -> Result<(), String> {
    let cfg = config();
    let _ = host_ap_down();
    disable_station_autoconnect(cfg);
    ensure_station_profile(cfg, ssid, psk)?;
    nmcli_ok(&[
        "connection",
        "up",
        &cfg.host_station_conn,
        "ifname",
        &cfg.iface,
    ])
}

pub fn host_station_down() -> Result<(), String> {
    nmcli_ignore(&["connection", "down", &config().host_station_conn]);
    Ok(())
}

pub fn host_station_wait_lease(timeout_sec: &str) -> Result<String, String> {
    let timeout: u64 = timeout_sec.parse().unwrap_or(45);
    wait_hotspot_lease(config(), timeout)
}

pub fn host_scan_has_ssid(ssid: &str) -> Result<(), String> {
    let cfg = config();
    let deadline = Instant::now() + Duration::from_secs(45);
    let mut last = String::new();
    while Instant::now() < deadline {
        match wifi_scan_has_ssid(cfg, ssid) {
            Ok(()) => return Ok(()),
            Err(err) => last = err,
        }
        std::thread::sleep(Duration::from_secs(2));
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

/// GET /wifi-manager/v1.0/hotspot_credentials raw JSON body.
pub fn fetch_hotspot_credentials_json(blueos_base: &str) -> Result<String, String> {
    let base = blueos_base.trim_end_matches('/');
    let url = format!("{base}/wifi-manager/v1.0/hotspot_credentials");
    let (body, code) = http_get(&url, "15")?;
    if code != "200" {
        return Err(format!("GET /hotspot_credentials HTTP {code}: {body}"));
    }
    Ok(body.trim().to_string())
}

/// POST /wifi-manager/v1.0/hotspot_credentials with a JSON body.
pub fn post_hotspot_credentials_json(blueos_base: &str, body: &str) -> Result<(), String> {
    let base = blueos_base.trim_end_matches('/');
    let url = format!("{base}/wifi-manager/v1.0/hotspot_credentials");
    let compact = serde_json::to_string(
        &serde_json::from_str::<serde_json::Value>(body)
            .map_err(|err| format!("hotspot credentials JSON: {err}"))?,
    )
    .map_err(|err| format!("serialize hotspot credentials: {err}"))?;
    let want_ssid = serde_json::from_str::<serde_json::Value>(&compact)
        .ok()
        .and_then(|v| v.get("ssid")?.as_str().map(str::to_string));
    let output = Command::new("curl")
        .args([
            "-sS",
            "-m",
            "120",
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            compact.as_str(),
            "-w",
            "\n%{http_code}",
            url.as_str(),
        ])
        .output()
        .map_err(|err| format!("curl POST hotspot_credentials: {err}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let (resp, code) = text.rsplit_once('\n').unwrap_or((text.as_ref(), "000"));
    if code.trim() == "200" {
        return Ok(());
    }
    // Applying credentials can drop the HTTP connection (curl → 000); confirm via GET.
    std::thread::sleep(Duration::from_secs(2));
    let got = fetch_hotspot_credentials_json(blueos_base)?;
    let got_ssid = serde_json::from_str::<serde_json::Value>(&got)
        .ok()
        .and_then(|v| v.get("ssid")?.as_str().map(str::to_string));
    if want_ssid.is_some() && want_ssid == got_ssid {
        return Ok(());
    }
    Err(format!(
        "POST /hotspot_credentials HTTP {code}: {resp} (ssid now {got_ssid:?})"
    ))
}

/// Bring up host RF required before a journey's HTTP mutate steps.
pub fn rf_setup(journey_id: JourneyId) -> Result<(), String> {
    if needs_host_ap(journey_id) {
        host_ap_ensure()?;
        host_ap_up("wpa2")?;
        return Ok(());
    }
    if needs_host_station(journey_id) {
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
    let cfg = config();
    let ssid = if cfg.e2e_hotspot_ssid.is_empty() {
        SMOKE_HOTSPOT_SSID
    } else {
        cfg.e2e_hotspot_ssid.as_str()
    };
    let psk = if cfg.e2e_hotspot_psk.is_empty() {
        SMOKE_HOTSPOT_PSK
    } else {
        cfg.e2e_hotspot_psk.as_str()
    };
    host_scan_has_ssid(ssid)?;
    host_station_up(ssid, psk)?;
    host_station_wait_lease("45")
}

/// GET /wifi-manager/v1.0/status → associated SSID + lease IP.
pub fn dut_wifi_status(blueos_base: &str) -> Result<DutWifiStatus, String> {
    let base = blueos_base.trim_end_matches('/');
    let url = format!("{base}/wifi-manager/v1.0/status");
    let (body, code) = http_get(&url, "15")?;
    if code != "200" {
        return Err(format!("GET /status HTTP {code}: {body}"));
    }
    let value: serde_json::Value =
        serde_json::from_str(&body).map_err(|err| format!("parse /status: {err}"))?;
    let ssid = value
        .get("ssid")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let ip_address = value
        .get("ip_address")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    Ok(DutWifiStatus { ssid, ip_address })
}

/// GET /wifi-manager/v1.0/status → associated SSID (None if idle / missing).
pub fn dut_status_ssid(blueos_base: &str) -> Result<Option<String>, String> {
    Ok(dut_wifi_status(blueos_base)?.ssid)
}

fn wait_dut_ssid(
    blueos_base: &str,
    want: Option<&str>,
    timeout_sec: u64,
) -> Result<Option<String>, String> {
    let deadline = Instant::now() + Duration::from_secs(timeout_sec);
    let mut last = None;
    while Instant::now() < deadline {
        last = dut_status_ssid(blueos_base)?;
        match (want, last.as_deref()) {
            (None, None) => return Ok(None),
            (Some(expected), Some(got)) if got == expected => return Ok(last),
            (None, Some(_)) | (Some(_), None) | (Some(_), Some(_)) => {}
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    Err(format!(
        "timeout waiting for status ssid={want:?} (last={last:?})"
    ))
}

/// Poll until DUT reports `ssid` + non-empty `ip_address`.
pub fn wait_dut_lease(blueos_base: &str, ssid: &str, timeout_sec: u64) -> Result<String, String> {
    let deadline = Instant::now() + Duration::from_secs(timeout_sec);
    let mut last = DutWifiStatus {
        ssid: None,
        ip_address: None,
    };
    while Instant::now() < deadline {
        last = dut_wifi_status(blueos_base)?;
        if last.ssid.as_deref() == Some(ssid) {
            if let Some(ip) = last.ip_address.as_ref() {
                return Ok(ip.clone());
            }
        }
        std::thread::sleep(Duration::from_secs(1));
    }
    Err(format!(
        "timeout waiting for lease on {ssid} (last ssid={:?} ip={:?})",
        last.ssid, last.ip_address
    ))
}

pub fn host_ping(ip: &str) -> Result<(), String> {
    let count = config().ping_count.to_string();
    let output = Command::new("ping")
        .args(["-c", &count, "-W", "2", ip])
        .output()
        .map_err(|err| format!("ping {ip}: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ping {ip} failed: {stderr}"));
    }
    Ok(())
}

/// Client L3: wait for DUT wlan lease on the smoke SSID, then ping it from the host.
pub fn l3_assert_client_lease(blueos_base: &str) -> Result<String, String> {
    let ip = wait_dut_lease(blueos_base, SMOKE_CLIENT_SSID, 45)?;
    host_ping(&ip)?;
    Ok(ip)
}

/// Hotspot L3: ping BlueOS soft-AP gateway from the host station.
pub fn l3_assert_hotspot_gateway() -> Result<(), String> {
    host_ping(&config().hotspot_gateway.to_string())
}

pub fn wants_client_l3(journey_id: JourneyId) -> bool {
    matches!(
        journey_id,
        JourneyId::ConnectToWifiNetwork
            | JourneyId::ConnectToHiddenWifiNetwork
            | JourneyId::ForceWifiNetworkPassword
            | JourneyId::ReconnectToSavedWifiNetwork
            | JourneyId::AutoconnectToSavedWifiNetwork
    )
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

/// After GET /disconnect: confirm status SSID clears.
pub fn run_assert_disconnected(blueos_base: &str) -> Result<(), String> {
    wait_dut_ssid(blueos_base, None, 30)?;
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
    fn config_dir_has_example_env() {
        assert!(
            config_dir().join("config.example.env").is_file(),
            "expected config.example.env under {}",
            config_dir().display()
        );
    }

    #[test]
    fn connect_body_matches_default_ssid() {
        assert!(connect_smoke_body().contains(SMOKE_CLIENT_SSID));
        assert!(connect_smoke_body().contains(SMOKE_CLIENT_PSK));
    }

    #[test]
    fn ipv4_prefix_membership() {
        let gw: Ipv4Addr = "192.168.42.1".parse().unwrap();
        assert!(ipv4_in_prefix("192.168.42.133".parse().unwrap(), gw, 24));
        assert!(!ipv4_in_prefix("10.42.0.118".parse().unwrap(), gw, 24));
    }

    #[test]
    fn ap_mode_parse() {
        assert_eq!(ApMode::parse("wpa2").unwrap(), ApMode::Wpa2);
        assert!(ApMode::parse("nope").is_err());
    }
}
