use crate::id::{PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
use crate::lifecycle::ObservedLifecycle;
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{Evidence, Evidenced, Observed, ObservedSet};
use crate::resource::{Resource, ResourceOwnership};

pub fn observed_facts() -> ObservedFacts {
    ObservedFacts {
        id: ServiceId("disk_usage".to_string()),
        aliases: ObservedSet::known(vec![Evidenced::new(
            "disk-usage".to_string(),
            Evidence {
                file: "core/services/disk_usage/main.py".to_string(),
                line: 26,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 147,
            },
        ),
        entrypoint: Observed::known(
            "$SERVICES_PATH/disk_usage/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 147,
            },
        ),
        tmux_name: Observed::known(
            "disk_usage".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 147,
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
                line: 147,
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 147,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/disk-usage/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 139,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(9151),
            Evidence {
                file: "core/services/disk_usage/main.py".to_string(),
                line: 28,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/disk_usage".to_string()),
            Evidence {
                file: "core/services/disk_usage/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/disk-usage/".to_string()),
                    port: PortRef::Literal(9151),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/disk_usage/main.py".to_string(),
                    line: 469,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/disk_usage/main.py".to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "du".to_string(),
                },
                Evidence {
                    file: "core/services/disk_usage/main.py".to_string(),
                    line: 211,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "disktest".to_string(),
                },
                Evidence {
                    file: "core/services/disk_usage/main.py".to_string(),
                    line: 319,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/disk-usage/log".to_string()],
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
                path: PathRef("/".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/disk_usage/main.py".to_string(),
                line: 281,
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
                    ServiceId("recorder".to_string()),
                    ServiceId("recorder_extractor".to_string()),
                ],
                ordered_before: vec![ServiceId("customization".to_string())],
            },
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 326,
            },
        ),
        logs_path: Observed::unknown(
            "init_logger publishes to zenoh only; no on-disk log path set in disk_usage source",
        ),
        zenoh_log_topic: Observed::known(
            "services/disk-usage/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/disk_usage/main.py".to_string(),
                line: 484,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}
