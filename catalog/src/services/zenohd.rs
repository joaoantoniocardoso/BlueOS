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
use crate::resource::{Resource, ResourceOwnership};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline};
use crate::service::{Authority, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

const RUNTIME_CAPTURE: &str = "runtime-captures/zenohd__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e; RepoDigest sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId::Zenohd,
        state_contracts: GroundedSet::unknown(
            "zenohd has no service-level state machine (card states Unknown); external Rust binary with no traced lifecycle states; running_baseline GET contracts captured in artifact only",
        ),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/@/local/router", 2.5, 7.4, 8.1, 60),
            runtime_slo(HttpMethod::Get, "/@/router/local", 1.5, 2.0, 3.0, 60),
            runtime_slo(HttpMethod::Get, "/@/**", 107.5, 143.3, 153.7, 60),
        ]),
        resource_usage: GroundedSet::known(vec![runtime_resource(
            "running_baseline",
            Distribution {
                mean: 9.12,
                median: 4.04,
                p95: 58.59,
                min: 2.92,
                max: 75.59,
                sd: 16.74,
            },
            Distribution {
                mean: 42.1,
                median: 42.0,
                p95: 46.9,
                min: 27.4,
                max: 46.9,
                sd: 4.2,
            },
            90,
        )]),
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "zenohd is the platform-independent on-host Zenoh pub/sub router".into(),
                    "runtime captured on Navigator only; Rust binary RSS ~42 MB, CPU ~9% mean with routing bursts".into(),
                    "adminspace key-expression API under /zenoh/ (not versioned REST)".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "Tier-1 GET-only adminspace capture; no PUT/DELETE config mutations exercised",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId::Zenohd,
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
        id: ServiceId::Zenohd,
        aliases: ObservedSet::unknown(
            "external binary; no in-repo alias declarations in this repository",
        ),
        kind: Observed::known(
            ServiceKind::Binary,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 128,
            },
        ),
        entrypoint: Observed::known(
            "ZENOH_BACKEND_FS_ROOT=$TOOLS_PATH/zenoh zenohd -c $TOOLS_PATH/zenoh/blueos-zenoh.json5"
                .to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 128,
            },
        ),
        tmux_name: Observed::known(
            "zenohd".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 128,
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
                line: 128,
            },
        ),
        nice: Observed::unknown("command line has no nice wrapper"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 128,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![
            Evidenced::new(
                PathRef("/zenoh/".to_string()),
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 238,
                },
            ),
            Evidenced::new(
                PathRef("/zenoh-api/".to_string()),
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 253,
                },
            ),
        ]),
        listen: ObservedSet::known(vec![
            Evidenced::new(
                PortRef::Literal(7117),
                Evidence {
                    file: "core/tools/zenoh/blueos-zenoh.json5".to_string(),
                    line: 9,
                },
            ),
            Evidenced::new(
                PortRef::Literal(7118),
                Evidence {
                    file: "core/tools/zenoh/blueos-zenoh.json5".to_string(),
                    line: 10,
                },
            ),
        ]),
        git_path: Observed::unknown(
            "external Zenoh router binary (zenohd); no source tree in this repository",
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/zenoh/".to_string()),
                    port: PortRef::Literal(7117),
                    versions: vec![],
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 246,
                },
            ),
            Evidenced::new(
                Interface::Websocket {
                    path: PathRef("/zenoh-api/".to_string()),
                    port: PortRef::Literal(7118),
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 261,
                },
            ),
        ]),
        resources: ObservedSet::known(vec![
            Evidenced::new(
                Resource {
                    path: PathRef("$TOOLS_PATH/zenoh".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/start-blueos-core".to_string(),
                    line: 128,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("$TOOLS_PATH/zenoh/blueos-zenoh.json5".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/start-blueos-core".to_string(),
                    line: 128,
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
                ],
                ordered_before: vec![
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
                line: 326,
            },
        ),
        logs_path: Observed::unknown(
            "external binary; no on-disk log path in start-blueos-core launch args",
        ),
        zenoh_log_topic: Observed::unknown(
            "zenohd is the Zenoh pub/sub router; does not publish logs to a zenoh topic via commonwealth init_logger",
        ),
        sentry: Observed::unknown(
            "external binary; no init_sentry or equivalent traced in this repository",
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId::Zenohd,
        singleton: Asserted::established(
            true,
            "single Normal-tier tmux instance; one zenohd process is the on-host Zenoh pub/sub router",
        ),
        bounded_context: Asserted::established(
            "message-bus".to_string(),
            "provisional 2.0 domain: local Zenoh pub/sub router bridging service publishers and subscribers on the vehicle",
        ),
        journey_refs: AssertedSet::established(vec![Rationaled::new(
            JourneyId::InspectZenohNetwork,
            "Zenoh Inspector page connects over WebSocket to browse live network topology and pub/sub topics",
        )]),
        tier: Asserted::established(
            CriticalityTier::Important,
            "Normal-tier shared message bus; many services publish and subscribe through zenohd for logs and kraken handlers, but autopilot MAVLink control does not route through Zenoh and the vehicle remains controllable when down",
        ),
        offline_required: Asserted::established(
            true,
            "local on-host pub/sub router with REST and WebSocket plugins on localhost; no internet or WAN dependency",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root in start-blueos-core Normal-tier launch line; binds router ports and writes filesystem storage backend",
        ),
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "Zenoh Inspector journey is read-only inspection; adminspace config queries are not modeled as irreversible or vehicle-arm operations per rubric",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId::InspectZenohNetwork,
                "Zenoh Inspector WebSocket session to /zenoh-api/ browses live topology and pub/sub topic payloads",
            ),
            Rationaled::new(
                CapabilityId::RoutePubsubMessages,
                "sole on-host Zenoh router; observed Rest /zenoh/ and WebSocket /zenoh-api/ planes carry inter-service pub/sub including commonwealth log publishers",
            ),
        ]),
        authorities: AssertedSet::established(vec![Rationaled::new(
            Authority::ZenohBroker,
            "sole catalog Zenoh router; every service that publishes or subscribes on the BlueOS message bus depends on this process",
        )]),
        states: AssertedSet::unknown(
            "external Zenoh router binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(vec![]),
        resources: AssertedSet::established(vec![
            Rationaled::new(
                Resource {
                    path: PathRef("$TOOLS_PATH/zenoh".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "ZENOH_BACKEND_FS_ROOT filesystem storage backend for Zenoh router state and persisted pub/sub data",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("$TOOLS_PATH/zenoh/blueos-zenoh.json5".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                "router configuration selecting REST admin plugin port 7117 and remote-api WebSocket port 7118",
            ),
        ]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                vec!["start-blueos-core create_service".to_string()],
                "observed lifecycle trigger: tmux creation at boot in Normal tier",
            ),
            ordered_after: Asserted::established(
                vec![
                    ServiceId::ArdupilotManager,
                    ServiceId::CableGuy,
                    ServiceId::MavlinkCameraManager,
                    ServiceId::Mavlink2rest,
                    ServiceId::Kraken,
                    ServiceId::Wifi,
                ],
                "observed ordered_after in start-blueos-core Normal block",
            ),
            ordered_before: Asserted::established(
                vec![
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
                "observed ordered_before lists zenohd before remaining Normal and SERVICES-tier peers",
            ),
            shutdown: Asserted::unknown(
                "external Zenoh router binary; no explicit shutdown handler traced in this repository",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade restart semantics for the external zenohd binary not traced in this repository",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST /zenoh/ and WebSocket /zenoh-api/ listener availability serve as health signals"
                .to_string(),
            "no dedicated /health route traced; router process continuity and plugin listeners serve as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "core infrastructure message bus; does not install or host third-party extensions",
        ),
        api_stable: Asserted::unknown(
            "external binary with empty observed REST versions list; upstream Zenoh plugin API stability not established from this repository",
        ),
        permissions_model: Asserted::established(
            "no auth middleware traced; REST and WebSocket routes are unauthenticated on the LAN".to_string(),
            "external binary proxied by nginx without observed permission checks; LAN trust model matches other core REST bridges",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "router_process_down".to_string(),
                "process exit or port bind failure stops all on-host Zenoh pub/sub routing",
            ),
            Rationaled::new(
                "pubsub_transport_partition".to_string(),
                "publishers and subscribers lose message delivery when the router is unreachable while local processes may still run",
            ),
            Rationaled::new(
                "rest_admin_plugin_down".to_string(),
                "port 7117 REST admin/data plugin unavailable blocks /zenoh/ consumers including config inspection",
            ),
            Rationaled::new(
                "websocket_remote_api_down".to_string(),
                "port 7118 remote-api WebSocket unavailable blocks Zenoh Inspector live topology and topic browsing",
            ),
        ]),
        blast_radius: Asserted::established(
            "inter-service Zenoh pub/sub stops: service log topics, kraken zenoh handlers, and the Zenoh Inspector lose messaging; autopilot MAVLink routing and direct vehicle control remain intact"
                .to_string(),
            "shared message-bus outage degrades observability and extension pub/sub but does not remove ardupilot_manager MAVLink control paths",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream zenohd and plugin API deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    }
}
