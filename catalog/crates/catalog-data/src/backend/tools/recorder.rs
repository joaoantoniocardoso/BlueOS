use catalog_kernel::criticality::CriticalityTier;
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::refs::PathRef;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use catalog_model::edge::{Bus, Connection, FailureImpact, SyncMode};
use catalog_model::lifecycle::{Lifecycle, ObservedLifecycle};
use catalog_model::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use catalog_model::resource::{Resource, ResourceOwnership};
use catalog_model::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts};
use catalog_model::service::{Authority, Service, ServiceJudgment};
use catalog_model::trust::{PrivilegeLevel, UserConfirmation};

use catalog_kernel::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_REPODIGEST;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_REPODIGEST;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Recorder,
        state_contracts: GroundedSet::unknown(
            "recorder has no HTTP routes and no service-level state machine; idle running_baseline (process up, empty recording dir) captured in artifact only",
        ),
        slo_baselines: GroundedSet::unknown(
            "recorder is a background recording daemon with no HTTP interface — no route latency baselines to measure",
        ),
        resource_usage: GroundedSet::known(&[runtime_resource(
            "running_baseline",
            Distribution {
                mean: 6.18,
                median: 6.08,
                p95: 8.74,
                min: 3.89,
                max: 10.45,
                sd: 1.4,
            },
            Distribution {
                mean: 9.4,
                median: 9.4,
                p95: 9.4,
                min: 9.4,
                max: 9.4,
                sd: 0.0,
            },
            60,
        )]),
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "blueos-recorder is a platform-independent background MCAP recording daemon",
                    "runtime captured on Navigator only; idle Rust binary RSS ~9.4 MB, CPU ~6% mean across 60 samples",
                    "recording dir /usr/blueos/userdata/recorder empty at capture — no active MCAP session recording",
                ],
            },
            runtime_prov("runtime-captures/recorder__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "Tier-1 idle sampling and recording-dir observation only; no recording session triggered or settings mutations exercised",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
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
        runtime_prov("runtime-captures/recorder__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts =
    ObservedFacts {
        id: ServiceId::Recorder,
        aliases: ObservedSet::unknown(
            "external binary; no in-repo alias declarations in this repository",
        ),
        kind: Observed::known(
            ServiceKind::Binary,
            Evidence {
                file: "core/start-blueos-core",
                line: 145,
                anchor: "'recorder',250,0,0,0,\"blueos-recorder --recorder-path /usr/b",
            },
        ),
        entrypoint: Observed::known(
            "blueos-recorder --recorder-path /usr/blueos/userdata/recorder",
            Evidence {
                file: "core/start-blueos-core",
                line: 145,
                anchor: "'recorder',250,0,0,0,\"blueos-recorder --recorder-path /usr/b",
            },
        ),
        tmux_name: Observed::known(
            "recorder",
            Evidence {
                file: "core/start-blueos-core",
                line: 145,
                anchor: "'recorder',250,0,0,0,\"blueos-recorder --recorder-path /usr/b",
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
                line: 145,
                anchor: "'recorder',250,0,0,0,\"blueos-recorder --recorder-path /usr/b",
            },
        ),
        nice: Observed::unknown("command line has no nice wrapper"),
        run_as: Observed::known(
            "root",
            Evidence {
                file: "core/start-blueos-core",
                line: 145,
                anchor: "'recorder',250,0,0,0,\"blueos-recorder --recorder-path /usr/b",
            },
        ),
        nginx_prefixes: ObservedSet::known(&[]),
        listen: ObservedSet::known(&[]),
        git_path: Observed::unknown(
            "external blueos-recorder binary (installed via core/tools/recorder/bootstrap.sh from github.com/bluerobotics/blueos-recorder); no source tree in this repository",
        ),
        interfaces: ObservedSet::known(&[]),
        resources: ObservedSet::known(&[Evidenced::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/recorder"),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/start-blueos-core",
                line: 145,
                anchor: "'recorder',250,0,0,0,\"blueos-recorder --recorder-path /usr/b",
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
                ],
                ordered_before: &[
                    ServiceId::RecorderExtractor,
                    ServiceId::DiskUsage,
                    ServiceId::Customization,
                ],
            },
            Evidence {
                file: "core/start-blueos-core",
                line: 326,
                anchor: "for TUPLE in \"${SERVICES[@]}\"; do",
            },
        ),
        logs_path: Observed::unknown(
            "external blueos-recorder binary; no --log-path or log file in start-blueos-core launch args",
        ),
        zenoh_log_topic: Observed::unknown(
            "external blueos-recorder binary; does not use commonwealth init_logger zenoh publisher",
        ),
        sentry: Observed::unknown(
            "external blueos-recorder binary; no init_sentry or equivalent traced in this repository",
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    };

pub const SERVICE_DEFINITION: ServiceJudgment =
    ServiceJudgment {
        id: ServiceId::Recorder,
        singleton: Asserted::established(
            true,
            "single Normal-tier tmux instance; one blueos-recorder process writes MCAP session recordings",
        ),
        bounded_context: Asserted::established(
            "session-recording",
            "provisional 2.0 domain: background capture of the vehicle data stream into MCAP files on local disk",
        ),
        journey_refs: AssertedSet::established(&[]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "Normal-tier background data-capture daemon; vehicle flight and MAVLink control do not depend on session recording — outage stops new MCAP files only",
        ),
        offline_required: Asserted::established(
            true,
            "writes MCAP session recordings to local /usr/blueos/userdata/recorder; no internet or WAN dependency",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root in start-blueos-core Normal-tier launch line",
        ),
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "no irreversible, untrusted-code, or vehicle-arm operations; continuous MCAP writes are additive session data, not destructive mutations of existing user files",
        ),
        capabilities: AssertedSet::established(&[Rationaled::new(
            CapabilityId::RecordVehicleDataStream,
            "observed entrypoint blueos-recorder --recorder-path /usr/blueos/userdata/recorder; background daemon captures the vehicle data stream into MCAP session files",
        )]),
        authorities: AssertedSet::established(&[Rationaled::new(
            Authority::Other("session_recorder"),
            "sole catalog service that produces MCAP session recordings; recorder_extractor consumes and extracts them but does not write MCAP",
        )]),
        states: AssertedSet::unknown(
            "external blueos-recorder binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(&[Rationaled::new(
            Connection {
                from: ServiceId::Recorder,
                to: ServiceId::RecorderExtractor,
                via: Bus::File,
                sync: SyncMode::Async,
                endpoint: "/usr/blueos/userdata/recorder",
                purpose: "produce MCAP session recordings consumed by recorder_extractor for MP4 extraction and gallery serving"
                    ,
                required_at_boot: false,
                failure_impact: FailureImpact::Degraded,
            },
            "observed SharedWrite on /usr/blueos/userdata/recorder pairs with recorder_extractor File ReadWrite interface on the same path; producer-to-consumer filesystem coupling via Bus::File",
        )]),
        resources: AssertedSet::established(&[Rationaled::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/recorder"),
                ownership: ResourceOwnership::SharedWrite,
            },
            "observed --recorder-path; blueos-recorder writes MCAP session recordings into the shared recorder directory",
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
                    ServiceId::Filebrowser,
                    ServiceId::Versionchooser,
                    ServiceId::Pardal,
                    ServiceId::Ping,
                    ServiceId::UserTerminal,
                    ServiceId::Ttyd,
                    ServiceId::Nginx,
                    ServiceId::BagOfHolding,
                ],
                "observed ordered_after in start-blueos-core Normal block",
            ),
            ordered_before: Asserted::established(
                &[
                    ServiceId::RecorderExtractor,
                    ServiceId::DiskUsage,
                    ServiceId::Customization,
                ],
                "observed ordered_before lists recorder before recorder_extractor, disk_usage, and customization",
            ),
            shutdown: Asserted::unknown(
                "external blueos-recorder binary; no explicit shutdown handler traced in this repository",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade restart semantics for the external blueos-recorder binary not traced in this repository",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; no HTTP listener or dedicated health endpoint",
            "no nginx route or listen port; blueos-recorder process continuity in tmux serves as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "background session recording daemon; does not install or host third-party extensions",
        ),
        api_stable: Asserted::unknown(
            "external blueos-recorder binary; MCAP output format stability not established from this repository",
        ),
        permissions_model: Asserted::established(
            "no operator-facing API; background daemon with no nginx route or REST interface",
            "no observed interfaces or journeys; recording runs autonomously without caller permission checks",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "recorder_process_down",
                "process exit or tmux session loss stops new MCAP session recording; existing files remain on disk",
            ),
            Rationaled::new(
                "recorder_path_unavailable",
                "missing or unwritable /usr/blueos/userdata/recorder blocks MCAP file creation",
            ),
            Rationaled::new(
                "disk_full_from_recording",
                "continuous MCAP capture can fill userdata storage and block new recordings or contend with other services",
            ),
        ]),
        blast_radius: Asserted::established(
            "no new session MCAP recordings captured; existing recordings and recorder_extractor playback remain available; vehicle flight and MAVLink control unaffected"
                ,
            "recorder outage is data-capture convenience loss only; recorder_extractor continues serving prior MP4 extractions from existing MCAP files",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream blueos-recorder deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Recorder,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
