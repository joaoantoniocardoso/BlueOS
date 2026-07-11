use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, ServiceId};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, GroundedItem, GroundedSet, Observed, ObservedSet, Provenance,
    Rationaled,
};
use crate::runtime::{Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts};
use crate::service::ServiceDefinition;
use crate::trust::{PrivilegeLevel, UserConfirmation};

const RUNTIME_CAPTURE: &str = "runtime-captures/user_terminal__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e; RepoDigest sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a), Raspberry Pi 4, Navigator";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId::UserTerminal,
        state_contracts: GroundedSet::unknown(
            "user_terminal has no HTTP routes and no service-level state machine; tmux session liveness captured in artifact only",
        ),
        slo_baselines: GroundedSet::unknown(
            "user_terminal has no HTTP interface — no nginx route or listen port; no HTTP latency baselines to measure",
        ),
        resource_usage: GroundedSet::known(vec![runtime_resource(
            "idle_bash",
            Distribution {
                mean: 0.0,
                median: 0.0,
                p95: 0.0,
                min: 0.0,
                max: 0.0,
                sd: 0.0,
            },
            Distribution {
                mean: 2.5,
                median: 2.5,
                p95: 2.5,
                min: 2.5,
                max: 2.5,
                sd: 0.0,
            },
            60,
        )]),
        platform_matrix: GroundedSet::known(vec![GroundedItem::new(
            PlatformBehavior {
                platform: "navigator".into(),
                firmware: None,
                notes: vec![
                    "user_terminal is a platform-independent persistent root shell tmux session".into(),
                    "runtime captured on Navigator only; idle bash pane RSS ~2.5 MB, CPU 0% across 60 samples".into(),
                    "tmux session user_terminal alive since boot; pane PID 1040 runs idle -bash; no HTTP interface".into(),
                ],
            },
            runtime_prov("#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "Tier-1 read-only external observation only; no shell interaction, keystrokes, or settings mutations exercised",
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
        id: ServiceId::UserTerminal,
        aliases: ObservedSet::unknown(
            "no in-repo alias declarations; tmux session launched by shell invocation only",
        ),
        kind: Observed::known(
            ServiceKind::Shell,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 141,
            },
        ),
        entrypoint: Observed::known(
            "cat /etc/motd".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 141,
            },
        ),
        tmux_name: Observed::known(
            "user_terminal".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 141,
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
                memory_mb: Some(0),
                cpu_percent: Some(0),
                io_weight: None,
            },
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 141,
            },
        ),
        nice: Observed::unknown("command line has no nice wrapper"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 141,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![]),
        listen: ObservedSet::known(vec![]),
        git_path: Observed::unknown(
            "no in-repo program source; entrypoint is coreutils cat shell invocation",
        ),
        interfaces: ObservedSet::known(vec![]),
        resources: ObservedSet::known(vec![]),
        lifecycle: Observed::known(
            ObservedLifecycle {
                triggers: vec!["start-blueos-core create_service".to_string()],
                ordered_after: vec![
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
                ],
                ordered_before: vec![
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
                file: "core/start-blueos-core".to_string(),
                line: 326,
            },
        ),
        logs_path: Observed::unknown(
            "shell tmux session; no --log-path or log file in start-blueos-core launch args",
        ),
        zenoh_log_topic: Observed::unknown(
            "shell tmux session; does not use commonwealth init_logger zenoh publisher",
        ),
        sentry: Observed::unknown(
            "shell tmux session; no init_sentry or equivalent traced in this repository",
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId::UserTerminal,
        singleton: Asserted::established(
            true,
            "single Normal-tier tmux session named user_terminal; one persistent interactive root shell",
        ),
        bounded_context: Asserted::established(
            "interactive-shell".to_string(),
            "provisional 2.0 domain: persistent root tmux session for interactive host-shell access inside the core container",
        ),
        journey_refs: AssertedSet::established(vec![]),
        tier: Asserted::established(
            CriticalityTier::Auxiliary,
            "debug/convenience shell; vehicle flight and MAVLink control operate fully without this tmux session — operators can SSH or use ttyd which recreates the session on attach",
        ),
        offline_required: Asserted::established(
            true,
            "local tmux shell session inside the core container; no internet or WAN dependency",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root in start-blueos-core Normal-tier launch line; session runs as root inside the container",
        ),
        dangerous_operations: AssertedSet::established(vec![]),
        user_confirmation: Asserted::established(
            UserConfirmation::NotRequired,
            "no user-facing operation surface on this service; arbitrary_root_shell_execution danger is modeled on ttyd where operators actually open the web terminal (rubric pairs dangerous_operations with user-facing gates — not double-counted here)",
        ),
        capabilities: AssertedSet::established(vec![Rationaled::new(
            CapabilityId::ProvideInteractiveRootShell,
            "observed Shell kind with tmux session user_terminal; persistent interactive root shell created at boot (entrypoint cat /etc/motd then shell)",
        )]),
        authorities: AssertedSet::established(vec![]),
        states: AssertedSet::unknown(
            "shell tmux session; no in-repo state machine or lifecycle states traced",
        ),
        edges: AssertedSet::established(vec![]),
        resources: AssertedSet::established(vec![]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                vec!["start-blueos-core create_service".to_string()],
                "observed lifecycle trigger: tmux creation at boot in Normal tier",
            ),
            ordered_after: Asserted::established(
                vec![
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
                ],
                "observed ordered_after in start-blueos-core Normal block",
            ),
            ordered_before: Asserted::established(
                vec![
                    ServiceId::Ttyd,
                    ServiceId::Nginx,
                    ServiceId::BagOfHolding,
                    ServiceId::Recorder,
                    ServiceId::RecorderExtractor,
                    ServiceId::DiskUsage,
                    ServiceId::Customization,
                ],
                "observed ordered_before lists user_terminal before ttyd and remaining Normal and SERVICES-tier peers",
            ),
            shutdown: Asserted::unknown(
                "shell tmux session; no explicit shutdown handler traced in this repository",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade restart semantics for the user_terminal tmux session not traced in this repository",
            ),
        },
        health: Asserted::established(
            "implicit: tmux session liveness; user_terminal tmux session existence serves as health signal".to_string(),
            "no dedicated health route; tmux session continuity serves as health signal for the persistent shell",
        ),
        is_platform: Asserted::established(
            false,
            "core infrastructure shell session; does not install or host third-party extensions",
        ),
        api_stable: Asserted::unknown(
            "shell tmux session; no API surface traced in this repository",
        ),
        permissions_model: Asserted::established(
            "no auth layer; direct tmux attach only — not operator-facing without ttyd or SSH".to_string(),
            "no observed permission checks on the tmux session itself; access is gated by container/host access control outside this service",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "user_terminal_tmux_down".to_string(),
                "tmux session exit or loss removes the boot-created persistent shell; ttyd recreates a session on attach via tmux new -s user_terminal",
            ),
            Rationaled::new(
                "motd_banner_lost".to_string(),
                "if the boot session is replaced by ttyd's tmux new fallback, the initial cat /etc/motd banner from the boot entrypoint is not replayed",
            ),
        ]),
        blast_radius: Asserted::established(
            "boot-created tmux session and MOTD banner may be lost; ttyd recreates an interactive shell on attach and vehicle flight control remains intact"
                .to_string(),
            "near-zero operational impact — ttyd self-heals with tmux new -s user_terminal; flight and MAVLink routing unaffected",
        ),
        compatibility_policy: Asserted::unknown(
            "shell tmux session; no versioned compatibility surface traced in this repository",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found for shell tmux session"),
    }
}
