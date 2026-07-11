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

const RUNTIME_CAPTURE: &str = "runtime-captures/bridget__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId("bridget".into()),
        state_contracts: GroundedSet::unknown(
            "bridget has no service-level state machine (card states Unknown); per-bridge bridges subprocess lifecycle is runtime-managed",
        ),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/serial_ports", 23.8, 42.1, 65.5, 60),
            runtime_slo(HttpMethod::Get, "/bridges", 8.7, 12.9, 17.9, 60),
            runtime_slo(HttpMethod::Get, "/", 5.5, 8.7, 10.7, 60),
        ]),
        resource_usage: GroundedSet::known(vec![runtime_resource(
            "running_baseline",
            Distribution {
                mean: 1.73,
                median: 0.00,
                p95: 1.05,
                min: 0.00,
                max: 78.92,
                sd: 9.85,
            },
            Distribution {
                mean: 38.1,
                median: 38.1,
                p95: 38.1,
                min: 38.1,
                max: 38.1,
                sd: 0.0,
            },
            90,
        )]),
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "bridget lists serial ports via linux2rest and manages operator-configured serial-to-UDP bridges; platform-independent".into(),
                    "runtime captured on Navigator only with NO bridges configured; RSS ~38.1 MB flat, CPU ~1.73% mean with occasional spikes during capture".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "POST /bridges and DELETE /bridges persist bridge configuration to /usr/blueos/userdata/settings/bridget; not exercised (mutating; no serial hardware to bridge)",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId("bridget".into()),
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
        id: ServiceId("bridget".to_string()),
        aliases: ObservedSet::known(vec![Evidenced::new(
            "bridget".to_string(),
            Evidence {
                file: "core/services/bridget/main.py".to_string(),
                line: 16,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 131,
            },
        ),
        entrypoint: Observed::known(
            "nice -19 $RUN_AS_REGULAR_USER_BEGIN $SERVICES_PATH/bridget/main.py $RUN_AS_REGULAR_USER_END"
                .to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 131,
            },
        ),
        tmux_name: Observed::known(
            "bridget".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 131,
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
                line: 131,
            },
        ),
        nice: Observed::known(
            -19,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 131,
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
            PathRef("/bridget/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 98,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(27353),
            Evidence {
                file: "core/services/bridget/main.py".to_string(),
                line: 83,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/bridget".to_string()),
            Evidence {
                file: "core/services/bridget/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/bridget/".to_string()),
                    port: PortRef::Literal(27353),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/bridget/main.py".to_string(),
                    line: 64,
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
                    url: "http://localhost:6030/serial".to_string(),
                },
                Evidence {
                    file: "core/services/bridget/bridget.py".to_string(),
                    line: 62,
                },
            ),
            Evidenced::new(
                Interface::Settings {
                    path: PathRef("/usr/blueos/userdata/settings/bridget".to_string()),
                },
                Evidence {
                    file: "core/services/bridget/bridget.py".to_string(),
                    line: 51,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/usr/blueos/userdata/settings/bridget".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/bridget/bridget.py".to_string(),
                    line: 51,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/bridget/log".to_string()],
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
                path: PathRef("/usr/blueos/userdata/settings/bridget".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/bridget/bridget.py".to_string(),
                line: 51,
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
                ],
                ordered_before: vec![
                    ServiceId("commander".to_string()),
                    ServiceId("nmea_injector".to_string()),
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
            "init_logger publishes to zenoh only; no on-disk log path set in bridget source",
        ),
        zenoh_log_topic: Observed::known(
            "services/bridget/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/bridget/main.py".to_string(),
                line: 80,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("bridget".to_string()),
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one bridget process owns all operator-configured serial bridges and bridges subprocesses",
        ),
        bounded_context: Asserted::established(
            "serial-device-bridging".to_string(),
            "provisional 2.0 domain: operator-configured serial-to-UDP bridges for arbitrary onboard serial devices",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId("view_configured_serial_bridges".into()),
                "Serial Bridges page lists configured bridges and available serial ports via GET /bridges and GET /serial_ports",
            ),
            Rationaled::new(
                JourneyId("create_serial_to_udp_bridge".into()),
                "creation dialog POST /bridges adds a serial-to-UDP bridge, spawns bridges subprocess, and persists settings",
            ),
            Rationaled::new(
                JourneyId("remove_serial_bridge".into()),
                "bridge card DELETE /bridges removes a configured serial bridge and stops its subprocess",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "advanced/pirate opt-in serial bridging; vehicle flight and MAVLink control operate without user-configured serial bridges",
        ),
        offline_required: Asserted::established(
            true,
            "serial port enumeration, UDP bridges, and userdata settings persistence are local; no internet required",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Regular,
            "observed run_as blueos via RUN_AS_REGULAR_USER wrapper; reads/writes userdata settings and spawns bridges without root",
        ),
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "bridge create/remove is reversible operator-initiated reconfiguration; no irreversible, untrusted-code, or vehicle-arm operations per rubric",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("list_configured_serial_bridges".to_string()),
                "GET /bridges and GET /serial_ports return configured bridges and available serial ports for the Serial Bridges page",
            ),
            Rationaled::new(
                CapabilityId("create_serial_to_udp_bridge".to_string()),
                "POST /bridges starts a bridges subprocess for the chosen serial path, baud, and UDP endpoint and persists the configuration",
            ),
            Rationaled::new(
                CapabilityId("remove_serial_bridge".to_string()),
                "DELETE /bridges stops and removes the matching serial-to-UDP bridge configuration",
            ),
        ]),
        authorities: AssertedSet::established(vec![Rationaled::new(
            Authority::Other("serial_bridge_manager".to_string()),
            "sole catalog service that manages operator-configured arbitrary serial-to-UDP bridges; ping owns Ping-family sonar auto-detection separately",
        )]),
        states: AssertedSet::unknown(
            "no cataloged state machine; per-bridge bridges subprocess lifecycle is runtime-managed",
        ),
        edges: AssertedSet::established(vec![Rationaled::new(
            Edge {
                from: ServiceId("bridget".to_string()),
                to: ServiceId("linux2rest".to_string()),
                via: Bus::Rest,
                sync: SyncMode::Sync,
                endpoint: "localhost:6030/serial".to_string(),
                purpose: "enumerate host serial ports via linux2rest".to_string(),
                required_at_boot: false,
                failure_impact: FailureImpact::Degraded,
            },
            "observed OutboundHttp http://localhost:6030/serial (bridget.py:62) pairs with linux2rest listen 6030",
        )]),
        resources: AssertedSet::established(vec![Rationaled::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/settings/bridget".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            "persisted bridge configurations in settings-2.json consumed only by bridget",
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
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                vec![
                    ServiceId("commander".to_string()),
                    ServiceId("nmea_injector".to_string()),
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
                "observed ordered_before lists bridget before remaining SERVICES-tier peers including customization",
            ),
            shutdown: Asserted::unknown(
                "main.py has no explicit shutdown handler; bridge subprocess cleanup relies on bridget process lifecycle",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight bridge subprocesses and userdata settings migration not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET /bridges availability serves as health signal".to_string(),
            "no dedicated /health route; uvicorn availability and configured bridge list continuity serve as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "Serial Bridges manager; does not install or host third-party extensions",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 router exposed under /bridget/ via VersionedFastAPI",
        ),
        permissions_model: Asserted::established(
            "no separate permissions manifest; REST routes are unauthenticated".to_string(),
            "bridget routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "linux2rest_unreachable".to_string(),
                "GET /serial_ports proxy to localhost:6030/serial fails when linux2rest is down; bridge creation cannot list available ports",
            ),
            Rationaled::new(
                "bridge_subprocess_crash".to_string(),
                "bridges binary exits for a configured bridge; UDP endpoint stops until bridget respawns or operator recreates the bridge",
            ),
            Rationaled::new(
                "serial_port_contention".to_string(),
                "misconfigured bridge binds a serial device already used by autopilot telemetry or another payload; both consumers may fail or see corrupted data",
            ),
            Rationaled::new(
                "udp_endpoint_conflict".to_string(),
                "chosen UDP port may conflict if another process binds the same endpoint on the host or network",
            ),
            Rationaled::new(
                "settings_persistence_failure".to_string(),
                "userdata settings write fails; new bridge may not survive reboot or may be lost on process restart",
            ),
        ]),
        blast_radius: Asserted::established(
            "Serial Bridges UI and configured serial-to-UDP links unavailable; surface tools cannot reach bridged serial devices; a misconfigured active bridge may expose or contend for a serial port used elsewhere"
                .to_string(),
            "outage blocks optional serial network bridging; misconfigured bridge affects only the chosen serial device and UDP endpoint, not arm/disarm or motion commands directly",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and userdata settings migration stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
