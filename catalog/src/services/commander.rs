use crate::id::{PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
use crate::lifecycle::ObservedLifecycle;
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{Evidence, Evidenced, Observed, ObservedSet};
use crate::resource::{Resource, ResourceOwnership};

pub fn observed_facts() -> ObservedFacts {
    ObservedFacts {
        id: ServiceId("commander".to_string()),
        aliases: ObservedSet::known(vec![Evidenced::new(
            "commander".to_string(),
            Evidence {
                file: "core/services/commander/main.py".to_string(),
                line: 25,
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 132,
            },
        ),
        entrypoint: Observed::known(
            "$SERVICES_PATH/commander/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 132,
            },
        ),
        tmux_name: Observed::known(
            "commander".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 132,
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
                line: 132,
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 132,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/commander/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 108,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(9100),
            Evidence {
                file: "core/services/commander/main.py".to_string(),
                line: 299,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/commander".to_string()),
            Evidence {
                file: "core/services/commander/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/commander/".to_string()),
                    port: PortRef::Literal(9100),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 240,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/var/logs/blueos".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 26,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/shortcuts/ardupilot_logs/logs/".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/.ssh".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 257,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/home/{user}/.ssh/authorized_keys".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 263,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "<caller-supplied host shell command>".to_string(),
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 62,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ssh".to_string(),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/commands.py".to_string(),
                    line: 47,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sshpass".to_string(),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/commands.py".to_string(),
                    line: 21,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ssh-keygen".to_string(),
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 269,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ls".to_string(),
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 296,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sudo timedatectl set-ntp false; sudo date -s '@{unix_time_seconds}'; sudo timedatectl set-ntp true".to_string(),
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 83,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sudo reboot".to_string(),
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 94,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sudo shutdown --poweroff -h now".to_string(),
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 97,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "raspi-config nonint get_legacy".to_string(),
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 104,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sudo raspi-config nonint do_legacy {argument}".to_string(),
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 121,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sudo vcgencmd otp_dump".to_string(),
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 136,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sudo vcgencmd bootloader_version".to_string(),
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 138,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sudo vcgencmd version".to_string(),
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 140,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sudo rpi-eeprom-update".to_string(),
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 154,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sudo rpi-eeprom-update -a -d".to_string(),
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 161,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/commander/log".to_string()],
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
                    path: PathRef("/var/logs/blueos".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 182,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/shortcuts/ardupilot_logs/logs/".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 215,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/.ssh".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 257,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/home/{user}/.ssh/authorized_keys".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py".to_string(),
                    line: 282,
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
                ],
                ordered_before: vec![
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
            "init_logger publishes to zenoh only; no on-disk log path set in commander source",
        ),
        zenoh_log_topic: Observed::known(
            "services/commander/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/commander/main.py".to_string(),
                line: 292,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}
