use crate::id::{PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
use crate::lifecycle::ObservedLifecycle;
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{Evidence, Evidenced, Observed, ObservedSet};
use crate::resource::{Resource, ResourceOwnership};

pub fn observed_facts() -> ObservedFacts {
    ObservedFacts {
        id: ServiceId("versionchooser".to_string()),
        aliases: ObservedSet::known(vec![
            Evidenced::new(
                "versionchooser".to_string(),
                Evidence {
                    file: "core/start-blueos-core".to_string(),
                    line: 138,
                },
            ),
            Evidenced::new(
                "version-chooser".to_string(),
                Evidence {
                    file: "core/services/versionchooser/main.py".to_string(),
                    line: 12,
                },
            ),
        ]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 138,
            },
        ),
        entrypoint: Observed::known(
            "$BLUEOS_PYTHON_BIN_SECONDARY $SERVICES_PATH/versionchooser/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 138,
            },
        ),
        tmux_name: Observed::known(
            "versionchooser".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 138,
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
                line: 138,
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 138,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/version-chooser/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 220,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(8081),
            Evidence {
                file: "core/services/versionchooser/args.py".to_string(),
                line: 26,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/versionchooser".to_string()),
            Evidence {
                file: "core/services/versionchooser/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/version-chooser/".to_string()),
                    port: PortRef::Literal(8081),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/versionchooser/api/app.py".to_string(),
                    line: 30,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "https://index.docker.io".to_string(),
                },
                Evidence {
                    file: "core/services/versionchooser/utils/dockerhub.py".to_string(),
                    line: 49,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "https://hub.docker.com/".to_string(),
                },
                Evidence {
                    file: "core/services/versionchooser/utils/dockerhub.py".to_string(),
                    line: 50,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "https://auth.docker.io".to_string(),
                },
                Evidence {
                    file: "core/services/versionchooser/utils/dockerhub.py".to_string(),
                    line: 99,
                },
            ),
            Evidenced::new(
                Interface::Settings {
                    path: PathRef("/root/.config/bootstrap/startup.json".to_string()),
                },
                Evidence {
                    file: "core/services/versionchooser/utils/chooser.py".to_string(),
                    line: 16,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/var/run/docker.sock".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/api/v1/routers/version.py".to_string(),
                    line: 23,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.docker/config.json".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/docker_login.py".to_string(),
                    line: 14,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/home/pi/.docker/config.json".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/docker_login.py".to_string(),
                    line: 13,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/version-chooser/log".to_string()],
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
                    path: PathRef("/var/run/docker.sock".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/api/v1/routers/version.py".to_string(),
                    line: 23,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/bootstrap/startup.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/utils/chooser.py".to_string(),
                    line: 16,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.docker/config.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/docker_login.py".to_string(),
                    line: 14,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/home/pi/.docker/config.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/versionchooser/docker_login.py".to_string(),
                    line: 13,
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
                    ServiceId("helper".to_string()),
                    ServiceId("iperf3".to_string()),
                    ServiceId("linux2rest".to_string()),
                    ServiceId("filebrowser".to_string()),
                ],
                ordered_before: vec![
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
            "init_logger publishes to zenoh only; no on-disk log path set in versionchooser source",
        ),
        zenoh_log_topic: Observed::known(
            "services/version-chooser/log".to_string(),
            Evidence {
                file: "core/services/versionchooser/main.py".to_string(),
                line: 12,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/versionchooser/main.py".to_string(),
                line: 21,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}
