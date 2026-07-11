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

const RUNTIME_CAPTURE: &str = "runtime-captures/beacon__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId::Beacon,
        state_contracts: GroundedSet::unknown(
            "beacon has no service-level state machine (card states Unknown); it runs a periodic mDNS re-advertisement loop",
        ),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/vehicle_name", 5.0, 7.0, 7.7, 40),
            runtime_slo(HttpMethod::Get, "/hostname", 4.7, 6.8, 6.9, 40),
            runtime_slo(HttpMethod::Get, "/services", 10.5, 14.8, 15.7, 40),
            runtime_slo(HttpMethod::Get, "/ip", 6.1, 7.8, 8.1, 40),
        ]),
        resource_usage: GroundedSet::known(vec![runtime_resource(
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
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "beacon advertises mDNS and serves identity regardless of flight controller; platform-independent".into(),
                    "runtime captured on Navigator only; RSS ~39.9 MB flat, CPU ~0.79% mean (spikes from ~10s mDNS loop)".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "POST /vehicle_name and POST /hostname persist to /root/.config/beacon/settings-4.json but were not exercised (would rename the live vehicle)",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId::Beacon,
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
        id: ServiceId::Beacon,
        aliases: ObservedSet::known(vec![Evidenced::new(
            "beacon".to_string(),
            Evidence {
                file: "core/services/beacon/main.py".to_string(),
                line: 25,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 130,
            },
        ),
        entrypoint: Observed::known(
            "$SERVICES_PATH/beacon/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 130,
            },
        ),
        tmux_name: Observed::known(
            "beacon".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 130,
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
                line: 130,
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 130,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/beacon/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 91,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(9111),
            Evidence {
                file: "core/services/beacon/main.py".to_string(),
                line: 348,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/beacon".to_string()),
            Evidence {
                file: "core/services/beacon/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/beacon/".to_string()),
                    port: PortRef::Literal(9111),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/beacon/main.py".to_string(),
                    line: 321,
                },
            ),
            Evidenced::new(
                Interface::Settings {
                    path: PathRef("/root/.config/beacon/settings-4.json".to_string()),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py".to_string(),
                    line: 69,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("core/services/beacon/default-settings.json".to_string()),
                    mode: FileAccessMode::Read,
                },
                Evidence {
                    file: "core/services/beacon/main.py".to_string(),
                    line: 96,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/beacon".to_string()),
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
                    topics_produced: vec!["services/beacon/log".to_string()],
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
                    path: PathRef("/root/.config/beacon".to_string()),
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
                    path: PathRef("/root/.config/beacon/settings-4.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 69,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("core/services/beacon/default-settings.json".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/services/beacon/main.py".to_string(),
                    line: 96,
                },
            ),
        ]),
        lifecycle: Observed::known(
            ObservedLifecycle {
                triggers: vec!["start-blueos-core create_service".to_string()],
                ordered_after: vec![
                    ServiceId::ArdupilotManager,
                    ServiceId::CableGuy,
                    ServiceId::MavlinkCameraManager,
                    ServiceId::Mavlink2rest,
                    ServiceId::Kraken,
                    ServiceId::Wifi,
                    ServiceId::Zenohd,
                ],
                ordered_before: vec![
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
                file: "core/start-blueos-core".to_string(),
                line: 326,
            },
        ),
        logs_path: Observed::unknown(
            "init_logger publishes to zenoh only; no on-disk log path set in beacon source",
        ),
        zenoh_log_topic: Observed::known(
            "services/beacon/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/beacon/main.py".to_string(),
                line: 337,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId::Beacon,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one Beacon process owns all mDNS runners",
        ),
        bounded_context: Asserted::established(
            "vehicle-identity-and-discovery".to_string(),
            "provisional 2.0 domain: mDNS LAN advertisement plus persisted vehicle name and hostname identity",
        ),
        journey_refs: AssertedSet::established(vec![
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
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "vehicle rename and hostname change are reversible settings; no irreversible, untrusted-code, or vehicle-arm operations",
        ),
        capabilities: AssertedSet::established(vec![
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
        authorities: AssertedSet::established(vec![Rationaled::new(
            Authority::Other("mdns_advertiser".to_string()),
            "sole publisher of BlueOS mDNS records on LAN interfaces; no other catalog service advertises blueos.local",
        )]),
        states: AssertedSet::unknown(
            "no cataloged state machine; mDNS runners and settings reload are periodic loop state",
        ),
        edges: AssertedSet::established(vec![]),
        resources: AssertedSet::established(vec![
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/beacon".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "SettingsV4 pykson manager directory for beacon identity and mDNS interface configuration",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/beacon/settings-4.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "persisted vehicle_name, hostname, and per-interface mDNS advertisement settings",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("core/services/beacon/default-settings.json".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                "load_default_settings seeds domain names and service types on first run",
            ),
        ]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                vec!["start-blueos-core create_service".to_string()],
                "observed lifecycle trigger: tmux creation at boot in SERVICES tier",
            ),
            ordered_after: Asserted::established(
                vec![
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
                vec![
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
                "beacon.stop unregisters all mDNS runners on uvicorn exit".to_string(),
                "main.py awaits beacon.stop after server.serve returns",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight mDNS registrations and settings migration not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns service name; mDNS run loop every 10s".to_string(),
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
            "no separate permissions manifest; REST routes are unauthenticated".to_string(),
            "beacon routes have no auth decorator or extension-style permissions JSON",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "mdns_registration_failure".to_string(),
                "run() logs warnings when runner.register_services raises; affected interface stays undiscoverable",
            ),
            Rationaled::new(
                "interface_runner_creation_failure".to_string(),
                "create_default_runners and create_user_runners skip interfaces when AsyncRunner construction fails",
            ),
            Rationaled::new(
                "service_info_value_error".to_string(),
                "create_async_service_infos ValueError skips individual service advertisements on an interface",
            ),
            Rationaled::new(
                "stale_mdns_after_hostname_change".to_string(),
                "hostname change updates settings; runner diff on next loop cycle re-registers domains",
            ),
        ]),
        blast_radius: Asserted::established(
            "mDNS hostname discovery and vehicle name sidebar stale; operators can still reach BlueOS by IP; MAVLink unaffected"
                .to_string(),
            "beacon outage blocks LAN name resolution UX but not autopilot, nginx core paths, or FC control",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and SettingsV4 migration stability not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
