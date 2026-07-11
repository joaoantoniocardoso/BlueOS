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

const RUNTIME_CAPTURE: &str = "runtime-captures/filebrowser__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e; RepoDigest sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId("filebrowser".into()),
        state_contracts: GroundedSet::unknown(
            "filebrowser has no service-level state machine (card states Unknown); external Go binary with no traced lifecycle states; running_baseline GET contracts captured in artifact only",
        ),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/file-browser/", 2.8, 5.6, 6.3, 60),
            runtime_slo(HttpMethod::Get, "/file-browser/health", 1.7, 2.3, 3.1, 60),
        ]),
        resource_usage: GroundedSet::known(vec![runtime_resource(
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
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "filebrowser is the platform-independent browser web file-manager provider".into(),
                    "runtime captured on Navigator only; idle Go binary RSS ~15.9 MB, CPU ~0% median".into(),
                    "File-mutating /file-browser/api/* and authenticated session flows not probed (Tier-1 GET-only)".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "Tier-1 GET-only capture; login, file mutations, and settings changes not exercised",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId("filebrowser".into()),
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
        id: ServiceId("filebrowser".to_string()),
        aliases: ObservedSet::unknown(
            "external binary; no in-repo alias declarations in this repository",
        ),
        kind: Observed::known(
            ServiceKind::Binary,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 137,
            },
        ),
        entrypoint: Observed::known(
            "nice -19 filebrowser --database /etc/filebrowser/filebrowser.db --baseurl /file-browser"
                .to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 137,
            },
        ),
        tmux_name: Observed::known(
            "filebrowser".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 137,
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
                line: 137,
            },
        ),
        nice: Observed::known(
            -19,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 137,
            },
        ),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 137,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/file-browser/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 120,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(7777),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 121,
            },
        )]),
        git_path: Observed::unknown(
            "external filebrowser binary; no source tree in this repository",
        ),
        interfaces: ObservedSet::known(vec![Evidenced::new(
            Interface::Rest {
                path_prefix: PathRef("/file-browser/".to_string()),
                port: PortRef::Literal(7777),
                versions: vec![],
            },
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 121,
            },
        )]),
        resources: ObservedSet::known(vec![Evidenced::new(
            Resource {
                path: PathRef("/etc/filebrowser/filebrowser.db".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 137,
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
                ],
                ordered_before: vec![
                    ServiceId("versionchooser".to_string()),
                    ServiceId("pardal".to_string()),
                    ServiceId("ping".to_string()),
                    ServiceId("user_terminal".to_string()),
                    ServiceId("ttyd".to_string()),
                    ServiceId("nginx".to_string()),
                    ServiceId("bag_of_holding".to_string()),
                    ServiceId("recorder".to_string()),
                    ServiceId("recorder_extractor".to_string()),
                    ServiceId("disk_usage".to_string()),
                    ServiceId("customization".to_string()),
                ],
            },
            Evidence {
                file: "core/start-blueos-core".to_string(),
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
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("filebrowser".to_string()),
        singleton: Asserted::established(
            true,
            "single Normal-tier tmux instance; one filebrowser process is the sole browser web file-manager provider",
        ),
        bounded_context: Asserted::established(
            "web-file-browser".to_string(),
            "provisional 2.0 domain: browser-based file management over the local filesystem via the upstream filebrowser SPA",
        ),
        journey_refs: AssertedSet::established(vec![Rationaled::new(
            JourneyId("manage_blueos_files".into()),
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
        dangerous_operations: AssertedSet::established(vec![Rationaled::new(
            DangerousOperation::Other("arbitrary_file_mutation".to_string()),
            "journey manage_blueos_files (advanced:431) exposes view/edit/download/upload over /file-browser/ as root; delete and move are standard filebrowser SPA operations — irreversible mutation of arbitrary root-owned files, comparable to nginx webdav_file_mutation but via a full file-manager UI",
        )]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "rubric requires Required when dangerous_operations is non-empty; File Browser page is gated behind Advanced/pirate visibility per journey",
        ),
        capabilities: AssertedSet::established(vec![Rationaled::new(
            CapabilityId("manage_blueos_files".to_string()),
            "journey capability: operator opens File Browser page and uses the embedded filebrowser SPA to view, edit, download, and upload BlueOS files",
        )]),
        authorities: AssertedSet::established(vec![Rationaled::new(
            Authority::Other("file_browser".to_string()),
            "sole catalog service that exposes a browser-based file-manager UI; nginx /upload/ WebDAV is a separate PUT/DELETE endpoint on /usr/blueos/ only, not a full filesystem file manager",
        )]),
        states: AssertedSet::unknown(
            "external filebrowser binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(vec![]),
        resources: AssertedSet::established(vec![Rationaled::new(
            Resource {
                path: PathRef("/etc/filebrowser/filebrowser.db".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            "SQLite database persists filebrowser users, settings, and application state across restarts",
        )]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                vec!["start-blueos-core create_service".to_string()],
                "observed lifecycle trigger: tmux creation at boot in Normal tier",
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
                ],
                "observed ordered_after in start-blueos-core Normal block",
            ),
            ordered_before: Asserted::established(
                vec![
                    ServiceId("versionchooser".to_string()),
                    ServiceId("pardal".to_string()),
                    ServiceId("ping".to_string()),
                    ServiceId("user_terminal".to_string()),
                    ServiceId("ttyd".to_string()),
                    ServiceId("nginx".to_string()),
                    ServiceId("bag_of_holding".to_string()),
                    ServiceId("recorder".to_string()),
                    ServiceId("recorder_extractor".to_string()),
                    ServiceId("disk_usage".to_string()),
                    ServiceId("customization".to_string()),
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
                .to_string(),
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
            "no auth middleware traced; Rest /file-browser/ is unauthenticated on the LAN".to_string(),
            "external binary proxied by nginx without observed permission checks; LAN trust model — root filesystem mutation exposed without filebrowser-layer confirmation",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "filebrowser_process_down".to_string(),
                "process exit or tmux session loss removes browser web file-manager access",
            ),
            Rationaled::new(
                "rest_listener_unavailable".to_string(),
                "port 7777 Rest listener down blocks /file-browser/ connections from the File Browser page",
            ),
            Rationaled::new(
                "filebrowser_database_unavailable".to_string(),
                "missing or corrupt /etc/filebrowser/filebrowser.db prevents filebrowser startup or user authentication state",
            ),
        ]),
        blast_radius: Asserted::established(
            "operators lose browser-based file management and must use SSH/scp or the ttyd terminal instead; vehicle flight and MAVLink control remain intact"
                .to_string(),
            "web file-manager outage is a convenience loss only; autopilot MAVLink routing and direct vehicle control are unaffected",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream filebrowser deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    }
}
