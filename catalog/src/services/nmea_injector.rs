use crate::criticality::CriticalityTier;
use crate::edge::{Bus, Connection, FailureImpact, SyncMode};
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, PortKind};
use crate::journey::{HttpMethod, RouteRef};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use crate::resource::{Resource, ResourceOwnership};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline};
use crate::service::{Authority, Service, ServiceJudgment};
use crate::trust::{PrivilegeLevel, UserConfirmation};

use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::NmeaInjector,
        state_contracts: GroundedSet::unknown(
            "nmea_injector has no service-level state machine (card states Unknown); TrafficController manages a dynamic set of listener sockets",
        ),
        slo_baselines: GroundedSet::known(&[runtime_slo(
            HttpMethod::Get,
            "/socks",
            8.3,
            11.3,
            12.2,
            40,
        )]),
        resource_usage: GroundedSet::known(&[runtime_resource(
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
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "nmea_injector runs regardless of flight controller; it POSTs GPS_INPUT to mavlink2rest; platform-independent",
                    "runtime captured on Navigator only with NO sockets configured; RSS ~39.5 MB flat, CPU ~0.26% mean",
                    "active NMEA ingest (not captured) would add per-message parse + mavlink2rest POST load",
                ],
            },
            runtime_prov("runtime-captures/nmea_injector__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "POST /socks and DELETE /socks persist the socket spec list to /root/.config/nmea-injector/settings-1.json; not exercised",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::NmeaInjector,
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
        runtime_prov(
            "runtime-captures/nmea_injector__pi4_navigator_master.json#slo_running_baseline",
        ),
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
        runtime_prov("runtime-captures/nmea_injector__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts = ObservedFacts {
    id: ServiceId::NmeaInjector,
    aliases: ObservedSet::known(&[Evidenced::new(
        "nmea-injector",
        Evidence {
            file: "core/services/nmea_injector/main.py",
            line: 17,
            anchor: "SERVICE_NAME = \"nmea-injector\"",
        },
    )]),
    kind: Observed::known(
        ServiceKind::PythonService,
        Evidence {
            file: "core/start-blueos-core",
            line: 133,
            anchor: "'nmea_injector',250,0,0,0,\"nice -19 $SERVICES_PATH/nmea_inje",
        },
    ),
    entrypoint: Observed::known(
        "nice -19 $SERVICES_PATH/nmea_injector/main.py",
        Evidence {
            file: "core/start-blueos-core",
            line: 133,
            anchor: "'nmea_injector',250,0,0,0,\"nice -19 $SERVICES_PATH/nmea_inje",
        },
    ),
    tmux_name: Observed::known(
        "nmea_injector",
        Evidence {
            file: "core/start-blueos-core",
            line: 133,
            anchor: "'nmea_injector',250,0,0,0,\"nice -19 $SERVICES_PATH/nmea_inje",
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
            line: 133,
            anchor: "'nmea_injector',250,0,0,0,\"nice -19 $SERVICES_PATH/nmea_inje",
        },
    ),
    nice: Observed::known(
        19,
        Evidence {
            file: "core/start-blueos-core",
            line: 133,
            anchor: "'nmea_injector',250,0,0,0,\"nice -19 $SERVICES_PATH/nmea_inje",
        },
    ),
    run_as: Observed::known(
        "root",
        Evidence {
            file: "core/start-blueos-core",
            line: 133,
            anchor: "'nmea_injector',250,0,0,0,\"nice -19 $SERVICES_PATH/nmea_inje",
        },
    ),
    nginx_prefixes: ObservedSet::known(&[Evidenced::new(
        PathRef("/nmea-injector/"),
        Evidence {
            file: "core/tools/nginx/nginx.conf",
            line: 158,
            anchor: "location /nmea-injector/ {",
        },
    )]),
    listen: ObservedSet::known(&[Evidenced::new(
        PortRef::Literal(2748),
        Evidence {
            file: "core/services/nmea_injector/main.py",
            line: 88,
            anchor: "config = Config(app=app, host=\"0.0.0.0\", port=2748, log_conf",
        },
    )]),
    git_path: Observed::known(
        PathRef("core/services/nmea_injector"),
        Evidence {
            file: "core/services/nmea_injector/main.py",
            line: 1,
            anchor: "#! /usr/bin/env python3",
        },
    ),
    interfaces: ObservedSet::known(&[
        Evidenced::new(
            PortKind::Rest {
                path_prefix: PathRef("/nmea-injector/"),
                port: PortRef::Literal(2748),
                versions: &["v1.0"],
            },
            Evidence {
                file: "core/services/nmea_injector/main.py",
                line: 69,
                anchor: "app = VersionedFastAPI(app, version=\"1.0.0\", prefix_format=\"",
            },
        ),
        Evidenced::new(
            PortKind::OutboundHttp {
                url: "localhost:6040",
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/mavlink_comm/MavlinkComm.py",
                line: 24,
                anchor: "self.m2r_address = \"localhost:6040\"",
            },
        ),
        Evidenced::new(
            PortKind::Settings {
                path: PathRef("/root/.config/nmea-injector/settings-1.json"),
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py",
                line: 69,
                anchor: "return self.config_folder.joinpath(f\"{PyksonManager.SETTINGS",
            },
        ),
        Evidenced::new(
            PortKind::File {
                path: PathRef("/root/.config/nmea-injector"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py",
                line: 27,
                anchor: "else pathlib.Path(appdirs.user_config_dir(self.project_name)",
            },
        ),
        Evidenced::new(
            PortKind::Zenoh {
                topics_produced: &["services/nmea-injector/log"],
                topics_consumed: &[],
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                line: 78,
                anchor: "topic = f\"services/{service_name}/log\"",
            },
        ),
    ]),
    resources: ObservedSet::known(&[
        Evidenced::new(
            Resource {
                path: PathRef("/root/.config/nmea-injector"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py",
                line: 27,
                anchor: "else pathlib.Path(appdirs.user_config_dir(self.project_name)",
            },
        ),
        Evidenced::new(
            Resource {
                path: PathRef("/root/.config/nmea-injector/settings-1.json"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py",
                line: 69,
                anchor: "return self.config_folder.joinpath(f\"{PyksonManager.SETTINGS",
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
                ServiceId::Zenohd,
                ServiceId::Beacon,
                ServiceId::Bridget,
                ServiceId::Commander,
            ],
            ordered_before: &[
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
        "init_logger publishes to zenoh only; no on-disk log path set in nmea_injector source",
    ),
    zenoh_log_topic: Observed::known(
        "services/nmea-injector/log",
        Evidence {
            file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
            line: 78,
            anchor: "topic = f\"services/{service_name}/log\"",
        },
    ),
    sentry: Observed::known(
        true,
        Evidence {
            file: "core/services/nmea_injector/main.py",
            line: 85,
            anchor: "await init_sentry_async(SERVICE_NAME)",
        },
    ),
    openapi_refs: ObservedSet::unknown("not yet extracted"),
};

pub const SERVICE_DEFINITION: ServiceJudgment =
    ServiceJudgment {
        id: ServiceId::NmeaInjector,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one nmea_injector process owns all NMEA listen sockets",
        ),
        bounded_context: Asserted::established(
            "external-gps-nmea-injection",
            "provisional 2.0 domain: opt-in external NMEA ingest parsed into MAVLink GPS_INPUT for the autopilot",
        ),
        journey_refs: AssertedSet::established(&[
            Rationaled::new(
                JourneyId::ViewConfiguredNmeaSockets,
                "NMEA Injector page lists configured sockets via GET /socks",
            ),
            Rationaled::new(
                JourneyId::AddExternalNmeaGpsSocket,
                "creation dialog submits socket kind, port, and component ID via POST /socks",
            ),
            Rationaled::new(
                JourneyId::RemoveConfiguredNmeaSocket,
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
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "socket add/remove is reversible reconfiguration; no irreversible, untrusted-code, or vehicle-arm operations per rubric",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::ListNmeaSockets,
                "GET /socks returns configured NMEA sockets with kind, port, and MAVLink component ID",
            ),
            Rationaled::new(
                CapabilityId::CreateNmeaSocket,
                "POST /socks opens a UDP or TCP listen socket and persists the spec in SettingsV1",
            ),
            Rationaled::new(
                CapabilityId::RemoveNmeaSocket,
                "DELETE /socks closes the matching listen socket and removes it from SettingsV1",
            ),
        ]),
        authorities: AssertedSet::established(&[Rationaled::new(
            Authority::Other("nmea_gps_injector"),
            "sole catalog service that ingests external NMEA and produces MAVLink GPS_INPUT messages via mavlink2rest",
        )]),
        states: AssertedSet::unknown(
            "no cataloged state machine; socket listeners and settings reload are managed inside TrafficController",
        ),
        edges: AssertedSet::established(&[Rationaled::new(
            Connection {
                from: ServiceId::NmeaInjector,
                to: ServiceId::Mavlink2rest,
                via: Bus::Rest,
                sync: SyncMode::Async,
                endpoint: "localhost:6040",
                purpose: "inject external GPS as MAVLink GPS_INPUT via mavlink2rest",
                required_at_boot: false,
                failure_impact: FailureImpact::Degraded,
            },
            "observed OutboundHttp localhost:6040 (MavlinkComm.py:24) pairs with mavlink2rest listen 6040",
        )]),
        resources: AssertedSet::established(&[
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/nmea-injector"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "SettingsV1 pykson manager directory for persisted NMEA socket specifications",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/nmea-injector/settings-1.json"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "persisted socket kind, port, and MAVLink component ID list consumed only by nmea_injector",
            ),
        ]),
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
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                &[
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
            "implicit: process liveness via tmux; REST GET / returns service name",
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
            "no separate permissions manifest; REST routes are unauthenticated",
            "nmea_injector routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "injecting_wrong_position_affects_navigation",
                "malformed or spoofed NMEA forwarded as GPS_INPUT can skew autopilot position estimates while sockets are active",
            ),
            Rationaled::new(
                "mavlink2rest_unreachable",
                "MavlinkMessenger POST to localhost:6040 fails when mavlink2rest is down; NMEA data is dropped",
            ),
            Rationaled::new(
                "invalid_nmea_parse_failure",
                "pynmea2.parse errors on non-NMEA datagrams prevent GPS_INPUT forwarding for that message",
            ),
            Rationaled::new(
                "port_conflict_on_socket_create",
                "add_sock fails when the requested UDP/TCP port is already bound on the host",
            ),
            Rationaled::new(
                "remove_nonexistent_socket",
                "DELETE /socks returns error when the specified kind, port, and component ID is not configured",
            ),
        ]),
        blast_radius: Asserted::established(
            "external GPS injection unavailable; autopilot falls back to onboard GPS; misconfigured active sockets can corrupt position data"
                ,
            "outage stops optional external GPS path; live misconfiguration affects navigation estimates but not arm/disarm or motion commands directly",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and SettingsV1 migration stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::NmeaInjector,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
