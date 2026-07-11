use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::Interface;
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

const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e; RepoDigest sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a), Raspberry Pi 4, Navigator";

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Filebrowser,
        state_contracts: GroundedSet::unknown(
            "filebrowser has no service-level state machine (card states Unknown); external Go binary with no traced lifecycle states; running_baseline GET contracts captured in artifact only",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(HttpMethod::Get, "/file-browser/", 2.8, 5.6, 6.3, 60),
            runtime_slo(HttpMethod::Get, "/file-browser/health", 1.7, 2.3, 3.1, 60),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
            "running_baseline",
            Distribution {
                mean: 0.0,
                median: 0.0,
                p95: 0.0,
                min: 0.0,
                max: 0.0,
                sd: 0.0,
            },
            Distribution {
                mean: 15.9,
                median: 15.9,
                p95: 15.9,
                min: 15.9,
                max: 15.9,
                sd: 0.0,
            },
            90,
        )]),
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "filebrowser is the platform-independent browser web file-manager provider",
                    "runtime captured on Navigator only; idle Go binary RSS ~15.9 MB, CPU ~0% median",
                    "File-mutating /file-browser/api/* and authenticated session flows not probed (Tier-1 GET-only)",
                ],
            },
            runtime_prov("runtime-captures/filebrowser__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "Tier-1 GET-only capture; login, file mutations, and settings changes not exercised",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::Filebrowser,
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
            "runtime-captures/filebrowser__pi4_navigator_master.json#slo_running_baseline",
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
        runtime_prov("runtime-captures/filebrowser__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts = ObservedFacts {
    id: ServiceId::Filebrowser,
    aliases: ObservedSet::unknown(
        "external binary; no in-repo alias declarations in this repository",
    ),
    kind: Observed::known(
        ServiceKind::Binary,
        Evidence {
            file: "core/start-blueos-core",
            line: 137,
        },
    ),
    entrypoint: Observed::known(
        "nice -19 filebrowser --database /etc/filebrowser/filebrowser.db --baseurl /file-browser",
        Evidence {
            file: "core/start-blueos-core",
            line: 137,
        },
    ),
    tmux_name: Observed::known(
        "filebrowser",
        Evidence {
            file: "core/start-blueos-core",
            line: 137,
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
            line: 137,
        },
    ),
    nice: Observed::known(
        -19,
        Evidence {
            file: "core/start-blueos-core",
            line: 137,
        },
    ),
    run_as: Observed::known(
        "root",
        Evidence {
            file: "core/start-blueos-core",
            line: 137,
        },
    ),
    nginx_prefixes: ObservedSet::known(&[Evidenced::new(
        PathRef("/file-browser/"),
        Evidence {
            file: "core/tools/nginx/nginx.conf",
            line: 120,
        },
    )]),
    listen: ObservedSet::known(&[Evidenced::new(
        PortRef::Literal(7777),
        Evidence {
            file: "core/tools/nginx/nginx.conf",
            line: 121,
        },
    )]),
    git_path: Observed::unknown("external filebrowser binary; no source tree in this repository"),
    interfaces: ObservedSet::known(&[Evidenced::new(
        Interface::Rest {
            path_prefix: PathRef("/file-browser/"),
            port: PortRef::Literal(7777),
            versions: &[],
        },
        Evidence {
            file: "core/tools/nginx/nginx.conf",
            line: 121,
        },
    )]),
    resources: ObservedSet::known(&[Evidenced::new(
        Resource {
            path: PathRef("/etc/filebrowser/filebrowser.db"),
            ownership: ResourceOwnership::SharedWrite,
        },
        Evidence {
            file: "core/start-blueos-core",
            line: 137,
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
            ],
            ordered_before: &[
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
        "external filebrowser binary; no --log-path or log file in start-blueos-core launch args",
    ),
    zenoh_log_topic: Observed::unknown(
        "external filebrowser binary; does not use commonwealth init_logger zenoh publisher",
    ),
    sentry: Observed::unknown(
        "external filebrowser binary; no init_sentry or equivalent traced in this repository",
    ),
    openapi_refs: ObservedSet::unknown("not yet extracted"),
};

pub const SERVICE_DEFINITION: ServiceDefinition =
    ServiceDefinition {
        id: ServiceId::Filebrowser,
        singleton: Asserted::established(
            true,
            "single Normal-tier tmux instance; one filebrowser process is the sole browser web file-manager provider",
        ),
        bounded_context: Asserted::established(
            "web-file-browser",
            "provisional 2.0 domain: browser-based file management over the local filesystem via the upstream filebrowser SPA",
        ),
        journey_refs: AssertedSet::established(&[Rationaled::new(
            JourneyId::ManageBlueosFiles,
            "File Browser page embeds the upstream filebrowser SPA at /file-browser/ for viewing, editing, downloading, and uploading files",
        )]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "advanced/pirate file-management convenience; vehicle flight and MAVLink control operate fully without browser file editing — operators can use SSH/scp or the ttyd terminal instead",
        ),
        offline_required: Asserted::established(
            true,
            "local filesystem browse and REST SPA on localhost; no internet or WAN dependency for file view/edit/download/upload",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root in start-blueos-core Normal-tier launch line; edits, deletes, moves, and uploads arbitrary files across the filesystem as root",
        ),
        dangerous_operations: AssertedSet::established(&[Rationaled::new(
            DangerousOperation::Other("arbitrary_file_mutation"),
            "journey manage_blueos_files (advanced:431) exposes view/edit/download/upload over /file-browser/ as root; delete and move are standard filebrowser SPA operations — irreversible mutation of arbitrary root-owned files, comparable to nginx webdav_file_mutation but via a full file-manager UI",
        )]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "rubric requires Required when dangerous_operations is non-empty; File Browser page is gated behind Advanced/pirate visibility per journey",
        ),
        capabilities: AssertedSet::established(&[Rationaled::new(
            CapabilityId::ManageBlueosFiles,
            "journey capability: operator opens File Browser page and uses the embedded filebrowser SPA to view, edit, download, and upload BlueOS files",
        )]),
        authorities: AssertedSet::established(&[Rationaled::new(
            Authority::Other("file_browser"),
            "sole catalog service that exposes a browser-based file-manager UI; nginx /upload/ WebDAV is a separate PUT/DELETE endpoint on /usr/blueos/ only, not a full filesystem file manager",
        )]),
        states: AssertedSet::unknown(
            "external filebrowser binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(&[]),
        resources: AssertedSet::established(&[Rationaled::new(
            Resource {
                path: PathRef("/etc/filebrowser/filebrowser.db"),
                ownership: ResourceOwnership::SharedWrite,
            },
            "SQLite database persists filebrowser users, settings, and application state across restarts",
        )]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                &["start-blueos-core create_service"],
                "observed lifecycle trigger: tmux creation at boot in Normal tier",
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
                ],
                "observed ordered_after in start-blueos-core Normal block",
            ),
            ordered_before: Asserted::established(
                &[
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
                "observed ordered_before lists filebrowser before versionchooser and remaining Normal and SERVICES-tier peers",
            ),
            shutdown: Asserted::unknown(
                "external filebrowser binary; no explicit shutdown handler traced in this repository",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade restart semantics for the external filebrowser binary not traced in this repository",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; Rest /file-browser/ listener availability (proxied to filebrowser on port 7777) serves as health signal"
                ,
            "no dedicated /health route traced; filebrowser process continuity and Rest listener serve as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "core infrastructure file-management UI; does not install or host third-party extensions",
        ),
        api_stable: Asserted::unknown(
            "external binary; upstream filebrowser REST API stability not established from this repository",
        ),
        permissions_model: Asserted::established(
            "no auth middleware traced; Rest /file-browser/ is unauthenticated on the LAN",
            "external binary proxied by nginx without observed permission checks; LAN trust model — root filesystem mutation exposed without filebrowser-layer confirmation",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "filebrowser_process_down",
                "process exit or tmux session loss removes browser web file-manager access",
            ),
            Rationaled::new(
                "rest_listener_unavailable",
                "port 7777 Rest listener down blocks /file-browser/ connections from the File Browser page",
            ),
            Rationaled::new(
                "filebrowser_database_unavailable",
                "missing or corrupt /etc/filebrowser/filebrowser.db prevents filebrowser startup or user authentication state",
            ),
        ]),
        blast_radius: Asserted::established(
            "operators lose browser-based file management and must use SSH/scp or the ttyd terminal instead; vehicle flight and MAVLink control remain intact"
                ,
            "web file-manager outage is a convenience loss only; autopilot MAVLink routing and direct vehicle control are unaffected",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream filebrowser deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    };
