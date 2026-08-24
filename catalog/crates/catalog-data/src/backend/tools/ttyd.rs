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
use catalog_model::interface::PortKind;
use catalog_model::journey::{HttpMethod, RouteRef};
use catalog_model::lifecycle::{Lifecycle, ObservedLifecycle};
use catalog_model::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use catalog_model::runtime::{
    Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline,
};
use catalog_model::service::{Authority, Service, ServiceJudgment};
use catalog_model::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

use catalog_kernel::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_REPODIGEST;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_REPODIGEST;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Ttyd,
        state_contracts: GroundedSet::unknown(
            "ttyd has no service-level state machine (card states Unknown); external C binary with no traced lifecycle states; running_baseline GET contracts captured in artifact only",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/terminal/", 10.6, 16.1, 20.1, 60),
            runtime_slo(HttpMethod::Get, "/terminal/token", 1.3, 2.5, 3.4, 60),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
            "running_baseline",
            Distribution {
                mean: 0.13,
                median: 0.0,
                p95: 0.0,
                min: 0.0,
                max: 11.61,
                sd: 1.22,
            },
            Distribution {
                mean: 1.3,
                median: 1.3,
                p95: 1.3,
                min: 1.3,
                max: 1.3,
                sd: 0.0,
            },
            90,
        )]),
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "ttyd is the platform-independent browser web-terminal provider",
                    "runtime captured on Navigator only; idle C binary RSS ~1.3 MB, CPU ~0% median",
                    "WebSocket /terminal/ interactive root shell not probed (Tier-1 GET-only)",
                ],
            },
            runtime_prov("runtime-captures/ttyd__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "Tier-1 GET-only capture; websocket shell and settings mutations not exercised",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::Ttyd,
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
        runtime_prov("runtime-captures/ttyd__pi4_navigator_master.json#slo_running_baseline"),
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
        runtime_prov("runtime-captures/ttyd__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts =
    ObservedFacts {
        id: ServiceId::Ttyd,
        aliases: ObservedSet::unknown(
            "external binary; no in-repo alias declarations in this repository",
        ),
        kind: Observed::known(
            ServiceKind::Binary,
            Evidence {
                file: "core/start-blueos-core",
                line: 142,
                anchor: "'ttyd',250,0,0,0,'nice -19 ttyd -p 8088 sh -c \"/usr/bin/tmux",
            },
        ),
        entrypoint: Observed::known(
            "nice -19 ttyd -p 8088 sh -c \"/usr/bin/tmux attach -t user_terminal || /usr/bin/tmux new -s user_terminal\""
                ,
            Evidence {
                file: "core/start-blueos-core",
                line: 142,
                anchor: "'ttyd',250,0,0,0,'nice -19 ttyd -p 8088 sh -c \"/usr/bin/tmux",
            },
        ),
        tmux_name: Observed::known(
            "ttyd",
            Evidence {
                file: "core/start-blueos-core",
                line: 142,
                anchor: "'ttyd',250,0,0,0,'nice -19 ttyd -p 8088 sh -c \"/usr/bin/tmux",
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
                memory_mb: Some(250),
                cpu_percent: Some(0),
                io_weight: None,
            },
            Evidence {
                file: "core/start-blueos-core",
                line: 142,
                anchor: "'ttyd',250,0,0,0,'nice -19 ttyd -p 8088 sh -c \"/usr/bin/tmux",
            },
        ),
        nice: Observed::known(
            -19,
            Evidence {
                file: "core/start-blueos-core",
                line: 142,
                anchor: "'ttyd',250,0,0,0,'nice -19 ttyd -p 8088 sh -c \"/usr/bin/tmux",
            },
        ),
        run_as: Observed::known(
            "root",
            Evidence {
                file: "core/start-blueos-core",
                line: 142,
                anchor: "'ttyd',250,0,0,0,'nice -19 ttyd -p 8088 sh -c \"/usr/bin/tmux",
            },
        ),
        nginx_prefixes: ObservedSet::known(&[Evidenced::new(
            PathRef("/terminal/"),
            Evidence {
                file: "core/tools/nginx/nginx.conf",
                line: 213,
                anchor: "location /terminal/ {",
            },
        )]),
        listen: ObservedSet::known(&[Evidenced::new(
            PortRef::Literal(8088),
            Evidence {
                file: "core/start-blueos-core",
                line: 142,
                anchor: "'ttyd',250,0,0,0,'nice -19 ttyd -p 8088 sh -c \"/usr/bin/tmux",
            },
        )]),
        git_path: Observed::unknown("external ttyd binary; no source tree in this repository"),
        interfaces: ObservedSet::known(&[
            Evidenced::new(
                PortKind::Websocket {
                    path: PathRef("/terminal/"),
                    port: PortRef::Literal(8088),
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf",
                    line: 216,
                    anchor: "proxy_set_header Upgrade $http_upgrade;",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "sh -c \"/usr/bin/tmux attach -t user_terminal || /usr/bin/tmux new -s user_terminal\""
                        ,
                },
                Evidence {
                    file: "core/start-blueos-core",
                    line: 142,
                    anchor: "'ttyd',250,0,0,0,'nice -19 ttyd -p 8088 sh -c \"/usr/bin/tmux",
                },
            ),
        ]),
        resources: ObservedSet::unknown(
            "no config or data paths in start-blueos-core launch; ttyd attaches an existing tmux session only",
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
                    ServiceId::Linux2rest,
                    ServiceId::Filebrowser,
                    ServiceId::Versionchooser,
                    ServiceId::Pardal,
                    ServiceId::Ping,
                    ServiceId::UserTerminal,
                ],
                ordered_before: &[
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
            "external ttyd binary; no --log-path or log file in start-blueos-core launch args",
        ),
        zenoh_log_topic: Observed::unknown(
            "external ttyd binary; does not use commonwealth init_logger zenoh publisher",
        ),
        sentry: Observed::unknown(
            "external ttyd binary; no init_sentry or equivalent traced in this repository",
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    };

pub const SERVICE_DEFINITION: ServiceJudgment =
    ServiceJudgment {
        id: ServiceId::Ttyd,
        singleton: Asserted::established(
            true,
            "single Normal-tier tmux instance; one ttyd process is the sole browser web-terminal provider",
        ),
        bounded_context: Asserted::established(
            "web-terminal",
            "provisional 2.0 domain: browser-based interactive shell access into the blueos-core container",
        ),
        journey_refs: AssertedSet::established(&[Rationaled::new(
            JourneyId::AccessWebTerminal,
            "Terminal page embeds ttyd web terminal over WebSocket at /terminal/ attached to user_terminal tmux",
        )]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "advanced/pirate developer convenience; vehicle flight and MAVLink control operate fully without the browser shell — operators can SSH instead",
        ),
        offline_required: Asserted::established(
            true,
            "local tmux attach and WebSocket shell on localhost; no internet or WAN dependency",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root in start-blueos-core Normal-tier launch line; exposes an interactive root shell inside the core container",
        ),
        dangerous_operations: AssertedSet::established(&[Rationaled::new(
            DangerousOperation::Other("arbitrary_root_shell_execution"),
            "WebSocket /terminal/ grants a full interactive root shell in the core container — arbitrary irreversible host operations beyond any single REST delete endpoint",
        )]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "rubric requires Required when dangerous_operations is non-empty; Terminal page is gated behind Advanced/pirate visibility per journey",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::AccessWebTerminal,
                "journey capability: operator opens Terminal page and uses the embedded web terminal attached to user_terminal tmux",
            ),
            Rationaled::new(
                CapabilityId::ProvideShellOverWebsocket,
                "observed Websocket interface at /terminal/ on port 8088 carries an interactive Linux shell session",
            ),
        ]),
        authorities: AssertedSet::established(&[Rationaled::new(
            Authority::Other("web_terminal"),
            "sole catalog service that exposes a browser-based interactive shell; no other service provides WebSocket terminal access",
        )]),
        states: AssertedSet::unknown(
            "external ttyd binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(&[Rationaled::new(
            Connection {
                from: ServiceId::Ttyd,
                to: ServiceId::UserTerminal,
                via: Bus::Subprocess,
                sync: SyncMode::Sync,
                endpoint: "user_terminal",
                purpose: "attach the web terminal to the interactive root-shell tmux session"
                    ,
                required_at_boot: false,
                failure_impact: FailureImpact::Degraded,
            },
            "observed Subprocess sh -c tmux attach -t user_terminal || tmux new -s user_terminal (start-blueos-core:142); ttyd self-heals when the boot session is absent so the edge is not boot-hard",
        )]),
        resources: AssertedSet::unknown(
            "no config or data paths in start-blueos-core launch; ttyd attaches an existing tmux session only",
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
                    ServiceId::Linux2rest,
                    ServiceId::Filebrowser,
                    ServiceId::Versionchooser,
                    ServiceId::Pardal,
                    ServiceId::Ping,
                    ServiceId::UserTerminal,
                ],
                "observed ordered_after in start-blueos-core Normal block; ttyd starts after user_terminal creates the tmux session",
            ),
            ordered_before: Asserted::established(
                &[
                    ServiceId::Nginx,
                    ServiceId::BagOfHolding,
                    ServiceId::Recorder,
                    ServiceId::RecorderExtractor,
                    ServiceId::DiskUsage,
                    ServiceId::Customization,
                ],
                "observed ordered_before lists ttyd before nginx and remaining SERVICES-tier peers",
            ),
            shutdown: Asserted::unknown(
                "external ttyd binary; no explicit shutdown handler traced in this repository",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade restart semantics for the external ttyd binary not traced in this repository",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; WebSocket /terminal/ listener availability (proxied to ttyd on port 8088) serves as health signal"
                ,
            "no dedicated /health route traced; ttyd process continuity and WebSocket listener serve as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "core infrastructure shell access; does not install or host third-party extensions",
        ),
        api_stable: Asserted::unknown(
            "external binary; upstream ttyd WebSocket protocol stability not established from this repository",
        ),
        permissions_model: Asserted::established(
            "no auth middleware traced; WebSocket /terminal/ is unauthenticated on the LAN",
            "external binary proxied by nginx without observed permission checks; LAN trust model — full root shell exposed without ttyd-layer confirmation",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "ttyd_process_down",
                "process exit or tmux session loss removes browser web-terminal access",
            ),
            Rationaled::new(
                "websocket_listener_unavailable",
                "port 8088 WebSocket listener down blocks /terminal/ connections from the Terminal page",
            ),
            Rationaled::new(
                "user_terminal_tmux_unavailable",
                "missing or unreachable user_terminal tmux session prevents attaching an interactive shell",
            ),
        ]),
        blast_radius: Asserted::established(
            "operators lose browser-based root shell access and must use SSH instead; vehicle flight and MAVLink control remain intact"
                ,
            "web-terminal outage is a convenience loss only; autopilot MAVLink routing and direct vehicle control are unaffected",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream ttyd deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Ttyd,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
