use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, PortKind};
use crate::journey::{HttpMethod, RouteRef};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use crate::resource::{Resource, ResourceOwnership};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline};
use crate::service::{Authority, Service, ServiceJudgment};
use crate::trust::{PrivilegeLevel, UserConfirmation};

use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::CableGuy,
        state_contracts: GroundedSet::unknown(
            "cable_guy has no service-level state machine (card states Unknown); a manager watchdog reconciles interface state periodically",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/interfaces", 12.4, 24.4, 26.5, 40),
            runtime_slo(HttpMethod::Get, "/ethernet", 11.0, 19.7, 19.8, 40),
            runtime_slo(HttpMethod::Get, "/host_dns", 1215.2, 1302.9, 1314.2, 40),
            runtime_slo(HttpMethod::Get, "/route?interface_name=eth0", 25.9, 32.8, 35.9, 40),
            runtime_slo(HttpMethod::Get, "/dhcp/details/eth0", 7.7, 15.7, 22.7, 40),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
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
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "cable_guy manages host wired interfaces regardless of flight controller; platform-independent",
                    "runtime captured on Navigator only; RSS ~53.1 MB flat, CPU ~2.47% mean (watchdog reconciliation spikes)",
                    "GET /host_dns is ~1.2 s: it shells out (cat/lsattr on /etc/resolv.conf) per request",
                ],
            },
            runtime_prov("runtime-captures/cable_guy__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "mutating routes persist to /root/.config/cable-guy/settings-2.json, /etc/dhcpcd.conf, /etc/resolv.conf but were not exercised (network-lockout hazard)",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::CableGuy,
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
        runtime_prov("runtime-captures/cable_guy__pi4_navigator_master.json#slo_running_baseline"),
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
        runtime_prov("runtime-captures/cable_guy__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts = ObservedFacts {
    id: ServiceId::CableGuy,
    aliases: ObservedSet::known(&[
        Evidenced::new(
            "cable_guy",
            Evidence {
                file: "core/start-blueos-core",
                line: 119,
                anchor: "'cable_guy',0,0,0,0,\"$SERVICES_PATH/cable_guy/main.py\"",
            },
        ),
        Evidenced::new(
            "cable-guy",
            Evidence {
                file: "core/services/cable_guy/config.py",
                line: 6,
                anchor: "SERVICE_NAME = \"cable-guy\"",
            },
        ),
    ]),
    kind: Observed::known(
        ServiceKind::PythonService,
        Evidence {
            file: "core/start-blueos-core",
            line: 119,
            anchor: "'cable_guy',0,0,0,0,\"$SERVICES_PATH/cable_guy/main.py\"",
        },
    ),
    entrypoint: Observed::known(
        "$SERVICES_PATH/cable_guy/main.py",
        Evidence {
            file: "core/start-blueos-core",
            line: 119,
            anchor: "'cable_guy',0,0,0,0,\"$SERVICES_PATH/cable_guy/main.py\"",
        },
    ),
    tmux_name: Observed::known(
        "cable_guy",
        Evidence {
            file: "core/start-blueos-core",
            line: 119,
            anchor: "'cable_guy',0,0,0,0,\"$SERVICES_PATH/cable_guy/main.py\"",
        },
    ),
    startup_tier: Observed::known(
        StartupTier::Priority,
        Evidence {
            file: "core/start-blueos-core",
            line: 117,
            anchor: "PRIORITY_SERVICES=(",
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
            line: 119,
            anchor: "'cable_guy',0,0,0,0,\"$SERVICES_PATH/cable_guy/main.py\"",
        },
    ),
    nice: Observed::unknown("no nice prefix in start tuple"),
    run_as: Observed::known(
        "root",
        Evidence {
            file: "core/services/cable_guy/main.py",
            line: 193,
            anchor: "if os.geteuid() != 0:",
        },
    ),
    nginx_prefixes: ObservedSet::known(&[Evidenced::new(
        PathRef("/cable-guy/"),
        Evidence {
            file: "core/tools/nginx/nginx.conf",
            line: 103,
            anchor: "location /cable-guy/ {",
        },
    )]),
    listen: ObservedSet::known(&[Evidenced::new(
        PortRef::Literal(9090),
        Evidence {
            file: "core/services/cable_guy/main.py",
            line: 183,
            anchor: "config = Config(app=app, host=\"0.0.0.0\", port=9090, log_conf",
        },
    )]),
    git_path: Observed::known(
        PathRef("core/services/cable_guy"),
        Evidence {
            file: "core/services/cable_guy/main.py",
            line: 1,
            anchor: "#! /usr/bin/env python3",
        },
    ),
    interfaces: ObservedSet::known(&[
        Evidenced::new(
            PortKind::Rest {
                path_prefix: PathRef("/cable-guy/"),
                port: PortRef::Literal(9090),
                versions: &["v1.0"],
            },
            Evidence {
                file: "core/services/cable_guy/main.py",
                line: 163,
                anchor: "prefix_format=\"/v{major}.{minor}\",",
            },
        ),
        Evidenced::new(
            PortKind::Settings {
                path: PathRef("/root/.config/cable-guy/settings-2.json"),
            },
            Evidence {
                file:
                    "core/libs/commonwealth/src/commonwealth/settings/managers/pydantic_manager.py",
                line: 73,
                anchor: "return self.config_folder.joinpath(f\"{PydanticManager.SETTIN",
            },
        ),
        Evidenced::new(
            PortKind::File {
                path: PathRef("/root/.config/cable-guy"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file:
                    "core/libs/commonwealth/src/commonwealth/settings/managers/pydantic_manager.py",
                line: 27,
                anchor: "else pathlib.Path(appdirs.user_config_dir(self.project_name)",
            },
        ),
        Evidenced::new(
            PortKind::File {
                path: PathRef("/root/.config/cable-guy/settings-1.json"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/services/cable_guy/api/settings.py",
                line: 40,
                anchor: "settings_v1_file = file_path.parent / \"settings-1.json\"",
            },
        ),
        Evidenced::new(
            PortKind::File {
                path: PathRef("/root/.config/cable-guy/settings.json"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/services/cable_guy/api/settings.py",
                line: 47,
                anchor: "old_settings_file_path = file_path.parent / \"settings.json\"",
            },
        ),
        Evidenced::new(
            PortKind::File {
                path: PathRef("/etc/resolv.conf"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/services/cable_guy/api/dns.py",
                line: 9,
                anchor: "RESOLVCONF_FILE_PATH: str = \"/etc/resolv.conf\"",
            },
        ),
        Evidenced::new(
            PortKind::File {
                path: PathRef("/etc/dhcpcd.conf"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/services/cable_guy/networksetup.py",
                line: 254,
                anchor: "dhcpcd_conf_path = \"/etc/dhcpcd.conf\"",
            },
        ),
        Evidenced::new(
            PortKind::File {
                path: PathRef("/var/lib/dnsmasq"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/DHCPServerManager.py",
                line: 50,
                anchor: "lease_dir: pathlib.Path = pathlib.Path(\"/var/lib/dnsmasq\"),",
            },
        ),
        Evidenced::new(
            PortKind::Subprocess {
                command: "cat '{filename}'",
            },
            Evidence {
                file: "core/services/cable_guy/api/dns.py",
                line: 56,
                anchor: "output = run_command(f\"cat '{filename}'\")",
            },
        ),
        Evidenced::new(
            PortKind::Subprocess {
                command: "lsattr {filename}",
            },
            Evidence {
                file: "core/services/cable_guy/api/dns.py",
                line: 64,
                anchor: "output = run_command(f\"lsattr {filename}\")",
            },
        ),
        Evidenced::new(
            PortKind::Subprocess {
                command: "sudo chattr -i {filename}",
            },
            Evidence {
                file: "core/services/cable_guy/api/dns.py",
                line: 80,
                anchor: "output = run_command(f\"sudo chattr -i {filename}\")",
            },
        ),
        Evidenced::new(
            PortKind::Subprocess {
                command: "echo '{content}' | sudo tee {filename}",
            },
            Evidence {
                file: "core/services/cable_guy/api/dns.py",
                line: 86,
                anchor: "output = run_command(f\"echo '{content}' | sudo tee {filename",
            },
        ),
        Evidenced::new(
            PortKind::Subprocess {
                command: "sudo chattr +i {filename}",
            },
            Evidence {
                file: "core/services/cable_guy/api/dns.py",
                line: 74,
                anchor: "output = run_command(f\"sudo chattr +i {filename}\")",
            },
        ),
        Evidenced::new(
            PortKind::Subprocess {
                command: "timeout 5 dhclient -d -v {interface_name} 2>&1 || echo 'timeout'",
            },
            Evidence {
                file: "core/services/cable_guy/networksetup.py",
                line: 124,
                anchor: "command = f\"timeout 5 dhclient -d -v {interface_name} 2>&1 |",
            },
        ),
        Evidenced::new(
            PortKind::Subprocess {
                command: "ifmetric",
            },
            Evidence {
                file: "core/services/cable_guy/api/manager.py",
                line: 605,
                anchor: "[\"ifmetric\", name, str(priority)],",
            },
        ),
        Evidenced::new(
            PortKind::Subprocess { command: "dnsmasq" },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/DHCPServerManager.py",
                line: 95,
                anchor: "return \"dnsmasq\"",
            },
        ),
        Evidenced::new(
            PortKind::Zenoh {
                topics_produced: &["services/cable-guy/log"],
                topics_consumed: &[],
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                line: 78,
                anchor: "topic = f\"services/{service_name}/log\"",
            },
        ),
    ]),
    resources: ObservedSet::known(&[
        Evidenced::new(
            Resource {
                path: PathRef("/root/.config/cable-guy"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file:
                    "core/libs/commonwealth/src/commonwealth/settings/managers/pydantic_manager.py",
                line: 27,
                anchor: "else pathlib.Path(appdirs.user_config_dir(self.project_name)",
            },
        ),
        Evidenced::new(
            Resource {
                path: PathRef("/root/.config/cable-guy/settings-2.json"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file:
                    "core/libs/commonwealth/src/commonwealth/settings/managers/pydantic_manager.py",
                line: 73,
                anchor: "return self.config_folder.joinpath(f\"{PydanticManager.SETTIN",
            },
        ),
        Evidenced::new(
            Resource {
                path: PathRef("/etc/resolv.conf"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/cable_guy/api/dns.py",
                line: 86,
                anchor: "output = run_command(f\"echo '{content}' | sudo tee {filename",
            },
        ),
        Evidenced::new(
            Resource {
                path: PathRef("/etc/dhcpcd.conf"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/cable_guy/networksetup.py",
                line: 355,
                anchor: "with open(\"/etc/dhcpcd.conf\", \"a+\", encoding=\"utf-8\") as f:",
            },
        ),
        Evidenced::new(
            Resource {
                path: PathRef("/var/lib/dnsmasq"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/DHCPServerManager.py",
                line: 79,
                anchor: "lease_dir.mkdir(parents=True, exist_ok=True)",
            },
        ),
    ]),
    lifecycle: Observed::known(
        ObservedLifecycle {
            triggers: &["start-blueos-core create_service"],
            ordered_after: &[ServiceId::ArdupilotManager],
            ordered_before: &[
                ServiceId::MavlinkCameraManager,
                ServiceId::Mavlink2rest,
                ServiceId::Kraken,
                ServiceId::Wifi,
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
            line: 318,
            anchor: "for TUPLE in \"${PRIORITY_SERVICES[@]}\"; do",
        },
    ),
    logs_path: Observed::unknown(
        "init_logger publishes to zenoh only; no on-disk log path set in cable_guy source",
    ),
    zenoh_log_topic: Observed::known(
        "services/cable-guy/log",
        Evidence {
            file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
            line: 78,
            anchor: "topic = f\"services/{service_name}/log\"",
        },
    ),
    sentry: Observed::known(
        true,
        Evidence {
            file: "core/services/cable_guy/main.py",
            line: 181,
            anchor: "await init_sentry_async(SERVICE_NAME)",
        },
    ),
    openapi_refs: ObservedSet::unknown("not yet extracted"),
};

pub const SERVICE_DEFINITION: ServiceJudgment =
    ServiceJudgment {
        id: ServiceId::CableGuy,
        singleton: Asserted::established(
            true,
            "single PRIORITY-tier tmux instance; one cable_guy process on port 9090 owns wired network configuration",
        ),
        bounded_context: Asserted::established(
            "wired-network-configuration",
            "provisional 2.0 domain: wired interface IP addresses, routes, host DNS, interface metrics, and onboard DHCP",
        ),
        journey_refs: AssertedSet::established(&[
            Rationaled::new(
                JourneyId::AssignStaticIpAddress,
                "ethernet tray POST /address adds a static IPv4 address to a wired interface",
            ),
            Rationaled::new(
                JourneyId::AcquireDynamicIpAddress,
                "ethernet tray POST /dynamic_ip triggers dhclient DHCP acquisition on a wired interface",
            ),
            Rationaled::new(
                JourneyId::EnableOnboardDhcpServer,
                "ethernet tray POST /dhcp starts dnsmasq as a local DHCP server on a wired interface",
            ),
            Rationaled::new(
                JourneyId::DisableOnboardDhcpServer,
                "ethernet tray DELETE /dhcp stops the onboard dnsmasq DHCP server on a wired interface",
            ),
            Rationaled::new(
                JourneyId::SetNetworkInterfacePriority,
                "internet tray POST /set_interfaces_priority persists interface metric ordering for default routes",
            ),
            Rationaled::new(
                JourneyId::ConfigureHostDns,
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
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "dangerous_operations empty per rubric v1.0; network changes are reversible reconfiguration, not irreversible destructive ops",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::AssignStaticIp,
                "POST /address adds a static IPv4 address to the named wired interface",
            ),
            Rationaled::new(
                CapabilityId::AcquireDynamicIp,
                "POST /dynamic_ip runs dhclient to acquire a dynamic address on the named interface",
            ),
            Rationaled::new(
                CapabilityId::EnableDhcpServer,
                "POST /dhcp starts dnsmasq as an onboard DHCP server on the interface gateway address",
            ),
            Rationaled::new(
                CapabilityId::DisableDhcpServer,
                "DELETE /dhcp stops the onboard dnsmasq DHCP server on the named interface",
            ),
            Rationaled::new(
                CapabilityId::SetInterfacePriority,
                "POST /set_interfaces_priority persists interface metric ordering used for default-route preference",
            ),
            Rationaled::new(
                CapabilityId::ConfigureHostDns,
                "POST /host_dns updates host nameserver entries and optional immutability lock on /etc/resolv.conf",
            ),
            Rationaled::new(
                CapabilityId::ListNetworkInterfaces,
                "GET /interfaces returns all network interfaces with addresses, routes, and DHCP state",
            ),
            Rationaled::new(
                CapabilityId::ListEthernetInterfaces,
                "GET /ethernet returns wired ethernet and USB-OTG interfaces",
            ),
            Rationaled::new(
                CapabilityId::RetrieveHostDns,
                "GET /host_dns returns current host nameserver entries and resolv.conf lock state",
            ),
            Rationaled::new(
                CapabilityId::GetInterfaceRoutes,
                "GET /route returns routing table entries for the named interface",
            ),
            Rationaled::new(
                CapabilityId::GetDhcpServerDetails,
                "GET /dhcp/details/{interface_name} returns onboard DHCP server configuration per interface",
            ),
            Rationaled::new(
                CapabilityId::GetDhcpServerLeases,
                "GET /dhcp/leases/{interface_name} returns active dnsmasq DHCP leases per interface",
            ),
        ]),
        authorities: AssertedSet::established(&[
            Rationaled::new(
                Authority::Other("wired_network_controller"),
                "sole REST surface for wired interface IP addresses, routes, and metric priority; wifi service owns wlan separately",
            ),
            Rationaled::new(
                Authority::Other("host_dns_writer"),
                "sole writer of /etc/resolv.conf via chattr and tee; no other cataloged service mutates host DNS",
            ),
            Rationaled::new(
                Authority::Other("onboard_dhcp_server_operator"),
                "manager of onboard dnsmasq DHCP servers on WIRED interfaces (wifi runs a separate dnsmasq for its uap0 hotspot; /var/lib/dnsmasq lease dir is SharedWrite)",
            ),
        ]),
        states: AssertedSet::unknown(
            "no cataloged state machine; interface manager and DHCP watchdog run periodic reconciliation loops",
        ),
        edges: AssertedSet::established(&[]),
        resources: AssertedSet::established(&[
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/cable-guy"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "SettingsV2 pydantic manager directory for persisted interface and DHCP configuration",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/cable-guy/settings-2.json"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "persisted cable_guy SettingsV2 network interface, DHCP, and priority state",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/etc/resolv.conf"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "host DNS nameserver file updated via chattr unlock, tee write, and optional re-lock",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/etc/dhcpcd.conf"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "dhcpcd static and metric configuration written for wired interface management",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/var/lib/dnsmasq"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "dnsmasq lease and DHCP state directory for onboard DHCP servers",
            ),
        ]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                &["start-blueos-core create_service"],
                "observed lifecycle trigger: tmux creation at boot in PRIORITY tier",
            ),
            ordered_after: Asserted::established(
                &[ServiceId::ArdupilotManager],
                "observed ordered_after in start-blueos-core PRIORITY block lists cable_guy after autopilot only",
            ),
            ordered_before: Asserted::established(
                &[
                    ServiceId::MavlinkCameraManager,
                    ServiceId::Mavlink2rest,
                    ServiceId::Kraken,
                    ServiceId::Wifi,
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
                "observed ordered_before lists cable_guy before remaining PRIORITY and SERVICES-tier peers including nginx",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit on process termination",
                "main.py awaits server.serve with no explicit shutdown hook beyond uvicorn exit",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight interface configuration and dhcpcd migration not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns HTML title; manager.initialize and watchdog task at boot",
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
            "no separate permissions manifest; REST routes are unauthenticated",
            "cable_guy routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "network_reconfiguration_lockout",
                "incorrect static IP, route, or DNS change can sever the operator connection until physical or alternate-interface recovery",
            ),
            Rationaled::new(
                "dhclient_acquisition_timeout",
                "POST /dynamic_ip may return timeout output when dhclient does not obtain a lease within five seconds",
            ),
            Rationaled::new(
                "dnsmasq_start_failure",
                "POST /dhcp logs errors and leaves the interface without a DHCP server when dnsmasq fails to start",
            ),
            Rationaled::new(
                "resolv_conf_write_failure",
                "POST /host_dns may fail when chattr or tee cannot update the immutable /etc/resolv.conf file",
            ),
            Rationaled::new(
                "interface_watchdog_reconciliation",
                "manager watchdog periodically reconciles interface state; transient mismatches may appear until the next cycle",
            ),
        ]),
        blast_radius: Asserted::established(
            "wired network reachability and operator UI access; MAVLink on the FC may continue but BlueOS web UI and LAN routing can become unreachable",
            "misconfigured IP, route, or DNS can lock out the operator; outage blocks ethernet tray and internet-indicator UX",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and SettingsV2 migration stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::CableGuy,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
