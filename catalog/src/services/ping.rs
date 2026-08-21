use crate::criticality::CriticalityTier;
use crate::edge::{Bus, Edge, FailureImpact, SyncMode};
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
use crate::service::{Authority, Service, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Ping,
        state_contracts: GroundedSet::unknown(
            "ping has no service-level state machine (card states Unknown); PingManager probe loop and per-device bridge subprocesses are runtime-managed",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/sensors", 10.9, 35.2, 39.7, 60),
            runtime_slo(HttpMethod::Get, "/", 5.7, 10.1, 14.8, 60),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
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
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "ping auto-detects Ping-family sonar on serial/USB and local ethernet; platform-independent",
                    "runtime captured on Navigator only with NO sonar hardware; RSS ~40.2 MB flat, CPU ~0.96% mean with occasional discovery spikes",
                ],
            },
            runtime_prov("runtime-captures/ping__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "POST /sensors persists Ping1D settings to /usr/blueos/userdata/settings/ping; not exercised (mutating; no sonar hardware)",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::Ping,
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
        runtime_prov("runtime-captures/ping__pi4_navigator_master.json#slo_running_baseline"),
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
        runtime_prov("runtime-captures/ping__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts =
    ObservedFacts {
        id: ServiceId::Ping,
        aliases: ObservedSet::known(&[Evidenced::new(
            "ping",
            Evidence {
                file: "core/services/ping/main.py",
                line: 20,
                anchor: "SERVICE_NAME = \"ping\"",
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core",
                line: 140,
                anchor: "'ping',0,0,0,0,\"nice -19 $RUN_AS_REGULAR_USER_BEGIN $SERVICE",
            },
        ),
        entrypoint: Observed::known(
            "nice -19 $RUN_AS_REGULAR_USER_BEGIN $SERVICES_PATH/ping/main.py $RUN_AS_REGULAR_USER_END"
                ,
            Evidence {
                file: "core/start-blueos-core",
                line: 140,
                anchor: "'ping',0,0,0,0,\"nice -19 $RUN_AS_REGULAR_USER_BEGIN $SERVICE",
            },
        ),
        tmux_name: Observed::known(
            "ping",
            Evidence {
                file: "core/start-blueos-core",
                line: 140,
                anchor: "'ping',0,0,0,0,\"nice -19 $RUN_AS_REGULAR_USER_BEGIN $SERVICE",
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
                line: 140,
                anchor: "'ping',0,0,0,0,\"nice -19 $RUN_AS_REGULAR_USER_BEGIN $SERVICE",
            },
        ),
        nice: Observed::known(
            -19,
            Evidence {
                file: "core/start-blueos-core",
                line: 140,
                anchor: "'ping',0,0,0,0,\"nice -19 $RUN_AS_REGULAR_USER_BEGIN $SERVICE",
            },
        ),
        run_as: Observed::known(
            "blueos",
            Evidence {
                file: "core/start-blueos-core",
                line: 22,
                anchor: "RUN_AS_REGULAR_USER_BEGIN=\"sudo -u blueos bash -c \\\"source /",
            },
        ),
        nginx_prefixes: ObservedSet::known(&[Evidenced::new(
            PathRef("/ping/"),
            Evidence {
                file: "core/tools/nginx/nginx.conf",
                line: 233,
                anchor: "location /ping/ {",
            },
        )]),
        listen: ObservedSet::known(&[Evidenced::new(
            PortRef::Literal(9110),
            Evidence {
                file: "core/services/ping/main.py",
                line: 86,
                anchor: "config = Config(app=app, host=\"0.0.0.0\", port=9110, log_conf",
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/ping"),
            Evidence {
                file: "core/services/ping/main.py",
                line: 1,
                anchor: "#! /usr/bin/env python3",
            },
        ),
        interfaces: ObservedSet::known(&[
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/ping/"),
                    port: PortRef::Literal(9110),
                    versions: &["v1.0"],
                },
                Evidence {
                    file: "core/services/ping/main.py",
                    line: 54,
                    anchor: "app = VersionedFastAPI(app, version=\"1.0.0\", prefix_format=\"",
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "{bridges} -u {ip}:{port} -p {serial_port.device}:{baud} {automatic_disconnect_clients}"
                        ,
                },
                Evidence {
                    file: "core/libs/bridges/src/bridges/bridges.py",
                    line: 29,
                    anchor: "command_line = f\"{bridges} -u {ip}:{port} -p {serial_port.de",
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "localhost:6040",
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/mavlink_comm/MavlinkComm.py"
                        ,
                    line: 24,
                    anchor: "self.m2r_address = \"localhost:6040\"",
                },
            ),
            Evidenced::new(
                Interface::Settings {
                    path: PathRef("/usr/blueos/userdata/settings/ping"),
                },
                Evidence {
                    file: "core/services/ping/ping1d_driver.py",
                    line: 22,
                    anchor: "self.manager = Manager(SERVICE_NAME, SettingsV1, USERDATA / ",
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/usr/blueos/userdata/settings/ping"),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/ping/ping1d_driver.py",
                    line: 22,
                    anchor: "self.manager = Manager(SERVICE_NAME, SettingsV1, USERDATA / ",
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: &["services/ping/log"],
                    topics_consumed: &[],
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                    line: 78,
                    anchor: "topic = f\"services/{service_name}/log\"",
                },
            ),
        ]),
        resources: ObservedSet::known(&[Evidenced::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/settings/ping"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/ping/ping1d_driver.py",
                line: 22,
                anchor: "self.manager = Manager(SERVICE_NAME, SettingsV1, USERDATA / ",
            },
        )]),
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
                ],
                ordered_before: &[
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
            "init_logger publishes to zenoh only; no on-disk log path set in ping source",
        ),
        zenoh_log_topic: Observed::known(
            "services/ping/log",
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                line: 78,
                anchor: "topic = f\"services/{service_name}/log\"",
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/ping/main.py",
                line: 80,
                anchor: "await init_sentry_async(SERVICE_NAME)",
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    };

pub const SERVICE_DEFINITION: ServiceDefinition =
    ServiceDefinition {
        id: ServiceId::Ping,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one ping process owns all sonar probes and bridges subprocesses",
        ),
        bounded_context: Asserted::established(
            "ping-sonar-integration",
            "provisional 2.0 domain: auto-detect Ping-family sonar devices, spawn UDP bridges, and optional MAVLink rangefinder forwarding",
        ),
        journey_refs: AssertedSet::established(&[
            Rationaled::new(
                JourneyId::ViewDetectedSonarDevices,
                "Ping Sonar Devices page lists auto-detected Ping1D and Ping360 sensors via GET /sensors",
            ),
            Rationaled::new(
                JourneyId::ConnectPingViewerToSonar,
                "operator uses the UDP bridge port shown on the device card to reach the sonar from Ping Viewer",
            ),
            Rationaled::new(
                JourneyId::EnablePing1dRangefinderMavlink,
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
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "sensor detection, UDP bridging, and MAVLink distance toggle are reversible reconfiguration; no irreversible, untrusted-code, or vehicle-arm operations per rubric",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::ListDetectedPingSensors,
                "GET /sensors returns auto-detected Ping1D and Ping360 devices with bridge ports and serial paths",
            ),
            Rationaled::new(
                CapabilityId::ConnectPingViewerToSonar,
                "per-device UDP bridge exposes the sonar on an assigned port for Ping Viewer on the surface computer",
            ),
            Rationaled::new(
                CapabilityId::EnablePing1dMavlinkDistance,
                "POST /sensors persists Ping1D settings to toggle mavlink_driver DISTANCE_SENSOR forwarding via mavlink2rest",
            ),
        ]),
        authorities: AssertedSet::established(&[Rationaled::new(
            Authority::Other("ping_sonar_manager"),
            "sole catalog service that auto-detects Ping-family sonar devices and spawns bridges subprocesses per device",
        )]),
        states: AssertedSet::unknown(
            "no cataloged state machine; PingManager probe loop and per-device bridge subprocesses are runtime-managed",
        ),
        edges: AssertedSet::established(&[Rationaled::new(
            Edge {
                from: ServiceId::Ping,
                to: ServiceId::Mavlink2rest,
                via: Bus::Rest,
                sync: SyncMode::Async,
                endpoint: "localhost:6040",
                purpose: "forward sonar DISTANCE_SENSOR to the vehicle via mavlink2rest",
                required_at_boot: false,
                failure_impact: FailureImpact::Degraded,
            },
            "observed OutboundHttp localhost:6040 (MavlinkComm.py:24) pairs with mavlink2rest listen 6040",
        )]),
        resources: AssertedSet::established(&[Rationaled::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/settings/ping"),
                ownership: ResourceOwnership::SharedWrite,
            },
            "persisted Ping1D sensor settings including mavlink_driver toggle consumed only by ping",
        )]),
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
                    ServiceId::Pardal,
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                &[
                    ServiceId::UserTerminal,
                    ServiceId::Ttyd,
                    ServiceId::Nginx,
                    ServiceId::BagOfHolding,
                    ServiceId::Recorder,
                    ServiceId::RecorderExtractor,
                    ServiceId::DiskUsage,
                    ServiceId::Customization,
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
            "implicit: process liveness via tmux; REST GET /sensors availability serves as health signal",
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
            "no separate permissions manifest; REST routes are unauthenticated",
            "ping routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "sonar_not_detected",
                "serial and ethernet probes find no Ping-protocol device; GET /sensors returns an empty list",
            ),
            Rationaled::new(
                "bridge_subprocess_crash",
                "bridges binary exits for a device; UDP endpoint stops until PingManager respawns the bridge",
            ),
            Rationaled::new(
                "wrong_distance_affects_depth_hold",
                "stale or incorrect Ping1D distance forwarded as DISTANCE_SENSOR can skew autopilot depth-hold while mavlink_driver is enabled",
            ),
            Rationaled::new(
                "mavlink2rest_unreachable",
                "MavlinkMessenger POST to localhost:6040 fails when mavlink2rest is down; MAVLink distance forwarding stops",
            ),
            Rationaled::new(
                "udp_bridge_port_conflict",
                "assigned bridge port from the 9090/9092 range may conflict if another process binds the same UDP port",
            ),
        ]),
        blast_radius: Asserted::established(
            "Ping sonar UI and UDP bridges unavailable; Ping Viewer cannot connect; enabled MAVLink rangefinder forwarding may feed wrong depth estimates to the autopilot"
                ,
            "outage blocks optional sonar telemetry path; misconfigured active mavlink_driver affects depth-hold estimates but not arm/disarm or motion commands directly",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and userdata settings migration stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Ping,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
