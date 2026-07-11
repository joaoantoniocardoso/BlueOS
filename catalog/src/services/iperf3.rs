use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, PortRef, ServiceId};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts};
use crate::service::ServiceDefinition;
use crate::trust::{PrivilegeLevel, UserConfirmation};

const RUNTIME_CAPTURE: &str = "runtime-captures/iperf3__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e; RepoDigest sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId("iperf3".into()),
        state_contracts: GroundedSet::unknown(
            "iperf3 has no HTTP routes and no service-level state machine; running_baseline TCP :5201 listener health captured in artifact only",
        ),
        slo_baselines: GroundedSet::unknown(
            "iperf3 is a raw TCP iperf bandwidth server on port 5201, not HTTP — no HTTP latency baselines to measure",
        ),
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
                mean: 1.3,
                median: 1.3,
                p95: 1.3,
                min: 1.3,
                max: 1.3,
                sd: 0.0,
            },
            60,
        )]),
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "iperf3 is a platform-independent LAN bandwidth diagnostic server".into(),
                    "runtime captured on Navigator only; idle C binary RSS ~1.3 MB, CPU 0% across 60 samples".into(),
                    "TCP :5201 listener confirmed LISTEN (iperf3 pid=772); no throughput test run".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "Tier-1 idle sampling and listener confirmation only; no throughput test or settings mutations exercised",
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
        id: ServiceId("iperf3".to_string()),
        aliases: ObservedSet::unknown(
            "external binary; no in-repo alias declarations in this repository",
        ),
        kind: Observed::known(
            ServiceKind::Binary,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 135,
            },
        ),
        entrypoint: Observed::known(
            " iperf3 --server --port 5201".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 135,
            },
        ),
        tmux_name: Observed::known(
            "iperf3".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 135,
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
                line: 135,
            },
        ),
        nice: Observed::unknown("command line has no nice wrapper"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 135,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(5201),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 135,
            },
        )]),
        git_path: Observed::unknown("external iperf3 binary; no source tree in this repository"),
        interfaces: ObservedSet::known(vec![]),
        resources: ObservedSet::unknown(
            "no config or data paths in start-blueos-core launch; stateless bandwidth server",
        ),
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
                ],
                ordered_before: vec![
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
            "external iperf3 binary; no --log-path or log file in start-blueos-core launch args",
        ),
        zenoh_log_topic: Observed::unknown(
            "external iperf3 binary; does not use commonwealth init_logger zenoh publisher",
        ),
        sentry: Observed::unknown(
            "external iperf3 binary; no init_sentry or equivalent traced in this repository",
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("iperf3".to_string()),
        singleton: Asserted::established(
            true,
            "single Normal-tier tmux instance; one iperf3 --server process listens on port 5201",
        ),
        bounded_context: Asserted::established(
            "network-bandwidth-test".to_string(),
            "provisional 2.0 domain: raw iperf3 server for external LAN throughput benchmarking",
        ),
        journey_refs: AssertedSet::established(vec![]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "Normal-tier diagnostic bandwidth server; vehicle flight and MAVLink control do not depend on iperf3 throughput measurement",
        ),
        offline_required: Asserted::established(
            true,
            "LAN iperf3 client-to-server measurement uses local network only; no internet or WAN dependency",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root in start-blueos-core Normal-tier launch line; iperf3 does not strictly require root but is launched as root",
        ),
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "no irreversible, untrusted-code, or vehicle-arm operations; passive bandwidth measurement is a reversible transient side effect only",
        ),
        capabilities: AssertedSet::established(vec![Rationaled::new(
            CapabilityId("serve_iperf_bandwidth_test".to_string()),
            "observed entrypoint iperf3 --server --port 5201; external iperf3 -c clients connect to the raw TCP listener for LAN throughput measurement",
        )]),
        authorities: AssertedSet::established(vec![]),
        states: AssertedSet::unknown(
            "external iperf3 binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(vec![]),
        resources: AssertedSet::unknown(
            "no config or data paths in start-blueos-core launch; stateless bandwidth server",
        ),
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
                ],
                "observed ordered_after in start-blueos-core Normal block",
            ),
            ordered_before: Asserted::established(
                vec![
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
                    ServiceId("disk_usage".to_string()),
                    ServiceId("customization".to_string()),
                ],
                "observed ordered_before lists iperf3 before linux2rest and remaining Normal and SERVICES-tier peers",
            ),
            shutdown: Asserted::unknown(
                "external iperf3 binary; no explicit shutdown handler traced in this repository",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade restart semantics for the external iperf3 binary not traced in this repository",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; TCP listener on port 5201 serves as health signal".to_string(),
            "no dedicated /health route; iperf3 process continuity and port 5201 listener availability serve as health signal",
        ),
        is_platform: Asserted::established(
            false,
            "diagnostic bandwidth server; does not install or host third-party extensions",
        ),
        api_stable: Asserted::unknown(
            "external binary; upstream iperf3 wire protocol stability not established from this repository",
        ),
        permissions_model: Asserted::established(
            "no auth layer; external iperf3 clients connect directly to port 5201 on the LAN".to_string(),
            "not proxied by nginx; no observed permission checks — LAN trust model for raw iperf protocol access",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "iperf3_process_down".to_string(),
                "process exit or tmux session loss stops the iperf3 server and blocks external bandwidth benchmarks",
            ),
            Rationaled::new(
                "listener_unavailable".to_string(),
                "port 5201 bind failure or firewall blocks iperf3 -c client connections from the LAN",
            ),
            Rationaled::new(
                "active_test_link_saturation".to_string(),
                "concurrent iperf3 throughput tests can transiently saturate LAN bandwidth and contend with live video or telemetry",
            ),
        ]),
        blast_radius: Asserted::established(
            "external iperf3 bandwidth benchmarking unavailable; vehicle flight and MAVLink control remain intact"
                .to_string(),
            "iperf3 outage is diagnostic convenience loss only; active tests may momentarily contend with operator links but do not mutate vehicle configuration",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream iperf3 deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    }
}
