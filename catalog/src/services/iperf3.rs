use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, PortRef, ServiceId};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, GroundedSet, Observed, ObservedSet,
    Provenance, Rationaled,
};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts};
use crate::service::{Service, ServiceDefinition};
use crate::trust::{PrivilegeLevel, UserConfirmation};

use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_REPODIGEST;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_REPODIGEST;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Iperf3,
        state_contracts: GroundedSet::unknown(
            "iperf3 has no HTTP routes and no service-level state machine; running_baseline TCP :5201 listener health captured in artifact only",
        ),
        slo_baselines: GroundedSet::unknown(
            "iperf3 is a raw TCP iperf bandwidth server on port 5201, not HTTP — no HTTP latency baselines to measure",
        ),
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
                mean: 1.3,
                median: 1.3,
                p95: 1.3,
                min: 1.3,
                max: 1.3,
                sd: 0.0,
            },
            60,
        )]),
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "iperf3 is a platform-independent LAN bandwidth diagnostic server",
                    "runtime captured on Navigator only; idle C binary RSS ~1.3 MB, CPU 0% across 60 samples",
                    "TCP :5201 listener confirmed LISTEN (iperf3 pid=772); no throughput test run",
                ],
            },
            runtime_prov("runtime-captures/iperf3__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "Tier-1 idle sampling and listener confirmation only; no throughput test or settings mutations exercised",
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
        runtime_prov("runtime-captures/iperf3__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts = ObservedFacts {
    id: ServiceId::Iperf3,
    aliases: ObservedSet::unknown(
        "external binary; no in-repo alias declarations in this repository",
    ),
    kind: Observed::known(
        ServiceKind::Binary,
        Evidence {
            file: "core/start-blueos-core",
            line: 135,
        },
    ),
    entrypoint: Observed::known(
        " iperf3 --server --port 5201",
        Evidence {
            file: "core/start-blueos-core",
            line: 135,
        },
    ),
    tmux_name: Observed::known(
        "iperf3",
        Evidence {
            file: "core/start-blueos-core",
            line: 135,
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
            line: 135,
        },
    ),
    nice: Observed::unknown("command line has no nice wrapper"),
    run_as: Observed::known(
        "root",
        Evidence {
            file: "core/start-blueos-core",
            line: 135,
        },
    ),
    nginx_prefixes: ObservedSet::known(&[]),
    listen: ObservedSet::known(&[Evidenced::new(
        PortRef::Literal(5201),
        Evidence {
            file: "core/start-blueos-core",
            line: 135,
        },
    )]),
    git_path: Observed::unknown("external iperf3 binary; no source tree in this repository"),
    interfaces: ObservedSet::known(&[]),
    resources: ObservedSet::unknown(
        "no config or data paths in start-blueos-core launch; stateless bandwidth server",
    ),
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
            ],
            ordered_before: &[
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
        "external iperf3 binary; no --log-path or log file in start-blueos-core launch args",
    ),
    zenoh_log_topic: Observed::unknown(
        "external iperf3 binary; does not use commonwealth init_logger zenoh publisher",
    ),
    sentry: Observed::unknown(
        "external iperf3 binary; no init_sentry or equivalent traced in this repository",
    ),
    openapi_refs: ObservedSet::unknown("not yet extracted"),
};

pub const SERVICE_DEFINITION: ServiceDefinition =
    ServiceDefinition {
        id: ServiceId::Iperf3,
        singleton: Asserted::established(
            true,
            "single Normal-tier tmux instance; one iperf3 --server process listens on port 5201",
        ),
        bounded_context: Asserted::established(
            "network-bandwidth-test",
            "provisional 2.0 domain: raw iperf3 server for external LAN throughput benchmarking",
        ),
        journey_refs: AssertedSet::established(&[]),
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
        dangerous_operations: AssertedSet::established(&[]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "no irreversible, untrusted-code, or vehicle-arm operations; passive bandwidth measurement is a reversible transient side effect only",
        ),
        capabilities: AssertedSet::established(&[Rationaled::new(
            CapabilityId::ServeIperfBandwidthTest,
            "observed entrypoint iperf3 --server --port 5201; external iperf3 -c clients connect to the raw TCP listener for LAN throughput measurement",
        )]),
        authorities: AssertedSet::established(&[]),
        states: AssertedSet::unknown(
            "external iperf3 binary; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(&[]),
        resources: AssertedSet::unknown(
            "no config or data paths in start-blueos-core launch; stateless bandwidth server",
        ),
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
                ],
                "observed ordered_after in start-blueos-core Normal block",
            ),
            ordered_before: Asserted::established(
                &[
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
                    ServiceId::DiskUsage,
                    ServiceId::Customization,
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
            "implicit: process liveness via tmux; TCP listener on port 5201 serves as health signal",
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
            "no auth layer; external iperf3 clients connect directly to port 5201 on the LAN",
            "not proxied by nginx; no observed permission checks — LAN trust model for raw iperf protocol access",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "iperf3_process_down",
                "process exit or tmux session loss stops the iperf3 server and blocks external bandwidth benchmarks",
            ),
            Rationaled::new(
                "listener_unavailable",
                "port 5201 bind failure or firewall blocks iperf3 -c client connections from the LAN",
            ),
            Rationaled::new(
                "active_test_link_saturation",
                "concurrent iperf3 throughput tests can transiently saturate LAN bandwidth and contend with live video or telemetry",
            ),
        ]),
        blast_radius: Asserted::established(
            "external iperf3 bandwidth benchmarking unavailable; vehicle flight and MAVLink control remain intact"
                ,
            "iperf3 outage is diagnostic convenience loss only; active tests may momentarily contend with operator links but do not mutate vehicle configuration",
        ),
        compatibility_policy: Asserted::unknown(
            "upstream iperf3 deprecation policy not established from this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for external binary"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Iperf3,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
