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
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Versionchooser,
        state_contracts: GroundedSet::unknown(
            "versionchooser has no service-level state machine (card states Unknown); version selection is implicit in startup.json + Docker image state",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/version/current", 98.5, 134.9, 179.3, 40),
            runtime_slo(HttpMethod::Get, "/version/available/local", 104.2, 121.5, 140.0, 40),
            runtime_slo(HttpMethod::Get, "/bootstrap/current", 93.4, 113.8, 140.0, 40),
            runtime_slo(HttpMethod::Get, "/docker/accounts", 4.4, 7.9, 8.1, 40),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
            "running_baseline",
            Distribution {
                mean: 0.82,
                median: 0.86,
                p95: 2.84,
                min: 0.00,
                max: 3.17,
                sd: 0.98,
            },
            Distribution {
                mean: 53.1,
                median: 53.1,
                p95: 53.1,
                min: 53.1,
                max: 53.1,
                sd: 0.0,
            },
            60,
        )]),
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "versionchooser manages the BlueOS core Docker image regardless of flight controller; platform-independent",
                    "runtime captured on Navigator only; RSS ~53.1 MB flat, CPU ~0.82% mean",
                    "version/bootstrap GETs are ~90-105 ms (query the Docker daemon over docker.sock)",
                ],
            },
            runtime_prov("runtime-captures/versionchooser__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "POST /version/current and POST /bootstrap/current persist to /root/.config/bootstrap/startup.json; docker login writes ~/.docker/config.json; not exercised (system-integrity hazard)",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::Versionchooser,
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
            "runtime-captures/versionchooser__pi4_navigator_master.json#slo_running_baseline",
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
        runtime_prov("runtime-captures/versionchooser__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts = ObservedFacts {
    id: ServiceId::Versionchooser,
    aliases: ObservedSet::known(&[
        Evidenced::new(
            "versionchooser",
            Evidence {
                file: "core/start-blueos-core",
                line: 138,
            },
        ),
        Evidenced::new(
            "version-chooser",
            Evidence {
                file: "core/services/versionchooser/main.py",
                line: 12,
            },
        ),
    ]),
    kind: Observed::known(
        ServiceKind::PythonService,
        Evidence {
            file: "core/start-blueos-core",
            line: 138,
        },
    ),
    entrypoint: Observed::known(
        "$BLUEOS_PYTHON_BIN_SECONDARY $SERVICES_PATH/versionchooser/main.py",
        Evidence {
            file: "core/start-blueos-core",
            line: 138,
        },
    ),
    tmux_name: Observed::known(
        "versionchooser",
        Evidence {
            file: "core/start-blueos-core",
            line: 138,
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
            line: 138,
        },
    ),
    nice: Observed::unknown("no nice prefix in start tuple"),
    run_as: Observed::known(
        "root",
        Evidence {
            file: "core/start-blueos-core",
            line: 138,
        },
    ),
    nginx_prefixes: ObservedSet::known(&[Evidenced::new(
        PathRef("/version-chooser/"),
        Evidence {
            file: "core/tools/nginx/nginx.conf",
            line: 220,
        },
    )]),
    listen: ObservedSet::known(&[Evidenced::new(
        PortRef::Literal(8081),
        Evidence {
            file: "core/services/versionchooser/args.py",
            line: 26,
        },
    )]),
    git_path: Observed::known(
        PathRef("core/services/versionchooser"),
        Evidence {
            file: "core/services/versionchooser/main.py",
            line: 1,
        },
    ),
    interfaces: ObservedSet::known(&[
        Evidenced::new(
            Interface::Rest {
                path_prefix: PathRef("/version-chooser/"),
                port: PortRef::Literal(8081),
                versions: &["v1.0"],
            },
            Evidence {
                file: "core/services/versionchooser/api/app.py",
                line: 30,
            },
        ),
        Evidenced::new(
            Interface::OutboundHttp {
                url: "https://index.docker.io",
            },
            Evidence {
                file: "core/services/versionchooser/utils/dockerhub.py",
                line: 49,
            },
        ),
        Evidenced::new(
            Interface::OutboundHttp {
                url: "https://hub.docker.com/",
            },
            Evidence {
                file: "core/services/versionchooser/utils/dockerhub.py",
                line: 50,
            },
        ),
        Evidenced::new(
            Interface::OutboundHttp {
                url: "https://auth.docker.io",
            },
            Evidence {
                file: "core/services/versionchooser/utils/dockerhub.py",
                line: 99,
            },
        ),
        Evidenced::new(
            Interface::Settings {
                path: PathRef("/root/.config/bootstrap/startup.json"),
            },
            Evidence {
                file: "core/services/versionchooser/utils/chooser.py",
                line: 16,
            },
        ),
        Evidenced::new(
            Interface::File {
                path: PathRef("/var/run/docker.sock"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/services/versionchooser/api/v1/routers/version.py",
                line: 23,
            },
        ),
        Evidenced::new(
            Interface::File {
                path: PathRef("/root/.docker/config.json"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/services/versionchooser/docker_login.py",
                line: 14,
            },
        ),
        Evidenced::new(
            Interface::File {
                path: PathRef("/home/pi/.docker/config.json"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/services/versionchooser/docker_login.py",
                line: 13,
            },
        ),
        Evidenced::new(
            Interface::Zenoh {
                topics_produced: &["services/version-chooser/log"],
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
                path: PathRef("/var/run/docker.sock"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/versionchooser/api/v1/routers/version.py",
                line: 23,
            },
        ),
        Evidenced::new(
            Resource {
                path: PathRef("/root/.config/bootstrap/startup.json"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/versionchooser/utils/chooser.py",
                line: 16,
            },
        ),
        Evidenced::new(
            Resource {
                path: PathRef("/root/.docker/config.json"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/versionchooser/docker_login.py",
                line: 14,
            },
        ),
        Evidenced::new(
            Resource {
                path: PathRef("/home/pi/.docker/config.json"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/versionchooser/docker_login.py",
                line: 13,
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
                ServiceId::NmeaInjector,
                ServiceId::Helper,
                ServiceId::Iperf3,
                ServiceId::Linux2rest,
                ServiceId::Filebrowser,
            ],
            ordered_before: &[
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
        "init_logger publishes to zenoh only; no on-disk log path set in versionchooser source",
    ),
    zenoh_log_topic: Observed::known(
        "services/version-chooser/log",
        Evidence {
            file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
            line: 78,
        },
    ),
    sentry: Observed::known(
        true,
        Evidence {
            file: "core/services/versionchooser/main.py",
            line: 21,
        },
    ),
    openapi_refs: ObservedSet::unknown("not yet extracted"),
};

pub const SERVICE_DEFINITION: ServiceDefinition =
    ServiceDefinition {
        id: ServiceId::Versionchooser,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one VersionChooser process",
        ),
        bounded_context: Asserted::established(
            "blueos-version-management",
            "provisional 2.0 domain: core OS image pull, switch, delete, bootstrap alignment, and registry auth",
        ),
        journey_refs: AssertedSet::established(&[
            Rationaled::new(
                JourneyId::UpdateBlueosVersion,
                "simplified update flow pulls and switches to a newer stable or beta core release",
            ),
            Rationaled::new(
                JourneyId::SwitchLocalBlueosVersion,
                "pirate-mode local cards apply a previously installed core image without re-downloading",
            ),
            Rationaled::new(
                JourneyId::PullBlueosVersionWithoutSwitch,
                "remote Versions section fetches a tag to local storage before apply",
            ),
            Rationaled::new(
                JourneyId::DeleteLocalBlueosVersion,
                "local version cards delete non-current core images when enough versions remain",
            ),
            Rationaled::new(
                JourneyId::DockerRegistryLogin,
                "Docker Login dialog authenticates the daemon and lists connected accounts",
            ),
            Rationaled::new(
                JourneyId::UpdateBootstrapImage,
                "current-version card updates blueos-bootstrap to match the running core release",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Important,
            "not vehicle-flight-critical, but sole updater of the core OS image; a bad switch or delete can brick or downgrade BlueOS",
        ),
        offline_required: Asserted::established(
            false,
            "primary update and pull journeys require Docker Hub or registry network access; local switch/delete alone do not define the service",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; read-write docker.sock can pull, switch, and restart the core container",
        ),
        dangerous_operations: AssertedSet::established(&[
            Rationaled::new(
                DangerousOperation::Upgrade,
                "POST /version/current and update journeys switch the running blueos-core image and restart into it",
            ),
            Rationaled::new(
                DangerousOperation::Other("delete_core_image"),
                "DELETE /version/delete removes a locally stored core image; recovery may require re-pull or factory restore",
            ),
            Rationaled::new(
                DangerousOperation::Other("pull_untrusted_image"),
                "POST /version/pull accepts arbitrary repository and tag, including custom registries after docker login",
            ),
        ]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "switching core OS images, deleting local versions, and pulling third-party images can brick or compromise the vehicle computer",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::UpdateBlueosVersion,
                "simplified Version Chooser pulls and applies a newer stable or beta core release",
            ),
            Rationaled::new(
                CapabilityId::SwitchBlueosVersion,
                "POST /version/current applies a previously installed local core image",
            ),
            Rationaled::new(
                CapabilityId::PullBlueosVersion,
                "POST /version/pull streams docker pull for a remote repository tag",
            ),
            Rationaled::new(
                CapabilityId::DeleteLocalBlueosVersion,
                "DELETE /version/delete removes a non-current local core image",
            ),
            Rationaled::new(
                CapabilityId::DockerRegistryLogin,
                "POST /docker/login and GET /docker/accounts manage registry credentials",
            ),
            Rationaled::new(
                CapabilityId::UpdateBootstrapImage,
                "POST /bootstrap/current switches blueos-bootstrap to a pulled tag",
            ),
            Rationaled::new(
                CapabilityId::ListLocalBlueosVersions,
                "GET /version/available/local lists locally installed core images",
            ),
            Rationaled::new(
                CapabilityId::ListRemoteBlueosVersions,
                "GET /version/available/{repository}/{image} lists remote tags for a repository",
            ),
            Rationaled::new(
                CapabilityId::GetCurrentBlueosVersion,
                "GET /version/current returns the configured running core image and metadata",
            ),
            Rationaled::new(
                CapabilityId::GetCurrentBootstrapVersion,
                "GET /bootstrap/current returns the running blueos-bootstrap container image",
            ),
            Rationaled::new(
                CapabilityId::ListDockerAccounts,
                "GET /docker/accounts lists Docker registry accounts logged in on the host",
            ),
        ]),
        authorities: AssertedSet::established(&[
            Rationaled::new(
                Authority::Other("blueos_version_controller"),
                "sole writer of startup.json core image selection; bootstrap and the next boot read this path",
            ),
            Rationaled::new(
                Authority::Other("bootstrap_image_controller"),
                "sole service that stops, recreates, and starts the blueos-bootstrap container",
            ),
            Rationaled::new(
                Authority::UserdataWriter(PathRef("/root/.config/bootstrap/startup.json")),
                "owns the bootstrap startup config that records which core image tag boots on next restart",
            ),
        ]),
        states: AssertedSet::unknown(
            "no cataloged state machine; current core and bootstrap versions are implicit in startup.json and Docker",
        ),
        edges: AssertedSet::established(&[]),
        resources: AssertedSet::established(&[
            Rationaled::new(
                Resource {
                    path: PathRef("/var/run/docker.sock"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "primary Docker API client for core and bootstrap image pull, switch, and container lifecycle",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/bootstrap/startup.json"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "bootstrap startup config selecting the core image tag for the next boot",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.docker/config.json"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "root-user Docker registry credentials written by docker login",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/home/pi/.docker/config.json"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "pi-user Docker registry credentials written when login targets the default user",
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
                    ServiceId::NmeaInjector,
                    ServiceId::Helper,
                    ServiceId::Iperf3,
                    ServiceId::Linux2rest,
                    ServiceId::Filebrowser,
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                &[
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
                "observed ordered_before lists versionchooser before remaining SERVICES-tier peers",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit logs Version Chooser service stopped",
                "main.py finally block after server.serve returns",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight pull streams and bootstrap container handoff not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns service name",
            "no dedicated /health route; uvicorn availability serves as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "manages the platform core image but does not install or host third-party extension containers like Kraken",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 router exposed under /version-chooser/ via VersionedFastAPI",
        ),
        permissions_model: Asserted::established(
            "pirate mode gates advanced version UI in the frontend; REST routes have no separate permissions manifest"
                ,
            "advanced switch, pull, delete, and docker login journeys require pirate mode in VersionChooser.vue",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "docker_daemon_unavailable",
                "aiodocker client operations fail when /var/run/docker.sock is unreachable",
            ),
            Rationaled::new(
                "image_pull_failure",
                "pull_version streams errors when the registry is unreachable or the tag is missing",
            ),
            Rationaled::new(
                "invalid_startup_json",
                "get_current_image_and_tag returns None when startup.json lacks core image or tag keys",
            ),
            Rationaled::new(
                "registry_auth_failure",
                "private or rate-limited pulls fail without prior POST /docker/login credentials",
            ),
            Rationaled::new(
                "bootstrap_switch_failure",
                "set_bootstrap_version aborts when the target bootstrap image is not present locally",
            ),
            Rationaled::new(
                "switch_to_missing_image",
                "set_version returns 412 when the requested core image tag is not installed locally",
            ),
        ]),
        blast_radius: Asserted::established(
            "wrong core or bootstrap image selection can brick BlueOS on next restart; already-running core may persist until switch"
                ,
            "versionchooser outage blocks OS updates and rollbacks but does not directly stop autopilot or MAVLink",
        ),
        compatibility_policy: Asserted::unknown(
            "core image tag compatibility matrix and bootstrap/core pairing policy not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Versionchooser,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
