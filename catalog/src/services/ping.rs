use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
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

const RUNTIME_CAPTURE: &str = "runtime-captures/ping__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId("ping".into()),
        state_contracts: GroundedSet::unknown(
            "ping has no service-level state machine (card states Unknown); PingManager probe loop and per-device bridge subprocesses are runtime-managed",
        ),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/sensors", 10.9, 35.2, 39.7, 60),
            runtime_slo(HttpMethod::Get, "/", 5.7, 10.1, 14.8, 60),
        ]),
        resource_usage: GroundedSet::known(vec![runtime_resource(
            "running_baseline",
            Distribution {
                mean: 0.96,
                median: 0.94,
                p95: 2.00,
                min: 0.00,
                max: 26.25,
                sd: 2.78,
            },
            Distribution {
                mean: 40.2,
                median: 40.2,
                p95: 40.2,
                min: 40.2,
                max: 40.2,
                sd: 0.0,
            },
            90,
        )]),
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "ping auto-detects Ping-family sonar on serial/USB and local ethernet; platform-independent".into(),
                    "runtime captured on Navigator only with NO sonar hardware; RSS ~40.2 MB flat, CPU ~0.96% mean with occasional discovery spikes".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "POST /sensors persists Ping1D settings to /usr/blueos/userdata/settings/ping; not exercised (mutating; no sonar hardware)",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId("ping".into()),
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
        id: ServiceId("ping".to_string()),
        aliases: ObservedSet::known(vec![Evidenced::new(
            "ping".to_string(),
            Evidence {
                file: "core/services/ping/main.py".to_string(),
                line: 20,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 140,
            },
        ),
        entrypoint: Observed::known(
            "nice -19 $RUN_AS_REGULAR_USER_BEGIN $SERVICES_PATH/ping/main.py $RUN_AS_REGULAR_USER_END"
                .to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 140,
            },
        ),
        tmux_name: Observed::known(
            "ping".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 140,
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
                line: 140,
            },
        ),
        nice: Observed::known(
            -19,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 140,
            },
        ),
        run_as: Observed::known(
            "blueos".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 22,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/ping/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 233,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(9110),
            Evidence {
                file: "core/services/ping/main.py".to_string(),
                line: 86,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/ping".to_string()),
            Evidence {
                file: "core/services/ping/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/ping/".to_string()),
                    port: PortRef::Literal(9110),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/ping/main.py".to_string(),
                    line: 54,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "{bridges} -u {ip}:{port} -p {serial_port.device}:{baud} {automatic_disconnect_clients}"
                        .to_string(),
                },
                Evidence {
                    file: "core/libs/bridges/src/bridges/bridges.py".to_string(),
                    line: 29,
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
                Interface::Settings {
                    path: PathRef("/usr/blueos/userdata/settings/ping".to_string()),
                },
                Evidence {
                    file: "core/services/ping/ping1d_driver.py".to_string(),
                    line: 22,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/usr/blueos/userdata/settings/ping".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/ping/ping1d_driver.py".to_string(),
                    line: 22,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/ping/log".to_string()],
                    topics_consumed: vec![],
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                    line: 78,
                },
            ),
        ]),
        resources: ObservedSet::known(vec![Evidenced::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/settings/ping".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/ping/ping1d_driver.py".to_string(),
                line: 22,
            },
        )]),
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
                    ServiceId("pardal".to_string()),
                ],
                ordered_before: vec![
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
            "init_logger publishes to zenoh only; no on-disk log path set in ping source",
        ),
        zenoh_log_topic: Observed::known(
            "services/ping/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/ping/main.py".to_string(),
                line: 80,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("ping".to_string()),
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one ping process owns all sonar probes and bridges subprocesses",
        ),
        bounded_context: Asserted::established(
            "ping-sonar-integration".to_string(),
            "provisional 2.0 domain: auto-detect Ping-family sonar devices, spawn UDP bridges, and optional MAVLink rangefinder forwarding",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId("view_detected_sonar_devices".into()),
                "Ping Sonar Devices page lists auto-detected Ping1D and Ping360 sensors via GET /sensors",
            ),
            Rationaled::new(
                JourneyId("connect_ping_viewer_to_sonar".into()),
                "operator uses the UDP bridge port shown on the device card to reach the sonar from Ping Viewer",
            ),
            Rationaled::new(
                JourneyId("enable_ping1d_rangefinder_mavlink".into()),
                "Ping1D card MAVLink Distances switch posts sensor settings to toggle mavlink_driver",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "opt-in Ping sonar peripheral; vehicle flight and MAVLink control do not require sonar bridging or rangefinder forwarding",
        ),
        offline_required: Asserted::established(
            true,
            "serial/USB probing, UDP bridges, and optional DISTANCE_SENSOR POST to localhost mavlink2rest operate without internet",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Regular,
            "observed run_as blueos via RUN_AS_REGULAR_USER wrapper; reads/writes userdata settings and spawns bridges without root",
        ),
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "sensor detection, UDP bridging, and MAVLink distance toggle are reversible reconfiguration; no irreversible, untrusted-code, or vehicle-arm operations per rubric",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("list_detected_ping_sensors".to_string()),
                "GET /sensors returns auto-detected Ping1D and Ping360 devices with bridge ports and serial paths",
            ),
            Rationaled::new(
                CapabilityId("connect_ping_viewer_to_sonar".to_string()),
                "per-device UDP bridge exposes the sonar on an assigned port for Ping Viewer on the surface computer",
            ),
            Rationaled::new(
                CapabilityId("enable_ping1d_mavlink_distance".to_string()),
                "POST /sensors persists Ping1D settings to toggle mavlink_driver DISTANCE_SENSOR forwarding via mavlink2rest",
            ),
        ]),
        authorities: AssertedSet::established(vec![Rationaled::new(
            Authority::Other("ping_sonar_manager".to_string()),
            "sole catalog service that auto-detects Ping-family sonar devices and spawns bridges subprocesses per device",
        )]),
        states: AssertedSet::unknown(
            "no cataloged state machine; PingManager probe loop and per-device bridge subprocesses are runtime-managed",
        ),
        edges: AssertedSet::unknown(
            "observed OutboundHttp to localhost:6040 (mavlink2rest) for DISTANCE_SENSOR; target not cataloged yet so no validated ServiceId edge",
        ),
        resources: AssertedSet::established(vec![Rationaled::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/settings/ping".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            "persisted Ping1D sensor settings including mavlink_driver toggle consumed only by ping",
        )]),
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
                    ServiceId("pardal".to_string()),
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                vec![
                    ServiceId("user_terminal".to_string()),
                    ServiceId("ttyd".to_string()),
                    ServiceId("nginx".to_string()),
                    ServiceId("bag_of_holding".to_string()),
                    ServiceId("recorder".to_string()),
                    ServiceId("recorder_extractor".to_string()),
                    ServiceId("disk_usage".to_string()),
                    ServiceId("customization".to_string()),
                ],
                "observed ordered_before lists ping before remaining SERVICES-tier peers including customization",
            ),
            shutdown: Asserted::unknown(
                "main.py has no explicit shutdown handler; bridge subprocess cleanup relies on PingManager lifecycle",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight bridge subprocesses and userdata settings migration not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET /sensors availability serves as health signal".to_string(),
            "no dedicated /health route; uvicorn availability and probe loop continuity serve as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "Ping sonar device manager; does not install or host third-party extensions",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 router exposed under /ping/ via VersionedFastAPI",
        ),
        permissions_model: Asserted::established(
            "no separate permissions manifest; REST routes are unauthenticated".to_string(),
            "ping routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "sonar_not_detected".to_string(),
                "serial and ethernet probes find no Ping-protocol device; GET /sensors returns an empty list",
            ),
            Rationaled::new(
                "bridge_subprocess_crash".to_string(),
                "bridges binary exits for a device; UDP endpoint stops until PingManager respawns the bridge",
            ),
            Rationaled::new(
                "wrong_distance_affects_depth_hold".to_string(),
                "stale or incorrect Ping1D distance forwarded as DISTANCE_SENSOR can skew autopilot depth-hold while mavlink_driver is enabled",
            ),
            Rationaled::new(
                "mavlink2rest_unreachable".to_string(),
                "MavlinkMessenger POST to localhost:6040 fails when mavlink2rest is down; MAVLink distance forwarding stops",
            ),
            Rationaled::new(
                "udp_bridge_port_conflict".to_string(),
                "assigned bridge port from the 9090/9092 range may conflict if another process binds the same UDP port",
            ),
        ]),
        blast_radius: Asserted::established(
            "Ping sonar UI and UDP bridges unavailable; Ping Viewer cannot connect; enabled MAVLink rangefinder forwarding may feed wrong depth estimates to the autopilot"
                .to_string(),
            "outage blocks optional sonar telemetry path; misconfigured active mavlink_driver affects depth-hold estimates but not arm/disarm or motion commands directly",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and userdata settings migration stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
