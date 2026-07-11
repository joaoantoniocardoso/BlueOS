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
use crate::service::{Authority, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

const RUNTIME_CAPTURE: &str = "runtime-captures/nmea_injector__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId("nmea_injector".into()),
        state_contracts: GroundedSet::unknown(
            "nmea_injector has no service-level state machine (card states Unknown); TrafficController manages a dynamic set of listener sockets",
        ),
        slo_baselines: GroundedSet::known(vec![runtime_slo(
            HttpMethod::Get,
            "/socks",
            8.3,
            11.3,
            12.2,
            40,
        )]),
        resource_usage: GroundedSet::known(vec![runtime_resource(
            "running_baseline",
            Distribution {
                mean: 0.26,
                median: 0.00,
                p95: 1.01,
                min: 0.00,
                max: 1.83,
                sd: 0.46,
            },
            Distribution {
                mean: 39.5,
                median: 39.5,
                p95: 39.5,
                min: 39.5,
                max: 39.5,
                sd: 0.0,
            },
            60,
        )]),
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "nmea_injector runs regardless of flight controller; it POSTs GPS_INPUT to mavlink2rest; platform-independent".into(),
                    "runtime captured on Navigator only with NO sockets configured; RSS ~39.5 MB flat, CPU ~0.26% mean".into(),
                    "active NMEA ingest (not captured) would add per-message parse + mavlink2rest POST load".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "POST /socks and DELETE /socks persist the socket spec list to /root/.config/nmea-injector/settings-1.json; not exercised",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId("nmea_injector".into()),
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
        id: ServiceId("nmea_injector".to_string()),
        aliases: ObservedSet::known(vec![Evidenced::new(
            "nmea-injector".to_string(),
            Evidence {
                file: "core/services/nmea_injector/main.py".to_string(),
                line: 17,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 133,
            },
        ),
        entrypoint: Observed::known(
            "nice -19 $SERVICES_PATH/nmea_injector/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 133,
            },
        ),
        tmux_name: Observed::known(
            "nmea_injector".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 133,
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
                line: 133,
            },
        ),
        nice: Observed::known(
            19,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 133,
            },
        ),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 133,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/nmea-injector/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 158,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(2748),
            Evidence {
                file: "core/services/nmea_injector/main.py".to_string(),
                line: 88,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/nmea_injector".to_string()),
            Evidence {
                file: "core/services/nmea_injector/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/nmea-injector/".to_string()),
                    port: PortRef::Literal(2748),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/nmea_injector/main.py".to_string(),
                    line: 69,
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
                    path: PathRef("/root/.config/nmea-injector/settings-1.json".to_string()),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 69,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/nmea-injector".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/nmea-injector/log".to_string()],
                    topics_consumed: vec![],
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                    line: 78,
                },
            ),
        ]),
        resources: ObservedSet::known(vec![
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/nmea-injector".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/nmea-injector/settings-1.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 69,
                },
            ),
        ]),
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
                ],
                ordered_before: vec![
                    ServiceId("helper".to_string()),
                    ServiceId("iperf3".to_string()),
                    ServiceId("linux2rest".to_string()),
                    ServiceId("filebrowser".to_string()),
                    ServiceId("versionchooser".to_string()),
                    ServiceId("pardal".to_string()),
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
            "init_logger publishes to zenoh only; no on-disk log path set in nmea_injector source",
        ),
        zenoh_log_topic: Observed::known(
            "services/nmea-injector/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/nmea_injector/main.py".to_string(),
                line: 85,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("nmea_injector".to_string()),
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one nmea_injector process owns all NMEA listen sockets",
        ),
        bounded_context: Asserted::established(
            "external-gps-nmea-injection".to_string(),
            "provisional 2.0 domain: opt-in external NMEA ingest parsed into MAVLink GPS_INPUT for the autopilot",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId("view_configured_nmea_sockets".into()),
                "NMEA Injector page lists configured sockets via GET /socks",
            ),
            Rationaled::new(
                JourneyId("add_external_nmea_gps_socket".into()),
                "creation dialog submits socket kind, port, and component ID via POST /socks",
            ),
            Rationaled::new(
                JourneyId("remove_configured_nmea_socket".into()),
                "socket card remove button deletes the matching socket via DELETE /socks",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "opt-in external GPS injection; core vehicle operation uses the autopilot's own GPS and does not require this service",
        ),
        offline_required: Asserted::established(
            true,
            "NMEA listen sockets and GPS_INPUT POST to localhost mavlink2rest operate without internet",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; binds UDP/TCP listen sockets on 0.0.0.0 and writes /root/.config/nmea-injector settings",
        ),
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "socket add/remove is reversible reconfiguration; no irreversible, untrusted-code, or vehicle-arm operations per rubric",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("list_nmea_sockets".to_string()),
                "GET /socks returns configured NMEA sockets with kind, port, and MAVLink component ID",
            ),
            Rationaled::new(
                CapabilityId("create_nmea_socket".to_string()),
                "POST /socks opens a UDP or TCP listen socket and persists the spec in SettingsV1",
            ),
            Rationaled::new(
                CapabilityId("remove_nmea_socket".to_string()),
                "DELETE /socks closes the matching listen socket and removes it from SettingsV1",
            ),
        ]),
        authorities: AssertedSet::established(vec![Rationaled::new(
            Authority::Other("nmea_gps_injector".to_string()),
            "sole catalog service that ingests external NMEA and produces MAVLink GPS_INPUT messages via mavlink2rest",
        )]),
        states: AssertedSet::unknown(
            "no cataloged state machine; socket listeners and settings reload are managed inside TrafficController",
        ),
        edges: AssertedSet::established(vec![Rationaled::new(
            Edge {
                from: ServiceId("nmea_injector".to_string()),
                to: ServiceId("mavlink2rest".to_string()),
                via: Bus::Rest,
                sync: SyncMode::Async,
                endpoint: "localhost:6040".to_string(),
                purpose: "inject external GPS as MAVLink GPS_INPUT via mavlink2rest".to_string(),
                required_at_boot: false,
                failure_impact: FailureImpact::Degraded,
            },
            "observed OutboundHttp localhost:6040 (MavlinkComm.py:24) pairs with mavlink2rest listen 6040",
        )]),
        resources: AssertedSet::established(vec![
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/nmea-injector".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "SettingsV1 pykson manager directory for persisted NMEA socket specifications",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/nmea-injector/settings-1.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "persisted socket kind, port, and MAVLink component ID list consumed only by nmea_injector",
            ),
        ]),
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
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                vec![
                    ServiceId("helper".to_string()),
                    ServiceId("iperf3".to_string()),
                    ServiceId("linux2rest".to_string()),
                    ServiceId("filebrowser".to_string()),
                    ServiceId("versionchooser".to_string()),
                    ServiceId("pardal".to_string()),
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
                "observed ordered_before lists nmea_injector before remaining SERVICES-tier peers including customization",
            ),
            shutdown: Asserted::unknown(
                "main.py has no explicit shutdown handler; socket cleanup relies on TrafficController __del__",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight NMEA sockets and settings migration not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns service name".to_string(),
            "no dedicated /health route; uvicorn availability serves as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "external GPS injection utility; does not install or host third-party extensions",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 router exposed under /nmea-injector/ via VersionedFastAPI",
        ),
        permissions_model: Asserted::established(
            "no separate permissions manifest; REST routes are unauthenticated".to_string(),
            "nmea_injector routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "injecting_wrong_position_affects_navigation".to_string(),
                "malformed or spoofed NMEA forwarded as GPS_INPUT can skew autopilot position estimates while sockets are active",
            ),
            Rationaled::new(
                "mavlink2rest_unreachable".to_string(),
                "MavlinkMessenger POST to localhost:6040 fails when mavlink2rest is down; NMEA data is dropped",
            ),
            Rationaled::new(
                "invalid_nmea_parse_failure".to_string(),
                "pynmea2.parse errors on non-NMEA datagrams prevent GPS_INPUT forwarding for that message",
            ),
            Rationaled::new(
                "port_conflict_on_socket_create".to_string(),
                "add_sock fails when the requested UDP/TCP port is already bound on the host",
            ),
            Rationaled::new(
                "remove_nonexistent_socket".to_string(),
                "DELETE /socks returns error when the specified kind, port, and component ID is not configured",
            ),
        ]),
        blast_radius: Asserted::established(
            "external GPS injection unavailable; autopilot falls back to onboard GPS; misconfigured active sockets can corrupt position data"
                .to_string(),
            "outage stops optional external GPS path; live misconfiguration affects navigation estimates but not arm/disarm or motion commands directly",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and SettingsV1 migration stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
