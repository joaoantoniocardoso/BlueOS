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

const RUNTIME_CAPTURE: &str = "runtime-captures/cable_guy__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId("cable_guy".into()),
        state_contracts: GroundedSet::unknown(
            "cable_guy has no service-level state machine (card states Unknown); a manager watchdog reconciles interface state periodically",
        ),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/interfaces", 12.4, 24.4, 26.5, 40),
            runtime_slo(HttpMethod::Get, "/ethernet", 11.0, 19.7, 19.8, 40),
            runtime_slo(HttpMethod::Get, "/host_dns", 1215.2, 1302.9, 1314.2, 40),
            runtime_slo(HttpMethod::Get, "/route?interface_name=eth0", 25.9, 32.8, 35.9, 40),
            runtime_slo(HttpMethod::Get, "/dhcp/details/eth0", 7.7, 15.7, 22.7, 40),
        ]),
        resource_usage: GroundedSet::known(vec![runtime_resource(
            "running_baseline",
            Distribution {
                mean: 2.47,
                median: 0.00,
                p95: 11.94,
                min: 0.00,
                max: 17.35,
                sd: 4.53,
            },
            Distribution {
                mean: 53.1,
                median: 53.1,
                p95: 53.1,
                min: 53.1,
                max: 53.1,
                sd: 0.0,
            },
            60,
        )]),
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "cable_guy manages host wired interfaces regardless of flight controller; platform-independent".into(),
                    "runtime captured on Navigator only; RSS ~53.1 MB flat, CPU ~2.47% mean (watchdog reconciliation spikes)".into(),
                    "GET /host_dns is ~1.2 s: it shells out (cat/lsattr on /etc/resolv.conf) per request".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "mutating routes persist to /root/.config/cable-guy/settings-2.json, /etc/dhcpcd.conf, /etc/resolv.conf but were not exercised (network-lockout hazard)",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId("cable_guy".into()),
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
        id: ServiceId("cable_guy".to_string()),
        aliases: ObservedSet::known(vec![
            Evidenced::new(
                "cable_guy".to_string(),
                Evidence {
                    file: "core/start-blueos-core".to_string(),
                    line: 119,
                },
            ),
            Evidenced::new(
                "cable-guy".to_string(),
                Evidence {
                    file: "core/services/cable_guy/config.py".to_string(),
                    line: 6,
                },
            ),
        ]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 119,
            },
        ),
        entrypoint: Observed::known(
            "$SERVICES_PATH/cable_guy/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 119,
            },
        ),
        tmux_name: Observed::known(
            "cable_guy".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 119,
            },
        ),
        startup_tier: Observed::known(
            StartupTier::Priority,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 117,
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
                line: 119,
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/services/cable_guy/main.py".to_string(),
                line: 193,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/cable-guy/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 103,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(9090),
            Evidence {
                file: "core/services/cable_guy/main.py".to_string(),
                line: 183,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/cable_guy".to_string()),
            Evidence {
                file: "core/services/cable_guy/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/cable-guy/".to_string()),
                    port: PortRef::Literal(9090),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/cable_guy/main.py".to_string(),
                    line: 163,
                },
            ),
            Evidenced::new(
                Interface::Settings {
                    path: PathRef("/root/.config/cable-guy/settings-2.json".to_string()),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pydantic_manager.py"
                        .to_string(),
                    line: 73,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/cable-guy".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pydantic_manager.py"
                        .to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/cable-guy/settings-1.json".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/cable_guy/api/settings.py".to_string(),
                    line: 40,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/cable-guy/settings.json".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/cable_guy/api/settings.py".to_string(),
                    line: 47,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/etc/resolv.conf".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 9,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/etc/dhcpcd.conf".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/cable_guy/networksetup.py".to_string(),
                    line: 254,
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
                Interface::Subprocess {
                    command: "cat '{filename}'".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 56,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "lsattr {filename}".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 64,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sudo chattr -i {filename}".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 80,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "echo '{content}' | sudo tee {filename}".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 86,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sudo chattr +i {filename}".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 74,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "timeout 5 dhclient -d -v {interface_name} 2>&1 || echo 'timeout'".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/networksetup.py".to_string(),
                    line: 124,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ifmetric".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/api/manager.py".to_string(),
                    line: 605,
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
                Interface::Zenoh {
                    topics_produced: vec!["services/cable-guy/log".to_string()],
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
                    path: PathRef("/root/.config/cable-guy".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pydantic_manager.py"
                        .to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/cable-guy/settings-2.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pydantic_manager.py"
                        .to_string(),
                    line: 73,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/etc/resolv.conf".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 86,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/etc/dhcpcd.conf".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/cable_guy/networksetup.py".to_string(),
                    line: 355,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/var/lib/dnsmasq".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/DHCPServerManager.py".to_string(),
                    line: 79,
                },
            ),
        ]),
        lifecycle: Observed::known(
            ObservedLifecycle {
                triggers: vec!["start-blueos-core create_service".to_string()],
                ordered_after: vec![ServiceId("autopilot".to_string())],
                ordered_before: vec![
                    ServiceId("video".to_string()),
                    ServiceId("mavlink2rest".to_string()),
                    ServiceId("kraken".to_string()),
                    ServiceId("wifi".to_string()),
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
                line: 318,
            },
        ),
        logs_path: Observed::unknown(
            "init_logger publishes to zenoh only; no on-disk log path set in cable_guy source",
        ),
        zenoh_log_topic: Observed::known(
            "services/cable-guy/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/cable_guy/main.py".to_string(),
                line: 181,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("cable_guy".to_string()),
        singleton: Asserted::established(
            true,
            "single PRIORITY-tier tmux instance; one cable_guy process on port 9090 owns wired network configuration",
        ),
        bounded_context: Asserted::established(
            "wired-network-configuration".to_string(),
            "provisional 2.0 domain: wired interface IP addresses, routes, host DNS, interface metrics, and onboard DHCP",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId("assign_static_ip_address".into()),
                "ethernet tray POST /address adds a static IPv4 address to a wired interface",
            ),
            Rationaled::new(
                JourneyId("acquire_dynamic_ip_address".into()),
                "ethernet tray POST /dynamic_ip triggers dhclient DHCP acquisition on a wired interface",
            ),
            Rationaled::new(
                JourneyId("enable_onboard_dhcp_server".into()),
                "ethernet tray POST /dhcp starts dnsmasq as a local DHCP server on a wired interface",
            ),
            Rationaled::new(
                JourneyId("disable_onboard_dhcp_server".into()),
                "ethernet tray DELETE /dhcp stops the onboard dnsmasq DHCP server on a wired interface",
            ),
            Rationaled::new(
                JourneyId("set_network_interface_priority".into()),
                "internet tray POST /set_interfaces_priority persists interface metric ordering for default routes",
            ),
            Rationaled::new(
                JourneyId("configure_host_dns".into()),
                "internet tray POST /host_dns updates locked host nameserver entries in /etc/resolv.conf",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Important,
            "PRIORITY boot tier brings wired connectivity up before most services; MAVLink vehicle control does not depend on it but operator reachability and LAN UX do",
        ),
        offline_required: Asserted::established(
            true,
            "wired IP, route, DNS, DHCP, and interface-metric configuration operate on local interfaces without internet",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; mutates host interfaces, /etc/resolv.conf, /etc/dhcpcd.conf, and runs dnsmasq and dhclient",
        ),
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "dangerous_operations empty per rubric v1.0; network changes are reversible reconfiguration, not irreversible destructive ops",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("assign_static_ip".to_string()),
                "POST /address adds a static IPv4 address to the named wired interface",
            ),
            Rationaled::new(
                CapabilityId("acquire_dynamic_ip".to_string()),
                "POST /dynamic_ip runs dhclient to acquire a dynamic address on the named interface",
            ),
            Rationaled::new(
                CapabilityId("enable_dhcp_server".to_string()),
                "POST /dhcp starts dnsmasq as an onboard DHCP server on the interface gateway address",
            ),
            Rationaled::new(
                CapabilityId("disable_dhcp_server".to_string()),
                "DELETE /dhcp stops the onboard dnsmasq DHCP server on the named interface",
            ),
            Rationaled::new(
                CapabilityId("set_interface_priority".to_string()),
                "POST /set_interfaces_priority persists interface metric ordering used for default-route preference",
            ),
            Rationaled::new(
                CapabilityId("configure_host_dns".to_string()),
                "POST /host_dns updates host nameserver entries and optional immutability lock on /etc/resolv.conf",
            ),
            Rationaled::new(
                CapabilityId("list_network_interfaces".to_string()),
                "GET /interfaces returns all network interfaces with addresses, routes, and DHCP state",
            ),
            Rationaled::new(
                CapabilityId("list_ethernet_interfaces".to_string()),
                "GET /ethernet returns wired ethernet and USB-OTG interfaces",
            ),
            Rationaled::new(
                CapabilityId("retrieve_host_dns".to_string()),
                "GET /host_dns returns current host nameserver entries and resolv.conf lock state",
            ),
            Rationaled::new(
                CapabilityId("get_interface_routes".to_string()),
                "GET /route returns routing table entries for the named interface",
            ),
            Rationaled::new(
                CapabilityId("get_dhcp_server_details".to_string()),
                "GET /dhcp/details/{interface_name} returns onboard DHCP server configuration per interface",
            ),
            Rationaled::new(
                CapabilityId("get_dhcp_server_leases".to_string()),
                "GET /dhcp/leases/{interface_name} returns active dnsmasq DHCP leases per interface",
            ),
        ]),
        authorities: AssertedSet::established(vec![
            Rationaled::new(
                Authority::Other("wired_network_controller".to_string()),
                "sole REST surface for wired interface IP addresses, routes, and metric priority; wifi service owns wlan separately",
            ),
            Rationaled::new(
                Authority::Other("host_dns_writer".to_string()),
                "sole writer of /etc/resolv.conf via chattr and tee; no other cataloged service mutates host DNS",
            ),
            Rationaled::new(
                Authority::Other("onboard_dhcp_server_operator".to_string()),
                "manager of onboard dnsmasq DHCP servers on WIRED interfaces (wifi runs a separate dnsmasq for its uap0 hotspot; /var/lib/dnsmasq lease dir is SharedWrite)",
            ),
        ]),
        states: AssertedSet::unknown(
            "no cataloged state machine; interface manager and DHCP watchdog run periodic reconciliation loops",
        ),
        edges: AssertedSet::unknown(
            "no outbound coupling to other catalog services; network changes are local host configuration only",
        ),
        resources: AssertedSet::established(vec![
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/cable-guy".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "SettingsV2 pydantic manager directory for persisted interface and DHCP configuration",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/cable-guy/settings-2.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "persisted cable_guy SettingsV2 network interface, DHCP, and priority state",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/etc/resolv.conf".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "host DNS nameserver file updated via chattr unlock, tee write, and optional re-lock",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/etc/dhcpcd.conf".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "dhcpcd static and metric configuration written for wired interface management",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/var/lib/dnsmasq".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "dnsmasq lease and DHCP state directory for onboard DHCP servers",
            ),
        ]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                vec!["start-blueos-core create_service".to_string()],
                "observed lifecycle trigger: tmux creation at boot in PRIORITY tier",
            ),
            ordered_after: Asserted::established(
                vec![ServiceId("autopilot".to_string())],
                "observed ordered_after in start-blueos-core PRIORITY block lists cable_guy after autopilot only",
            ),
            ordered_before: Asserted::established(
                vec![
                    ServiceId("video".to_string()),
                    ServiceId("mavlink2rest".to_string()),
                    ServiceId("kraken".to_string()),
                    ServiceId("wifi".to_string()),
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
                "observed ordered_before lists cable_guy before remaining PRIORITY and SERVICES-tier peers including nginx",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit on process termination".to_string(),
                "main.py awaits server.serve with no explicit shutdown hook beyond uvicorn exit",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight interface configuration and dhcpcd migration not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns HTML title; manager.initialize and watchdog task at boot".to_string(),
            "no dedicated /health route; uvicorn availability and manager watchdog serve as health signals",
        ),
        is_platform: Asserted::established(
            false,
            "wired network configuration utility; does not install or host third-party extensions",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 router exposed under /cable-guy/ via VersionedFastAPI",
        ),
        permissions_model: Asserted::established(
            "no separate permissions manifest; REST routes are unauthenticated".to_string(),
            "cable_guy routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "network_reconfiguration_lockout".to_string(),
                "incorrect static IP, route, or DNS change can sever the operator connection until physical or alternate-interface recovery",
            ),
            Rationaled::new(
                "dhclient_acquisition_timeout".to_string(),
                "POST /dynamic_ip may return timeout output when dhclient does not obtain a lease within five seconds",
            ),
            Rationaled::new(
                "dnsmasq_start_failure".to_string(),
                "POST /dhcp logs errors and leaves the interface without a DHCP server when dnsmasq fails to start",
            ),
            Rationaled::new(
                "resolv_conf_write_failure".to_string(),
                "POST /host_dns may fail when chattr or tee cannot update the immutable /etc/resolv.conf file",
            ),
            Rationaled::new(
                "interface_watchdog_reconciliation".to_string(),
                "manager watchdog periodically reconciles interface state; transient mismatches may appear until the next cycle",
            ),
        ]),
        blast_radius: Asserted::established(
            "wired network reachability and operator UI access; MAVLink on the FC may continue but BlueOS web UI and LAN routing can become unreachable".to_string(),
            "misconfigured IP, route, or DNS can lock out the operator; outage blocks ethernet tray and internet-indicator UX",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and SettingsV2 migration stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
