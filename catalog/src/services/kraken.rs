use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
use crate::journey::{HttpMethod, RouteRef};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::GroundedSet;
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, Observed, ObservedSet, Provenance,
    Rationaled,
};
use crate::resource::{Resource, ResourceOwnership};
use crate::runtime::{
    Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SettingsMutation, SloBaseline,
};
use crate::service::{Authority, ServiceDefinition};
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

const RUNTIME_CAPTURE: &str = "runtime-captures/kraken__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId::Kraken,
        state_contracts: GroundedSet::unknown(
            "kraken has no service-level state machine (service_definition states: Unknown); \
             per-extension enabled/running state lives in settings + Docker, not modeled as kraken states",
        ),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/installed_extensions", 6.6, 12.6, 14.9),
            runtime_slo(HttpMethod::Get, "/list_containers", 41.7, 56.2, 61.7),
            runtime_slo(HttpMethod::Get, "/stats", 2026.7, 2046.7, 2052.4),
            runtime_slo(HttpMethod::Get, "/container/", 43.1, 58.3, 60.1),
            runtime_slo(HttpMethod::Get, "/manifest/consolidated", 846.2, 929.2, 939.1),
        ]),
        resource_usage: GroundedSet::known(vec![
            runtime_resource(
                "running_baseline",
                Distribution {
                    mean: 1.12,
                    median: 1.06,
                    p95: 3.10,
                    min: 0.0,
                    max: 3.27,
                    sd: 0.92,
                },
                flat_rss(91.8),
                60,
            ),
            runtime_resource(
                "with_extension_installed",
                Distribution {
                    mean: 1.11,
                    median: 0.99,
                    p95: 2.04,
                    min: 0.0,
                    max: 2.98,
                    sd: 0.77,
                },
                flat_rss(91.8),
                40,
            ),
        ]),
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "kraken behavior is platform-independent (Docker/extension management does not depend on the flight-controller board); not captured across boards".into(),
                    "runtime captured on Navigator only; RSS ~91.8 MB, CPU ~1.12% mean".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::known(vec![
            runtime_settings_mutation(
                "POST /extension/install (v1.0)",
                vec!["extensions[]"],
            ),
            runtime_settings_mutation(
                "POST /extension/{identifier}/disable or POST /extension/{identifier}/{tag}/enable (v2.0)",
                vec!["extensions[].enabled"],
            ),
            runtime_settings_mutation(
                "DELETE /extension/{identifier} (v2.0)",
                vec!["extensions[]"],
            ),
        ]),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId::Kraken,
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
) -> GroundedItem<SloBaseline> {
    GroundedItem::new(
        SloBaseline {
            route: runtime_route(method, path),
            latency_p50_ms: p50,
            latency_p95_ms: p95,
            latency_p99_ms: p99,
            sample_size: 40,
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

fn runtime_settings_mutation(
    trigger: &str,
    keys_changed: Vec<&str>,
) -> GroundedItem<SettingsMutation> {
    GroundedItem::new(
        SettingsMutation {
            trigger: trigger.into(),
            keys_changed: keys_changed.into_iter().map(str::to_string).collect(),
        },
        runtime_prov("#settings_mutations"),
    )
}

pub fn observed_facts() -> ObservedFacts {
    ObservedFacts {
        id: ServiceId::Kraken,
        aliases: ObservedSet::known(vec![Evidenced::new(
            "kraken".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 126,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 126,
            },
        ),
        entrypoint: Observed::known(
            "nice -19 $BLUEOS_PYTHON_BIN_SECONDARY $SERVICES_PATH/kraken/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 126,
            },
        ),
        tmux_name: Observed::known(
            "kraken".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 126,
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
                line: 126,
            },
        ),
        nice: Observed::known(
            -19,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 126,
            },
        ),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 126,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/kraken/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 153,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(9134),
            Evidence {
                file: "core/services/kraken/args.py".to_string(),
                line: 26,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/kraken".to_string()),
            Evidence {
                file: "core/services/kraken/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/kraken/".to_string()),
                    port: PortRef::Literal(9134),
                    versions: vec!["v1.0".to_string(), "v2.0".to_string()],
                },
                Evidence {
                    file: "core/services/kraken/api/app.py".to_string(),
                    line: 53,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "https://bluerobotics.github.io/BlueOS-Extensions-Repository/manifest.json"
                        .to_string(),
                },
                Evidence {
                    file: "core/services/kraken/config.py".to_string(),
                    line: 9,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "https://blueos.cloud/major_tom/install".to_string(),
                },
                Evidence {
                    file: "core/services/kraken/config.py".to_string(),
                    line: 16,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "http://0.0.0.0:9134".to_string(),
                },
                Evidence {
                    file: "core/services/kraken/main.py".to_string(),
                    line: 39,
                },
            ),
            Evidenced::new(
                Interface::Settings {
                    path: PathRef("/root/.config/kraken/settings-2.json".to_string()),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pykson_manager.py"
                        .to_string(),
                    line: 69,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/var/run/docker.sock".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/kraken/harbor/contexts.py".to_string(),
                    line: 14,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/kraken".to_string()),
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
                    topics_produced: vec!["services/kraken/log".to_string()],
                    topics_consumed: vec![],
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                    line: 78,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["extensions/logs/{safe_name}".to_string()],
                    topics_consumed: vec![],
                },
                Evidence {
                    file: "core/services/kraken/extension_logs.py".to_string(),
                    line: 153,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["kraken/extension/logs/request".to_string()],
                    topics_consumed: vec![],
                },
                Evidence {
                    file: "core/services/kraken/zenoh_handlers/extension_handler.py".to_string(),
                    line: 53,
                },
            ),
        ]),
        resources: ObservedSet::known(vec![
            Evidenced::new(
                Resource {
                    path: PathRef("/var/run/docker.sock".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/kraken/harbor/contexts.py".to_string(),
                    line: 14,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/kraken".to_string()),
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
                    path: PathRef("/root/.config/kraken/settings-2.json".to_string()),
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
                    ServiceId::ArdupilotManager,
                    ServiceId::CableGuy,
                    ServiceId::MavlinkCameraManager,
                    ServiceId::Mavlink2rest,
                ],
                ordered_before: vec![
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
            "init_logger publishes to zenoh only; no on-disk log path set in kraken source",
        ),
        zenoh_log_topic: Observed::known(
            "services/kraken/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/kraken/main.py".to_string(),
                line: 27,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId::Kraken,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; ManifestManager singleton and one Kraken process",
        ),
        bounded_context: Asserted::established(
            "extension-lifecycle-management".to_string(),
            "provisional 2.0 domain: Docker-based extension platform — manifests, install, configure, lifecycle",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId::AddCustomManifest,
                "POST /manifest/ registers external extension collection sources",
            ),
            Rationaled::new(
                JourneyId::BrowseExtensionStore,
                "Store tab lists extensions from consolidated manifests",
            ),
            Rationaled::new(
                JourneyId::ConfigureInstalledExtension,
                "Installed tab permissions, logs, restart, and disable routes",
            ),
            Rationaled::new(
                JourneyId::EditExtensionDevVersion,
                "PUT /extension/{identifier}/{tag} switches docker tag for dev versions",
            ),
            Rationaled::new(
                JourneyId::InstallCustomExtension,
                "POST /extension/ registers a custom Docker image via the blue plus flow",
            ),
            Rationaled::new(
                JourneyId::InstallExtension,
                "POST /extension/{identifier}/{tag}/install pulls and enables store extensions",
            ),
            Rationaled::new(
                JourneyId::UninstallExtension,
                "DELETE /extension/{identifier}/{tag} removes an installed extension version",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "SERVICES startup tier; core vehicle operation does not depend on Kraken, though individual extensions may be critical",
        ),
        offline_required: Asserted::established(
            false,
            "manifest fetch, store browse, and image pull require network; installed extensions continue under Docker offline",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; read-write /var/run/docker.sock can start arbitrary containers",
        ),
        dangerous_operations: AssertedSet::established(vec![
            Rationaled::new(
                DangerousOperation::Other("install_arbitrary_docker_image".to_string()),
                "POST /extension/ and install routes pull and register user-supplied Docker images",
            ),
            Rationaled::new(
                DangerousOperation::Other("run_privileged_containers".to_string()),
                "Extension.start creates containers from manifest HostConfig via the Docker API",
            ),
            Rationaled::new(
                DangerousOperation::Upgrade,
                "PUT /extension/{identifier}/{tag} and install update replace running extension images",
            ),
        ]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "installing and running third-party Docker images can affect vehicle networking, storage, and MAVLink consumers",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId::BrowseExtensionStore,
                "GET /manifest/consolidated and manifest-backed store listing",
            ),
            Rationaled::new(
                CapabilityId::ConfigureExtension,
                "PUT /extension/{identifier} edits permissions and user_permissions settings",
            ),
            Rationaled::new(
                CapabilityId::InstallExtension,
                "POST /extension/ and POST /extension/{identifier}/{tag}/install pull and register images",
            ),
            Rationaled::new(
                CapabilityId::ManageExtensionLifecycle,
                "restart/disable routes, starter task, and ContainerManager orchestration",
            ),
            Rationaled::new(
                CapabilityId::ManageManifests,
                "ManifestManager CRUD and POST /manifest/ for external collection sources",
            ),
            Rationaled::new(
                CapabilityId::UninstallExtension,
                "DELETE /extension/{identifier}/{tag} and Extension.uninstall container/image removal",
            ),
        ]),
        authorities: AssertedSet::established(vec![
            Rationaled::new(
                Authority::Other("docker_extension_orchestrator".to_string()),
                "sole BlueOS service that creates, starts, stops, and removes extension-* Docker containers",
            ),
            Rationaled::new(
                Authority::UserdataWriter(PathRef("/root/.config/kraken".to_string())),
                "owns extension and manifest settings under the kraken config directory",
            ),
        ]),
        states: AssertedSet::unknown(
            "no cataloged state machine; per-extension enabled/running is implicit in settings and Docker, not modeled as service states",
        ),
        edges: AssertedSet::established(vec![]),
        resources: AssertedSet::established(vec![
            Rationaled::new(
                Resource {
                    path: PathRef("/var/run/docker.sock".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "primary Docker API client; other processes may share the socket but Kraken is the extension orchestrator",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/kraken".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "service-owned settings tree for extensions and manifests",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/kraken/settings-2.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "SettingsV2 persistence file written by Manager save",
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
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                vec![
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
                "observed ordered_before lists Kraken before remaining SERVICES-tier peers",
            ),
            shutdown: Asserted::established(
                "jobs.stop and extension_log_publisher.shutdown on uvicorn server exit".to_string(),
                "main.py awaits server shutdown then stops JobsManager and Kraken background tasks",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight extension containers and image cache not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST root redirects; starter and cleaner background tasks".to_string(),
            "no dedicated /health route; background tasks and REST availability serve as health signals",
        ),
        is_platform: Asserted::established(
            true,
            "installs and runs third-party Docker containers as BlueOS extensions via harbor/ and extension/extension.py",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 and v2.0 routers exposed under /kraken/; v2 is the active extension API surface",
        ),
        permissions_model: Asserted::established(
            "extension manifest permissions JSON with optional user_permissions override in settings".to_string(),
            "permissions from Docker image labels; PUT /extension/{identifier} persists user_permissions overrides",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "docker_daemon_unavailable".to_string(),
                "init_dead_extensions and kill_dangling_containers abort when ContainerManager cannot list containers",
            ),
            Rationaled::new(
                "manifest_fetch_failure".to_string(),
                "ManifestBackendOffline and ManifestDataFetchFailed block store browse and version resolution",
            ),
            Rationaled::new(
                "image_pull_failure".to_string(),
                "ExtensionPullFailed on install/start when registry is unreachable or image incompatible",
            ),
            Rationaled::new(
                "extension_crash_loop".to_string(),
                "starter task retries with exponential backoff via Extension.start_attempts",
            ),
            Rationaled::new(
                "insufficient_storage".to_string(),
                "ExtensionInsufficientStorage when expanded_size exceeds available disk",
            ),
            Rationaled::new(
                "incompatible_extension".to_string(),
                "IncompatibleExtension when manifest has no compatible image digest for the platform",
            ),
        ]),
        blast_radius: Asserted::established(
            "extension install/configure unavailable; already-running extension containers may persist under Docker; core vehicle services unaffected"
                .to_string(),
            "Kraken outage blocks extension management but not autopilot, MAVLink, or nginx core paths",
        ),
        compatibility_policy: Asserted::unknown(
            "extension image compatibility matrix and API deprecation policy between v1 and v2 not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
