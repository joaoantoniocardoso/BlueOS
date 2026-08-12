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
use crate::service::{Authority, Service, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Wifi,
        state_contracts: GroundedSet::unknown(
            "wifi has no service-level state machine (card states Unknown); a wpa_supplicant event loop and hotspot watchdog reconcile periodically",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/status", 30.0, 44.4, 48.8, 40),
            runtime_slo(HttpMethod::Get, "/saved", 6.2, 9.5, 12.5, 40),
            runtime_slo(HttpMethod::Get, "/hotspot_extended_status", 7.3, 9.2, 9.9, 40),
            runtime_slo(HttpMethod::Get, "/smart_hotspot", 8.2, 10.5, 10.9, 40),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
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
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "wifi manages wlan0/uap0 regardless of flight controller; platform-independent",
                    "runtime captured on Navigator only; RSS ~45.9 MB flat, CPU ~0.37% mean (near-idle station mode)",
                    "GET /status is heaviest (~30 ms): it queries the wpa_supplicant control socket",
                ],
            },
            runtime_prov("runtime-captures/wifi__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "hotspot/smart-hotspot routes persist to /root/.config/wifi-manager/settings-1.json; wifi credentials persist via wpa_supplicant SAVE_CONFIG; not exercised (reachability hazard)",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::Wifi,
        method,
        path,
        version: None,
    }
}

const fn runtime_slo(
    method: HttpMethod,
    path: &'static str,
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
        runtime_prov("runtime-captures/wifi__pi4_navigator_master.json#slo_running_baseline"),
    )
}

const fn runtime_resource(
    condition: &'static str,
    cpu_pct: Distribution,
    rss_mb: Distribution,
    samples: u32,
) -> GroundedItem<ResourceUsage> {
    GroundedItem::new(
        ResourceUsage {
            condition,
            cpu_pct,
            rss_mb,
            samples,
        },
        runtime_prov("runtime-captures/wifi__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts =
    ObservedFacts {
        id: ServiceId::Wifi,
        aliases: ObservedSet::known(&[
            Evidenced::new(
                "wifi",
                Evidence {
                    file: "core/start-blueos-core",
                    line: 127,
                },
            ),
            Evidenced::new(
                "wifi-manager",
                Evidence {
                    file: "core/services/wifi/main.py",
                    line: 34,
                },
            ),
        ]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core",
                line: 127,
            },
        ),
        entrypoint: Observed::known(
            "$SERVICES_PATH/wifi/main.py --socket wlan0",
            Evidence {
                file: "core/start-blueos-core",
                line: 127,
            },
        ),
        tmux_name: Observed::known(
            "wifi",
            Evidence {
                file: "core/start-blueos-core",
                line: 127,
            },
        ),
        startup_tier: Observed::known(
            StartupTier::Normal,
            Evidence {
                file: "core/start-blueos-core",
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
                file: "core/start-blueos-core",
                line: 127,
            },
        ),
        nice: Observed::known(
            -19,
            Evidence {
                file: "core/start-blueos-core",
                line: 127,
            },
        ),
        run_as: Observed::known(
            "root",
            Evidence {
                file: "core/start-blueos-core",
                line: 127,
            },
        ),
        nginx_prefixes: ObservedSet::known(&[Evidenced::new(
            PathRef("/wifi-manager/"),
            Evidence {
                file: "core/tools/nginx/nginx.conf",
                line: 228,
            },
        )]),
        listen: ObservedSet::known(&[Evidenced::new(
            PortRef::Literal(9000),
            Evidence {
                file: "core/services/wifi/main.py",
                line: 184,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/wifi"),
            Evidence {
                file: "core/services/wifi/main.py",
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(&[
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/wifi-manager/"),
                    port: PortRef::Literal(9000),
                    versions: &["v1.0"],
                },
                Evidence {
                    file: "core/services/wifi/main.py",
                    line: 166,
                },
            ),
            Evidenced::new(
                Interface::Settings {
                    path: PathRef("/root/.config/wifi-manager/settings-1.json"),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        ,
                    line: 69,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/wifi-manager"),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        ,
                    line: 27,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/var/run/wpa_supplicant/"),
                    mode: FileAccessMode::Read,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/WifiManager.py",
                    line: 530,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/var/run/wpa_supplicant/wlan0"),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/WifiManager.py",
                    line: 560,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/tmp/wpa_playground"),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/wpa_supplicant.py",
                    line: 32,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/tmp/wpa_playground/wpa_supplicant_service_{os.getpid()}"),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/wpa_supplicant.py",
                    line: 43,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/etc/dhcpcd.conf"),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py",
                    line: 250,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/tmp/hostapd.conf"),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py",
                    line: 211,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/var/lib/dnsmasq"),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/DHCPServerManager.py",
                    line: 50,
                },
            ),
            Evidenced::new(
                Interface::Hardware {
                    device: PathRef("wlan0"),
                },
                Evidence {
                    file: "core/start-blueos-core",
                    line: 127,
                },
            ),
            Evidenced::new(
                Interface::Hardware {
                    device: PathRef("uap0"),
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py",
                    line: 39,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "dhcpcd -n wlan0",
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/WifiManager.py",
                    line: 394,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "hostapd -h",
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py",
                    line: 98,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "iw dev {self._base_interface} interface add {self._ap_interface_name} type __ap"
                        ,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py",
                    line: 142,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ifconfig {self._ap_interface_name} up",
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py",
                    line: 154,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "hostapd {self.config_path()}",
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py",
                    line: 164,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "dnsmasq",
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/DHCPServerManager.py",
                    line: 95,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ip link show {self._ap_interface}",
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py",
                    line: 60,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "iw dev {phys_name} interface add {self._ap_interface} type __ap",
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py",
                    line: 71,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ip link set {self._ap_interface} up",
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py",
                    line: 75,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "iw {phys_name} set power_save off",
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py",
                    line: 78,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "iw {self._ap_interface} set power_save off",
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py",
                    line: 79,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "iw dev {self._ap_interface} del",
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py",
                    line: 97,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "create_ap -n uap0 -g 192.168.42.1 --redirect-to-localhost {credentials.ssid} {credentials.password}"
                        ,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/networkmanager/networkmanager.py",
                    line: 305,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: &["services/wifi-manager/log"],
                    topics_consumed: &[],
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                    line: 78,
                },
            ),
        ]),
        resources: ObservedSet::known(&[
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/wifi-manager"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        ,
                    line: 27,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/wifi-manager/settings-1.json"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        ,
                    line: 69,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/var/run/wpa_supplicant/wlan0"),
                    ownership: ResourceOwnership::Exclusive,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/WifiManager.py",
                    line: 560,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/etc/dhcpcd.conf"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py",
                    line: 281,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/var/lib/dnsmasq"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/DHCPServerManager.py",
                    line: 50,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("wlan0"),
                    ownership: ResourceOwnership::Exclusive,
                },
                Evidence {
                    file: "core/start-blueos-core",
                    line: 127,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("uap0"),
                    ownership: ResourceOwnership::Exclusive,
                },
                Evidence {
                    file: "core/services/wifi/wifi_handlers/wpa_supplicant/Hotspot.py",
                    line: 39,
                },
            ),
        ]),
        lifecycle: Observed::known(
            ObservedLifecycle {
                triggers: &["start-blueos-core create_service"],
                ordered_after: &[
                    ServiceId::ArdupilotManager,
                    ServiceId::CableGuy,
                    ServiceId::MavlinkCameraManager,
                    ServiceId::Mavlink2rest,
                    ServiceId::Kraken,
                ],
                ordered_before: &[
                    ServiceId::Zenohd,
                    ServiceId::Beacon,
                    ServiceId::Bridget,
                    ServiceId::Commander,
                    ServiceId::NmeaInjector,
                    ServiceId::Helper,
                    ServiceId::Iperf3,
                    ServiceId::Linux2rest,
                    ServiceId::Filebrowser,
                    ServiceId::Versionchooser,
                    ServiceId::Pardal,
                    ServiceId::Ping,
                    ServiceId::UserTerminal,
                    ServiceId::Ttyd,
                    ServiceId::Nginx,
                    ServiceId::BagOfHolding,
                    ServiceId::Recorder,
                    ServiceId::RecorderExtractor,
                    ServiceId::DiskUsage,
                    ServiceId::Customization,
                ],
            },
            Evidence {
                file: "core/start-blueos-core",
                line: 326,
            },
        ),
        logs_path: Observed::unknown(
            "init_logger publishes to zenoh only; no on-disk log path set in wifi source",
        ),
        zenoh_log_topic: Observed::known(
            "services/wifi-manager/log",
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/wifi/main.py",
                line: 171,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    };

pub const SERVICE_DEFINITION: ServiceDefinition =
    ServiceDefinition {
        id: ServiceId::Wifi,
        singleton: Asserted::established(
            true,
            "single NORMAL-tier tmux instance; one wifi-manager process on port 9000 owns wlan0 wireless configuration",
        ),
        bounded_context: Asserted::established(
            "wireless-network-configuration",
            "provisional 2.0 domain: wlan scan/connect/disconnect, saved networks, hotspot, and smart-hotspot",
        ),
        journey_refs: AssertedSet::established(&[
            Rationaled::new(
                JourneyId::ConnectToWifiNetwork,
                "wifi tray scans networks and POST /connect joins the selected SSID",
            ),
            Rationaled::new(
                JourneyId::ConnectToHiddenWifiNetwork,
                "connection dialog POST /connect?hidden=true joins an operator-entered SSID",
            ),
            Rationaled::new(
                JourneyId::DisconnectFromWifiNetwork,
                "wifi tray disconnects the active wlan association via GET /disconnect",
            ),
            Rationaled::new(
                JourneyId::ForgetSavedWifiNetwork,
                "connection dialog removes a stored SSID via POST /remove",
            ),
            Rationaled::new(
                JourneyId::ForceWifiNetworkPassword,
                "connection dialog Force new password re-submits credentials via POST /connect",
            ),
            Rationaled::new(
                JourneyId::ReconnectToSavedWifiNetwork,
                "connection dialog reconnects to a saved SSID without re-entering the password",
            ),
            Rationaled::new(
                JourneyId::RejectInvalidWifiCredentials,
                "failed POST /connect with wrong password surfaces a connection error",
            ),
            Rationaled::new(
                JourneyId::DetectWifiApLoss,
                "GET /status reflects loss of the associated SSID when the AP disappears",
            ),
            Rationaled::new(
                JourneyId::AutoconnectToSavedWifiNetwork,
                "wpa_supplicant autoconnect rejoins a saved SSID when the AP returns",
            ),
            Rationaled::new(
                JourneyId::ToggleHotspot,
                "wifi tray hotspot button enables or disables the onboard access point via POST /hotspot",
            ),
            Rationaled::new(
                JourneyId::ConfigureHotspotCredentials,
                "wifi settings dialog persists hotspot SSID and password via POST /hotspot_credentials",
            ),
            Rationaled::new(
                JourneyId::ToggleSmartHotspot,
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
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "dangerous_operations empty per rubric v1.0; wireless changes are reversible reconfiguration, not irreversible destructive ops",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::ConnectWifiNetwork,
                "POST /connect joins the named SSID with supplied credentials",
            ),
            Rationaled::new(
                CapabilityId::DisconnectWifiNetwork,
                "GET /disconnect drops the active wlan association",
            ),
            Rationaled::new(
                CapabilityId::RemoveSavedWifiNetwork,
                "POST /remove deletes a stored SSID from saved networks",
            ),
            Rationaled::new(
                CapabilityId::ToggleHotspot,
                "POST /hotspot enables or disables the onboard wireless access point",
            ),
            Rationaled::new(
                CapabilityId::SetHotspotCredentials,
                "POST /hotspot_credentials persists hotspot SSID and password in SettingsV1",
            ),
            Rationaled::new(
                CapabilityId::ToggleSmartHotspot,
                "POST /smart_hotspot enables or disables auto-hotspot when no known network is connected",
            ),
            Rationaled::new(
                CapabilityId::ScanWifiNetworks,
                "GET /scan returns available BSS scan results from wpa_supplicant",
            ),
            Rationaled::new(
                CapabilityId::GetWifiStatus,
                "GET /status returns current wlan association state and interface details",
            ),
            Rationaled::new(
                CapabilityId::ListSavedWifiNetworks,
                "GET /saved returns stored SSIDs and connection metadata",
            ),
            Rationaled::new(
                CapabilityId::GetHotspotStatus,
                "GET /hotspot, /hotspot_extended_status, /smart_hotspot, and /hotspot_credentials report hotspot and smart-hotspot state",
            ),
        ]),
        authorities: AssertedSet::established(&[
            Rationaled::new(
                Authority::Other("wireless_network_controller"),
                "sole REST surface for wlan0 scan, connect, disconnect, and saved networks; cable_guy owns wired interfaces separately",
            ),
            Rationaled::new(
                Authority::Other("wifi_hotspot_operator"),
                "sole manager of uap0 access point, hostapd, dnsmasq DHCP for hotspot, and smart-hotspot watchdog",
            ),
        ]),
        states: AssertedSet::unknown(
            "no cataloged state machine; wpa_supplicant event loop and hotspot watchdog run periodic reconciliation",
        ),
        edges: AssertedSet::established(&[]),
        resources: AssertedSet::established(&[
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/wifi-manager"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "SettingsV1 pykson manager directory for persisted hotspot and smart-hotspot configuration",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/wifi-manager/settings-1.json"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "persisted wifi-manager SettingsV1 hotspot credentials and smart-hotspot flag",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/var/run/wpa_supplicant/wlan0"),
                    ownership: ResourceOwnership::Exclusive,
                },
                "exclusive wpa_supplicant control socket for wlan0 association and scan commands",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/etc/dhcpcd.conf"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "dhcpcd static configuration written when hotspot alters interface addressing; hostapd config generated at /tmp/hostapd.conf",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/var/lib/dnsmasq"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "dnsmasq lease and DHCP state directory for hotspot clients",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("wlan0"),
                    ownership: ResourceOwnership::Exclusive,
                },
                "exclusive control of the primary wlan station interface",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("uap0"),
                    ownership: ResourceOwnership::Exclusive,
                },
                "exclusive control of the virtual access-point interface created for hotspot",
            ),
        ]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                &["start-blueos-core create_service"],
                "observed lifecycle trigger: tmux creation at boot in NORMAL tier",
            ),
            ordered_after: Asserted::established(
                &[
                    ServiceId::ArdupilotManager,
                    ServiceId::CableGuy,
                    ServiceId::MavlinkCameraManager,
                    ServiceId::Mavlink2rest,
                    ServiceId::Kraken,
                ],
                "observed ordered_after in start-blueos-core NORMAL block lists wifi after PRIORITY-tier peers",
            ),
            ordered_before: Asserted::established(
                &[
                    ServiceId::Zenohd,
                    ServiceId::Beacon,
                    ServiceId::Bridget,
                    ServiceId::Commander,
                    ServiceId::NmeaInjector,
                    ServiceId::Helper,
                    ServiceId::Iperf3,
                    ServiceId::Linux2rest,
                    ServiceId::Filebrowser,
                    ServiceId::Versionchooser,
                    ServiceId::Pardal,
                    ServiceId::Ping,
                    ServiceId::UserTerminal,
                    ServiceId::Ttyd,
                    ServiceId::Nginx,
                    ServiceId::BagOfHolding,
                    ServiceId::Recorder,
                    ServiceId::RecorderExtractor,
                    ServiceId::DiskUsage,
                    ServiceId::Customization,
                ],
                "observed ordered_before lists wifi before remaining SERVICES-tier peers including nginx",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit on process termination",
                "main.py awaits server.serve with no explicit shutdown hook beyond uvicorn exit",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight wlan association and SettingsV1 migration not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET /status returns wlan state; wpa_supplicant event loop at boot",
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
            "no separate permissions manifest; REST routes are unauthenticated",
            "wifi routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "wireless_reconfiguration_lockout",
                "incorrect connect, disconnect, or hotspot change can sever the operator connection until physical or alternate-interface recovery",
            ),
            Rationaled::new(
                "wpa_supplicant_connection_failure",
                "POST /connect may fail when credentials are wrong or wpa_supplicant cannot complete association within timeout",
            ),
            Rationaled::new(
                "hotspot_start_failure",
                "POST /hotspot enable may fail when hostapd, iw virtual interface creation, or dnsmasq cannot start",
            ),
            Rationaled::new(
                "scan_busy",
                "GET /scan returns HTTP 425 when a scan is already in progress",
            ),
            Rationaled::new(
                "smart_hotspot_watchdog_mismatch",
                "smart-hotspot watchdog may enable or disable hotspot asynchronously; transient state mismatches until the next cycle",
            ),
        ]),
        blast_radius: Asserted::established(
            "wireless network reachability and operator UI access; MAVLink on the FC may continue but BlueOS web UI over wifi or hotspot can become unreachable",
            "misconfigured wlan or hotspot can lock out the operator; outage blocks wifi tray UX and wireless LAN access",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and SettingsV1 migration stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Wifi,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
