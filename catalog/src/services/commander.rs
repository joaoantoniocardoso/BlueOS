use crate::criticality::CriticalityTier;
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled,
};
use crate::resource::{Resource, ResourceOwnership};
use crate::service::{Authority, ServiceDefinition};
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

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

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("commander".to_string()),
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one commander process on port 9100",
        ),
        bounded_context: Asserted::established(
            "onboard-host-control".to_string(),
            "provisional 2.0 domain: privileged host power, time, firmware, settings reset, and shell command execution for the frontend",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId("reboot_onboard_computer".into()),
                "power menu POST /shutdown with reboot type restarts the companion computer",
            ),
            Rationaled::new(
                JourneyId("shutdown_onboard_computer".into()),
                "power menu POST /shutdown with poweroff type shuts down the companion computer",
            ),
            Rationaled::new(
                JourneyId("sync_system_time".into()),
                "App.vue load posts browser unix time to POST /set_time when drift exceeds five minutes",
            ),
            Rationaled::new(
                JourneyId("enable_legacy_camera_support".into()),
                "video manager toggles POST /raspi_config/camera_legacy then chains reboot journey",
            ),
            Rationaled::new(
                JourneyId("inspect_raspberry_eeprom_bootloader".into()),
                "firmware tab GET /raspi/vcgencmd and GET /raspi/eeprom_update report Pi bootloader state",
            ),
            Rationaled::new(
                JourneyId("update_raspberry_eeprom_bootloader".into()),
                "firmware tab POST /raspi/eeprom_update applies rpi-eeprom-update when versions are stale",
            ),
            Rationaled::new(
                JourneyId("reset_blueos_settings".into()),
                "settings page POST /settings/reset deletes service config while preserving bootstrap",
            ),
            Rationaled::new(
                JourneyId("run_host_command".into()),
                "commander store POST /command/host runs arbitrary privileged shell commands",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::Important,
            "default UX paths for power menu, clock sync, and settings reset depend on commander; MAVLink vehicle control does not",
        ),
        offline_required: Asserted::established(
            true,
            "reboot, shutdown, timedatectl, raspi-config, vcgencmd, and local settings deletion work without internet",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; executes sudo reboot, shutdown, timedatectl, raspi-config, and caller-supplied shell commands",
        ),
        dangerous_operations: AssertedSet::established(vec![
            Rationaled::new(
                DangerousOperation::Reboot,
                "POST /shutdown with reboot type schedules sudo reboot after a five-second delay",
            ),
            Rationaled::new(
                DangerousOperation::Other("shutdown_poweroff".to_string()),
                "POST /shutdown with poweroff type schedules sudo shutdown --poweroff; distinct from reboot and drops all onboard services",
            ),
            Rationaled::new(
                DangerousOperation::SettingsReset,
                "POST /settings/reset deletes service configuration under appdirs.user_config_dir except bootstrap paths",
            ),
            Rationaled::new(
                DangerousOperation::FirmwareFlash,
                "POST /raspi/eeprom_update runs sudo rpi-eeprom-update -a -d to flash Pi bootloader EEPROM firmware",
            ),
            Rationaled::new(
                DangerousOperation::Other("arbitrary_host_command".to_string()),
                "POST /command/host executes untrusted caller-supplied shell via run_command as root",
            ),
        ]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "dangerous_operations non-empty; REST routes gate destructive ops behind i_know_what_i_am_doing=true",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("reboot_onboard_computer".to_string()),
                "POST /shutdown with ShutdownType.REBOOT schedules companion-computer reboot",
            ),
            Rationaled::new(
                CapabilityId("shutdown_onboard_computer".to_string()),
                "POST /shutdown with ShutdownType.POWEROFF schedules companion-computer poweroff",
            ),
            Rationaled::new(
                CapabilityId("sync_system_time".to_string()),
                "POST /set_time adjusts system clock via timedatectl when browser time drifts beyond five minutes",
            ),
            Rationaled::new(
                CapabilityId("configure_legacy_camera".to_string()),
                "GET/POST /raspi_config/camera_legacy reads and sets raspi-config legacy camera mode",
            ),
            Rationaled::new(
                CapabilityId("inspect_raspberry_eeprom".to_string()),
                "GET /raspi/vcgencmd and GET /raspi/eeprom_update report Pi firmware, bootloader, and EEPROM update status",
            ),
            Rationaled::new(
                CapabilityId("update_raspberry_eeprom".to_string()),
                "POST /raspi/eeprom_update applies available Pi EEPROM and USB-controller firmware updates",
            ),
            Rationaled::new(
                CapabilityId("reset_blueos_settings".to_string()),
                "POST /settings/reset deletes BlueOS service settings while preserving bootstrap and ardupilot-manager state",
            ),
            Rationaled::new(
                CapabilityId("run_host_command".to_string()),
                "POST /command/host runs a caller-supplied shell command and returns stdout, stderr, and return code",
            ),
            Rationaled::new(
                CapabilityId("setup_ssh".to_string()),
                "startup setup_ssh generates /root/.config/.ssh keys and appends the public key to the SSH user's authorized_keys",
            ),
        ]),
        authorities: AssertedSet::established(vec![
            Rationaled::new(
                Authority::Other("host_power_controller".to_string()),
                "sole REST surface for sudo reboot and shutdown --poweroff; no other cataloged service exposes onboard power control",
            ),
            Rationaled::new(
                Authority::Other("host_command_executor".to_string()),
                "sole POST /command/host arbitrary shell executor consumed by the frontend commander store",
            ),
        ]),
        states: AssertedSet::unknown(
            "no cataloged state machine; shutdown scheduling and SSH key setup are one-shot request or boot-time side effects",
        ),
        edges: AssertedSet::established(vec![]),
        resources: AssertedSet::established(vec![
            Rationaled::new(
                Resource {
                    path: PathRef("/var/logs/blueos".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "POST /services/remove_log and remove_log_stream delete files under the BlueOS log folder",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/shortcuts/ardupilot_logs/logs/".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "POST /services/remove_mavlink_log deletes MAVLink log files under the mavlink log folder",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/.ssh".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "setup_ssh at boot creates and stores the container SSH key pair",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/home/{user}/.ssh/authorized_keys".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "setup_ssh appends the generated public key to the SSH user's authorized_keys",
            ),
        ]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                vec!["start-blueos-core create_service".to_string()],
                "observed lifecycle trigger: tmux creation at boot in SERVICES tier",
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
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                vec![
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
                "observed ordered_before lists commander before helper and remaining SERVICES-tier peers",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit on process termination".to_string(),
                "main.py awaits server.serve with no explicit shutdown hook beyond uvicorn exit",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight reboot scheduling and SSH key persistence not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns HTML title; setup_ssh runs once at boot".to_string(),
            "no dedicated /health route; uvicorn availability and boot-time SSH setup serve as health signals",
        ),
        is_platform: Asserted::established(
            false,
            "executes host maintenance commands but does not install or run third-party extension containers",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1.0 router exposed under /commander/ via VersionedFastAPI",
        ),
        permissions_model: Asserted::established(
            "no separate permissions manifest; destructive REST routes require i_know_what_i_am_doing=true".to_string(),
            "check_what_i_am_doing rejects requests without explicit operator acknowledgment",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "i_know_what_i_am_doing_rejected".to_string(),
                "check_what_i_am_doing returns HTTP 400 when the acknowledgment query param is false",
            ),
            Rationaled::new(
                "host_command_subprocess_failure".to_string(),
                "command_host returns non-zero return_code and stderr when run_command fails",
            ),
            Rationaled::new(
                "raspi_config_legacy_failure".to_string(),
                "raspi_config_camera_legacy routes return HTTP 400 when raspi-config subprocess exits non-zero",
            ),
            Rationaled::new(
                "ssh_setup_failure".to_string(),
                "setup_ssh logs errors and continues when key generation or authorized_keys write fails",
            ),
            Rationaled::new(
                "settings_reset_partial_failure".to_string(),
                "delete_everything during settings reset may leave some config paths if deletion raises",
            ),
        ]),
        blast_radius: Asserted::established(
            "reboot or poweroff takes down the entire companion computer and all BlueOS services; MAVLink on the FC may continue but onboard UI, logging, and routing stop".to_string(),
            "host power control can remove every onboard service at once; not vehicle-critical for live FC control but high operational impact",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and shutdown hold-time semantics not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
