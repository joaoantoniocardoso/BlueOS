use crate::id::{PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
use crate::lifecycle::ObservedLifecycle;
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{Evidence, Evidenced, Observed, ObservedSet};
use crate::resource::{Resource, ResourceOwnership};

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
