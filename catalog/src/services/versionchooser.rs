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
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

const RUNTIME_CAPTURE: &str = "runtime-captures/versionchooser__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId::Versionchooser,
        state_contracts: GroundedSet::unknown(
            "versionchooser has no service-level state machine (card states Unknown); version selection is implicit in startup.json + Docker image state",
        ),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/version/current", 98.5, 134.9, 179.3, 40),
            runtime_slo(HttpMethod::Get, "/version/available/local", 104.2, 121.5, 140.0, 40),
            runtime_slo(HttpMethod::Get, "/bootstrap/current", 93.4, 113.8, 140.0, 40),
            runtime_slo(HttpMethod::Get, "/docker/accounts", 4.4, 7.9, 8.1, 40),
        ]),
        resource_usage: GroundedSet::known(vec![runtime_resource(
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
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "versionchooser manages the BlueOS core Docker image regardless of flight controller; platform-independent".into(),
                    "runtime captured on Navigator only; RSS ~53.1 MB flat, CPU ~0.82% mean".into(),
                    "version/bootstrap GETs are ~90-105 ms (query the Docker daemon over docker.sock)".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "POST /version/current and POST /bootstrap/current persist to /root/.config/bootstrap/startup.json; docker login writes ~/.docker/config.json; not exercised (system-integrity hazard)",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId::Versionchooser,
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
        id: ServiceId::Versionchooser,
        aliases: ObservedSet::known(vec![
            Evidenced::new(
                "versionchooser".to_string(),
                Evidence {
                    file: "core/start-blueos-core".to_string(),
                    line: 138,
                },
            ),
            Evidenced::new(
                "version-chooser".to_string(),
                Evidence {
                    file: "core/services/versionchooser/main.py".to_string(),
                    line: 12,
                },
            ),
        ]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 138,
            },
        ),
        entrypoint: Observed::known(
            "$BLUEOS_PYTHON_BIN_SECONDARY $SERVICES_PATH/versionchooser/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 138,
            },
        ),
        tmux_name: Observed::known(
            "versionchooser".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 138,
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
                line: 138,
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 138,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/version-chooser/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 220,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(8081),
            Evidence {
                file: "core/services/versionchooser/args.py".to_string(),
                line: 26,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/versionchooser".to_string()),
            Evidence {
                file: "core/services/versionchooser/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/version-chooser/".to_string()),
                    port: PortRef::Literal(8081),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/versionchooser/api/app.py".to_string(),
                    line: 30,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "https://index.docker.io".to_string(),
                },
                Evidence {
                    file: "core/services/versionchooser/utils/dockerhub.py".to_string(),
                    line: 49,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "https://hub.docker.com/".to_string(),
                },
                Evidence {
                    file: "core/services/versionchooser/utils/dockerhub.py".to_string(),
                    line: 50,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "https://auth.docker.io".to_string(),
                },
                Evidence {
                    file: "core/services/versionchooser/utils/dockerhub.py".to_string(),
                    line: 99,
                },
            ),
            Evidenced::new(
                Interface::Settings {
                    path: PathRef("/root/.config/bootstrap/startup.json".to_string()),
                },
                Evidence {
                    file: "core/services/versionchooser/utils/chooser.py".to_string(),
                    line: 16,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/var/run/docker.sock".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/api/v1/routers/version.py".to_string(),
                    line: 23,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.docker/config.json".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/docker_login.py".to_string(),
                    line: 14,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/home/pi/.docker/config.json".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/docker_login.py".to_string(),
                    line: 13,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/version-chooser/log".to_string()],
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
                    path: PathRef("/var/run/docker.sock".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/api/v1/routers/version.py".to_string(),
                    line: 23,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/bootstrap/startup.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/utils/chooser.py".to_string(),
                    line: 16,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.docker/config.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/docker_login.py".to_string(),
                    line: 14,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/home/pi/.docker/config.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/docker_login.py".to_string(),
                    line: 13,
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
                    ServiceId::Beacon,
                    ServiceId::Bridget,
                    ServiceId::Commander,
                    ServiceId::NmeaInjector,
                    ServiceId::Helper,
                    ServiceId::Iperf3,
                    ServiceId::Linux2rest,
                    ServiceId::Filebrowser,
                ],
                ordered_before: vec![
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
            "init_logger publishes to zenoh only; no on-disk log path set in versionchooser source",
        ),
        zenoh_log_topic: Observed::known(
            "services/version-chooser/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/versionchooser/main.py".to_string(),
                line: 21,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId::Versionchooser,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one VersionChooser process",
        ),
        bounded_context: Asserted::established(
            "blueos-version-management".to_string(),
            "provisional 2.0 domain: core OS image pull, switch, delete, bootstrap alignment, and registry auth",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId("update_blueos_version".into()),
                "simplified update flow pulls and switches to a newer stable or beta core release",
            ),
            Rationaled::new(
                JourneyId("switch_local_blueos_version".into()),
                "pirate-mode local cards apply a previously installed core image without re-downloading",
            ),
            Rationaled::new(
                JourneyId("pull_blueos_version_without_switch".into()),
                "remote Versions section fetches a tag to local storage before apply",
            ),
            Rationaled::new(
                JourneyId("delete_local_blueos_version".into()),
                "local version cards delete non-current core images when enough versions remain",
            ),
            Rationaled::new(
                JourneyId("docker_registry_login".into()),
                "Docker Login dialog authenticates the daemon and lists connected accounts",
            ),
            Rationaled::new(
                JourneyId("update_bootstrap_image".into()),
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
        dangerous_operations: AssertedSet::established(vec![
            Rationaled::new(
                DangerousOperation::Upgrade,
                "POST /version/current and update journeys switch the running blueos-core image and restart into it",
            ),
            Rationaled::new(
                DangerousOperation::Other("delete_core_image".to_string()),
                "DELETE /version/delete removes a locally stored core image; recovery may require re-pull or factory restore",
            ),
            Rationaled::new(
                DangerousOperation::Other("pull_untrusted_image".to_string()),
                "POST /version/pull accepts arbitrary repository and tag, including custom registries after docker login",
            ),
        ]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "switching core OS images, deleting local versions, and pulling third-party images can brick or compromise the vehicle computer",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("update_blueos_version".to_string()),
                "simplified Version Chooser pulls and applies a newer stable or beta core release",
            ),
            Rationaled::new(
                CapabilityId("switch_blueos_version".to_string()),
                "POST /version/current applies a previously installed local core image",
            ),
            Rationaled::new(
                CapabilityId("pull_blueos_version".to_string()),
                "POST /version/pull streams docker pull for a remote repository tag",
            ),
            Rationaled::new(
                CapabilityId("delete_local_blueos_version".to_string()),
                "DELETE /version/delete removes a non-current local core image",
            ),
            Rationaled::new(
                CapabilityId("docker_registry_login".to_string()),
                "POST /docker/login and GET /docker/accounts manage registry credentials",
            ),
            Rationaled::new(
                CapabilityId("update_bootstrap_image".to_string()),
                "POST /bootstrap/current switches blueos-bootstrap to a pulled tag",
            ),
            Rationaled::new(
                CapabilityId("list_local_blueos_versions".to_string()),
                "GET /version/available/local lists locally installed core images",
            ),
            Rationaled::new(
                CapabilityId("list_remote_blueos_versions".to_string()),
                "GET /version/available/{repository}/{image} lists remote tags for a repository",
            ),
            Rationaled::new(
                CapabilityId("get_current_blueos_version".to_string()),
                "GET /version/current returns the configured running core image and metadata",
            ),
            Rationaled::new(
                CapabilityId("get_current_bootstrap_version".to_string()),
                "GET /bootstrap/current returns the running blueos-bootstrap container image",
            ),
            Rationaled::new(
                CapabilityId("list_docker_accounts".to_string()),
                "GET /docker/accounts lists Docker registry accounts logged in on the host",
            ),
        ]),
        authorities: AssertedSet::established(vec![
            Rationaled::new(
                Authority::Other("blueos_version_controller".to_string()),
                "sole writer of startup.json core image selection; bootstrap and the next boot read this path",
            ),
            Rationaled::new(
                Authority::Other("bootstrap_image_controller".to_string()),
                "sole service that stops, recreates, and starts the blueos-bootstrap container",
            ),
            Rationaled::new(
                Authority::UserdataWriter(PathRef("/root/.config/bootstrap/startup.json".to_string())),
                "owns the bootstrap startup config that records which core image tag boots on next restart",
            ),
        ]),
        states: AssertedSet::unknown(
            "no cataloged state machine; current core and bootstrap versions are implicit in startup.json and Docker",
        ),
        edges: AssertedSet::established(vec![]),
        resources: AssertedSet::established(vec![
            Rationaled::new(
                Resource {
                    path: PathRef("/var/run/docker.sock".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "primary Docker API client for core and bootstrap image pull, switch, and container lifecycle",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/bootstrap/startup.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "bootstrap startup config selecting the core image tag for the next boot",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.docker/config.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "root-user Docker registry credentials written by docker login",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/home/pi/.docker/config.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "pi-user Docker registry credentials written when login targets the default user",
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
                vec![
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
                "uvicorn server exit logs Version Chooser service stopped".to_string(),
                "main.py finally block after server.serve returns",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight pull streams and bootstrap container handoff not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns service name".to_string(),
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
                .to_string(),
            "advanced switch, pull, delete, and docker login journeys require pirate mode in VersionChooser.vue",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "docker_daemon_unavailable".to_string(),
                "aiodocker client operations fail when /var/run/docker.sock is unreachable",
            ),
            Rationaled::new(
                "image_pull_failure".to_string(),
                "pull_version streams errors when the registry is unreachable or the tag is missing",
            ),
            Rationaled::new(
                "invalid_startup_json".to_string(),
                "get_current_image_and_tag returns None when startup.json lacks core image or tag keys",
            ),
            Rationaled::new(
                "registry_auth_failure".to_string(),
                "private or rate-limited pulls fail without prior POST /docker/login credentials",
            ),
            Rationaled::new(
                "bootstrap_switch_failure".to_string(),
                "set_bootstrap_version aborts when the target bootstrap image is not present locally",
            ),
            Rationaled::new(
                "switch_to_missing_image".to_string(),
                "set_version returns 412 when the requested core image tag is not installed locally",
            ),
        ]),
        blast_radius: Asserted::established(
            "wrong core or bootstrap image selection can brick BlueOS on next restart; already-running core may persist until switch"
                .to_string(),
            "versionchooser outage blocks OS updates and rollbacks but does not directly stop autopilot or MAVLink",
        ),
        compatibility_policy: Asserted::unknown(
            "core image tag compatibility matrix and bootstrap/core pairing policy not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
