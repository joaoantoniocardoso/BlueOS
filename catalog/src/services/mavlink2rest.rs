use crate::criticality::CriticalityTier;
use crate::edge::{Bus, Edge, FailureImpact, SyncMode};
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{Interface, MavlinkRole};
use crate::journey::{HttpMethod, RouteRef};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline};
use crate::service::{Authority, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

const RUNTIME_CAPTURE: &str = "runtime-captures/mavlink2rest__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId::Mavlink2rest,
        state_contracts: GroundedSet::unknown(
            "mavlink2rest has no service-level state machine (card states Unknown); external Rust binary with no traced lifecycle states",
        ),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/v1/mavlink", 3.9, 6.3, 8.3, 60),
            runtime_slo(HttpMethod::Get, "/", 1.3, 1.7, 3.1, 60),
            runtime_slo(HttpMethod::Get, "/v1/mavlink/HEARTBEAT", 3.4, 4.9, 5.2, 60),
        ]),
        resource_usage: GroundedSet::known(vec![runtime_resource(
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
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "mavlink2rest consumes udpout:127.0.0.1:14001 from ardupilot_manager; platform-independent".into(),
                    "runtime captured on Navigator only; Rust binary RSS ~8.8 MB flat, CPU ~1.33% mean with occasional serialization spikes".into(),
                    "observed REST API path prefix /v1 at runtime (observed_facts versions list empty)".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "mavlink2rest has no traced on-disk settings paths; POST /v1/mavlink injects MAVLink to the vehicle — not exercised",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId::Mavlink2rest,
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
        id: ServiceId::Mavlink2rest,
        aliases: ObservedSet::unknown(
            "external binary; no in-repo alias declarations in this repository",
        ),
        kind: Observed::known(
            ServiceKind::Binary,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 121,
            },
        ),
        entrypoint: Observed::known(
            "mavlink2rest --connect=udpout:127.0.0.1:14001 --server [::]:6040 --system-id $MAV_SYSTEM_ID --component-id $MAV_COMPONENT_ID_ONBOARD_COMPUTER4"
                .to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 121,
            },
        ),
        tmux_name: Observed::known(
            "mavlink2rest".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 121,
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
                line: 121,
            },
        ),
        nice: Observed::unknown("command line has no nice wrapper"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 121,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/mavlink2rest/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 169,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(6040),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 121,
            },
        )]),
        git_path: Observed::unknown(
            "external Rust binary (upstream github.com/bluerobotics/mavlink2rest); no source tree in this repository",
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/mavlink2rest/".to_string()),
                    port: PortRef::Literal(6040),
                    versions: vec![],
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 174,
                },
            ),
            Evidenced::new(
                Interface::Mavlink {
                    role: MavlinkRole::Consumer,
                    connect: "udpout:127.0.0.1:14001".to_string(),
                },
                Evidence {
                    file: "core/start-blueos-core".to_string(),
                    line: 121,
                },
            ),
        ]),
        resources: ObservedSet::unknown(
            "no settings paths or userdata files referenced in start-blueos-core launch args",
        ),
        lifecycle: Observed::known(
            ObservedLifecycle {
                triggers: vec!["start-blueos-core create_service".to_string()],
                ordered_after: vec![
                    ServiceId::ArdupilotManager,
                    ServiceId::CableGuy,
                    ServiceId::MavlinkCameraManager,
                ],
                ordered_before: vec![
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
                file: "core/start-blueos-core".to_string(),
                line: 318,
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
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId::Mavlink2rest,
        singleton: Asserted::established(
            true,
            "single Priority-tier tmux instance; one mavlink2rest process bridges the vehicle MAVLink stream to REST/WebSocket",
        ),
        bounded_context: Asserted::established(
            "vehicle-mavlink-access".to_string(),
            "provisional 2.0 domain: bridge vehicle MAVLink telemetry and commands to HTTP REST and WebSocket consumers",
        ),
        journey_refs: AssertedSet::established(vec![Rationaled::new(
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
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "transparent MAVLink bridge; no autonomous irreversible, untrusted-code, or vehicle-arm operations per rubric — caller-initiated MAVLink send risk captured in blast_radius",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId::InspectLiveMavlinkMessages,
                "MAVLink Inspector WebSocket stream and REST message listing for operator inspection",
            ),
            Rationaled::new(
                CapabilityId::AccessMavlinkOverRest,
                "observed Rest /mavlink2rest/ and Mavlink Consumer interfaces; MavlinkMessenger and frontend vehicle store POST/read MAVLink via localhost:6040",
            ),
        ]),
        authorities: AssertedSet::established(vec![Rationaled::new(
            Authority::Other("mavlink_rest_bridge".to_string()),
            "sole catalog service that exposes the vehicle MAVLink stream and send path over HTTP REST/WebSocket",
        )]),
        states: AssertedSet::unknown(
            "external Rust binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(vec![Rationaled::new(
            Edge {
                from: ServiceId::Mavlink2rest,
                to: ServiceId::ArdupilotManager,
                via: Bus::Mavlink,
                sync: SyncMode::Async,
                endpoint: "udpout:127.0.0.1:14001".to_string(),
                purpose: "consume vehicle MAVLink stream from ardupilot_manager router endpoint".to_string(),
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
                vec!["start-blueos-core create_service".to_string()],
                "observed lifecycle trigger: tmux creation at boot in Priority tier",
            ),
            ordered_after: Asserted::established(
                vec![
                    ServiceId::ArdupilotManager,
                    ServiceId::CableGuy,
                    ServiceId::MavlinkCameraManager,
                ],
                "observed ordered_after in start-blueos-core Priority block",
            ),
            ordered_before: Asserted::established(
                vec![
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
                .to_string(),
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
            "no auth middleware traced; REST and WebSocket routes are unauthenticated on the LAN".to_string(),
            "external binary proxied by nginx without observed permission checks; LAN trust model matches other core REST bridges",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "mavlink_router_endpoint_unreachable".to_string(),
                "udpout:127.0.0.1:14001 has no router listener when ardupilot_manager is down or endpoint not yet created",
            ),
            Rationaled::new(
                "rest_bridge_down".to_string(),
                "process exit or port 6040 bind failure blocks all REST/WebSocket MAVLink consumers including the frontend vehicle store",
            ),
            Rationaled::new(
                "websocket_stream_stale".to_string(),
                "MAVLink Inspector live view stops updating when the WebSocket bridge disconnects while REST may still respond",
            ),
            Rationaled::new(
                "mavlink_send_surface_abuse".to_string(),
                "unauthenticated REST POST can forward arbitrary MAVLink messages including param writes and mode commands to the vehicle",
            ),
        ]),
        blast_radius: Asserted::established(
            "BlueOS UI and dependent services lose vehicle telemetry, parameter access, and MAVLink send path; unauthenticated REST POST can inject arbitrary MAVLink to the autopilot while direct GCS/router control remains available"
                .to_string(),
            "data-plane bridge outage degrades BlueOS vehicle integration but does not remove the ardupilot_manager MAVLink router or GCS link; exposed send surface is a security concern independent of outage",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream mavlink2rest API deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    }
}
