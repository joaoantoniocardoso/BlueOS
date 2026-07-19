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
        service: ServiceId::Bridget,
        state_contracts: GroundedSet::unknown(
            "bridget has no service-level state machine (card states Unknown); per-bridge bridges subprocess lifecycle is runtime-managed",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/serial_ports", 23.8, 42.1, 65.5, 60),
            runtime_slo(HttpMethod::Get, "/bridges", 8.7, 12.9, 17.9, 60),
            runtime_slo(HttpMethod::Get, "/", 5.5, 8.7, 10.7, 60),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
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
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "bridget lists serial ports via linux2rest and manages operator-configured serial-to-UDP bridges; platform-independent",
                    "runtime captured on Navigator only with NO bridges configured; RSS ~38.1 MB flat, CPU ~1.73% mean with occasional spikes during capture",
                ],
            },
            runtime_prov("runtime-captures/bridget__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "POST /bridges and DELETE /bridges persist bridge configuration to /usr/blueos/userdata/settings/bridget; not exercised (mutating; no serial hardware to bridge)",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::Bridget,
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
        runtime_prov("runtime-captures/bridget__pi4_navigator_master.json#slo_running_baseline"),
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
        runtime_prov("runtime-captures/bridget__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts =
    ObservedFacts {
        id: ServiceId::Bridget,
        aliases: ObservedSet::known(&[Evidenced::new(
            "bridget",
            Evidence {
                file: "core/services/bridget/main.py",
                line: 16,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core",
                line: 131,
            },
        ),
        entrypoint: Observed::known(
            "nice -19 $RUN_AS_REGULAR_USER_BEGIN $SERVICES_PATH/bridget/main.py $RUN_AS_REGULAR_USER_END"
                ,
            Evidence {
                file: "core/start-blueos-core",
                line: 131,
            },
        ),
        tmux_name: Observed::known(
            "bridget",
            Evidence {
                file: "core/start-blueos-core",
                line: 131,
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
                memory_mb: Some(0),
                cpu_percent: Some(0),
                io_weight: None,
            },
            Evidence {
                file: "core/start-blueos-core",
                line: 131,
            },
        ),
        nice: Observed::known(
            -19,
            Evidence {
                file: "core/start-blueos-core",
                line: 131,
            },
        ),
        run_as: Observed::known(
            "blueos",
            Evidence {
                file: "core/start-blueos-core",
                line: 22,
            },
        ),
        nginx_prefixes: ObservedSet::known(&[Evidenced::new(
            PathRef("/bridget/"),
            Evidence {
                file: "core/tools/nginx/nginx.conf",
                line: 98,
            },
        )]),
        listen: ObservedSet::known(&[Evidenced::new(
            PortRef::Literal(27353),
            Evidence {
                file: "core/services/bridget/main.py",
                line: 83,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/bridget"),
            Evidence {
                file: "core/services/bridget/main.py",
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(&[
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/bridget/"),
                    port: PortRef::Literal(27353),
                    versions: &["v1.0"],
                },
                Evidence {
                    file: "core/services/bridget/main.py",
                    line: 64,
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
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "http://localhost:6030/serial",
                },
                Evidence {
                    file: "core/services/bridget/bridget.py",
                    line: 62,
                },
            ),
            Evidenced::new(
                Interface::Settings {
                    path: PathRef("/usr/blueos/userdata/settings/bridget"),
                },
                Evidence {
                    file: "core/services/bridget/bridget.py",
                    line: 51,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/usr/blueos/userdata/settings/bridget"),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/bridget/bridget.py",
                    line: 51,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: &["services/bridget/log"],
                    topics_consumed: &[],
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                    line: 78,
                },
            ),
        ]),
        resources: ObservedSet::known(&[Evidenced::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/settings/bridget"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/bridget/bridget.py",
                line: 51,
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
                ],
                ordered_before: &[
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
            },
        ),
        logs_path: Observed::unknown(
            "init_logger publishes to zenoh only; no on-disk log path set in bridget source",
        ),
        zenoh_log_topic: Observed::known(
            "services/bridget/log",
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/bridget/main.py",
                line: 80,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    };

pub const SERVICE_DEFINITION: ServiceDefinition =
    ServiceDefinition {
        id: ServiceId::Bridget,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one bridget process owns all operator-configured serial bridges and bridges subprocesses",
        ),
        bounded_context: Asserted::established(
            "serial-device-bridging",
            "provisional 2.0 domain: operator-configured serial-to-UDP bridges for arbitrary onboard serial devices",
        ),
        journey_refs: AssertedSet::established(&[
            Rationaled::new(
                JourneyId::ViewConfiguredSerialBridges,
                "Serial Bridges page lists configured bridges and available serial ports via GET /bridges and GET /serial_ports",
            ),
            Rationaled::new(
                JourneyId::CreateSerialToUdpBridge,
                "creation dialog POST /bridges adds a serial-to-UDP bridge, spawns bridges subprocess, and persists settings",
            ),
            Rationaled::new(
                JourneyId::RemoveSerialBridge,
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
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "bridge create/remove is reversible operator-initiated reconfiguration; no irreversible, untrusted-code, or vehicle-arm operations per rubric",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::ListConfiguredSerialBridges,
                "GET /bridges and GET /serial_ports return configured bridges and available serial ports for the Serial Bridges page",
            ),
            Rationaled::new(
                CapabilityId::CreateSerialToUdpBridge,
                "POST /bridges starts a bridges subprocess for the chosen serial path, baud, and UDP endpoint and persists the configuration",
            ),
            Rationaled::new(
                CapabilityId::RemoveSerialBridge,
                "DELETE /bridges stops and removes the matching serial-to-UDP bridge configuration",
            ),
        ]),
        authorities: AssertedSet::established(&[Rationaled::new(
            Authority::Other("serial_bridge_manager"),
            "sole catalog service that manages operator-configured arbitrary serial-to-UDP bridges; ping owns Ping-family sonar auto-detection separately",
        )]),
        states: AssertedSet::unknown(
            "no cataloged state machine; per-bridge bridges subprocess lifecycle is runtime-managed",
        ),
        edges: AssertedSet::established(&[Rationaled::new(
            Edge {
                from: ServiceId::Bridget,
                to: ServiceId::Linux2rest,
                via: Bus::Rest,
                sync: SyncMode::Sync,
                endpoint: "localhost:6030/serial",
                purpose: "enumerate host serial ports via linux2rest",
                required_at_boot: false,
                failure_impact: FailureImpact::Degraded,
            },
            "observed OutboundHttp http://localhost:6030/serial (bridget.py:62) pairs with linux2rest listen 6030",
        )]),
        resources: AssertedSet::established(&[Rationaled::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/settings/bridget"),
                ownership: ResourceOwnership::SharedWrite,
            },
            "persisted bridge configurations in settings-2.json consumed only by bridget",
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
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                &[
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
            "implicit: process liveness via tmux; REST GET /bridges availability serves as health signal",
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
            "no separate permissions manifest; REST routes are unauthenticated",
            "bridget routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "linux2rest_unreachable",
                "GET /serial_ports proxy to localhost:6030/serial fails when linux2rest is down; bridge creation cannot list available ports",
            ),
            Rationaled::new(
                "bridge_subprocess_crash",
                "bridges binary exits for a configured bridge; UDP endpoint stops until bridget respawns or operator recreates the bridge",
            ),
            Rationaled::new(
                "serial_port_contention",
                "misconfigured bridge binds a serial device already used by autopilot telemetry or another payload; both consumers may fail or see corrupted data",
            ),
            Rationaled::new(
                "udp_endpoint_conflict",
                "chosen UDP port may conflict if another process binds the same endpoint on the host or network",
            ),
            Rationaled::new(
                "settings_persistence_failure",
                "userdata settings write fails; new bridge may not survive reboot or may be lost on process restart",
            ),
        ]),
        blast_radius: Asserted::established(
            "Serial Bridges UI and configured serial-to-UDP links unavailable; surface tools cannot reach bridged serial devices; a misconfigured active bridge may expose or contend for a serial port used elsewhere"
                ,
            "outage blocks optional serial network bridging; misconfigured bridge affects only the chosen serial device and UDP endpoint, not arm/disarm or motion commands directly",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and userdata settings migration stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Bridget,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
