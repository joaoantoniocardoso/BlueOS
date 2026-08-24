use catalog_kernel::criticality::CriticalityTier;
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::refs::{PathRef, PortRef};
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use catalog_model::edge::{Bus, Connection, FailureImpact, SyncMode};
use catalog_model::interface::{MavlinkRole, PortKind};
use catalog_model::journey::{HttpMethod, RouteRef};
use catalog_model::lifecycle::{Lifecycle, ObservedLifecycle};
use catalog_model::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use catalog_model::runtime::{
    Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline,
};
use catalog_model::service::{Authority, Service, ServiceJudgment};
use catalog_model::trust::{PrivilegeLevel, UserConfirmation};

use catalog_kernel::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Mavlink2rest,
        state_contracts: GroundedSet::unknown(
            "mavlink2rest has no service-level state machine (card states Unknown); external Rust binary with no traced lifecycle states",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/v1/mavlink", 3.9, 6.3, 8.3, 60),
            runtime_slo(HttpMethod::Get, "/", 1.3, 1.7, 3.1, 60),
            runtime_slo(HttpMethod::Get, "/v1/mavlink/HEARTBEAT", 3.4, 4.9, 5.2, 60),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
            "running_baseline",
            Distribution {
                mean: 1.33,
                median: 1.01,
                p95: 2.03,
                min: 0.0,
                max: 33.59,
                sd: 3.51,
            },
            Distribution {
                mean: 8.8,
                median: 8.8,
                p95: 8.8,
                min: 8.8,
                max: 8.8,
                sd: 0.0,
            },
            90,
        )]),
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "mavlink2rest consumes udpout:127.0.0.1:14001 from ardupilot_manager; platform-independent",
                    "runtime captured on Navigator only; Rust binary RSS ~8.8 MB flat, CPU ~1.33% mean with occasional serialization spikes",
                    "observed REST API path prefix /v1 at runtime (observed_facts versions list empty)",
                ],
            },
            runtime_prov("runtime-captures/mavlink2rest__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "mavlink2rest has no traced on-disk settings paths; POST /v1/mavlink injects MAVLink to the vehicle — not exercised",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::Mavlink2rest,
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
        runtime_prov(
            "runtime-captures/mavlink2rest__pi4_navigator_master.json#slo_running_baseline",
        ),
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
        runtime_prov("runtime-captures/mavlink2rest__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts =
    ObservedFacts {
        id: ServiceId::Mavlink2rest,
        aliases: ObservedSet::unknown(
            "external binary; no in-repo alias declarations in this repository",
        ),
        kind: Observed::known(
            ServiceKind::Binary,
            Evidence {
                file: "core/start-blueos-core",
                line: 121,
                anchor: "'mavlink2rest',0,0,0,0,\"mavlink2rest --connect=udpout:127.0.",
            },
        ),
        entrypoint: Observed::known(
            "mavlink2rest --connect=udpout:127.0.0.1:14001 --server [::]:6040 --system-id $MAV_SYSTEM_ID --component-id $MAV_COMPONENT_ID_ONBOARD_COMPUTER4"
                ,
            Evidence {
                file: "core/start-blueos-core",
                line: 121,
                anchor: "'mavlink2rest',0,0,0,0,\"mavlink2rest --connect=udpout:127.0.",
            },
        ),
        tmux_name: Observed::known(
            "mavlink2rest",
            Evidence {
                file: "core/start-blueos-core",
                line: 121,
                anchor: "'mavlink2rest',0,0,0,0,\"mavlink2rest --connect=udpout:127.0.",
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
                line: 121,
                anchor: "'mavlink2rest',0,0,0,0,\"mavlink2rest --connect=udpout:127.0.",
            },
        ),
        nice: Observed::unknown("command line has no nice wrapper"),
        run_as: Observed::known(
            "root",
            Evidence {
                file: "core/start-blueos-core",
                line: 121,
                anchor: "'mavlink2rest',0,0,0,0,\"mavlink2rest --connect=udpout:127.0.",
            },
        ),
        nginx_prefixes: ObservedSet::known(&[Evidenced::new(
            PathRef("/mavlink2rest/"),
            Evidence {
                file: "core/tools/nginx/nginx.conf",
                line: 169,
                anchor: "location /mavlink2rest/ {",
            },
        )]),
        listen: ObservedSet::known(&[Evidenced::new(
            PortRef::Literal(6040),
            Evidence {
                file: "core/start-blueos-core",
                line: 121,
                anchor: "'mavlink2rest',0,0,0,0,\"mavlink2rest --connect=udpout:127.0.",
            },
        )]),
        git_path: Observed::unknown(
            "external Rust binary (upstream github.com/bluerobotics/mavlink2rest); no source tree in this repository",
        ),
        interfaces: ObservedSet::known(&[
            Evidenced::new(
                PortKind::Rest {
                    path_prefix: PathRef("/mavlink2rest/"),
                    port: PortRef::Literal(6040),
                    versions: &[],
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf",
                    line: 174,
                    anchor: "proxy_pass http://127.0.0.1:6040/;",
                },
            ),
            Evidenced::new(
                PortKind::Mavlink {
                    role: MavlinkRole::Consumer,
                    connect: "udpout:127.0.0.1:14001",
                },
                Evidence {
                    file: "core/start-blueos-core",
                    line: 121,
                    anchor: "'mavlink2rest',0,0,0,0,\"mavlink2rest --connect=udpout:127.0.",
                },
            ),
        ]),
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
                ],
                ordered_before: &[
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

pub const SERVICE_DEFINITION: ServiceJudgment =
    ServiceJudgment {
        id: ServiceId::Mavlink2rest,
        singleton: Asserted::established(
            true,
            "single Priority-tier tmux instance; one mavlink2rest process bridges the vehicle MAVLink stream to REST/WebSocket",
        ),
        bounded_context: Asserted::established(
            "vehicle-mavlink-access",
            "provisional 2.0 domain: bridge vehicle MAVLink telemetry and commands to HTTP REST and WebSocket consumers",
        ),
        journey_refs: AssertedSet::established(&[Rationaled::new(
            JourneyId::InspectMavlinkMessagesInBrowser,
            "MAVLink Inspector page filters, lists, and expands live MAVLink messages from the vehicle stream",
        )]),
        tier: Asserted::established(
            CriticalityTier::Important,
            "Priority-tier data-plane bridge; frontend vehicle store, nmea_injector, ping, and helper lose MAVLink REST access when down, but GCS and router paths still reach the autopilot",
        ),
        offline_required: Asserted::established(
            true,
            "bridges local MAVLink from udpout:127.0.0.1:14001 to on-host REST/WebSocket; no internet dependency",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root in start-blueos-core Priority-tier launch line",
        ),
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "transparent MAVLink bridge; no autonomous irreversible, untrusted-code, or vehicle-arm operations per rubric — caller-initiated MAVLink send risk captured in blast_radius",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::InspectLiveMavlinkMessages,
                "MAVLink Inspector WebSocket stream and REST message listing for operator inspection",
            ),
            Rationaled::new(
                CapabilityId::AccessMavlinkOverRest,
                "observed Rest /mavlink2rest/ and Mavlink Consumer interfaces; MavlinkMessenger and frontend vehicle store POST/read MAVLink via localhost:6040",
            ),
        ]),
        authorities: AssertedSet::established(&[Rationaled::new(
            Authority::Other("mavlink_rest_bridge"),
            "sole catalog service that exposes the vehicle MAVLink stream and send path over HTTP REST/WebSocket",
        )]),
        states: AssertedSet::unknown(
            "external Rust binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(&[Rationaled::new(
            Connection {
                from: ServiceId::Mavlink2rest,
                to: ServiceId::ArdupilotManager,
                via: Bus::Mavlink,
                sync: SyncMode::Async,
                endpoint: "udpout:127.0.0.1:14001",
                purpose: "consume vehicle MAVLink stream from ardupilot_manager router endpoint",
                required_at_boot: true,
                failure_impact: FailureImpact::ServiceUnavailable,
            },
            "observed Mavlink Consumer connect udpout:127.0.0.1:14001 pairs with ardupilot_manager udpin:127.0.0.1:14001 Endpoint created by the MAVLink router owner",
        )]),
        resources: AssertedSet::unknown(
            "observed artifact has no settings paths or userdata files; external binary with no traced on-disk resources",
        ),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                &["start-blueos-core create_service"],
                "observed lifecycle trigger: tmux creation at boot in Priority tier",
            ),
            ordered_after: Asserted::established(
                &[
                    ServiceId::ArdupilotManager,
                    ServiceId::CableGuy,
                    ServiceId::MavlinkCameraManager,
                ],
                "observed ordered_after in start-blueos-core Priority block",
            ),
            ordered_before: Asserted::established(
                &[
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
                "observed ordered_before lists mavlink2rest before remaining Priority and SERVICES-tier peers",
            ),
            shutdown: Asserted::unknown(
                "external Rust binary; no explicit shutdown handler traced in this repository",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade restart semantics for the external mavlink2rest binary not traced in this repository",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST /mavlink2rest/ and WebSocket stream availability serve as health signals"
                ,
            "no dedicated /health route traced; bridge process continuity and HTTP listener serve as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "core infrastructure MAVLink bridge; does not install or host third-party extensions",
        ),
        api_stable: Asserted::unknown(
            "external binary with empty observed REST versions list; upstream API stability not established from this repository",
        ),
        permissions_model: Asserted::established(
            "no auth middleware traced; REST and WebSocket routes are unauthenticated on the LAN",
            "external binary proxied by nginx without observed permission checks; LAN trust model matches other core REST bridges",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "mavlink_router_endpoint_unreachable",
                "udpout:127.0.0.1:14001 has no router listener when ardupilot_manager is down or endpoint not yet created",
            ),
            Rationaled::new(
                "rest_bridge_down",
                "process exit or port 6040 bind failure blocks all REST/WebSocket MAVLink consumers including the frontend vehicle store",
            ),
            Rationaled::new(
                "websocket_stream_stale",
                "MAVLink Inspector live view stops updating when the WebSocket bridge disconnects while REST may still respond",
            ),
            Rationaled::new(
                "mavlink_send_surface_abuse",
                "unauthenticated REST POST can forward arbitrary MAVLink messages including param writes and mode commands to the vehicle",
            ),
        ]),
        blast_radius: Asserted::established(
            "BlueOS UI and dependent services lose vehicle telemetry, parameter access, and MAVLink send path; unauthenticated REST POST can inject arbitrary MAVLink to the autopilot while direct GCS/router control remains available"
                ,
            "data-plane bridge outage degrades BlueOS vehicle integration but does not remove the ardupilot_manager MAVLink router or GCS link; exposed send surface is a security concern independent of outage",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream mavlink2rest API deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Mavlink2rest,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
