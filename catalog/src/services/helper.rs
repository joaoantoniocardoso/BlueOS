use crate::id::{PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
use crate::lifecycle::ObservedLifecycle;
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{Evidence, Evidenced, Observed, ObservedSet};
use crate::resource::{Resource, ResourceOwnership};

pub fn observed_facts() -> ObservedFacts {
    ObservedFacts {
        id: ServiceId("helper".to_string()),
        aliases: ObservedSet::known(vec![Evidenced::new(
            "helper".to_string(),
            Evidence {
                file: "core/services/helper/main.py".to_string(),
                line: 43,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 134,
            },
        ),
        entrypoint: Observed::known(
            "$BLUEOS_PYTHON_BIN_SECONDARY $SERVICES_PATH/helper/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 134,
            },
        ),
        tmux_name: Observed::known(
            "helper".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 134,
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
                line: 134,
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 134,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/helper/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 124,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(81),
            Evidence {
                file: "core/services/helper/main.py".to_string(),
                line: 143,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/helper".to_string()),
            Evidence {
                file: "core/services/helper/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/helper/".to_string()),
                    port: PortRef::Literal(81),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 612,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/home/pi/tools/nginx/nginx.conf".to_string()),
                    mode: FileAccessMode::Read,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 629,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/home/pi/tools/nginx/extensions/".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 491,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/var/run/nginx.pid".to_string()),
                    mode: FileAccessMode::Read,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 453,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/etc/blueos/hardware-uuid".to_string()),
                    mode: FileAccessMode::Read,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 553,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/etc/blueos/uuid".to_string()),
                    mode: FileAccessMode::Read,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 570,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "127.0.0.1".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 294,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "http://localhost/version-chooser/v1.0/version/current".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 507,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "localhost:6040".to_string(),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/mavlink_comm/MavlinkComm.py"
                        .to_string(),
                    line: 24,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "firmware.ardupilot.org".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 56,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "amazon.com".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 61,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "telemetry.blueos.cloud".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 66,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "1.1.1.1".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 74,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "github.com".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 79,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ping".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 586,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "kill -HUP".to_string(),
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 456,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/helper/log".to_string()],
                    topics_consumed: vec![],
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                    line: 78,
                },
            ),
        ]),
        resources: ObservedSet::known(vec![
            Evidenced::new(
                Resource {
                    path: PathRef("/home/pi/tools/nginx/nginx.conf".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 629,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/home/pi/tools/nginx/extensions/".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 491,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/var/run/nginx.pid".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 453,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/etc/blueos/hardware-uuid".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 553,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/etc/blueos/uuid".to_string()),
                    ownership: ResourceOwnership::SharedRead,
                },
                Evidence {
                    file: "core/services/helper/main.py".to_string(),
                    line: 570,
                },
            ),
        ]),
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
                ],
                ordered_before: vec![
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
            "init_logger publishes to zenoh only; no on-disk log path set in helper source",
        ),
        zenoh_log_topic: Observed::known(
            "services/helper/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/helper/main.py".to_string(),
                line: 633,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}
