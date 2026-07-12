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
use crate::service::{Authority, Service, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Beacon,
        state_contracts: GroundedSet::unknown(
            "beacon has no service-level state machine (card states Unknown); it runs a periodic mDNS re-advertisement loop",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/vehicle_name", 5.0, 7.0, 7.7, 40),
            runtime_slo(HttpMethod::Get, "/hostname", 4.7, 6.8, 6.9, 40),
            runtime_slo(HttpMethod::Get, "/services", 10.5, 14.8, 15.7, 40),
            runtime_slo(HttpMethod::Get, "/ip", 6.1, 7.8, 8.1, 40),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
            "running_baseline",
            Distribution {
                mean: 0.79,
                median: 0.00,
                p95: 4.02,
                min: 0.00,
                max: 5.74,
                sd: 1.35,
            },
            Distribution {
                mean: 39.9,
                median: 39.9,
                p95: 39.9,
                min: 39.9,
                max: 39.9,
                sd: 0.0,
            },
            60,
        )]),
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "beacon advertises mDNS and serves identity regardless of flight controller; platform-independent",
                    "runtime captured on Navigator only; RSS ~39.9 MB flat, CPU ~0.79% mean (spikes from ~10s mDNS loop)",
                ],
            },
            runtime_prov("runtime-captures/beacon__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "POST /vehicle_name and POST /hostname persist to /root/.config/beacon/settings-4.json but were not exercised (would rename the live vehicle)",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::Beacon,
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
        runtime_prov("runtime-captures/beacon__pi4_navigator_master.json#slo_running_baseline"),
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
        runtime_prov("runtime-captures/beacon__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts = ObservedFacts {
    id: ServiceId::Beacon,
    aliases: ObservedSet::known(&[Evidenced::new(
        "beacon",
        Evidence {
            file: "core/services/beacon/main.py",
            line: 25,
        },
    )]),
    kind: Observed::known(
        ServiceKind::PythonService,
        Evidence {
            file: "core/start-blueos-core",
            line: 130,
        },
    ),
    entrypoint: Observed::known(
        "$SERVICES_PATH/beacon/main.py",
        Evidence {
            file: "core/start-blueos-core",
            line: 130,
        },
    ),
    tmux_name: Observed::known(
        "beacon",
        Evidence {
            file: "core/start-blueos-core",
            line: 130,
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
            line: 130,
        },
    ),
    nice: Observed::unknown("no nice prefix in start tuple"),
    run_as: Observed::known(
        "root",
        Evidence {
            file: "core/start-blueos-core",
            line: 130,
        },
    ),
    nginx_prefixes: ObservedSet::known(&[Evidenced::new(
        PathRef("/beacon/"),
        Evidence {
            file: "core/tools/nginx/nginx.conf",
            line: 91,
        },
    )]),
    listen: ObservedSet::known(&[Evidenced::new(
        PortRef::Literal(9111),
        Evidence {
            file: "core/services/beacon/main.py",
            line: 348,
        },
    )]),
    git_path: Observed::known(
        PathRef("core/services/beacon"),
        Evidence {
            file: "core/services/beacon/main.py",
            line: 1,
        },
    ),
    interfaces: ObservedSet::known(&[
        Evidenced::new(
            Interface::Rest {
                path_prefix: PathRef("/beacon/"),
                port: PortRef::Literal(9111),
                versions: &["v1.0"],
            },
            Evidence {
                file: "core/services/beacon/main.py",
                line: 321,
            },
        ),
        Evidenced::new(
            Interface::Settings {
                path: PathRef("/root/.config/beacon/settings-4.json"),
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py",
                line: 69,
            },
        ),
        Evidenced::new(
            Interface::File {
                path: PathRef("core/services/beacon/default-settings.json"),
                mode: FileAccessMode::Read,
            },
            Evidence {
                file: "core/services/beacon/main.py",
                line: 96,
            },
        ),
        Evidenced::new(
            Interface::File {
                path: PathRef("/root/.config/beacon"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py",
                line: 27,
            },
        ),
        Evidenced::new(
            Interface::Zenoh {
                topics_produced: &["services/beacon/log"],
                topics_consumed: &[],
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                line: 78,
            },
        ),
    ]),
    resources: ObservedSet::known(&[
        Evidenced::new(
            Resource {
                path: PathRef("/root/.config/beacon"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py",
                line: 27,
            },
        ),
        Evidenced::new(
            Resource {
                path: PathRef("/root/.config/beacon/settings-4.json"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py",
                line: 69,
            },
        ),
        Evidenced::new(
            Resource {
                path: PathRef("core/services/beacon/default-settings.json"),
                ownership: ResourceOwnership::SharedRead,
            },
            Evidence {
                file: "core/services/beacon/main.py",
                line: 96,
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
            ],
            ordered_before: &[
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
        },
    ),
    logs_path: Observed::unknown(
        "init_logger publishes to zenoh only; no on-disk log path set in beacon source",
    ),
    zenoh_log_topic: Observed::known(
        "services/beacon/log",
        Evidence {
            file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
            line: 78,
        },
    ),
    sentry: Observed::known(
        true,
        Evidence {
            file: "core/services/beacon/main.py",
            line: 337,
        },
    ),
    openapi_refs: ObservedSet::unknown("not yet extracted"),
};

pub const SERVICE_DEFINITION: ServiceDefinition =
    ServiceDefinition {
        id: ServiceId::Beacon,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one Beacon process owns all mDNS runners",
        ),
        bounded_context: Asserted::established(
            "vehicle-identity-and-discovery",
            "provisional 2.0 domain: mDNS LAN advertisement plus persisted vehicle name and hostname identity",
        ),
        journey_refs: AssertedSet::established(&[
            Rationaled::new(
                JourneyId::RenameVehicle,
                "sidebar vehicle identifier edit persists display name via POST /vehicle_name",
            ),
            Rationaled::new(
                JourneyId::ChangeMdnsHostname,
                "sidebar edit updates the mDNS hostname broadcast via POST /hostname",
            ),
            Rationaled::new(
                JourneyId::DiscoverBlueosOnNetwork,
                "beacon publishes blueos.local mDNS records so operators can open the web UI on the LAN",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Important,
            "LAN hostname discovery and vehicle identity UX depend on beacon; MAVLink vehicle control paths do not",
        ),
        offline_required: Asserted::established(
            true,
            "mDNS advertisement and identity REST API operate on local network interfaces without internet",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; binds mDNS on network interfaces and writes /root/.config/beacon settings",
        ),
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "vehicle rename and hostname change are reversible settings; no irreversible, untrusted-code, or vehicle-arm operations",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::SetVehicleName,
                "POST /vehicle_name persists the operator-facing vehicle display name in SettingsV4",
            ),
            Rationaled::new(
                CapabilityId::SetMdnsHostname,
                "POST /hostname updates default and per-interface mDNS domain names in settings",
            ),
            Rationaled::new(
                CapabilityId::AdvertiseMdnsDomains,
                "run() loop registers AsyncRunner mDNS services on filtered up interfaces every 10 seconds",
            ),
            Rationaled::new(
                CapabilityId::GetVehicleName,
                "GET /vehicle_name returns the persisted vehicle name with BlueROV2 default",
            ),
            Rationaled::new(
                CapabilityId::GetMdnsHostname,
                "GET /hostname returns the primary mDNS hostname from default.domain_names",
            ),
            Rationaled::new(
                CapabilityId::ListMdnsDomains,
                "GET /services returns currently broadcast MdnsEntry records from active runners",
            ),
            Rationaled::new(
                CapabilityId::ReportClientIp,
                "GET /ip returns client IP information from the incoming HTTP request",
            ),
        ]),
        authorities: AssertedSet::established(&[Rationaled::new(
            Authority::Other("mdns_advertiser"),
            "sole publisher of BlueOS mDNS records on LAN interfaces; no other catalog service advertises blueos.local",
        )]),
        states: AssertedSet::unknown(
            "no cataloged state machine; mDNS runners and settings reload are periodic loop state",
        ),
        edges: AssertedSet::established(&[]),
        resources: AssertedSet::established(&[
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/beacon"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "SettingsV4 pykson manager directory for beacon identity and mDNS interface configuration",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/beacon/settings-4.json"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "persisted vehicle_name, hostname, and per-interface mDNS advertisement settings",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("core/services/beacon/default-settings.json"),
                    ownership: ResourceOwnership::SharedRead,
                },
                "load_default_settings seeds domain names and service types on first run",
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
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                &[
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
                "observed ordered_before lists beacon before remaining SERVICES-tier peers including nginx",
            ),
            shutdown: Asserted::established(
                "beacon.stop unregisters all mDNS runners on uvicorn exit",
                "main.py awaits beacon.stop after server.serve returns",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight mDNS registrations and settings migration not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns service name; mDNS run loop every 10s",
            "no dedicated /health route; uvicorn availability and periodic mDNS registration serve as health signals",
        ),
        is_platform: Asserted::established(
            false,
            "identity and mDNS discovery utility; does not install or host third-party extensions",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 router exposed under /beacon/ via VersionedFastAPI",
        ),
        permissions_model: Asserted::established(
            "no separate permissions manifest; REST routes are unauthenticated",
            "beacon routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "mdns_registration_failure",
                "run() logs warnings when runner.register_services raises; affected interface stays undiscoverable",
            ),
            Rationaled::new(
                "interface_runner_creation_failure",
                "create_default_runners and create_user_runners skip interfaces when AsyncRunner construction fails",
            ),
            Rationaled::new(
                "service_info_value_error",
                "create_async_service_infos ValueError skips individual service advertisements on an interface",
            ),
            Rationaled::new(
                "stale_mdns_after_hostname_change",
                "hostname change updates settings; runner diff on next loop cycle re-registers domains",
            ),
        ]),
        blast_radius: Asserted::established(
            "mDNS hostname discovery and vehicle name sidebar stale; operators can still reach BlueOS by IP; MAVLink unaffected"
                ,
            "beacon outage blocks LAN name resolution UX but not autopilot, nginx core paths, or FC control",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and SettingsV4 migration stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Beacon,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
