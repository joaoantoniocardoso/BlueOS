use crate::criticality::CriticalityTier;
use crate::edge::{Bus, Edge, FailureImpact, SyncMode};
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
use crate::journey::{HttpMethod, RouteRef};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::GroundedSet;
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, Observed, ObservedSet, Provenance,
    Rationaled,
};
use crate::resource::{Resource, ResourceOwnership};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline};
use crate::service::{Authority, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

const RUNTIME_CAPTURE: &str = "runtime-captures/helper__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId::Helper,
        state_contracts: GroundedSet::unknown(
            "helper has no service-level state machine (card states Unknown)",
        ),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/web_services", 7.4, 8.6, 8.9, 30),
            runtime_slo(HttpMethod::Get, "/check_internet_access", 6.6, 12.2, 12.4, 30),
            runtime_slo(HttpMethod::Get, "/hardware_id", 5.7, 8.4, 8.9, 30),
            runtime_slo(HttpMethod::Get, "/ping?host=1.1.1.1", 28.2, 33.8, 43.4, 30),
        ]),
        resource_usage: GroundedSet::known(vec![runtime_resource(
            "running_baseline",
            Distribution {
                mean: 1.06,
                median: 0.00,
                p95: 1.00,
                min: 0.00,
                max: 43.14,
                sd: 5.51,
            },
            Distribution {
                mean: 62.1,
                median: 61.7,
                p95: 62.6,
                min: 61.7,
                max: 62.6,
                sd: 0.0,
            },
            60,
        )]),
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "helper behavior is platform-independent (local port scan, UUID reads, internet probes); not captured across boards".into(),
                    "runtime captured on Navigator only; RSS ~62.1 MB, CPU ~1.06% mean".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "helper has no settings persistence; nginx snippet writes are config side effects, not settings",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId::Helper,
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
        id: ServiceId::Helper,
        aliases: ObservedSet::known(vec![Evidenced::new(
            "helper".to_string(),
            Evidence {
                file: "core/services/helper/main.py".to_string(),
                line: 43,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 134,
            },
        ),
        entrypoint: Observed::known(
            "$BLUEOS_PYTHON_BIN_SECONDARY $SERVICES_PATH/helper/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 134,
            },
        ),
        tmux_name: Observed::known(
            "helper".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 134,
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
                memory_mb: Some(250),
                cpu_percent: Some(0),
                io_weight: None,
            },
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 134,
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 134,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/helper/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 124,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(81),
            Evidence {
                file: "core/services/helper/main.py".to_string(),
                line: 143,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/helper".to_string()),
            Evidence {
                file: "core/services/helper/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/helper/".to_string()),
                    port: PortRef::Literal(81),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 612,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/home/pi/tools/nginx/nginx.conf".to_string()),
                    mode: FileAccessMode::Read,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 629,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/home/pi/tools/nginx/extensions/".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 491,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/var/run/nginx.pid".to_string()),
                    mode: FileAccessMode::Read,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 453,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/etc/blueos/hardware-uuid".to_string()),
                    mode: FileAccessMode::Read,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 553,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/etc/blueos/uuid".to_string()),
                    mode: FileAccessMode::Read,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 570,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "127.0.0.1".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 294,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "http://localhost/version-chooser/v1.0/version/current".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 507,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "localhost:6040".to_string(),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/mavlink_comm/MavlinkComm.py"
                        .to_string(),
                    line: 24,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "firmware.ardupilot.org".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 56,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "amazon.com".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 61,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "telemetry.blueos.cloud".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 66,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "1.1.1.1".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 74,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "github.com".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 79,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ping".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 586,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "kill -HUP".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 456,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/helper/log".to_string()],
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
                    path: PathRef("/home/pi/tools/nginx/nginx.conf".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 629,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/home/pi/tools/nginx/extensions/".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 491,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/var/run/nginx.pid".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 453,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/etc/blueos/hardware-uuid".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 553,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/etc/blueos/uuid".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 570,
                },
            ),
        ]),
        lifecycle: Observed::known(
            ObservedLifecycle {
                triggers: vec!["start-blueos-core create_service".to_string()],
                ordered_after: vec![
                    ServiceId::ArdupilotManager,
                    ServiceId::CableGuy,
                    ServiceId::MavlinkCameraManager,
                    ServiceId::Mavlink2rest,
                    ServiceId::Kraken,
                    ServiceId::Wifi,
                    ServiceId::Zenohd,
                    ServiceId::Beacon,
                    ServiceId::Bridget,
                    ServiceId::Commander,
                    ServiceId::NmeaInjector,
                ],
                ordered_before: vec![
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
                file: "core/start-blueos-core".to_string(),
                line: 326,
            },
        ),
        logs_path: Observed::unknown(
            "init_logger publishes to zenoh only; no on-disk log path set in helper source",
        ),
        zenoh_log_topic: Observed::known(
            "services/helper/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/helper/main.py".to_string(),
                line: 633,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId::Helper,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one Helper process with shared KNOWN_SERVICES cache",
        ),
        bounded_context: Asserted::established(
            "connectivity-and-service-discovery".to_string(),
            "provisional 2.0 domain: internet reachability probes, HTTP service catalog, extension nginx routing",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId("monitor_internet_connectivity".into()),
                "header indicator polls GET /check_internet_access every 20 seconds",
            ),
            Rationaled::new(
                JourneyId("verify_internet_connectivity".into()),
                "setup wizard RequireInternet confirms probe websites before continuing",
            ),
            Rationaled::new(
                JourneyId("browse_available_web_services".into()),
                "Available Services sidebar page lists GET /web_services scan results",
            ),
            Rationaled::new(
                JourneyId("probe_interface_internet_connectivity".into()),
                "network priority menu calls GET /ping per interface while reordering",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Important,
            "frontend header, service discovery, and extension nginx routing depend on helper; vehicle MAVLink control paths do not",
        ),
        offline_required: Asserted::established(
            true,
            "local port scan, hardware/software IDs, and interface ping work without internet; probes degrade offline but the service must stay up",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; writes nginx extension snippets and sends kill -HUP to nginx",
        ),
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "no irreversible, untrusted-code, or vehicle-arm operations; nginx reload and extension route writes are reversible config",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("check_internet_connectivity".to_string()),
                "GET /check_internet_access probes configured external websites concurrently",
            ),
            Rationaled::new(
                CapabilityId("discover_web_services".to_string()),
                "GET /web_services scans listening TCP ports and fetches /register_service metadata",
            ),
            Rationaled::new(
                CapabilityId("probe_interface_connectivity".to_string()),
                "GET /ping runs ping -I {interface} against a host to test per-interface reachability",
            ),
            Rationaled::new(
                CapabilityId("report_hardware_id".to_string()),
                "GET /hardware_id returns the motherboard-derived UUID from /etc/blueos/hardware-uuid",
            ),
            Rationaled::new(
                CapabilityId("report_software_id".to_string()),
                "GET /software_id returns the install UUID from /etc/blueos/uuid",
            ),
            Rationaled::new(
                CapabilityId("register_web_service".to_string()),
                "scan_ports setup_nginx_route writes /extensionv2/{name}/ proxy snippets for services with metadata",
            ),
            Rationaled::new(
                CapabilityId("reload_nginx".to_string()),
                "reload_nginx sends kill -HUP to the nginx master pid after extension route changes",
            ),
        ]),
        authorities: AssertedSet::established(vec![
            Rationaled::new(
                Authority::UserdataWriter(PathRef("/home/pi/tools/nginx/extensions/".to_string())),
                "sole writer of extension v2 include snippets consumed by nginx.conf include directive",
            ),
            Rationaled::new(
                Authority::Other("nginx_reloader".to_string()),
                "sole sender of HUP to nginx for extension route updates after scan_ports",
            ),
        ]),
        states: AssertedSet::unknown(
            "no cataloged state machine; internet probe results and KNOWN_SERVICES cache are ephemeral request/periodic state",
        ),
        edges: AssertedSet::established(vec![
            Rationaled::new(
                Edge {
                    from: ServiceId::Helper,
                    to: ServiceId::Versionchooser,
                    via: Bus::Rest,
                    sync: SyncMode::Sync,
                    endpoint: "localhost/version-chooser/v1.0/version/current".to_string(),
                    purpose: "query current BlueOS version from version-chooser".to_string(),
                    required_at_boot: false,
                    failure_impact: FailureImpact::Degraded,
                },
                "observed OutboundHttp http://localhost/version-chooser/v1.0/version/current (helper/main.py:507) pairs with versionchooser listen 8081 and nginx prefix /version-chooser/; external connectivity probe URLs intentionally excluded",
            ),
            Rationaled::new(
                Edge {
                    from: ServiceId::Helper,
                    to: ServiceId::Mavlink2rest,
                    via: Bus::Rest,
                    sync: SyncMode::Async,
                    endpoint: "localhost:6040".to_string(),
                    purpose: "query vehicle MAVLink data (e.g. vehicle type/heartbeat) via mavlink2rest"
                        .to_string(),
                    required_at_boot: false,
                    failure_impact: FailureImpact::Degraded,
                },
                "observed OutboundHttp localhost:6040 (MavlinkComm.py:24) pairs with mavlink2rest listen 6040; external connectivity probe URLs intentionally excluded",
            ),
        ]),
        resources: AssertedSet::established(vec![
            Rationaled::new(
                Resource {
                    path: PathRef("/home/pi/tools/nginx/nginx.conf".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                "parse_nginx_file reads nginx.conf to map ports to service path prefixes",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/home/pi/tools/nginx/extensions/".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "setup_nginx_route writes per-extension proxy snippets included by nginx",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/var/run/nginx.pid".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                "reload_nginx reads the nginx master pid before kill -HUP",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/etc/blueos/hardware-uuid".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                "GET /hardware_id reads the motherboard-derived identifier file",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/etc/blueos/uuid".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                "GET /software_id reads the install UUID file",
            ),
        ]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                vec!["start-blueos-core create_service".to_string()],
                "observed lifecycle trigger: tmux creation at boot in SERVICES tier",
            ),
            ordered_after: Asserted::established(
                vec![
                    ServiceId::ArdupilotManager,
                    ServiceId::CableGuy,
                    ServiceId::MavlinkCameraManager,
                    ServiceId::Mavlink2rest,
                    ServiceId::Kraken,
                    ServiceId::Wifi,
                    ServiceId::Zenohd,
                    ServiceId::Beacon,
                    ServiceId::Bridget,
                    ServiceId::Commander,
                    ServiceId::NmeaInjector,
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                vec![
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
                "observed ordered_before lists helper before remaining SERVICES-tier peers including nginx",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit on process termination".to_string(),
                "main.py awaits server.serve with no explicit shutdown hook beyond uvicorn exit",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for KNOWN_SERVICES cache and extension nginx snippets not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns HTML title; periodic task every 60s".to_string(),
            "no dedicated /health route; uvicorn availability and background periodic task serve as health signals",
        ),
        is_platform: Asserted::established(
            false,
            "discovers and proxies extension HTTP services but does not install or run third-party containers",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 router exposed under /helper/ via VersionedFastAPI",
        ),
        permissions_model: Asserted::established(
            "no separate permissions manifest; REST routes are unauthenticated".to_string(),
            "helper routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "nginx_reload_failure".to_string(),
                "reload_nginx uses subprocess.run with check=False; HUP failure leaves stale extension routes",
            ),
            Rationaled::new(
                "internet_probe_unreachable".to_string(),
                "check_website timeouts mark sites offline; header indicator shows disconnected state",
            ),
            Rationaled::new(
                "service_scan_timeout".to_string(),
                "detect_service returns invalid ServiceInfo after repeated localhost probe timeouts",
            ),
            Rationaled::new(
                "factory_mode_notification_failure".to_string(),
                "check_and_notify_factory_mode logs and skips MAVLink statustext when version-chooser is unreachable",
            ),
            Rationaled::new(
                "uuid_read_failure".to_string(),
                "GET /hardware_id and GET /software_id return 400 when UUID files are missing or invalid",
            ),
        ]),
        blast_radius: Asserted::established(
            "internet indicator, service discovery UI, and extension nginx routes stale; core vehicle MAVLink and autopilot unaffected"
                .to_string(),
            "helper outage blocks connectivity UX and extension proxy registration but not FC control paths",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and probe website list stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
