use crate::criticality::CriticalityTier;
use crate::edge::{Bus, Edge, FailureImpact, SyncMode};
use crate::id::{CapabilityId, PathRef, ServiceId};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use crate::resource::{Resource, ResourceOwnership};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts};
use crate::service::{Authority, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

const RUNTIME_CAPTURE: &str = "runtime-captures/recorder__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e; RepoDigest sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId("recorder".into()),
        state_contracts: GroundedSet::unknown(
            "recorder has no HTTP routes and no service-level state machine; idle running_baseline (process up, empty recording dir) captured in artifact only",
        ),
        slo_baselines: GroundedSet::unknown(
            "recorder is a background recording daemon with no HTTP interface — no route latency baselines to measure",
        ),
        resource_usage: GroundedSet::known(vec![runtime_resource(
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
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "blueos-recorder is a platform-independent background MCAP recording daemon".into(),
                    "runtime captured on Navigator only; idle Rust binary RSS ~9.4 MB, CPU ~6% mean across 60 samples".into(),
                    "recording dir /usr/blueos/userdata/recorder empty at capture — no active MCAP session recording".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "Tier-1 idle sampling and recording-dir observation only; no recording session triggered or settings mutations exercised",
        ),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
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
        id: ServiceId("recorder".to_string()),
        aliases: ObservedSet::unknown(
            "external binary; no in-repo alias declarations in this repository",
        ),
        kind: Observed::known(
            ServiceKind::Binary,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 145,
            },
        ),
        entrypoint: Observed::known(
            "blueos-recorder --recorder-path /usr/blueos/userdata/recorder".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 145,
            },
        ),
        tmux_name: Observed::known(
            "recorder".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 145,
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
                line: 145,
            },
        ),
        nice: Observed::unknown("command line has no nice wrapper"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 145,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![]),
        listen: ObservedSet::known(vec![]),
        git_path: Observed::unknown(
            "external blueos-recorder binary (installed via core/tools/recorder/bootstrap.sh from github.com/bluerobotics/blueos-recorder); no source tree in this repository",
        ),
        interfaces: ObservedSet::known(vec![]),
        resources: ObservedSet::known(vec![Evidenced::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/recorder".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 145,
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
                ],
                ordered_before: vec![
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
            "external blueos-recorder binary; no --log-path or log file in start-blueos-core launch args",
        ),
        zenoh_log_topic: Observed::unknown(
            "external blueos-recorder binary; does not use commonwealth init_logger zenoh publisher",
        ),
        sentry: Observed::unknown(
            "external blueos-recorder binary; no init_sentry or equivalent traced in this repository",
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("recorder".to_string()),
        singleton: Asserted::established(
            true,
            "single Normal-tier tmux instance; one blueos-recorder process writes MCAP session recordings",
        ),
        bounded_context: Asserted::established(
            "session-recording".to_string(),
            "provisional 2.0 domain: background capture of the vehicle data stream into MCAP files on local disk",
        ),
        journey_refs: AssertedSet::established(vec![]),
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
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "no irreversible, untrusted-code, or vehicle-arm operations; continuous MCAP writes are additive session data, not destructive mutations of existing user files",
        ),
        capabilities: AssertedSet::established(vec![Rationaled::new(
            CapabilityId("record_vehicle_data_stream".to_string()),
            "observed entrypoint blueos-recorder --recorder-path /usr/blueos/userdata/recorder; background daemon captures the vehicle data stream into MCAP session files",
        )]),
        authorities: AssertedSet::established(vec![Rationaled::new(
            Authority::Other("session_recorder".to_string()),
            "sole catalog service that produces MCAP session recordings; recorder_extractor consumes and extracts them but does not write MCAP",
        )]),
        states: AssertedSet::unknown(
            "external blueos-recorder binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(vec![Rationaled::new(
            Edge {
                from: ServiceId("recorder".to_string()),
                to: ServiceId("recorder_extractor".to_string()),
                via: Bus::File,
                sync: SyncMode::Async,
                endpoint: "/usr/blueos/userdata/recorder".to_string(),
                purpose: "produce MCAP session recordings consumed by recorder_extractor for MP4 extraction and gallery serving"
                    .to_string(),
                required_at_boot: false,
                failure_impact: FailureImpact::Degraded,
            },
            "observed SharedWrite on /usr/blueos/userdata/recorder pairs with recorder_extractor File ReadWrite interface on the same path; producer-to-consumer filesystem coupling via Bus::File",
        )]),
        resources: AssertedSet::established(vec![Rationaled::new(
            Resource {
                path: PathRef("/usr/blueos/userdata/recorder".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            "observed --recorder-path; blueos-recorder writes MCAP session recordings into the shared recorder directory",
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
                    ServiceId("filebrowser".to_string()),
                    ServiceId("versionchooser".to_string()),
                    ServiceId("pardal".to_string()),
                    ServiceId("ping".to_string()),
                    ServiceId("user_terminal".to_string()),
                    ServiceId("ttyd".to_string()),
                    ServiceId("nginx".to_string()),
                    ServiceId("bag_of_holding".to_string()),
                ],
                "observed ordered_after in start-blueos-core Normal block",
            ),
            ordered_before: Asserted::established(
                vec![
                    ServiceId("recorder_extractor".to_string()),
                    ServiceId("disk_usage".to_string()),
                    ServiceId("customization".to_string()),
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
            "implicit: process liveness via tmux; no HTTP listener or dedicated health endpoint".to_string(),
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
            "no operator-facing API; background daemon with no nginx route or REST interface".to_string(),
            "no observed interfaces or journeys; recording runs autonomously without caller permission checks",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "recorder_process_down".to_string(),
                "process exit or tmux session loss stops new MCAP session recording; existing files remain on disk",
            ),
            Rationaled::new(
                "recorder_path_unavailable".to_string(),
                "missing or unwritable /usr/blueos/userdata/recorder blocks MCAP file creation",
            ),
            Rationaled::new(
                "disk_full_from_recording".to_string(),
                "continuous MCAP capture can fill userdata storage and block new recordings or contend with other services",
            ),
        ]),
        blast_radius: Asserted::established(
            "no new session MCAP recordings captured; existing recordings and recorder_extractor playback remain available; vehicle flight and MAVLink control unaffected"
                .to_string(),
            "recorder outage is data-capture convenience loss only; recorder_extractor continues serving prior MP4 extractions from existing MCAP files",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream blueos-recorder deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    }
}
