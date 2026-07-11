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

const RUNTIME_CAPTURE: &str = "runtime-captures/recorder_extractor__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId::RecorderExtractor,
        state_contracts: GroundedSet::unknown(
            "recorder_extractor has no service-level state machine (card states Unknown); background MCAP extraction and REST handlers are runtime-managed",
        ),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/files", 5.8, 9.7, 11.0, 60),
            runtime_slo(HttpMethod::Get, "/status", 5.5, 8.0, 15.1, 60),
            runtime_slo(HttpMethod::Get, "/", 4.6, 6.6, 9.7, 60),
        ]),
        resource_usage: GroundedSet::known(vec![runtime_resource(
            "running_baseline",
            Distribution {
                mean: 0.31,
                median: 0.00,
                p95: 1.02,
                min: 0.00,
                max: 1.92,
                sd: 0.47,
            },
            Distribution {
                mean: 35.6,
                median: 35.6,
                p95: 35.6,
                min: 35.6,
                max: 35.6,
                sd: 0.0,
            },
            120,
        )]),
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "recorder_extractor extracts MCAP to MP4, serves recordings over REST, and manages the gallery; platform-independent".into(),
                    "runtime captured on Navigator only with NO MP4 recordings present; RSS ~35.6 MB flat, CPU ~0.31% mean with small periodic MCAP loop activity".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "recorder_extractor has no settings.json; DELETE /files/{filename} permanently removes recordings — not exercised (destructive; no recordings present)",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId::RecorderExtractor,
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
        id: ServiceId::RecorderExtractor,
        aliases: ObservedSet::known(vec![Evidenced::new(
            "recorder-extractor".to_string(),
            Evidence {
                file: "core/services/recorder_extractor/main.py".to_string(),
                line: 26,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 146,
            },
        ),
        entrypoint: Observed::known(
            "$SERVICES_PATH/recorder_extractor/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 146,
            },
        ),
        tmux_name: Observed::known(
            "recorder_extractor".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 146,
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
                line: 146,
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 146,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/recorder-extractor/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 129,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(9150),
            Evidence {
                file: "core/services/recorder_extractor/main.py".to_string(),
                line: 28,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/recorder_extractor".to_string()),
            Evidence {
                file: "core/services/recorder_extractor/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/recorder-extractor/".to_string()),
                    port: PortRef::Literal(9150),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/recorder_extractor/main.py".to_string(),
                    line: 434,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/usr/blueos/userdata/recorder".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/recorder_extractor/main.py".to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "mcap doctor {path}".to_string(),
                },
                Evidence {
                    file: "core/services/recorder_extractor/main.py".to_string(),
                    line: 124,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "mcap recover {path} -o {tmp}".to_string(),
                },
                Evidence {
                    file: "core/services/recorder_extractor/main.py".to_string(),
                    line: 149,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "mcap-foxglove-video-extract {mcap} all --output {dir}".to_string(),
                },
                Evidence {
                    file: "core/services/recorder_extractor/main.py".to_string(),
                    line: 281,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "gst-discoverer-1.0 file://{path}".to_string(),
                },
                Evidence {
                    file: "core/services/recorder_extractor/main.py".to_string(),
                    line: 201,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "gst-play-1.0 --start-position={target_sec} --videosink={pipeline} --audiosink=fakesink --no-interactive -q file://{path}"
                        .to_string(),
                },
                Evidence {
                    file: "core/services/recorder_extractor/main.py".to_string(),
                    line: 230,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/recorder-extractor/log".to_string()],
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
                path: PathRef("/usr/blueos/userdata/recorder".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/recorder_extractor/main.py".to_string(),
                line: 27,
            },
        )]),
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
                    ServiceId::Nginx,
                    ServiceId::BagOfHolding,
                    ServiceId::Recorder,
                ],
                ordered_before: vec![
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
            "init_logger publishes to zenoh only; no on-disk log path set in recorder_extractor source",
        ),
        zenoh_log_topic: Observed::known(
            "services/recorder-extractor/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/recorder_extractor/main.py".to_string(),
                line: 454,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId::RecorderExtractor,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one recorder_extractor process",
        ),
        bounded_context: Asserted::established(
            "video-recording-playback-and-management".to_string(),
            "provisional 2.0 domain: MCAP-to-MP4 extraction, recording gallery REST, thumbnails, download, and deletion",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId("browse_video_recordings".into()),
                "Records page lists MP4 recordings with thumbnails and MCAP extraction processing status",
            ),
            Rationaled::new(
                JourneyId("download_video_recording".into()),
                "download button or in-dialog player streams the MP4 via GET /files/{filename}",
            ),
            Rationaled::new(
                JourneyId("delete_video_recording".into()),
                "recording card delete button removes the MP4 file via DELETE /files/{filename}",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "SERVICES startup tier; core vehicle operation does not depend on video playback, download, or recording management",
        ),
        offline_required: Asserted::established(
            true,
            "listing, thumbnails, download, deletion, and MCAP extraction operate on local files under /usr/blueos/userdata/recorder",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; DELETE /files/{filename} unlinks recordings and background extraction writes MP4s under userdata",
        ),
        dangerous_operations: AssertedSet::established(vec![Rationaled::new(
            DangerousOperation::Other("delete_recording".to_string()),
            "DELETE /files/{filename} permanently removes the MP4 via path.unlink(); irreversible user-data loss",
        )]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "rubric requires Required when dangerous_operations is non-empty; RecordsView deleteRecording issues DELETE immediately without a confirm dialog today",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("browse_video_recordings".to_string()),
                "GET /files lists MP4 recordings; GET /status reports MCAP extraction progress; GET /files/{filename}/thumbnail serves JPEG previews",
            ),
            Rationaled::new(
                CapabilityId("download_video_recording".to_string()),
                "GET /files/{filename} streams or downloads an MP4 recording for playback or export",
            ),
            Rationaled::new(
                CapabilityId("delete_video_recording".to_string()),
                "DELETE /files/{filename} permanently removes an MP4 recording from the gallery",
            ),
        ]),
        authorities: AssertedSet::established(vec![Rationaled::new(
            Authority::Other("video_recording_server".to_string()),
            "sole catalog service that extracts MCAP to MP4, serves recordings over REST, and deletes operator recordings; recorder writes MCAP into the shared dir but is uncataloged",
        )]),
        states: AssertedSet::unknown(
            "no cataloged state machine; background MCAP extraction and REST handlers are runtime-managed",
        ),
        edges: AssertedSet::established(vec![]),
        resources: AssertedSet::established(vec![Rationaled::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/recorder".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            "shared recorder directory: recorder writes MCAP recordings; recorder_extractor extracts MP4s, serves thumbnails/downloads, and deletes files",
        )]),
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
                    ServiceId::Versionchooser,
                    ServiceId::Pardal,
                    ServiceId::Ping,
                    ServiceId::UserTerminal,
                    ServiceId::Ttyd,
                    ServiceId::Nginx,
                    ServiceId::BagOfHolding,
                    ServiceId::Recorder,
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                vec![
                    ServiceId::DiskUsage,
                    ServiceId::Customization,
                ],
                "observed ordered_before lists recorder_extractor before disk_usage and customization",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit cancels background MCAP extraction task".to_string(),
                "main() finally block cancels extract_mcap_recordings asyncio task after server.serve returns",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight MCAP extraction and partial MP4 writes not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns service name".to_string(),
            "no dedicated /health route; uvicorn availability serves as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "video recording playback utility; does not install or host third-party extensions",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 router exposed under /recorder-extractor/ via VersionedFastAPI",
        ),
        permissions_model: Asserted::established(
            "no service-level auth; any caller reaching nginx may list, download, or delete recordings"
                .to_string(),
            "REST endpoints accept any caller reaching nginx; delete is not gated server-side",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "recording_not_found".to_string(),
                "resolve_recording returns 404 when the requested filename is missing under the recorder directory",
            ),
            Rationaled::new(
                "delete_recording_failure".to_string(),
                "DELETE /files/{filename} returns 500 when path.unlink raises",
            ),
            Rationaled::new(
                "mcap_extraction_failure".to_string(),
                "background extract_mcap_recordings logs subprocess errors when mcap or gstreamer tools fail",
            ),
            Rationaled::new(
                "thumbnail_generation_failure".to_string(),
                "GET /files/{filename}/thumbnail may fail when gst-play cannot render a frame from the MP4",
            ),
        ]),
        blast_radius: Asserted::established(
            "recording playback, download, and deletion unavailable; MCAP-to-MP4 extraction stops; core vehicle services unaffected"
                .to_string(),
            "recorder_extractor outage blocks the Records UI and leaves new MCAP files unextracted but does not stop autopilot, MAVLink, or nginx core paths",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and mcap/gstreamer binary version coupling not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
