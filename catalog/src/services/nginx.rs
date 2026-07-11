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

const RUNTIME_CAPTURE: &str = "runtime-captures/nginx__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e; RepoDigest sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId::Nginx,
        state_contracts: GroundedSet::unknown(
            "nginx has no service-level state machine (card states Unknown); external C binary with no traced lifecycle states; running_baseline GET contracts captured in artifact only",
        ),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/status", 0.5, 0.7, 0.9, 60),
            runtime_slo(HttpMethod::Get, "/", 0.8, 1.3, 2.0, 60),
            runtime_slo(HttpMethod::Get, "/userdata/", 0.8, 1.1, 1.7, 60),
            runtime_slo(HttpMethod::Get, "/assets/", 3.9, 5.6, 6.9, 60),
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
                mean: 5.5,
                median: 5.5,
                p95: 5.5,
                min: 5.5,
                max: 5.5,
                sd: 0.0,
            },
            90,
        )]),
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "nginx is the platform-independent HTTP ingress reverse proxy".into(),
                    "multi-process: master PID 1178 RSS ~5.5 MB + 5 www-data workers aggregate RSS ~16.5 MB (total ~22.0 MB at snapshot)".into(),
                    "GET /status is unconditional 204 liveness only, not backend-reachability".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "Tier-1 GET-only capture; /upload/ WebDAV mutations not exercised",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId::Nginx,
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
        id: ServiceId::Nginx,
        aliases: ObservedSet::unknown(
            "external binary; no in-repo alias declarations in this repository",
        ),
        kind: Observed::known(
            ServiceKind::Binary,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 143,
            },
        ),
        entrypoint: Observed::known(
            "nice -18 nginx -g \"daemon off;\" -c $TOOLS_PATH/nginx/nginx.conf".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 143,
            },
        ),
        tmux_name: Observed::known(
            "nginx".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 143,
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
                line: 143,
            },
        ),
        nice: Observed::known(
            -18,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 143,
            },
        ),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 143,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![
            Evidenced::new(
                PathRef("/".to_string()),
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 268,
                },
            ),
            Evidenced::new(
                PathRef("/status".to_string()),
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 62,
                },
            ),
            Evidenced::new(
                PathRef("/assets/".to_string()),
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 282,
                },
            ),
            Evidenced::new(
                PathRef("/upload/".to_string()),
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 288,
                },
            ),
            Evidenced::new(
                PathRef("/userdata/".to_string()),
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 300,
                },
            ),
            Evidenced::new(
                PathRef("/cache/".to_string()),
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 66,
                },
            ),
        ]),
        listen: ObservedSet::known(vec![
            Evidenced::new(
                PortRef::Literal(80),
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 51,
                },
            ),
            Evidenced::new(
                PortRef::Literal(2770),
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 42,
                },
            ),
        ]),
        git_path: Observed::unknown("external nginx binary; no source tree in this repository"),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/".to_string()),
                    port: PortRef::Literal(80),
                    versions: vec![],
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 51,
                },
            ),
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/".to_string()),
                    port: PortRef::Literal(2770),
                    versions: vec![],
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 42,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "https://$target".to_string(),
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 73,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/usr/blueos/".to_string()),
                    mode: FileAccessMode::Write,
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 290,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/usr/blueos".to_string()),
                    mode: FileAccessMode::Read,
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 301,
                },
            ),
        ]),
        resources: ObservedSet::known(vec![
            Evidenced::new(
                Resource {
                    path: PathRef("$TOOLS_PATH/nginx/nginx.conf".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/start-blueos-core".to_string(),
                    line: 143,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/home/pi/frontend".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 269,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/usr/blueos/".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 290,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/usr/blueos".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 301,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/var/cache/nginx".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 31,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/var/log/nginx".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 37,
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
                    ServiceId::Versionchooser,
                    ServiceId::Pardal,
                    ServiceId::Ping,
                    ServiceId::UserTerminal,
                    ServiceId::Ttyd,
                ],
                ordered_before: vec![
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
        logs_path: Observed::known(
            PathRef("/var/log/nginx".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 37,
            },
        ),
        zenoh_log_topic: Observed::unknown(
            "external nginx binary; does not use commonwealth init_logger zenoh publisher",
        ),
        sentry: Observed::unknown(
            "external nginx binary; no init_sentry or equivalent traced in this repository",
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId::Nginx,
        singleton: Asserted::established(
            true,
            "single Normal-tier tmux instance; one nginx master process is the sole HTTP ingress on port 80",
        ),
        bounded_context: Asserted::established(
            "http-ingress".to_string(),
            "provisional 2.0 domain: HTTP reverse proxy and frontend server — the vehicle's web front door",
        ),
        journey_refs: AssertedSet::established(vec![Rationaled::new(
            JourneyId::AccessBlueosWebInterface,
            "operator opens the BlueOS web interface in a browser via nginx port 80",
        )]),
        tier: Asserted::established(
            CriticalityTier::Important,
            "Normal-tier sole HTTP ingress; all browser UI and nginx-proxied REST APIs are unreachable when down, but autopilot MAVLink control via ardupilot_manager and GCS does not route through nginx and the vehicle remains controllable",
        ),
        offline_required: Asserted::established(
            true,
            "frontend SPA serving, reverse proxy to localhost backends, WebDAV /upload/, and /userdata/ reads are fully local; only /cache/ outbound HTTPS proxy is an optional online enhancement",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root in start-blueos-core Normal-tier launch line; master binds port 80 and spawns www-data worker processes",
        ),
        dangerous_operations: AssertedSet::established(vec![Rationaled::new(
            DangerousOperation::Other("webdav_file_mutation".to_string()),
            "observed /upload/ WebDAV dav_methods PUT DELETE MKCOL COPY MOVE on /usr/blueos/ alias; DELETE and MOVE can irreversibly remove or relocate files over unauthenticated LAN HTTP",
        )]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "rubric requires Required when dangerous_operations is non-empty; nginx does not prompt today — /upload/ DAV DELETE/MOVE is an unauthenticated LAN endpoint comparable to recorder_extractor delete_recording",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId::AccessBlueosWebInterface,
                "journey capability: nginx listens on port 80 and serves the frontend SPA at / for browser access to configure vehicle services",
            ),
            Rationaled::new(
                CapabilityId::ServeFrontendSpa,
                "observed Rest / on port 80 serves static frontend from /home/pi/frontend at location /",
            ),
            Rationaled::new(
                CapabilityId::ReverseProxyBackendServices,
                "observed nginx.conf location blocks proxy_pass every catalog backend; sole HTTP ingress routing operator and frontend traffic to localhost services",
            ),
            Rationaled::new(
                CapabilityId::ServeWebdavUploads,
                "observed /upload/ WebDAV alias to /usr/blueos/ with dav_methods PUT DELETE MKCOL COPY MOVE and File write interface",
            ),
            Rationaled::new(
                CapabilityId::CacheExternalHttp,
                "observed /cache/ OutboundHttp proxy to https://$target with resolver 8.8.8.8 for outbound HTTPS caching",
            ),
        ]),
        authorities: AssertedSet::established(vec![Rationaled::new(
            Authority::NginxProxy,
            "sole catalog HTTP reverse proxy and frontend server; every browser and LAN REST consumer reaches backend services exclusively through nginx on port 80",
        )]),
        states: AssertedSet::unknown(
            "external nginx binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(vec![]),
        resources: AssertedSet::established(vec![
            Rationaled::new(
                Resource {
                    path: PathRef("$TOOLS_PATH/nginx/nginx.conf".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                "nginx master configuration defining listen ports, proxy routes, WebDAV, and cache locations",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/home/pi/frontend".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                "frontend SPA static assets served at / for operator browser access",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/usr/blueos/".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "WebDAV /upload/ alias target; nginx workers write uploaded files under /usr/blueos/",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/usr/blueos".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                "/userdata/ read serve root exposing /usr/blueos tree to operators",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/var/cache/nginx".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "on-disk cache for /cache/ outbound HTTPS proxy responses",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/var/log/nginx".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "nginx access and error logs",
            ),
        ]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                vec!["start-blueos-core create_service".to_string()],
                "observed lifecycle trigger: tmux creation at boot in Normal tier",
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
                    ServiceId::Versionchooser,
                    ServiceId::Pardal,
                    ServiceId::Ping,
                    ServiceId::UserTerminal,
                    ServiceId::Ttyd,
                ],
                "observed ordered_after in start-blueos-core Normal block; nginx starts after proxied backends are up",
            ),
            ordered_before: Asserted::established(
                vec![
                    ServiceId::BagOfHolding,
                    ServiceId::Recorder,
                    ServiceId::RecorderExtractor,
                    ServiceId::DiskUsage,
                    ServiceId::Customization,
                ],
                "observed ordered_before lists nginx before remaining SERVICES-tier peers",
            ),
            shutdown: Asserted::unknown(
                "external nginx binary; no explicit shutdown handler traced in this repository",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade restart semantics for the external nginx binary not traced in this repository",
            ),
        },
        health: Asserted::established(
            "GET /status returns HTTP 204 when nginx is online".to_string(),
            "observed /status location is an unconditional return 204 (nginx.conf:60-63); it signals nginx liveness only, not proxied-backend reachability; frontend api.ts polls it as the backend-online signal",
        ),
        is_platform: Asserted::established(
            false,
            "core infrastructure HTTP ingress; does not install or host third-party extensions",
        ),
        api_stable: Asserted::unknown(
            "external binary with empty observed REST versions list; upstream nginx API stability not established from this repository",
        ),
        permissions_model: Asserted::established(
            "no auth middleware traced; port 80 REST, WebDAV /upload/, and /userdata/ are unauthenticated on the LAN"
                .to_string(),
            "external binary without observed permission checks; LAN trust model — WebDAV DELETE/MOVE is exposed without nginx-layer confirmation",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "master_process_down".to_string(),
                "nginx master exit or tmux session loss removes the sole port 80 HTTP ingress",
            ),
            Rationaled::new(
                "port_80_bind_failure".to_string(),
                "cannot bind port 80 blocks all browser and LAN REST access to BlueOS",
            ),
            Rationaled::new(
                "frontend_spa_unavailable".to_string(),
                "/home/pi/frontend missing or unreadable prevents serving the operator web UI at /",
            ),
            Rationaled::new(
                "backend_proxy_unreachable".to_string(),
                "upstream localhost backend down causes 502/504 on proxied service routes while nginx itself may still serve /status and static assets",
            ),
        ]),
        blast_radius: Asserted::established(
            "all browser web UI and nginx-proxied REST APIs become unreachable; operator cannot configure or monitor via HTTP; autopilot MAVLink routing and direct GCS vehicle control remain intact"
                .to_string(),
            "sole HTTP ingress outage isolates web-based operations but does not remove ardupilot_manager MAVLink control paths",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream nginx deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    }
}
