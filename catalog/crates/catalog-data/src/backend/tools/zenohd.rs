use catalog_kernel::criticality::CriticalityTier;
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::refs::{PathRef, PortRef};
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use catalog_model::interface::PortKind;
use catalog_model::journey::{HttpMethod, RouteRef};
use catalog_model::lifecycle::{Lifecycle, ObservedLifecycle};
use catalog_model::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use catalog_model::resource::{Resource, ResourceOwnership};
use catalog_model::runtime::{
    Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline,
};
use catalog_model::service::{Authority, Service, ServiceJudgment};
use catalog_model::trust::{PrivilegeLevel, UserConfirmation};

use catalog_kernel::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_REPODIGEST;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_REPODIGEST;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Zenohd,
        state_contracts: GroundedSet::unknown(
            "zenohd has no service-level state machine (card states Unknown); external Rust binary with no traced lifecycle states; running_baseline GET contracts captured in artifact only",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/@/local/router", 2.5, 7.4, 8.1, 60),
            runtime_slo(HttpMethod::Get, "/@/router/local", 1.5, 2.0, 3.0, 60),
            runtime_slo(HttpMethod::Get, "/@/**", 107.5, 143.3, 153.7, 60),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
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
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "zenohd is the platform-independent on-host Zenoh pub/sub router",
                    "runtime captured on Navigator only; Rust binary RSS ~42 MB, CPU ~9% mean with routing bursts",
                    "adminspace key-expression API under /zenoh/ (not versioned REST)",
                ],
            },
            runtime_prov("runtime-captures/zenohd__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "Tier-1 GET-only adminspace capture; no PUT/DELETE config mutations exercised",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::Zenohd,
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
        runtime_prov("runtime-captures/zenohd__pi4_navigator_master.json#slo_running_baseline"),
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
        runtime_prov("runtime-captures/zenohd__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts =
    ObservedFacts {
        id: ServiceId::Zenohd,
        aliases: ObservedSet::unknown(
            "external binary; no in-repo alias declarations in this repository",
        ),
        kind: Observed::known(
            ServiceKind::Binary,
            Evidence {
                file: "core/start-blueos-core",
                line: 128,
                anchor: "'zenohd',0,0,0,0,\"ZENOH_BACKEND_FS_ROOT=$TOOLS_PATH/zenoh ze",
            },
        ),
        entrypoint: Observed::known(
            "ZENOH_BACKEND_FS_ROOT=$TOOLS_PATH/zenoh zenohd -c $TOOLS_PATH/zenoh/blueos-zenoh.json5"
                ,
            Evidence {
                file: "core/start-blueos-core",
                line: 128,
                anchor: "'zenohd',0,0,0,0,\"ZENOH_BACKEND_FS_ROOT=$TOOLS_PATH/zenoh ze",
            },
        ),
        tmux_name: Observed::known(
            "zenohd",
            Evidence {
                file: "core/start-blueos-core",
                line: 128,
                anchor: "'zenohd',0,0,0,0,\"ZENOH_BACKEND_FS_ROOT=$TOOLS_PATH/zenoh ze",
            },
        ),
        startup_tier: Observed::known(
            StartupTier::Normal,
            Evidence {
                file: "core/start-blueos-core",
                line: 124,
                anchor: "SERVICES=(",
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
                line: 128,
                anchor: "'zenohd',0,0,0,0,\"ZENOH_BACKEND_FS_ROOT=$TOOLS_PATH/zenoh ze",
            },
        ),
        nice: Observed::unknown("command line has no nice wrapper"),
        run_as: Observed::known(
            "root",
            Evidence {
                file: "core/start-blueos-core",
                line: 128,
                anchor: "'zenohd',0,0,0,0,\"ZENOH_BACKEND_FS_ROOT=$TOOLS_PATH/zenoh ze",
            },
        ),
        nginx_prefixes: ObservedSet::known(&[
            Evidenced::new(
                PathRef("/zenoh/"),
                Evidence {
                    file: "core/tools/nginx/nginx.conf",
                    line: 238,
                    anchor: "location /zenoh/ {",
                },
            ),
            Evidenced::new(
                PathRef("/zenoh-api/"),
                Evidence {
                    file: "core/tools/nginx/nginx.conf",
                    line: 253,
                    anchor: "location /zenoh-api/ {",
                },
            ),
        ]),
        listen: ObservedSet::known(&[
            Evidenced::new(
                PortRef::Literal(7117),
                Evidence {
                    file: "core/tools/zenoh/blueos-zenoh.json5",
                    line: 9,
                    anchor: "rest: { http_port: 7117 },",
                },
            ),
            Evidenced::new(
                PortRef::Literal(7118),
                Evidence {
                    file: "core/tools/zenoh/blueos-zenoh.json5",
                    line: 10,
                    anchor: "remote_api: { websocket_port: 7118 }",
                },
            ),
        ]),
        git_path: Observed::unknown(
            "external Zenoh router binary (zenohd); no source tree in this repository",
        ),
        interfaces: ObservedSet::known(&[
            Evidenced::new(
                PortKind::Rest {
                    path_prefix: PathRef("/zenoh/"),
                    port: PortRef::Literal(7117),
                    versions: &[],
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf",
                    line: 246,
                    anchor: "proxy_pass http://127.0.0.1:7117/;",
                },
            ),
            Evidenced::new(
                PortKind::Websocket {
                    path: PathRef("/zenoh-api/"),
                    port: PortRef::Literal(7118),
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf",
                    line: 261,
                    anchor: "proxy_pass http://127.0.0.1:7118/;",
                },
            ),
        ]),
        resources: ObservedSet::known(&[
            Evidenced::new(
                Resource {
                    path: PathRef("$TOOLS_PATH/zenoh"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/start-blueos-core",
                    line: 128,
                    anchor: "'zenohd',0,0,0,0,\"ZENOH_BACKEND_FS_ROOT=$TOOLS_PATH/zenoh ze",
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("$TOOLS_PATH/zenoh/blueos-zenoh.json5"),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/start-blueos-core",
                    line: 128,
                    anchor: "'zenohd',0,0,0,0,\"ZENOH_BACKEND_FS_ROOT=$TOOLS_PATH/zenoh ze",
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
                    ServiceId::Wifi,
                ],
                ordered_before: &[
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
                anchor: "for TUPLE in \"${SERVICES[@]}\"; do",
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
    };

pub const SERVICE_DEFINITION: ServiceJudgment =
    ServiceJudgment {
        id: ServiceId::Zenohd,
        singleton: Asserted::established(
            true,
            "single Normal-tier tmux instance; one zenohd process is the on-host Zenoh pub/sub router",
        ),
        bounded_context: Asserted::established(
            "message-bus",
            "provisional 2.0 domain: local Zenoh pub/sub router bridging service publishers and subscribers on the vehicle",
        ),
        journey_refs: AssertedSet::established(&[Rationaled::new(
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
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "Zenoh Inspector journey is read-only inspection; adminspace config queries are not modeled as irreversible or vehicle-arm operations per rubric",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::InspectZenohNetwork,
                "Zenoh Inspector WebSocket session to /zenoh-api/ browses live topology and pub/sub topic payloads",
            ),
            Rationaled::new(
                CapabilityId::RoutePubsubMessages,
                "sole on-host Zenoh router; observed Rest /zenoh/ and WebSocket /zenoh-api/ planes carry inter-service pub/sub including commonwealth log publishers",
            ),
        ]),
        authorities: AssertedSet::established(&[Rationaled::new(
            Authority::ZenohBroker,
            "sole catalog Zenoh router; every service that publishes or subscribes on the BlueOS message bus depends on this process",
        )]),
        states: AssertedSet::unknown(
            "external Zenoh router binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(&[]),
        resources: AssertedSet::established(&[
            Rationaled::new(
                Resource {
                    path: PathRef("$TOOLS_PATH/zenoh"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "ZENOH_BACKEND_FS_ROOT filesystem storage backend for Zenoh router state and persisted pub/sub data",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("$TOOLS_PATH/zenoh/blueos-zenoh.json5"),
                    ownership: ResourceOwnership::SharedRead,
                },
                "router configuration selecting REST admin plugin port 7117 and remote-api WebSocket port 7118",
            ),
        ]),
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
                ],
                "observed ordered_after in start-blueos-core Normal block",
            ),
            ordered_before: Asserted::established(
                &[
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
                ,
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
            "no auth middleware traced; REST and WebSocket routes are unauthenticated on the LAN",
            "external binary proxied by nginx without observed permission checks; LAN trust model matches other core REST bridges",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "router_process_down",
                "process exit or port bind failure stops all on-host Zenoh pub/sub routing",
            ),
            Rationaled::new(
                "pubsub_transport_partition",
                "publishers and subscribers lose message delivery when the router is unreachable while local processes may still run",
            ),
            Rationaled::new(
                "rest_admin_plugin_down",
                "port 7117 REST admin/data plugin unavailable blocks /zenoh/ consumers including config inspection",
            ),
            Rationaled::new(
                "websocket_remote_api_down",
                "port 7118 remote-api WebSocket unavailable blocks Zenoh Inspector live topology and topic browsing",
            ),
        ]),
        blast_radius: Asserted::established(
            "inter-service Zenoh pub/sub stops: service log topics, kraken zenoh handlers, and the Zenoh Inspector lose messaging; autopilot MAVLink routing and direct vehicle control remain intact"
                ,
            "shared message-bus outage degrades observability and extension pub/sub but does not remove ardupilot_manager MAVLink control paths",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream zenohd and plugin API deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Zenohd,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
