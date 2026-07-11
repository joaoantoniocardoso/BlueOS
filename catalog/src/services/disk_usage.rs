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
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline};
use crate::service::ServiceDefinition;
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

const RUNTIME_CAPTURE: &str = "runtime-captures/disk_usage__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId("disk_usage".into()),
        state_contracts: GroundedSet::unknown(
            "disk_usage has no service-level state machine (service_definition states: Unknown); \
             usage inspection and speed tests are stateless request handlers",
        ),
        slo_baselines: GroundedSet::known(vec![runtime_slo(
            HttpMethod::Get,
            "/disk/usage?depth=1",
            2722.3,
            2766.0,
            2766.0,
            8,
        )]),
        resource_usage: GroundedSet::known(vec![runtime_resource(
            "running_baseline",
            Distribution {
                mean: 0.29,
                median: 0.00,
                p95: 1.00,
                min: 0.00,
                max: 1.03,
                sd: 0.44,
            },
            flat_rss(35.0),
            60,
        )]),
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "disk_usage behavior is platform-independent (local du/disktest on root filesystem); not captured across boards".into(),
                    "runtime captured on Navigator only; RSS ~35.0 MB, CPU ~0.29% mean".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown("service has no settings persistence"),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId("disk_usage".into()),
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

pub fn observed_facts() -> ObservedFacts {
    ObservedFacts {
        id: ServiceId("disk_usage".to_string()),
        aliases: ObservedSet::known(vec![Evidenced::new(
            "disk-usage".to_string(),
            Evidence {
                file: "core/services/disk_usage/main.py".to_string(),
                line: 26,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 147,
            },
        ),
        entrypoint: Observed::known(
            "$SERVICES_PATH/disk_usage/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 147,
            },
        ),
        tmux_name: Observed::known(
            "disk_usage".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 147,
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
                line: 147,
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 147,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/disk-usage/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 139,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(9151),
            Evidence {
                file: "core/services/disk_usage/main.py".to_string(),
                line: 28,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/disk_usage".to_string()),
            Evidence {
                file: "core/services/disk_usage/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/disk-usage/".to_string()),
                    port: PortRef::Literal(9151),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/disk_usage/main.py".to_string(),
                    line: 469,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/disk_usage/main.py".to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "du".to_string(),
                },
                Evidence {
                    file: "core/services/disk_usage/main.py".to_string(),
                    line: 211,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "disktest".to_string(),
                },
                Evidence {
                    file: "core/services/disk_usage/main.py".to_string(),
                    line: 319,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/disk-usage/log".to_string()],
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
                path: PathRef("/".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/disk_usage/main.py".to_string(),
                line: 281,
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
                    ServiceId("bridget".to_string()),
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
                ],
                ordered_before: vec![ServiceId("customization".to_string())],
            },
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 326,
            },
        ),
        logs_path: Observed::unknown(
            "init_logger publishes to zenoh only; no on-disk log path set in disk_usage source",
        ),
        zenoh_log_topic: Observed::known(
            "services/disk-usage/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/disk_usage/main.py".to_string(),
                line: 484,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("disk_usage".to_string()),
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one disk_usage process",
        ),
        bounded_context: Asserted::established(
            "storage-diagnostics-and-maintenance".to_string(),
            "provisional 2.0 domain: local filesystem usage inspection, path deletion, and disk benchmarks",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId("inspect_disk_usage".into()),
                "Disk page loads du-backed usage tree and drills into subdirectories",
            ),
            Rationaled::new(
                JourneyId("free_disk_space".into()),
                "operator selects paths and DELETE /disk/paths/{target_path} reclaims storage",
            ),
            Rationaled::new(
                JourneyId("run_single_disk_speed_test".into()),
                "Speed Test tab runs GET /disk/speed disktest benchmark at one size",
            ),
            Rationaled::new(
                JourneyId("run_multi_size_disk_speed_test".into()),
                "GET /disk/speed/stream streams NDJSON points across progressive benchmark sizes",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "SERVICES startup tier; core vehicle operation does not depend on disk usage inspection or cleanup",
        ),
        offline_required: Asserted::established(
            true,
            "du, shutil, and disktest operate on the local filesystem without network access",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; DELETE /disk/paths can remove files anywhere under /",
        ),
        dangerous_operations: AssertedSet::established(vec![Rationaled::new(
            DangerousOperation::Other("delete_filesystem_paths".to_string()),
            "DELETE /disk/paths/{target_path} recursively removes files and directories via shutil",
        )]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "deleting filesystem paths is irreversible and can remove operator or system data",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("inspect_disk_usage".to_string()),
                "GET /disk/usage returns a du-backed usage tree for the requested path",
            ),
            Rationaled::new(
                CapabilityId("navigate_disk_usage".to_string()),
                "repeated GET /disk/usage with a subdirectory path drills into the tree",
            ),
            Rationaled::new(
                CapabilityId("delete_disk_paths".to_string()),
                "DELETE /disk/paths/{target_path} removes selected files or folders recursively",
            ),
            Rationaled::new(
                CapabilityId("run_disk_speed_test".to_string()),
                "GET /disk/speed runs one disktest write-and-verify pass at the requested size",
            ),
            Rationaled::new(
                CapabilityId("run_multi_size_disk_speed_test".to_string()),
                "GET /disk/speed/stream yields NDJSON benchmark points for each test size",
            ),
        ]),
        authorities: AssertedSet::established(vec![]),
        states: AssertedSet::unknown(
            "no cataloged state machine; disk usage and speed tests are stateless request handlers",
        ),
        edges: AssertedSet::unknown(
            "no outbound coupling to other catalog services; du and disktest are local subprocesses only",
        ),
        resources: AssertedSet::established(vec![Rationaled::new(
            Resource {
                path: PathRef("/".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            "observed SharedWrite on / for usage inspection, path deletion, and temp benchmark files",
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
                    ServiceId("bridget".to_string()),
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
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                vec![ServiceId("customization".to_string())],
                "observed ordered_before lists disk_usage before customization",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit logs Disk Usage service stopped".to_string(),
                "main.py finally block after server.serve returns",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight disk operations and temp benchmark files not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns service name".to_string(),
            "no dedicated /health route; uvicorn availability serves as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "storage diagnostics utility; does not install or host third-party extensions",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 router exposed under /disk-usage/ via VersionedFastAPI",
        ),
        permissions_model: Asserted::established(
            "protected system path roots block DELETE; no separate permissions manifest".to_string(),
            "is_protected_target refuses deletion under /bin, /etc, /lib, and other core roots",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "du_subprocess_failure".to_string(),
                "collect_disk_usage logs non-zero du return codes but may return partial trees",
            ),
            Rationaled::new(
                "disktest_binary_missing".to_string(),
                "GET /disk/speed returns 503 when disktest is not on PATH",
            ),
            Rationaled::new(
                "insufficient_storage_for_benchmark".to_string(),
                "run_single_speed_test returns 507 when temp dir lacks space for the requested test size",
            ),
            Rationaled::new(
                "protected_path_deletion_refused".to_string(),
                "DELETE /disk/paths returns 400 for paths under protected system roots",
            ),
            Rationaled::new(
                "invalid_or_missing_path".to_string(),
                "resolve_requested_path returns 404 or 400 for missing paths or paths outside /",
            ),
        ]),
        blast_radius: Asserted::established(
            "disk usage inspection, deletion, and benchmarks unavailable; core vehicle services unaffected"
                .to_string(),
            "disk_usage outage blocks storage maintenance UI but not autopilot, MAVLink, or nginx core paths",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and disktest binary version coupling not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
