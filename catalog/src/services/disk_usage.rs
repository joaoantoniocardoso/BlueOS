use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, PortKind};
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
use crate::service::{Service, ServiceJudgment};
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::DiskUsage,
        state_contracts: GroundedSet::unknown(
            "disk_usage has no service-level state machine (service_definition states: Unknown); \
             usage inspection and speed tests are stateless request handlers",
        ),
        slo_baselines: GroundedSet::known(&[runtime_slo(
            HttpMethod::Get,
            "/disk/usage?depth=1",
            2722.3,
            2766.0,
            2766.0,
            8,
        )]),
        resource_usage: GroundedSet::known(&[runtime_resource(
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
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "disk_usage behavior is platform-independent (local du/disktest on root filesystem); not captured across boards",
                    "runtime captured on Navigator only; RSS ~35.0 MB, CPU ~0.29% mean",
                ],
            },
            runtime_prov("runtime-captures/disk_usage__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown("service has no settings persistence"),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::DiskUsage,
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
        runtime_prov("runtime-captures/disk_usage__pi4_navigator_master.json#slo_running_baseline"),
    )
}

const fn flat_rss(mb: f64) -> Distribution {
    Distribution {
        mean: mb,
        median: mb,
        p95: mb,
        min: mb,
        max: mb,
        sd: 0.0,
    }
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
        runtime_prov("runtime-captures/disk_usage__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts = ObservedFacts {
    id: ServiceId::DiskUsage,
    aliases: ObservedSet::known(&[Evidenced::new(
        "disk-usage",
        Evidence {
            file: "core/services/disk_usage/main.py",
            line: 26,
            anchor: "SERVICE_NAME = \"disk-usage\"",
        },
    )]),
    kind: Observed::known(
        ServiceKind::PythonService,
        Evidence {
            file: "core/start-blueos-core",
            line: 147,
            anchor: "'disk_usage',250,0,0,0,\"$SERVICES_PATH/disk_usage/main.py\"",
        },
    ),
    entrypoint: Observed::known(
        "$SERVICES_PATH/disk_usage/main.py",
        Evidence {
            file: "core/start-blueos-core",
            line: 147,
            anchor: "'disk_usage',250,0,0,0,\"$SERVICES_PATH/disk_usage/main.py\"",
        },
    ),
    tmux_name: Observed::known(
        "disk_usage",
        Evidence {
            file: "core/start-blueos-core",
            line: 147,
            anchor: "'disk_usage',250,0,0,0,\"$SERVICES_PATH/disk_usage/main.py\"",
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
            line: 147,
            anchor: "'disk_usage',250,0,0,0,\"$SERVICES_PATH/disk_usage/main.py\"",
        },
    ),
    nice: Observed::unknown("no nice prefix in start tuple"),
    run_as: Observed::known(
        "root",
        Evidence {
            file: "core/start-blueos-core",
            line: 147,
            anchor: "'disk_usage',250,0,0,0,\"$SERVICES_PATH/disk_usage/main.py\"",
        },
    ),
    nginx_prefixes: ObservedSet::known(&[Evidenced::new(
        PathRef("/disk-usage/"),
        Evidence {
            file: "core/tools/nginx/nginx.conf",
            line: 139,
            anchor: "location /disk-usage/ {",
        },
    )]),
    listen: ObservedSet::known(&[Evidenced::new(
        PortRef::Literal(9151),
        Evidence {
            file: "core/services/disk_usage/main.py",
            line: 28,
            anchor: "PORT = 9151",
        },
    )]),
    git_path: Observed::known(
        PathRef("core/services/disk_usage"),
        Evidence {
            file: "core/services/disk_usage/main.py",
            line: 1,
            anchor: "#! /usr/bin/env python3",
        },
    ),
    interfaces: ObservedSet::known(&[
        Evidenced::new(
            PortKind::Rest {
                path_prefix: PathRef("/disk-usage/"),
                port: PortRef::Literal(9151),
                versions: &["v1.0"],
            },
            Evidence {
                file: "core/services/disk_usage/main.py",
                line: 469,
                anchor: "app = VersionedFastAPI(",
            },
        ),
        Evidenced::new(
            PortKind::File {
                path: PathRef("/"),
                mode: FileAccessMode::ReadWrite,
            },
            Evidence {
                file: "core/services/disk_usage/main.py",
                line: 27,
                anchor: "FILESYSTEM_ROOT = Path(\"/\")",
            },
        ),
        Evidenced::new(
            PortKind::Subprocess { command: "du" },
            Evidence {
                file: "core/services/disk_usage/main.py",
                line: 211,
                anchor: "args = [\"du\", \"-b\", str(path)]",
            },
        ),
        Evidenced::new(
            PortKind::Subprocess {
                command: "disktest",
            },
            Evidence {
                file: "core/services/disk_usage/main.py",
                line: 319,
                anchor: "disktest_binary = \"disktest\"",
            },
        ),
        Evidenced::new(
            PortKind::Zenoh {
                topics_produced: &["services/disk-usage/log"],
                topics_consumed: &[],
            },
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                line: 78,
                anchor: "topic = f\"services/{service_name}/log\"",
            },
        ),
    ]),
    resources: ObservedSet::known(&[Evidenced::new(
        Resource {
            path: PathRef("/"),
            ownership: ResourceOwnership::SharedWrite,
        },
        Evidence {
            file: "core/services/disk_usage/main.py",
            line: 281,
            anchor: "shutil.rmtree(resolved_path)",
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
            ],
            ordered_before: &[ServiceId::Customization],
        },
        Evidence {
            file: "core/start-blueos-core",
            line: 326,
            anchor: "for TUPLE in \"${SERVICES[@]}\"; do",
        },
    ),
    logs_path: Observed::unknown(
        "init_logger publishes to zenoh only; no on-disk log path set in disk_usage source",
    ),
    zenoh_log_topic: Observed::known(
        "services/disk-usage/log",
        Evidence {
            file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
            line: 78,
            anchor: "topic = f\"services/{service_name}/log\"",
        },
    ),
    sentry: Observed::known(
        true,
        Evidence {
            file: "core/services/disk_usage/main.py",
            line: 484,
            anchor: "await init_sentry_async(SERVICE_NAME)",
        },
    ),
    openapi_refs: ObservedSet::unknown("not yet extracted"),
};

pub const SERVICE_DEFINITION: ServiceJudgment =
    ServiceJudgment {
        id: ServiceId::DiskUsage,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one disk_usage process",
        ),
        bounded_context: Asserted::established(
            "storage-diagnostics-and-maintenance",
            "provisional 2.0 domain: local filesystem usage inspection, path deletion, and disk benchmarks",
        ),
        journey_refs: AssertedSet::established(&[
            Rationaled::new(
                JourneyId::InspectDiskUsage,
                "Disk page loads du-backed usage tree and drills into subdirectories",
            ),
            Rationaled::new(
                JourneyId::FreeDiskSpace,
                "operator selects paths and DELETE /disk/paths/{target_path} reclaims storage",
            ),
            Rationaled::new(
                JourneyId::RunSingleDiskSpeedTest,
                "Speed Test tab runs GET /disk/speed disktest benchmark at one size",
            ),
            Rationaled::new(
                JourneyId::RunMultiSizeDiskSpeedTest,
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
        dangerous_operations: AssertedSet::established(&[Rationaled::new(
            DangerousOperation::Other("delete_filesystem_paths"),
            "DELETE /disk/paths/{target_path} recursively removes files and directories via shutil",
        )]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "deleting filesystem paths is irreversible and can remove operator or system data",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::InspectDiskUsage,
                "GET /disk/usage returns a du-backed usage tree for the requested path",
            ),
            Rationaled::new(
                CapabilityId::NavigateDiskUsage,
                "repeated GET /disk/usage with a subdirectory path drills into the tree",
            ),
            Rationaled::new(
                CapabilityId::DeleteDiskPaths,
                "DELETE /disk/paths/{target_path} removes selected files or folders recursively",
            ),
            Rationaled::new(
                CapabilityId::RunDiskSpeedTest,
                "GET /disk/speed runs one disktest write-and-verify pass at the requested size",
            ),
            Rationaled::new(
                CapabilityId::RunMultiSizeDiskSpeedTest,
                "GET /disk/speed/stream yields NDJSON benchmark points for each test size",
            ),
        ]),
        authorities: AssertedSet::established(&[]),
        states: AssertedSet::unknown(
            "no cataloged state machine; disk usage and speed tests are stateless request handlers",
        ),
        edges: AssertedSet::established(&[]),
        resources: AssertedSet::established(&[Rationaled::new(
            Resource {
                path: PathRef("/"),
                ownership: ResourceOwnership::SharedWrite,
            },
            "observed SharedWrite on / for usage inspection, path deletion, and temp benchmark files",
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
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                &[ServiceId::Customization],
                "observed ordered_before lists disk_usage before customization",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit logs Disk Usage service stopped",
                "main.py finally block after server.serve returns",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight disk operations and temp benchmark files not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns service name",
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
            "protected system path roots block DELETE; no separate permissions manifest",
            "is_protected_target refuses deletion under /bin, /etc, /lib, and other core roots",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "du_subprocess_failure",
                "collect_disk_usage logs non-zero du return codes but may return partial trees",
            ),
            Rationaled::new(
                "disktest_binary_missing",
                "GET /disk/speed returns 503 when disktest is not on PATH",
            ),
            Rationaled::new(
                "insufficient_storage_for_benchmark",
                "run_single_speed_test returns 507 when temp dir lacks space for the requested test size",
            ),
            Rationaled::new(
                "protected_path_deletion_refused",
                "DELETE /disk/paths returns 400 for paths under protected system roots",
            ),
            Rationaled::new(
                "invalid_or_missing_path",
                "resolve_requested_path returns 404 or 400 for missing paths or paths outside /",
            ),
        ]),
        blast_radius: Asserted::established(
            "disk usage inspection, deletion, and benchmarks unavailable; core vehicle services unaffected"
                ,
            "disk_usage outage blocks storage maintenance UI but not autopilot, MAVLink, or nginx core paths",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and disktest binary version coupling not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::DiskUsage,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
