use crate::id::{PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
use crate::lifecycle::ObservedLifecycle;
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{Evidence, Evidenced, Observed, ObservedSet};
use crate::resource::{Resource, ResourceOwnership};

pub fn observed_facts() -> ObservedFacts {
    ObservedFacts {
        id: ServiceId("bag_of_holding".to_string()),
        aliases: ObservedSet::known(vec![Evidenced::new(
            "bag-of-holding".to_string(),
            Evidence {
                file: "core/services/bag_of_holding/main.py".to_string(),
                line: 21,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 144,
            },
        ),
        entrypoint: Observed::known(
            "$SERVICES_PATH/bag_of_holding/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 144,
            },
        ),
        tmux_name: Observed::known(
            "bag_of_holding".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 144,
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
                line: 144,
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 144,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/bag/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 86,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(9101),
            Evidence {
                file: "core/services/bag_of_holding/main.py".to_string(),
                line: 124,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/bag_of_holding".to_string()),
            Evidence {
                file: "core/services/bag_of_holding/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/bag/".to_string()),
                    port: PortRef::Literal(9101),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/bag_of_holding/main.py".to_string(),
                    line: 105,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/bag-of-holding".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/bag_of_holding/main.py".to_string(),
                    line: 22,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/bag-of-holding/log".to_string()],
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
                path: PathRef("/root/.config/bag-of-holding".to_string()),
                ownership: ResourceOwnership::SharedWrite,
            },
            Evidence {
                file: "core/services/bag_of_holding/main.py".to_string(),
                line: 61,
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
                ],
                ordered_before: vec![
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
            "init_logger publishes to zenoh only; no on-disk log path set in bag_of_holding source",
        ),
        zenoh_log_topic: Observed::known(
            "services/bag-of-holding/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/bag_of_holding/main.py".to_string(),
                line: 121,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}
