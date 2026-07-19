use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::Interface;
use crate::journey::{HttpMethod, RouteRef};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline};
use crate::service::{Authority, Service, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

// capture used RepoDigest as primary
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a), Raspberry Pi 4, Navigator";

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Linux2rest,
        state_contracts: GroundedSet::unknown(
            "linux2rest has no service-level state machine (card states Unknown); external Rust binary with no traced lifecycle states",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/system", 10.0, 16.2, 23.7, 60),
            runtime_slo(HttpMethod::Get, "/system/cpu", 1.6, 4.3, 12.2, 60),
            runtime_slo(HttpMethod::Get, "/system/memory", 1.4, 2.7, 3.2, 60),
            runtime_slo(HttpMethod::Get, "/system/disk", 1.4, 2.1, 2.4, 60),
            runtime_slo(HttpMethod::Get, "/system/network", 1.6, 2.8, 2.8, 60),
            runtime_slo(HttpMethod::Get, "/system/temperature", 2.0, 2.9, 4.1, 60),
            runtime_slo(HttpMethod::Get, "/system/process", 8.7, 13.5, 18.0, 60),
            runtime_slo(HttpMethod::Get, "/platform", 1.4, 1.7, 3.0, 60),
            runtime_slo(HttpMethod::Get, "/serial", 1.5, 1.9, 3.9, 60),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
            "running_baseline",
            Distribution {
                mean: 2.48,
                median: 0.0,
                p95: 10.84,
                min: 0.0,
                max: 40.92,
                sd: 5.77,
            },
            Distribution {
                mean: 28.1,
                median: 28.1,
                p95: 28.1,
                min: 28.1,
                max: 28.1,
                sd: 0.0,
            },
            90,
        )]),
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "linux2rest reports host platform/hardware via GET /platform; service behavior is platform-independent",
                    "runtime captured on Navigator only; Rust binary RSS ~28.1 MB flat, CPU ~2.48% mean with sampler-tick spikes",
                    "observed REST API is unversioned at runtime (routes directly under /system-information/, no /v1 prefix)",
                ],
            },
            runtime_prov("runtime-captures/linux2rest__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "linux2rest is read-only with no traced on-disk settings paths; no mutating routes exist",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::Linux2rest,
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
        runtime_prov("runtime-captures/linux2rest__pi4_navigator_master.json#slo_running_baseline"),
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
        runtime_prov("runtime-captures/linux2rest__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts =
    ObservedFacts {
        id: ServiceId::Linux2rest,
        aliases: ObservedSet::unknown(
            "external binary; no in-repo alias declarations in this repository",
        ),
        kind: Observed::known(
            ServiceKind::Binary,
            Evidence {
                file: "core/start-blueos-core",
                line: 136,
            },
        ),
        entrypoint: Observed::known(
            "linux2rest --log-settings netstat=30,platform=10,serial-ports=10,cpu=10,disk=30,info=10,memory=10,network=10,process=60,temperature=10,unix-time-seconds=10,usb=60"
                ,
            Evidence {
                file: "core/start-blueos-core",
                line: 136,
            },
        ),
        tmux_name: Observed::known(
            "linux2rest",
            Evidence {
                file: "core/start-blueos-core",
                line: 136,
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
                memory_mb: Some(250),
                cpu_percent: Some(0),
                io_weight: None,
            },
            Evidence {
                file: "core/start-blueos-core",
                line: 136,
            },
        ),
        nice: Observed::unknown("command line has no nice wrapper"),
        run_as: Observed::known(
            "root",
            Evidence {
                file: "core/start-blueos-core",
                line: 136,
            },
        ),
        nginx_prefixes: ObservedSet::known(&[Evidenced::new(
            PathRef("/system-information/"),
            Evidence {
                file: "core/tools/nginx/nginx.conf",
                line: 205,
            },
        )]),
        listen: ObservedSet::known(&[Evidenced::new(
            PortRef::Literal(6030),
            Evidence {
                file: "core/tools/nginx/nginx.conf",
                line: 207,
            },
        )]),
        git_path: Observed::unknown(
            "external Rust binary (upstream github.com/bluerobotics/linux2rest); no source tree in this repository",
        ),
        interfaces: ObservedSet::known(&[Evidenced::new(
            Interface::Rest {
                path_prefix: PathRef("/system-information/"),
                port: PortRef::Literal(6030),
                versions: &[],
            },
            Evidence {
                file: "core/tools/nginx/nginx.conf",
                line: 207,
            },
        )]),
        resources: ObservedSet::unknown(
            "no settings paths or userdata files referenced in start-blueos-core launch args",
        ),
        lifecycle: Observed::known(
            ObservedLifecycle {
                triggers: &["start-blueos-core create_service"],
                ordered_after: &[
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
                    ServiceId::Helper,
                    ServiceId::Iperf3,
                ],
                ordered_before: &[
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
            "external Rust binary; no on-disk log path in start-blueos-core launch args",
        ),
        zenoh_log_topic: Observed::unknown(
            "external Rust binary; does not use commonwealth init_logger zenoh publisher",
        ),
        sentry: Observed::unknown(
            "external Rust binary; no init_sentry or equivalent traced in this repository",
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    };

pub const SERVICE_DEFINITION: ServiceDefinition =
    ServiceDefinition {
        id: ServiceId::Linux2rest,
        singleton: Asserted::established(
            true,
            "single Normal-tier tmux instance; one linux2rest process serves host system information over REST",
        ),
        bounded_context: Asserted::established(
            "system-information-provider",
            "provisional 2.0 domain: read-only Linux host telemetry (CPU, memory, disk, network, processes, serial ports, USB) over HTTP",
        ),
        journey_refs: AssertedSet::established(&[Rationaled::new(
            JourneyId::ViewSystemInformation,
            "System Information page and System Monitor widgets poll linux2rest for live host metrics",
        )]),
        tier: Asserted::established(
            CriticalityTier::Important,
            "Normal-tier shared data source; bridget serial enumeration and helper service discovery depend on localhost:6030, but vehicle flight does not require live host telemetry",
        ),
        offline_required: Asserted::established(
            true,
            "reads local /proc, sysfs, and host interfaces only; no internet or WAN dependency",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root in start-blueos-core Normal-tier launch line; enumerates all processes, USB devices, and netstat as root",
        ),
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "read-only host telemetry REST API; no irreversible, untrusted-code, or vehicle-arm operations per rubric",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::ViewSystemInformation,
                "System Information page and monitor widgets display CPU, memory, disk, temperature, processes, and network data from observed Rest /system-information/",
            ),
            Rationaled::new(
                CapabilityId::ProvideSystemInformationOverRest,
                "observed Rest interface on port 6030; bridget GET localhost:6030/serial and helper port-6030 discovery consume machine-facing host telemetry",
            ),
        ]),
        authorities: AssertedSet::established(&[Rationaled::new(
            Authority::Other("system_information_provider"),
            "sole catalog service exposing consolidated Linux host system information over HTTP REST; bridget and helper depend on it instead of duplicating /proc reads",
        )]),
        states: AssertedSet::unknown(
            "external Rust binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(&[]),
        resources: AssertedSet::unknown(
            "observed artifact has no settings paths or userdata files; external binary with no traced on-disk resources",
        ),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                &["start-blueos-core create_service"],
                "observed lifecycle trigger: tmux creation at boot in Normal tier",
            ),
            ordered_after: Asserted::established(
                &[
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
                    ServiceId::Helper,
                    ServiceId::Iperf3,
                ],
                "observed ordered_after in start-blueos-core Normal block",
            ),
            ordered_before: Asserted::established(
                &[
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
                "observed ordered_before lists linux2rest before remaining Normal and SERVICES-tier peers",
            ),
            shutdown: Asserted::unknown(
                "external Rust binary; no explicit shutdown handler traced in this repository",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade restart semantics for the external linux2rest binary not traced in this repository",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST /system-information/ listener availability serves as health signal"
                ,
            "no dedicated /health route traced; process continuity and HTTP listener serve as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "core host telemetry provider; does not install or host third-party extensions",
        ),
        api_stable: Asserted::unknown(
            "external binary with empty observed REST versions list; upstream API stability not established from this repository",
        ),
        permissions_model: Asserted::established(
            "no auth middleware traced; REST routes are unauthenticated on the LAN",
            "external binary proxied by nginx without observed permission checks; LAN trust model matches other core REST bridges",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "rest_listener_down",
                "process exit or port 6030 bind failure blocks all system-information REST consumers including the frontend and bridget",
            ),
            Rationaled::new(
                "privileged_proc_read_failure",
                "root-only host queries (process list, USB, netstat) may return partial or empty data when kernel interfaces are unavailable",
            ),
            Rationaled::new(
                "serial_port_enumeration_stale",
                "bridget GET /serial_ports proxy to localhost:6030/serial fails when linux2rest is unreachable",
            ),
        ]),
        blast_radius: Asserted::established(
            "System Information UI loses live host metrics; bridget cannot enumerate serial ports; helper loses port-6030 system data; read-only outage with no vehicle-control impact"
                ,
            "shared host-telemetry provider outage degrades diagnostics and serial-bridge setup but does not affect autopilot, MAVLink routing, or flight safety",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream linux2rest API deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Linux2rest,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
