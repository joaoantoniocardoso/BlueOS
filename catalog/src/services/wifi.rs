use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
use crate::journey::{HttpMethod, RouteRef};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use crate::resource::{Resource, ResourceOwnership};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline};
use crate::service::{Authority, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

const RUNTIME_CAPTURE: &str = "runtime-captures/wifi__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId("wifi".into()),
        state_contracts: GroundedSet::unknown(
            "wifi has no service-level state machine (card states Unknown); a wpa_supplicant event loop and hotspot watchdog reconcile periodically",
        ),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/status", 30.0, 44.4, 48.8, 40),
            runtime_slo(HttpMethod::Get, "/saved", 6.2, 9.5, 12.5, 40),
            runtime_slo(HttpMethod::Get, "/hotspot_extended_status", 7.3, 9.2, 9.9, 40),
            runtime_slo(HttpMethod::Get, "/smart_hotspot", 8.2, 10.5, 10.9, 40),
        ]),
        resource_usage: GroundedSet::known(vec![runtime_resource(
            "running_baseline",
            Distribution {
                mean: 0.37,
                median: 0.00,
                p95: 1.02,
                min: 0.00,
                max: 1.07,
                sd: 0.47,
            },
            Distribution {
                mean: 45.9,
                median: 45.9,
                p95: 45.9,
                min: 45.9,
                max: 45.9,
                sd: 0.0,
            },
            60,
        )]),
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "wifi manages wlan0/uap0 regardless of flight controller; platform-independent".into(),
                    "runtime captured on Navigator only; RSS ~45.9 MB flat, CPU ~0.37% mean (near-idle station mode)".into(),
                    "GET /status is heaviest (~30 ms): it queries the wpa_supplicant control socket".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "hotspot/smart-hotspot routes persist to /root/.config/wifi-manager/settings-1.json; wifi credentials persist via wpa_supplicant SAVE_CONFIG; not exercised (reachability hazard)",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId("wifi".into()),
        method,
        path: path.into(),
        version: None,
    }
}

fn runtime_slo(
    method: HttpMethod,
    path: &str,
    p50: f64,
    p95: f64,
    p99: f64,
    sample_size: u32,
) -> GroundedItem<SloBaseline> {
    GroundedItem::new(
        SloBaseline {
            route: runtime_route(method, path),
            latency_p50_ms: p50,
            latency_p95_ms: p95,
            latency_p99_ms: p99,
            sample_size,
        },
        runtime_prov("#slo_running_baseline"),
    )
}

fn runtime_resource(
    condition: &str,
    cpu_pct: Distribution,
    rss_mb: Distribution,
    samples: u32,
) -> GroundedItem<ResourceUsage> {
    GroundedItem::new(
        ResourceUsage {
            condition: condition.into(),
            cpu_pct,
            rss_mb,
            samples,
        },
        runtime_prov("#resource_usage"),
    )
}

pub fn observed_facts() -> ObservedFacts {
    ObservedFacts {
        id: ServiceId("wifi".to_string()),
        aliases: ObservedSet::known(vec![
            Evidenced::new(
                "wifi".to_string(),
                Evidence {
                    file: "core/start-blueos-core".to_string(),
                    line: 127,
                },
            ),
            Evidenced::new(
                "wifi-manager".to_string(),
                Evidence {
                    file: "core/services/wifi/main.py".to_string(),
                    line: 34,
                },
            ),
        ]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 127,
            },
        ),
        entrypoint: Observed::known(
            "$SERVICES_PATH/wifi/main.py --socket wlan0".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 127,
            },
        ),
        tmux_name: Observed::known(
            "wifi".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 127,
            },
        ),
        startup_tier: Observed::known(
            StartupTier::Normal,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 124,
            },
        ),
        resource_limits: Observed::known(
            ResourceLimits {
                memory_mb: Some(0),
                cpu_percent: Some(0),
                io_weight: None,
            },
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 127,
            },
        ),
        nice: Observed::known(
            -19,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 127,
            },
        ),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 127,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/wifi-manager/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 228,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(9000),
            Evidence {
                file: "core/services/wifi/main.py".to_string(),
                line: 184,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/wifi".to_string()),
            Evidence {
                file: "core/services/wifi/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/wifi-manager/".to_string()),
                    port: PortRef::Literal(9000),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/wifi/main.py".to_string(),
                    line: 166,
                },
            ),
            Evidenced::new(
                Interface::Settings {
                    path: PathRef("/root/.config/wifi-manager/settings-1.json".to_string()),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 69,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/wifi-manager".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/var/run/wpa_supplicant/".to_string()),
                    mode: FileAccessMode::Read,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/WifiManager.py".to_string(),
                    line: 530,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/var/run/wpa_supplicant/wlan0".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/WifiManager.py".to_string(),
                    line: 560,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/tmp/wpa_playground".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/wpa_supplicant.py".to_string(),
                    line: 32,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/tmp/wpa_playground/wpa_supplicant_service_{os.getpid()}".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/wpa_supplicant.py".to_string(),
                    line: 43,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/etc/dhcpcd.conf".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py".to_string(),
                    line: 250,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/tmp/hostapd.conf".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py".to_string(),
                    line: 211,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/var/lib/dnsmasq".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/DHCPServerManager.py".to_string(),
                    line: 50,
                },
            ),
            Evidenced::new(
                Interface::Hardware {
                    device: PathRef("wlan0".to_string()),
                },
                Evidence {
                    file: "core/start-blueos-core".to_string(),
                    line: 127,
                },
            ),
            Evidenced::new(
                Interface::Hardware {
                    device: PathRef("uap0".to_string()),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py".to_string(),
                    line: 39,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "dhcpcd -n wlan0".to_string(),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/WifiManager.py".to_string(),
                    line: 394,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "hostapd -h".to_string(),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py".to_string(),
                    line: 98,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "iw dev {self._base_interface} interface add {self._ap_interface_name} type __ap"
                        .to_string(),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py".to_string(),
                    line: 142,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ifconfig {self._ap_interface_name} up".to_string(),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py".to_string(),
                    line: 154,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "hostapd {self.config_path()}".to_string(),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py".to_string(),
                    line: 164,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "dnsmasq".to_string(),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/DHCPServerManager.py".to_string(),
                    line: 95,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ip link show {self._ap_interface}".to_string(),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py".to_string(),
                    line: 60,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "iw dev {phys_name} interface add {self._ap_interface} type __ap".to_string(),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py".to_string(),
                    line: 71,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ip link set {self._ap_interface} up".to_string(),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py".to_string(),
                    line: 75,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "iw {phys_name} set power_save off".to_string(),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py".to_string(),
                    line: 78,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "iw {self._ap_interface} set power_save off".to_string(),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py".to_string(),
                    line: 79,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "iw dev {self._ap_interface} del".to_string(),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py".to_string(),
                    line: 97,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "create_ap -n uap0 -g 192.168.42.1 --redirect-to-localhost {credentials.ssid} {credentials.password}"
                        .to_string(),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py".to_string(),
                    line: 305,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/wifi-manager/log".to_string()],
                    topics_consumed: vec![],
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                    line: 78,
                },
            ),
        ]),
        resources: ObservedSet::known(vec![
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/wifi-manager".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/wifi-manager/settings-1.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 69,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/var/run/wpa_supplicant/wlan0".to_string()),
                    ownership: ResourceOwnership::Exclusive,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/WifiManager.py".to_string(),
                    line: 560,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/etc/dhcpcd.conf".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py".to_string(),
                    line: 281,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/var/lib/dnsmasq".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/DHCPServerManager.py".to_string(),
                    line: 50,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("wlan0".to_string()),
                    ownership: ResourceOwnership::Exclusive,
                },
                Evidence {
                    file: "core/start-blueos-core".to_string(),
                    line: 127,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("uap0".to_string()),
                    ownership: ResourceOwnership::Exclusive,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py".to_string(),
                    line: 39,
                },
            ),
        ]),
        lifecycle: Observed::known(
            ObservedLifecycle {
                triggers: vec!["start-blueos-core create_service".to_string()],
                ordered_after: vec![
                    ServiceId("autopilot".to_string()),
                    ServiceId("cable_guy".to_string()),
                    ServiceId("video".to_string()),
                    ServiceId("mavlink2rest".to_string()),
                    ServiceId("kraken".to_string()),
                ],
                ordered_before: vec![
                    ServiceId("zenohd".to_string()),
                    ServiceId("beacon".to_string()),
                    ServiceId("bridget".to_string()),
                    ServiceId("commander".to_string()),
                    ServiceId("nmea_injector".to_string()),
                    ServiceId("helper".to_string()),
                    ServiceId("iperf3".to_string()),
                    ServiceId("linux2rest".to_string()),
                    ServiceId("filebrowser".to_string()),
                    ServiceId("versionchooser".to_string()),
                    ServiceId("pardal".to_string()),
                    ServiceId("ping".to_string()),
                    ServiceId("user_terminal".to_string()),
                    ServiceId("ttyd".to_string()),
                    ServiceId("nginx".to_string()),
                    ServiceId("bag_of_holding".to_string()),
                    ServiceId("recorder".to_string()),
                    ServiceId("recorder_extractor".to_string()),
                    ServiceId("disk_usage".to_string()),
                    ServiceId("customization".to_string()),
                ],
            },
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 326,
            },
        ),
        logs_path: Observed::unknown(
            "init_logger publishes to zenoh only; no on-disk log path set in wifi source",
        ),
        zenoh_log_topic: Observed::known(
            "services/wifi-manager/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/wifi/main.py".to_string(),
                line: 171,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("wifi".to_string()),
        singleton: Asserted::established(
            true,
            "single NORMAL-tier tmux instance; one wifi-manager process on port 9000 owns wlan0 wireless configuration",
        ),
        bounded_context: Asserted::established(
            "wireless-network-configuration".to_string(),
            "provisional 2.0 domain: wlan scan/connect/disconnect, saved networks, hotspot, and smart-hotspot",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId("connect_to_wifi_network".into()),
                "wifi tray scans networks and POST /connect joins the selected SSID",
            ),
            Rationaled::new(
                JourneyId("disconnect_from_wifi_network".into()),
                "wifi tray disconnects the active wlan association via GET /disconnect",
            ),
            Rationaled::new(
                JourneyId("forget_saved_wifi_network".into()),
                "connection dialog removes a stored SSID via POST /remove",
            ),
            Rationaled::new(
                JourneyId("toggle_hotspot".into()),
                "wifi tray hotspot button enables or disables the onboard access point via POST /hotspot",
            ),
            Rationaled::new(
                JourneyId("configure_hotspot_credentials".into()),
                "wifi settings dialog persists hotspot SSID and password via POST /hotspot_credentials",
            ),
            Rationaled::new(
                JourneyId("toggle_smart_hotspot".into()),
                "wifi settings dialog enables auto-hotspot when no known network is connected via POST /smart_hotspot",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Important,
            "NORMAL boot tier brings wireless connectivity up after cable_guy; MAVLink vehicle control does not depend on it but operator reachability over wifi and hotspot UX do",
        ),
        offline_required: Asserted::established(
            true,
            "wlan scan, connect, disconnect, saved networks, hotspot, and smart-hotspot operate on local interfaces without internet",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; mutates wlan0/uap0, wpa_supplicant control socket, hostapd, dnsmasq, and /etc/dhcpcd.conf",
        ),
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "dangerous_operations empty per rubric v1.0; wireless changes are reversible reconfiguration, not irreversible destructive ops",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("connect_wifi_network".to_string()),
                "POST /connect joins the named SSID with supplied credentials",
            ),
            Rationaled::new(
                CapabilityId("disconnect_wifi_network".to_string()),
                "GET /disconnect drops the active wlan association",
            ),
            Rationaled::new(
                CapabilityId("remove_saved_wifi_network".to_string()),
                "POST /remove deletes a stored SSID from saved networks",
            ),
            Rationaled::new(
                CapabilityId("toggle_hotspot".to_string()),
                "POST /hotspot enables or disables the onboard wireless access point",
            ),
            Rationaled::new(
                CapabilityId("set_hotspot_credentials".to_string()),
                "POST /hotspot_credentials persists hotspot SSID and password in SettingsV1",
            ),
            Rationaled::new(
                CapabilityId("toggle_smart_hotspot".to_string()),
                "POST /smart_hotspot enables or disables auto-hotspot when no known network is connected",
            ),
            Rationaled::new(
                CapabilityId("scan_wifi_networks".to_string()),
                "GET /scan returns available BSS scan results from wpa_supplicant",
            ),
            Rationaled::new(
                CapabilityId("get_wifi_status".to_string()),
                "GET /status returns current wlan association state and interface details",
            ),
            Rationaled::new(
                CapabilityId("list_saved_wifi_networks".to_string()),
                "GET /saved returns stored SSIDs and connection metadata",
            ),
            Rationaled::new(
                CapabilityId("get_hotspot_status".to_string()),
                "GET /hotspot, /hotspot_extended_status, /smart_hotspot, and /hotspot_credentials report hotspot and smart-hotspot state",
            ),
        ]),
        authorities: AssertedSet::established(vec![
            Rationaled::new(
                Authority::Other("wireless_network_controller".to_string()),
                "sole REST surface for wlan0 scan, connect, disconnect, and saved networks; cable_guy owns wired interfaces separately",
            ),
            Rationaled::new(
                Authority::Other("wifi_hotspot_operator".to_string()),
                "sole manager of uap0 access point, hostapd, dnsmasq DHCP for hotspot, and smart-hotspot watchdog",
            ),
        ]),
        states: AssertedSet::unknown(
            "no cataloged state machine; wpa_supplicant event loop and hotspot watchdog run periodic reconciliation",
        ),
        edges: AssertedSet::unknown(
            "no outbound coupling to other catalog services; wireless changes are local host configuration only",
        ),
        resources: AssertedSet::established(vec![
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/wifi-manager".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "SettingsV1 pykson manager directory for persisted hotspot and smart-hotspot configuration",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/wifi-manager/settings-1.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "persisted wifi-manager SettingsV1 hotspot credentials and smart-hotspot flag",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/var/run/wpa_supplicant/wlan0".to_string()),
                    ownership: ResourceOwnership::Exclusive,
                },
                "exclusive wpa_supplicant control socket for wlan0 association and scan commands",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/etc/dhcpcd.conf".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "dhcpcd static configuration written when hotspot alters interface addressing; hostapd config generated at /tmp/hostapd.conf",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/var/lib/dnsmasq".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "dnsmasq lease and DHCP state directory for hotspot clients",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("wlan0".to_string()),
                    ownership: ResourceOwnership::Exclusive,
                },
                "exclusive control of the primary wlan station interface",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("uap0".to_string()),
                    ownership: ResourceOwnership::Exclusive,
                },
                "exclusive control of the virtual access-point interface created for hotspot",
            ),
        ]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                vec!["start-blueos-core create_service".to_string()],
                "observed lifecycle trigger: tmux creation at boot in NORMAL tier",
            ),
            ordered_after: Asserted::established(
                vec![
                    ServiceId("autopilot".to_string()),
                    ServiceId("cable_guy".to_string()),
                    ServiceId("video".to_string()),
                    ServiceId("mavlink2rest".to_string()),
                    ServiceId("kraken".to_string()),
                ],
                "observed ordered_after in start-blueos-core NORMAL block lists wifi after PRIORITY-tier peers",
            ),
            ordered_before: Asserted::established(
                vec![
                    ServiceId("zenohd".to_string()),
                    ServiceId("beacon".to_string()),
                    ServiceId("bridget".to_string()),
                    ServiceId("commander".to_string()),
                    ServiceId("nmea_injector".to_string()),
                    ServiceId("helper".to_string()),
                    ServiceId("iperf3".to_string()),
                    ServiceId("linux2rest".to_string()),
                    ServiceId("filebrowser".to_string()),
                    ServiceId("versionchooser".to_string()),
                    ServiceId("pardal".to_string()),
                    ServiceId("ping".to_string()),
                    ServiceId("user_terminal".to_string()),
                    ServiceId("ttyd".to_string()),
                    ServiceId("nginx".to_string()),
                    ServiceId("bag_of_holding".to_string()),
                    ServiceId("recorder".to_string()),
                    ServiceId("recorder_extractor".to_string()),
                    ServiceId("disk_usage".to_string()),
                    ServiceId("customization".to_string()),
                ],
                "observed ordered_before lists wifi before remaining SERVICES-tier peers including nginx",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit on process termination".to_string(),
                "main.py awaits server.serve with no explicit shutdown hook beyond uvicorn exit",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight wlan association and SettingsV1 migration not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET /status returns wlan state; wpa_supplicant event loop at boot".to_string(),
            "no dedicated /health route; uvicorn availability and wpa_supplicant control socket serve as health signals",
        ),
        is_platform: Asserted::established(
            false,
            "wireless network configuration utility; does not install or host third-party extensions",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 router exposed under /wifi-manager/ via VersionedFastAPI",
        ),
        permissions_model: Asserted::established(
            "no separate permissions manifest; REST routes are unauthenticated".to_string(),
            "wifi routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "wireless_reconfiguration_lockout".to_string(),
                "incorrect connect, disconnect, or hotspot change can sever the operator connection until physical or alternate-interface recovery",
            ),
            Rationaled::new(
                "wpa_supplicant_connection_failure".to_string(),
                "POST /connect may fail when credentials are wrong or wpa_supplicant cannot complete association within timeout",
            ),
            Rationaled::new(
                "hotspot_start_failure".to_string(),
                "POST /hotspot enable may fail when hostapd, iw virtual interface creation, or dnsmasq cannot start",
            ),
            Rationaled::new(
                "scan_busy".to_string(),
                "GET /scan returns HTTP 425 when a scan is already in progress",
            ),
            Rationaled::new(
                "smart_hotspot_watchdog_mismatch".to_string(),
                "smart-hotspot watchdog may enable or disable hotspot asynchronously; transient state mismatches until the next cycle",
            ),
        ]),
        blast_radius: Asserted::established(
            "wireless network reachability and operator UI access; MAVLink on the FC may continue but BlueOS web UI over wifi or hotspot can become unreachable".to_string(),
            "misconfigured wlan or hotspot can lock out the operator; outage blocks wifi tray UX and wireless LAN access",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and SettingsV1 migration stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
