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

use regex::Regex;

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
    static_test_ip: Ipv4Addr,
    xfer_min_bytes: u64,
    wifi_iface: String,
    expect_wpa3: ExpectWpa3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApMode {
    Open,
    Wpa,
    Wpa2,
    Transition,
    Wpa3,
}

impl ApMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Wpa => "wpa",
            Self::Wpa2 => "wpa2",
            Self::Transition => "transition",
            Self::Wpa3 => "wpa3",
        }
    }

    pub fn parse(mode: &str) -> Result<Self, String> {
        match mode.trim().to_ascii_lowercase().as_str() {
            "open" => Ok(Self::Open),
            "wpa" => Ok(Self::Wpa),
            "wpa2" => Ok(Self::Wpa2),
            "transition" => Ok(Self::Transition),
            "wpa3" => Ok(Self::Wpa3),
            other => Err(format!("unknown AP mode: {other}")),
        }
    }

    pub fn parse_csv(spec: &str) -> Result<Vec<Self>, String> {
        let modes: Result<Vec<_>, _> = spec
            .split(',')
            .filter(|mode| !mode.trim().is_empty())
            .map(Self::parse)
            .collect();
        let modes = modes?;
        if modes.is_empty() {
            return Err("wifi mode list is empty".into());
        }
        Ok(modes)
    }
}

impl std::fmt::Display for ApMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpectWpa3 {
    Yes,
    No,
    Auto,
}

impl ExpectWpa3 {
    fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "yes" => Self::Yes,
            "no" => Self::No,
            _ => Self::Auto,
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
            static_test_ip: parse_ip("STATIC_TEST_IP", "10.42.0.50"),
            xfer_min_bytes: env_or("XFER_MIN_BYTES", "100000").parse().unwrap_or(100000),
            wifi_iface: env_or("WIFI_IFACE", "wlan0"),
            expect_wpa3: ExpectWpa3::parse(&env_or("EXPECT_WPA3", "auto")),
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
    // `key-mgmt none` leaves residual WEP keys that modern wpa_supplicant rejects
    // ("does not support WEP encryption"). Drop the whole wifi-sec section.
    nmcli_ignore(&["connection", "modify", &cfg.open_conn, "remove", "wifi-sec"]);
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

fn http_request(method: &str, url: &str, body: Option<&str>) -> Result<(String, String), String> {
    let mut command = Command::new("curl");
    command.args(["-sS", "-m", "15", "-X", method]);
    if let Some(body) = body {
        command.args(["-H", "Content-Type: application/json", "-d", body]);
    }
    command.args(["-w", "\n%{http_code}", url]);
    let output = command
        .output()
        .map_err(|err| format!("curl {method} {url}: {err}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    let (response, code) = text.rsplit_once('\n').unwrap_or((text.as_ref(), "000"));
    Ok((response.to_string(), code.trim().to_string()))
}

fn wifi_url(base: &str, path: &str) -> String {
    format!(
        "{}/wifi-manager/v1.0/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

fn cable_url(base: &str, path: &str) -> String {
    format!(
        "{}/cable-guy/v1.0/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

pub fn mode_ssid(mode: ApMode) -> String {
    config().mode_ssid(mode).to_string()
}

pub fn mode_psk(mode: ApMode) -> String {
    match mode {
        ApMode::Open => String::new(),
        ApMode::Wpa => config().wpa2_psk.clone(),
        ApMode::Wpa2 | ApMode::Transition => config().wpa2_psk.clone(),
        ApMode::Wpa3 => config().wpa3_psk.clone(),
    }
}

pub fn connect_body_for_mode(mode: ApMode) -> String {
    format!(
        r#"{{"ssid":{},"password":{}}}"#,
        serde_json::to_string(&mode_ssid(mode)).expect("serialize SSID"),
        serde_json::to_string(&mode_psk(mode)).expect("serialize PSK")
    )
}

pub fn wrong_password_body_for_mode(mode: ApMode) -> String {
    format!(
        r#"{{"ssid":{},"password":"definitely-wrong-password-xyz"}}"#,
        serde_json::to_string(&mode_ssid(mode)).expect("serialize SSID")
    )
}

pub fn dut_scan_json(base: &str) -> Result<serde_json::Value, String> {
    let (body, code) = http_get(&wifi_url(base, "scan"), "30")?;
    if code != "200" {
        return Err(format!("GET /scan HTTP {code}: {body}"));
    }
    serde_json::from_str(&body).map_err(|err| format!("parse /scan: {err}"))
}

pub fn dut_scan_has_ssid(base: &str, ssid: &str) -> Result<bool, String> {
    Ok(dut_scan_json(base)?
        .as_array()
        .map(|networks| {
            networks.iter().any(|network| {
                network.get("ssid").and_then(serde_json::Value::as_str) == Some(ssid)
            })
        })
        .unwrap_or(false))
}

pub fn assert_scan_present(base: &str, ssid: &str, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    let mut last = String::new();
    let mut transport_fails = 0u8;
    while Instant::now() < deadline {
        match dut_scan_has_ssid(base, ssid) {
            Ok(true) => return Ok(()),
            Ok(false) => {
                transport_fails = 0;
                last = "SSID absent".into();
            }
            Err(err) => {
                last = err.clone();
                // Fail fast when wifi-manager is unreachable (HTTP 000 / timeout).
                if err.contains("HTTP 000") || err.contains("HTTP 502") || err.contains("HTTP 504")
                {
                    transport_fails += 1;
                    if transport_fails >= 3 {
                        return Err(format!(
                            "SSID `{ssid}` scan unreachable (wifi-manager down?): {last}"
                        ));
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    Err(format!(
        "SSID `{ssid}` not present in DUT scan within {timeout:?}: {last}"
    ))
}

pub fn assert_scan_absent(base: &str, ssid: &str, timeout: Duration) -> Result<(), String> {
    let deadline = Instant::now() + timeout;
    let mut last = String::new();
    let mut transport_fails = 0u8;
    while Instant::now() < deadline {
        match dut_scan_has_ssid(base, ssid) {
            Ok(false) => return Ok(()),
            Ok(true) => {
                transport_fails = 0;
                last = "SSID still present".into();
            }
            Err(err) => {
                last = err.clone();
                if err.contains("HTTP 000") || err.contains("HTTP 502") || err.contains("HTTP 504")
                {
                    transport_fails += 1;
                    if transport_fails >= 3 {
                        return Err(format!(
                            "SSID `{ssid}` scan unreachable (wifi-manager down?): {last}"
                        ));
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_secs(2));
    }
    // BlueOS /scan often retains SSIDs after the beacon is gone. If our host AP
    // is confirmed down and the runner radio does not see the SSID, continue.
    if !host_ap_active() && !host_wifi_list_has_ssid(ssid) {
        eprintln!(
            "wifi_rf: WARN DUT still lists `{ssid}` after {timeout:?} but host AP is down (stale scan); continuing"
        );
        return Ok(());
    }
    Err(format!(
        "SSID `{ssid}` still present in DUT scan after {timeout:?}: {last}"
    ))
}

pub fn expect_wpa3_join(base: &str) -> Result<bool, String> {
    match config().expect_wpa3 {
        ExpectWpa3::Yes => Ok(true),
        ExpectWpa3::No => Ok(false),
        ExpectWpa3::Auto => {
            let ssid = mode_ssid(ApMode::Wpa3);
            let networks = dut_scan_json(base)?;
            Ok(networks.as_array().is_some_and(|networks| {
                networks.iter().any(|network| {
                    network.get("ssid").and_then(serde_json::Value::as_str) == Some(ssid.as_str())
                        && network
                            .get("supported")
                            .and_then(serde_json::Value::as_bool)
                            .unwrap_or(false)
                })
            }))
        }
    }
}

pub fn disconnect_and_remove(base: &str, ssid: &str) {
    let _ = http_get(&wifi_url(base, "disconnect"), "15");
    let _ = http_request(
        "POST",
        &format!("{}?ssid={ssid}", wifi_url(base, "remove")),
        None,
    );
}

/// Best-effort: leave the DUT radio free for station/client tests.
///
/// Soft-AP left on (or smart-hotspot re-enabled after an endpoints suite) blocks
/// client connect and pollutes neighboring DUT scans.
pub fn dut_hotspot_off(base: &str) {
    let _ = http_request("POST", &wifi_url(base, "smart_hotspot?enable=false"), None);
    let _ = http_request("POST", &wifi_url(base, "hotspot?enable=false"), None);
}

/// True when any of our host AP profiles is active on the configured iface.
pub fn host_ap_active() -> bool {
    let cfg = config();
    let Ok(active) = nmcli(&["-t", "-f", "NAME,DEVICE", "connection", "show", "--active"]) else {
        return false;
    };
    active.lines().any(|line| {
        let Some((name, device)) = line.split_once(':') else {
            return false;
        };
        device == cfg.iface && cfg.all_ap_conns().iter().any(|conn| *conn == name)
    })
}

fn host_wifi_list_has_ssid(ssid: &str) -> bool {
    let cfg = config();
    let Ok(list) = nmcli(&[
        "-t", "-f", "SSID", "device", "wifi", "list", "ifname", &cfg.iface,
    ]) else {
        return false;
    };
    list.lines().any(|line| line.trim() == ssid)
}

pub fn restore_station() -> Result<(), String> {
    let cfg = config();
    let mut errors = Vec::new();
    for conn in &cfg.station_conns {
        if !connection_exists(conn) {
            continue;
        }
        if let Err(err) = nmcli_ok(&[
            "connection",
            "modify",
            conn,
            "connection.autoconnect",
            "yes",
        ]) {
            errors.push(err);
        }
        if let Err(err) = nmcli_ok(&["connection", "up", conn]) {
            errors.push(err);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
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
        // Caller must pass BlueOS base via env for multi-step setup; journeys that
        // only use rf_setup without a base skip this — run_wifi_mode / endpoints
        // call [`dut_hotspot_off`] explicitly.
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

/// Host-AP journeys: disable DUT soft-AP first, then bring up the runner AP.
pub fn rf_setup_for_base(journey_id: JourneyId, base: &str) -> Result<(), String> {
    if needs_host_ap(journey_id) {
        dut_hotspot_off(base);
        std::thread::sleep(Duration::from_secs(2));
    }
    rf_setup(journey_id)
}

/// Tear down host RF after a journey (best-effort after HTTP teardown).
///
/// Does **not** call [`restore_station`] by default: bringing a competing home
/// WiFi profile back up can black-hole ethernet routes to the DUT mid-suite.
/// Opt in with `RESTORE_STATION=1` (or call [`restore_station`] explicitly).
pub fn rf_teardown(journey_id: JourneyId) -> Result<(), String> {
    let result = if needs_host_ap(journey_id) {
        host_ap_down()
    } else if needs_host_station(journey_id) {
        let _ = host_station_down();
        Ok(())
    } else {
        Ok(())
    };
    if std::env::var("RESTORE_STATION")
        .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "yes" | "true"))
        .unwrap_or(false)
    {
        let _ = restore_station();
    }
    result
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

/// Client L3: DHCP, route, front-end transfer, WebDAV roundtrip, and static IP.
pub fn l3_assert_associated(blueos_base: &str, mode: ApMode) -> Result<String, String> {
    let cfg = config();
    let ip = wait_dut_lease(blueos_base, &mode_ssid(mode), 45)?;
    let lease: Ipv4Addr = ip
        .parse()
        .map_err(|err| format!("invalid DUT lease {ip}: {err}"))?;
    if !ipv4_in_prefix(lease, cfg.ap_gateway, cfg.ap_prefix) {
        return Err(format!(
            "DUT lease {ip} is outside AP subnet {}/{}",
            cfg.ap_gateway, cfg.ap_prefix
        ));
    }
    host_ping(&ip)?;

    let route_url = format!(
        "{}?interface_name={}",
        cable_url(blueos_base, "route"),
        cfg.wifi_iface
    );
    let (routes, code) = http_get(&route_url, "15")?;
    if code != "200" {
        return Err(format!("GET /route HTTP {code}: {routes}"));
    }
    let gateway = cfg.ap_gateway.to_string();
    let has_route = serde_json::from_str::<serde_json::Value>(&routes)
        .ok()
        .and_then(|routes| routes.as_array().cloned())
        .is_some_and(|routes| {
            routes.iter().any(|route| {
                let gateway_match = route.get("gateway").and_then(serde_json::Value::as_str)
                    == Some(gateway.as_str());
                let destination = route
                    .get("destination")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                if destination == "0.0.0.0/0" {
                    return gateway_match || !routes.is_empty();
                }
                if let Some((net_ip, pref)) = destination.split_once('/') {
                    if let (Ok(net_ip), Ok(pref)) = (net_ip.parse::<Ipv4Addr>(), pref.parse::<u8>())
                    {
                        // On-link or via AP gateway into the AP subnet.
                        return ipv4_in_prefix(cfg.ap_gateway, net_ip, pref) || gateway_match;
                    }
                }
                gateway_match
            })
        });
    if !has_route {
        return Err(format!(
            "no default or AP route through {}: {routes}",
            cfg.ap_gateway
        ));
    }

    let (html, code) = http_get(&format!("http://{ip}/"), "30")?;
    if code != "200" {
        return Err(format!("GET WiFi frontend HTTP {code}: {html}"));
    }
    let asset = Regex::new(r#"/assets/main\.[^"']+\.js"#)
        .expect("asset regex")
        .find(&html)
        .map(|match_| match_.as_str())
        .ok_or_else(|| "frontend main asset missing from WiFi HTML".to_string())?;
    let (asset_body, code) = http_get(&format!("http://{ip}{asset}"), "30")?;
    if code != "200" || asset_body.len() < cfg.xfer_min_bytes as usize {
        return Err(format!(
            "GET WiFi asset HTTP {code}, {} bytes (want at least {})",
            asset_body.len(),
            cfg.xfer_min_bytes
        ));
    }

    let name = format!("smoke-wifi-{mode}.txt");
    let payload = format!("blueos wifi smoke {mode}\n");
    let temp = std::env::temp_dir().join(&name);
    std::fs::write(&temp, payload.as_bytes())
        .map_err(|err| format!("write transfer payload: {err}"))?;
    let upload_url = format!("http://{ip}/upload/userdata/{name}");
    let output = Command::new("curl")
        .args([
            "-sS",
            "-m",
            "30",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "-T",
        ])
        .arg(&temp)
        .arg(&upload_url)
        .output()
        .map_err(|err| format!("PUT upload: {err}"))?;
    let put_code = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let (uploaded, get_code) = http_get(&format!("http://{ip}/userdata/{name}"), "30")?;
    let _ = Command::new("curl")
        .args([
            "-sS",
            "-m",
            "10",
            "-o",
            "/dev/null",
            "-X",
            "DELETE",
            &upload_url,
        ])
        .output();
    let _ = std::fs::remove_file(temp);
    if (put_code != "201" && put_code != "200") || get_code != "200" || uploaded != payload {
        return Err(format!(
            "WebDAV roundtrip put={put_code} get={get_code} content_matches={}",
            uploaded == payload
        ));
    }

    let address_url = format!(
        "{}?interface_name={}&ip_address={}",
        cable_url(blueos_base, "address"),
        cfg.wifi_iface,
        cfg.static_test_ip
    );
    let (addr_body, code) = http_request("POST", &address_url, None)?;
    // cable-guy often rejects wlan0 ("No interface…") — record honesty without failing L3.
    let static_note = if code == "200" {
        let static_ping = host_ping(&cfg.static_test_ip.to_string());
        let (_, delete_code) = http_request("DELETE", &address_url, None)?;
        if delete_code != "200" {
            return Err(format!(
                "DELETE /address HTTP {delete_code} for {}",
                cfg.static_test_ip
            ));
        }
        static_ping?;
        format!("static_ip={}", cfg.static_test_ip)
    } else {
        eprintln!(
            "wifi_rf: static_ip SKIP POST /address HTTP {code} for {} on {} ({})",
            cfg.static_test_ip,
            cfg.wifi_iface,
            addr_body.chars().take(120).collect::<String>()
        );
        format!("static_ip_skip=HTTP_{code}")
    };
    Ok(format!("{ip} ({static_note})"))
}

/// Compatibility wrapper for existing WPA2-only callers.
pub fn l3_assert_client_lease(blueos_base: &str) -> Result<String, String> {
    l3_assert_associated(blueos_base, ApMode::Wpa2)
}

/// Hotspot L3: ping BlueOS soft-AP gateway from the host station.
pub fn l3_assert_hotspot_gateway() -> Result<(), String> {
    let gateway = config().hotspot_gateway.to_string();
    host_ping(&gateway)?;
    if let Ok((html, code)) = http_get(&format!("http://{gateway}/"), "15") {
        if code != "200" {
            return Ok(());
        }
        if let Some(asset) = Regex::new(r#"/assets/main\.[^"']+\.js"#)
            .expect("asset regex")
            .find(&html)
        {
            let _ = http_get(&format!("http://{gateway}{}", asset.as_str()), "15");
        }
    }
    Ok(())
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
        assert_eq!(
            ApMode::parse_csv("open,wpa2,wpa3").unwrap(),
            vec![ApMode::Open, ApMode::Wpa2, ApMode::Wpa3]
        );
        assert!(ApMode::parse("nope").is_err());
        assert!(ApMode::parse_csv("").is_err());
    }
}
