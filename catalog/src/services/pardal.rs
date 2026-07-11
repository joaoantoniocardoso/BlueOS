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
use crate::service::ServiceDefinition;
use crate::trust::{PrivilegeLevel, UserConfirmation};

const RUNTIME_CAPTURE: &str = "runtime-captures/pardal__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId("pardal".into()),
        state_contracts: GroundedSet::unknown(
            "pardal has no service-level state machine (service_definition states: Unknown); \
             SPEED_TEST global and request handlers are ephemeral test state",
        ),
        slo_baselines: GroundedSet::known(vec![
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
        resource_usage: GroundedSet::known(vec![runtime_resource(
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
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "pardal behavior is platform-independent (local random-byte streaming and optional speedtest-cli WAN probes)".into(),
                    "runtime captured on Navigator only; RSS ~38.8 MB flat, CPU ~0.02% mean idle".into(),
                    "active LAN/WAN speed tests (not fully captured) add link saturation and speedtest-cli load".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "pardal has no settings manager or settings.json persistence",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId("pardal".into()),
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

fn flat_rss(mb: f64) -> Distribution {
    Distribution {
        mean: mb,
        median: mb,
        p95: mb,
        min: mb,
        max: mb,
        sd: 0.0,
    }
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
        id: ServiceId("pardal".to_string()),
        aliases: ObservedSet::known(vec![Evidenced::new(
            "pardal".to_string(),
            Evidence {
                file: "core/services/pardal/main.py".to_string(),
                line: 16,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 139,
            },
        ),
        entrypoint: Observed::known(
            "nice -19 $SERVICES_PATH/pardal/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 139,
            },
        ),
        tmux_name: Observed::known(
            "pardal".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 139,
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
                line: 139,
            },
        ),
        nice: Observed::known(
            -19,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 139,
            },
        ),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 139,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/network-test/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 197,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(9120),
            Evidence {
                file: "core/services/pardal/main.py".to_string(),
                line: 20,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/pardal".to_string()),
            Evidence {
                file: "core/services/pardal/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/network-test/".to_string()),
                    port: PortRef::Literal(9120),
                    versions: vec![],
                },
                Evidence {
                    file: "core/services/pardal/main.py".to_string(),
                    line: 155,
                },
            ),
            Evidenced::new(
                Interface::Websocket {
                    path: PathRef("/ws".to_string()),
                    port: PortRef::Literal(9120),
                },
                Evidence {
                    file: "core/services/pardal/main.py".to_string(),
                    line: 147,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/pardal/log".to_string()],
                    topics_consumed: vec![],
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                    line: 78,
                },
            ),
        ]),
        resources: ObservedSet::unknown(
            "no settings paths or persistent file writes in pardal source",
        ),
        lifecycle: Observed::known(
            ObservedLifecycle {
                triggers: vec!["start-blueos-core create_service".to_string()],
                ordered_after: vec![
                    ServiceId("autopilot".to_string()),
                    ServiceId("cable_guy".to_string()),
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
                ],
                ordered_before: vec![
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
                line: 326,
            },
        ),
        logs_path: Observed::unknown(
            "init_logger publishes to zenoh only; no on-disk log path set in pardal source",
        ),
        zenoh_log_topic: Observed::known(
            "services/pardal/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/pardal/main.py".to_string(),
                line: 142,
            },
        ),
        openapi_refs: ObservedSet::unknown(
            "aiohttp service; no OpenAPI or VersionedFastAPI in source",
        ),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("pardal".to_string()),
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one pardal aiohttp process",
        ),
        bounded_context: Asserted::established(
            "network-performance-diagnostics".to_string(),
            "provisional 2.0 domain: LAN throughput/latency probes and WAN speedtest-cli benchmarks",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId("run_lan_speed_test".into()),
                "Network Test Local tab streams /get_file and /post_file while /ws echoes latency",
            ),
            Rationaled::new(
                JourneyId("run_internet_speed_test".into()),
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
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "no irreversible, untrusted-code, or vehicle-arm operations; transient test traffic is reversible side effect only",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("run_lan_speed_test".to_string()),
                "GET /get_file and POST /post_file transfer test payloads while /ws echoes latency samples",
            ),
            Rationaled::new(
                CapabilityId("run_internet_speed_test".to_string()),
                "GET /internet_best_server, /internet_download_speed, and /internet_upload_speed run speedtest-cli WAN benchmarks",
            ),
        ]),
        authorities: AssertedSet::established(vec![]),
        states: AssertedSet::unknown(
            "no cataloged state machine; SPEED_TEST global and request handlers are ephemeral test state",
        ),
        edges: AssertedSet::unknown(
            "no outbound coupling to other catalog services; topside browser and speedtest-cli WAN endpoints are external",
        ),
        resources: AssertedSet::unknown(
            "no settings paths or persistent file writes in pardal source",
        ),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                vec!["start-blueos-core create_service".to_string()],
                "observed lifecycle trigger: tmux creation at boot in SERVICES tier",
            ),
            ordered_after: Asserted::established(
                vec![
                    ServiceId("autopilot".to_string()),
                    ServiceId("cable_guy".to_string()),
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
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                vec![
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
            "implicit: process liveness via tmux; REST GET / returns minimal HTML page".to_string(),
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
            "no separate permissions manifest; REST and WebSocket routes are unauthenticated".to_string(),
            "pardal routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "speedtest_not_initialized".to_string(),
                "/internet_download_speed and /internet_upload_speed raise when SPEED_TEST global was never set by /internet_best_server",
            ),
            Rationaled::new(
                "speedtest_cli_unavailable_offline".to_string(),
                "internet_best_server fails when vehicle has no WAN; startup may leave SPEED_TEST None until first successful server search",
            ),
            Rationaled::new(
                "websocket_echo_disconnect".to_string(),
                "LAN latency measurement stops when the /ws echo session closes mid-test",
            ),
            Rationaled::new(
                "large_transfer_resource_pressure".to_string(),
                "client_max_size 2 GiB and default 100 MiB /get_file streams can spike CPU and link utilization during active tests",
            ),
        ]),
        blast_radius: Asserted::established(
            "network test UI unavailable; active tests can transiently saturate LAN bandwidth and degrade live video or telemetry until the run finishes"
                .to_string(),
            "pardal outage blocks diagnostics only; concurrent large transfers may momentarily contend with operator links but do not mutate vehicle configuration",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and speedtest-cli server coupling not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
