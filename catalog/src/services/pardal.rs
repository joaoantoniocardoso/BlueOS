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
use crate::service::{Service, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Pardal,
        state_contracts: GroundedSet::unknown(
            "pardal has no service-level state machine (service_definition states: Unknown); \
             SPEED_TEST global and request handlers are ephemeral test state",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/", 2.6, 5.6, 7.8, 40),
            runtime_slo(
                HttpMethod::Get,
                "/internet_test_previous_result",
                2.7,
                5.1,
                5.2,
                40,
            ),
            runtime_slo(
                HttpMethod::Get,
                "/get_file?size=1048576",
                81.9,
                89.6,
                89.6,
                10,
            ),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
            "running_baseline",
            Distribution {
                mean: 0.02,
                median: 0.00,
                p95: 0.00,
                min: 0.00,
                max: 0.97,
                sd: 0.12,
            },
            flat_rss(38.8),
            60,
        )]),
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "pardal behavior is platform-independent (local random-byte streaming and optional speedtest-cli WAN probes)",
                    "runtime captured on Navigator only; RSS ~38.8 MB flat, CPU ~0.02% mean idle",
                    "active LAN/WAN speed tests (not fully captured) add link saturation and speedtest-cli load",
                ],
            },
            runtime_prov("runtime-captures/pardal__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "pardal has no settings manager or settings.json persistence",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::Pardal,
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
        runtime_prov("runtime-captures/pardal__pi4_navigator_master.json#slo_running_baseline"),
    )
}

const fn flat_rss(mb: f64) -> Distribution {
    Distribution {
        mean: mb,
        median: mb,
        p95: mb,
        min: mb,
        max: mb,
        sd: 0.0,
    }
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
        runtime_prov("runtime-captures/pardal__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts = ObservedFacts {
    id: ServiceId::Pardal,
    aliases: ObservedSet::known(&[Evidenced::new(
        "pardal",
        Evidence {
            file: "core/services/pardal/main.py",
            line: 16,
        },
    )]),
    kind: Observed::known(
        ServiceKind::PythonService,
        Evidence {
            file: "core/start-blueos-core",
            line: 139,
        },
    ),
    entrypoint: Observed::known(
        "nice -19 $SERVICES_PATH/pardal/main.py",
        Evidence {
            file: "core/start-blueos-core",
            line: 139,
        },
    ),
    tmux_name: Observed::known(
        "pardal",
        Evidence {
            file: "core/start-blueos-core",
            line: 139,
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
            line: 139,
        },
    ),
    nice: Observed::known(
        -19,
        Evidence {
            file: "core/start-blueos-core",
            line: 139,
        },
    ),
    run_as: Observed::known(
        "root",
        Evidence {
            file: "core/start-blueos-core",
            line: 139,
        },
    ),
    nginx_prefixes: ObservedSet::known(&[Evidenced::new(
        PathRef("/network-test/"),
        Evidence {
            file: "core/tools/nginx/nginx.conf",
            line: 197,
        },
    )]),
    listen: ObservedSet::known(&[Evidenced::new(
        PortRef::Literal(9120),
        Evidence {
            file: "core/services/pardal/main.py",
            line: 20,
        },
    )]),
    git_path: Observed::known(
        PathRef("core/services/pardal"),
        Evidence {
            file: "core/services/pardal/main.py",
            line: 1,
        },
    ),
    interfaces: ObservedSet::known(&[
        Evidenced::new(
            Interface::Rest {
                path_prefix: PathRef("/network-test/"),
                port: PortRef::Literal(9120),
                versions: &[],
            },
            Evidence {
                file: "core/services/pardal/main.py",
                line: 155,
            },
        ),
        Evidenced::new(
            Interface::Websocket {
                path: PathRef("/ws"),
                port: PortRef::Literal(9120),
            },
            Evidence {
                file: "core/services/pardal/main.py",
                line: 147,
            },
        ),
        Evidenced::new(
            Interface::Zenoh {
                topics_produced: &["services/pardal/log"],
                topics_consumed: &[],
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                line: 78,
            },
        ),
    ]),
    resources: ObservedSet::unknown("no settings paths or persistent file writes in pardal source"),
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
            ],
            ordered_before: &[
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
        "init_logger publishes to zenoh only; no on-disk log path set in pardal source",
    ),
    zenoh_log_topic: Observed::known(
        "services/pardal/log",
        Evidence {
            file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
            line: 78,
        },
    ),
    sentry: Observed::known(
        true,
        Evidence {
            file: "core/services/pardal/main.py",
            line: 142,
        },
    ),
    openapi_refs: ObservedSet::unknown("aiohttp service; no OpenAPI or VersionedFastAPI in source"),
};

pub const SERVICE_DEFINITION: ServiceDefinition =
    ServiceDefinition {
        id: ServiceId::Pardal,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one pardal aiohttp process",
        ),
        bounded_context: Asserted::established(
            "network-performance-diagnostics",
            "provisional 2.0 domain: LAN throughput/latency probes and WAN speedtest-cli benchmarks",
        ),
        journey_refs: AssertedSet::established(&[
            Rationaled::new(
                JourneyId::RunLanSpeedTest,
                "Network Test Local tab streams /get_file and /post_file while /ws echoes latency",
            ),
            Rationaled::new(
                JourneyId::RunInternetSpeedTest,
                "Network Test Internet tab runs speedtest-cli via /internet_* routes",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "SERVICES startup tier; vehicle flight and MAVLink control do not depend on bandwidth diagnostics",
        ),
        offline_required: Asserted::established(
            true,
            "LAN /get_file, /post_file, and /ws echo work without internet; service must stay available offline even though WAN test needs connectivity",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; measurement handlers only stream random bytes and discard uploads without host reconfiguration",
        ),
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "no irreversible, untrusted-code, or vehicle-arm operations; transient test traffic is reversible side effect only",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::RunLanSpeedTest,
                "GET /get_file and POST /post_file transfer test payloads while /ws echoes latency samples",
            ),
            Rationaled::new(
                CapabilityId::RunInternetSpeedTest,
                "GET /internet_best_server, /internet_download_speed, and /internet_upload_speed run speedtest-cli WAN benchmarks",
            ),
        ]),
        authorities: AssertedSet::established(&[]),
        states: AssertedSet::unknown(
            "no cataloged state machine; SPEED_TEST global and request handlers are ephemeral test state",
        ),
        edges: AssertedSet::established(&[]),
        resources: AssertedSet::unknown(
            "no settings paths or persistent file writes in pardal source",
        ),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                &["start-blueos-core create_service"],
                "observed lifecycle trigger: tmux creation at boot in SERVICES tier",
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
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                &[
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
                "observed ordered_before lists pardal before ping and remaining SERVICES-tier peers",
            ),
            shutdown: Asserted::unknown(
                "main.py awaits asyncio.Event forever; no graceful shutdown handler in source",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight speed tests and SPEED_TEST global state not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns minimal HTML page",
            "no dedicated /health route; aiohttp server availability serves as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "network diagnostics utility; does not install or host third-party extensions",
        ),
        api_stable: Asserted::established(
            false,
            "unversioned aiohttp routes under /network-test/; no VersionedFastAPI or OpenAPI in source",
        ),
        permissions_model: Asserted::established(
            "no separate permissions manifest; REST and WebSocket routes are unauthenticated",
            "pardal routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "speedtest_not_initialized",
                "/internet_download_speed and /internet_upload_speed raise when SPEED_TEST global was never set by /internet_best_server",
            ),
            Rationaled::new(
                "speedtest_cli_unavailable_offline",
                "internet_best_server fails when vehicle has no WAN; startup may leave SPEED_TEST None until first successful server search",
            ),
            Rationaled::new(
                "websocket_echo_disconnect",
                "LAN latency measurement stops when the /ws echo session closes mid-test",
            ),
            Rationaled::new(
                "large_transfer_resource_pressure",
                "client_max_size 2 GiB and default 100 MiB /get_file streams can spike CPU and link utilization during active tests",
            ),
        ]),
        blast_radius: Asserted::established(
            "network test UI unavailable; active tests can transiently saturate LAN bandwidth and degrade live video or telemetry until the run finishes"
                ,
            "pardal outage blocks diagnostics only; concurrent large transfers may momentarily contend with operator links but do not mutate vehicle configuration",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and speedtest-cli server coupling not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Pardal,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
