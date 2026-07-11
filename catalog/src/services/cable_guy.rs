use crate::id::{PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
use crate::lifecycle::ObservedLifecycle;
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{Evidence, Evidenced, Observed, ObservedSet};
use crate::resource::{Resource, ResourceOwnership};

pub fn observed_facts() -> ObservedFacts {
    ObservedFacts {
        id: ServiceId("cable_guy".to_string()),
        aliases: ObservedSet::known(vec![
            Evidenced::new(
                "cable_guy".to_string(),
                Evidence {
                    file: "core/start-blueos-core".to_string(),
                    line: 119,
                },
            ),
            Evidenced::new(
                "cable-guy".to_string(),
                Evidence {
                    file: "core/services/cable_guy/config.py".to_string(),
                    line: 6,
                },
            ),
        ]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 119,
            },
        ),
        entrypoint: Observed::known(
            "$SERVICES_PATH/cable_guy/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 119,
            },
        ),
        tmux_name: Observed::known(
            "cable_guy".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 119,
            },
        ),
        startup_tier: Observed::known(
            StartupTier::Priority,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 117,
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
                line: 119,
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/services/cable_guy/main.py".to_string(),
                line: 193,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![Evidenced::new(
            PathRef("/cable-guy/".to_string()),
            Evidence {
                file: "core/tools/nginx/nginx.conf".to_string(),
                line: 103,
            },
        )]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(9090),
            Evidence {
                file: "core/services/cable_guy/main.py".to_string(),
                line: 183,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/cable_guy".to_string()),
            Evidence {
                file: "core/services/cable_guy/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/cable-guy/".to_string()),
                    port: PortRef::Literal(9090),
                    versions: vec!["v1.0".to_string()],
                },
                Evidence {
                    file: "core/services/cable_guy/main.py".to_string(),
                    line: 163,
                },
            ),
            Evidenced::new(
                Interface::Settings {
                    path: PathRef("/root/.config/cable-guy/settings-2.json".to_string()),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pydantic_manager.py"
                        .to_string(),
                    line: 73,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/cable-guy".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pydantic_manager.py"
                        .to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/cable-guy/settings-1.json".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/cable_guy/api/settings.py".to_string(),
                    line: 40,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/root/.config/cable-guy/settings.json".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/cable_guy/api/settings.py".to_string(),
                    line: 47,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/etc/resolv.conf".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 9,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/etc/dhcpcd.conf".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/cable_guy/networksetup.py".to_string(),
                    line: 254,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/var/lib/dnsmasq".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/DHCPServerManager.py".to_string(),
                    line: 50,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "cat '{filename}'".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 56,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "lsattr {filename}".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 64,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sudo chattr -i {filename}".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 80,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "echo '{content}' | sudo tee {filename}".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 86,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "sudo chattr +i {filename}".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 74,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "timeout 5 dhclient -d -v {interface_name} 2>&1 || echo 'timeout'".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/networksetup.py".to_string(),
                    line: 124,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ifmetric".to_string(),
                },
                Evidence {
                    file: "core/services/cable_guy/api/manager.py".to_string(),
                    line: 605,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "dnsmasq".to_string(),
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/DHCPServerManager.py".to_string(),
                    line: 95,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/cable-guy/log".to_string()],
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
                    path: PathRef("/root/.config/cable-guy".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/settings/managers/pydantic_manager.py"
                        .to_string(),
                    line: 27,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/cable-guy/settings-2.json".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/cable_guy/api/manager.py".to_string(),
                    line: 60,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/etc/resolv.conf".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/cable_guy/api/dns.py".to_string(),
                    line: 86,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/etc/dhcpcd.conf".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/cable_guy/networksetup.py".to_string(),
                    line: 355,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/var/lib/dnsmasq".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/DHCPServerManager.py".to_string(),
                    line: 79,
                },
            ),
        ]),
        lifecycle: Observed::known(
            ObservedLifecycle {
                triggers: vec!["start-blueos-core create_service".to_string()],
                ordered_after: vec![ServiceId("autopilot".to_string())],
                ordered_before: vec![
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
                    ServiceId("disk_usage".to_string()),
                    ServiceId("customization".to_string()),
                ],
            },
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 318,
            },
        ),
        logs_path: Observed::unknown(
            "init_logger publishes to zenoh only; no on-disk log path set in cable_guy source",
        ),
        zenoh_log_topic: Observed::known(
            "services/cable-guy/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/cable_guy/main.py".to_string(),
                line: 181,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}
