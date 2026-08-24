use catalog_kernel::criticality::CriticalityTier;
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::refs::{PathRef, PortRef};
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::GroundedSet;
use catalog_kernel::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, Observed, ObservedSet, Provenance,
    Rationaled,
};
use catalog_model::interface::{FileAccessMode, PortKind};
use catalog_model::journey::{HttpMethod, RouteRef};
use catalog_model::lifecycle::{Lifecycle, ObservedLifecycle};
use catalog_model::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use catalog_model::resource::{Resource, ResourceOwnership};
use catalog_model::runtime::{
    Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SloBaseline,
};
use catalog_model::service::{Authority, Service, ServiceJudgment};
use catalog_model::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

use catalog_kernel::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

pub const RUNTIME_FACTS: RuntimeFacts =
    RuntimeFacts {
        service: ServiceId::Commander,
        state_contracts: GroundedSet::unknown(
            "commander has no service-level state machine",
        ),
        slo_baselines: GroundedSet::known(&[
            runtime_slo(
                HttpMethod::Get,
                "/raspi/vcgencmd?i_know_what_i_am_doing=true",
                1918.3,
                1991.4,
                1991.4,
                20,
            ),
            runtime_slo(
                HttpMethod::Get,
                "/raspi/eeprom_update?i_know_what_i_am_doing=true",
                805.9,
                876.3,
                876.3,
                20,
            ),
        ]),
        resource_usage: GroundedSet::known(&[runtime_resource(
            "running_baseline",
            Distribution {
                mean: 1.69,
                median: 2.04,
                p95: 3.17,
                min: 0.00,
                max: 3.21,
                sd: 0.83,
            },
            flat_rss(35.5),
            60,
        )]),
        platform_matrix: GroundedSet::known(&[GroundedItem::new(
            PlatformBehavior {
                platform: "navigator",
                firmware: None,
                notes: &[
                    "raspi/vcgencmd and raspi/eeprom_update are Pi-specific (vcgencmd, rpi-eeprom-update); may error on non-Pi boards",
                    "runtime captured on Navigator only; RSS ~35.5 MB, CPU ~1.69% mean",
                ],
            },
            runtime_prov("runtime-captures/commander__pi4_navigator_master.json#platform_matrix"),
        )]),
        settings_mutations: GroundedSet::unknown(
            "mutating endpoints (settings reset) are destructive and were not exercised; not captured",
        ),
    };

const fn runtime_prov(key: &'static str) -> Provenance {
    Provenance::runtime(key, RUNTIME_ENV)
}

const fn runtime_route(method: HttpMethod, path: &'static str) -> RouteRef {
    RouteRef {
        service: ServiceId::Commander,
        method,
        path,
        version: None,
    }
}

const fn runtime_slo(
    method: HttpMethod,
    path: &'static str,
    p50: f64,
    p95: f64,
    p99: f64,
    sample_size: u32,
) -> GroundedItem<SloBaseline> {
    GroundedItem::new(
        SloBaseline {
            route: runtime_route(method, path),
            latency_p50_ms: p50,
            latency_p95_ms: p95,
            latency_p99_ms: p99,
            sample_size,
        },
        runtime_prov("runtime-captures/commander__pi4_navigator_master.json#slo_running_baseline"),
    )
}

const fn flat_rss(mb: f64) -> Distribution {
    Distribution {
        mean: mb,
        median: mb,
        p95: mb,
        min: mb,
        max: mb,
        sd: 0.0,
    }
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
        runtime_prov("runtime-captures/commander__pi4_navigator_master.json#resource_usage"),
    )
}

pub const OBSERVED_FACTS: ObservedFacts =
    ObservedFacts {
        id: ServiceId::Commander,
        aliases: ObservedSet::known(&[Evidenced::new(
            "commander",
            Evidence {
                file: "core/services/commander/main.py",
                line: 25,
                anchor: "SERVICE_NAME = \"commander\"",
            },
        )]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core",
                line: 132,
                anchor: "'commander',250,0,0,0,\"$SERVICES_PATH/commander/main.py\"",
            },
        ),
        entrypoint: Observed::known(
            "$SERVICES_PATH/commander/main.py",
            Evidence {
                file: "core/start-blueos-core",
                line: 132,
                anchor: "'commander',250,0,0,0,\"$SERVICES_PATH/commander/main.py\"",
            },
        ),
        tmux_name: Observed::known(
            "commander",
            Evidence {
                file: "core/start-blueos-core",
                line: 132,
                anchor: "'commander',250,0,0,0,\"$SERVICES_PATH/commander/main.py\"",
            },
        ),
        startup_tier: Observed::known(
            StartupTier::Normal,
            Evidence {
                file: "core/start-blueos-core",
                line: 124,
                anchor: "SERVICES=(",
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
                line: 132,
                anchor: "'commander',250,0,0,0,\"$SERVICES_PATH/commander/main.py\"",
            },
        ),
        nice: Observed::unknown("no nice prefix in start tuple"),
        run_as: Observed::known(
            "root",
            Evidence {
                file: "core/start-blueos-core",
                line: 132,
                anchor: "'commander',250,0,0,0,\"$SERVICES_PATH/commander/main.py\"",
            },
        ),
        nginx_prefixes: ObservedSet::known(&[Evidenced::new(
            PathRef("/commander/"),
            Evidence {
                file: "core/tools/nginx/nginx.conf",
                line: 108,
                anchor: "location /commander/ {",
            },
        )]),
        listen: ObservedSet::known(&[Evidenced::new(
            PortRef::Literal(9100),
            Evidence {
                file: "core/services/commander/main.py",
                line: 299,
                anchor: "config = Config(app=app, host=\"0.0.0.0\", port=9100, log_conf",
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/commander"),
            Evidence {
                file: "core/services/commander/main.py",
                line: 1,
                anchor: "#! /usr/bin/env python3",
            },
        ),
        interfaces: ObservedSet::known(&[
            Evidenced::new(
                PortKind::Rest {
                    path_prefix: PathRef("/commander/"),
                    port: PortRef::Literal(9100),
                    versions: &["v1.0"],
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 240,
                    anchor: "app = VersionedFastAPI(app, version=\"1.0.0\", prefix_format=\"",
                },
            ),
            Evidenced::new(
                PortKind::File {
                    path: PathRef("/var/logs/blueos"),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 26,
                    anchor: "LOG_FOLDER_PATH = os.environ.get(\"BLUEOS_LOG_FOLDER_PATH\", \"",
                },
            ),
            Evidenced::new(
                PortKind::File {
                    path: PathRef("/shortcuts/ardupilot_logs/logs/"),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 27,
                    anchor: "MAVLINK_LOG_FOLDER_PATH = os.environ.get(\"BLUEOS_MAVLINK_LOG",
                },
            ),
            Evidenced::new(
                PortKind::File {
                    path: PathRef("/root/.config/.ssh"),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 257,
                    anchor: "key_path = Path(\"/root/.config/.ssh\")",
                },
            ),
            Evidenced::new(
                PortKind::File {
                    path: PathRef("/home/{user}/.ssh/authorized_keys"),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 263,
                    anchor: "authorized_keys = Path(f\"/home/{user}/.ssh/authorized_keys\")",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "<caller-supplied host shell command>",
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 62,
                    anchor: "output = run_command(command, False)",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "ssh",
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/commands.py",
                    line: 47,
                    anchor: "\"ssh\",",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "sshpass",
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/commands.py",
                    line: 21,
                    anchor: "\"sshpass\",",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "ssh-keygen",
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 269,
                    anchor: "subprocess.run([\"ssh-keygen\", \"-t\", \"rsa\", \"-f\", private_key",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "ls",
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 296,
                    anchor: "run_command(\"ls\")",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "sudo timedatectl set-ntp false; sudo date -s '@{unix_time_seconds}'; sudo timedatectl set-ntp true",
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 83,
                    anchor: "command = f\"sudo timedatectl set-ntp false; sudo date -s '@{",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "sudo reboot",
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 94,
                    anchor: "output = run_command(f\"(sleep {hold_time_seconds}; sudo rebo",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "sudo shutdown --poweroff -h now",
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 97,
                    anchor: "output = run_command(f\"(sleep {hold_time_seconds}; sudo shut",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "raspi-config nonint get_legacy",
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 104,
                    anchor: "output = await command_host(\"raspi-config nonint get_legacy\"",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "sudo raspi-config nonint do_legacy {argument}",
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 121,
                    anchor: "output = await command_host(f\"sudo raspi-config nonint do_le",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "sudo vcgencmd otp_dump",
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 136,
                    anchor: "output_vl085_firmware_version = await command_host(\"sudo vcg",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "sudo vcgencmd bootloader_version",
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 138,
                    anchor: "output_bootloader_version = await command_host(\"sudo vcgencm",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "sudo vcgencmd version",
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 140,
                    anchor: "output_raspberry_firmware_version = await command_host(\"sudo",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "sudo rpi-eeprom-update",
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 154,
                    anchor: "return await command_host(\"sudo rpi-eeprom-update\", True)",
                },
            ),
            Evidenced::new(
                PortKind::Subprocess {
                    command: "sudo rpi-eeprom-update -a -d",
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 161,
                    anchor: "return await command_host(\"sudo rpi-eeprom-update -a -d\", Tr",
                },
            ),
            Evidenced::new(
                PortKind::Zenoh {
                    topics_produced: &["services/commander/log"],
                    topics_consumed: &[],
                },
                Evidence {
                    file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                    line: 78,
                    anchor: "topic = f\"services/{service_name}/log\"",
                },
            ),
        ]),
        resources: ObservedSet::known(&[
            Evidenced::new(
                Resource {
                    path: PathRef("/var/logs/blueos"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 182,
                    anchor: "delete_everything(Path(LOG_FOLDER_PATH))",
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/shortcuts/ardupilot_logs/logs/"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 215,
                    anchor: "delete_everything(Path(MAVLINK_LOG_FOLDER_PATH))",
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/.ssh"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 257,
                    anchor: "key_path = Path(\"/root/.config/.ssh\")",
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/home/{user}/.ssh/authorized_keys"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/commander/main.py",
                    line: 282,
                    anchor: "authorized_keys.write_text(authorized_keys_text, \"utf-8\")",
                },
            ),
        ]),
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
                ],
                ordered_before: &[
                    ServiceId::NmeaInjector,
                    ServiceId::Helper,
                    ServiceId::Iperf3,
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
                anchor: "for TUPLE in \"${SERVICES[@]}\"; do",
            },
        ),
        logs_path: Observed::unknown(
            "init_logger publishes to zenoh only; no on-disk log path set in commander source",
        ),
        zenoh_log_topic: Observed::known(
            "services/commander/log",
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py",
                line: 78,
                anchor: "topic = f\"services/{service_name}/log\"",
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/commander/main.py",
                line: 292,
                anchor: "await init_sentry_async(SERVICE_NAME)",
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    };

pub const SERVICE_DEFINITION: ServiceJudgment =
    ServiceJudgment {
        id: ServiceId::Commander,
        singleton: Asserted::established(
            true,
            "single SERVICES-tier tmux instance; one commander process on port 9100",
        ),
        bounded_context: Asserted::established(
            "onboard-host-control",
            "provisional 2.0 domain: privileged host power, time, firmware, settings reset, and shell command execution for the frontend",
        ),
        journey_refs: AssertedSet::established(&[
            Rationaled::new(
                JourneyId::RebootOnboardComputer,
                "power menu POST /shutdown with reboot type restarts the companion computer",
            ),
            Rationaled::new(
                JourneyId::ShutdownOnboardComputer,
                "power menu POST /shutdown with poweroff type shuts down the companion computer",
            ),
            Rationaled::new(
                JourneyId::SyncSystemTime,
                "App.vue load posts browser unix time to POST /set_time when drift exceeds five minutes",
            ),
            Rationaled::new(
                JourneyId::EnableLegacyCameraSupport,
                "video manager toggles POST /raspi_config/camera_legacy then chains reboot journey",
            ),
            Rationaled::new(
                JourneyId::InspectRaspberryEepromBootloader,
                "firmware tab GET /raspi/vcgencmd and GET /raspi/eeprom_update report Pi bootloader state",
            ),
            Rationaled::new(
                JourneyId::UpdateRaspberryEepromBootloader,
                "firmware tab POST /raspi/eeprom_update applies rpi-eeprom-update when versions are stale",
            ),
            Rationaled::new(
                JourneyId::ResetBlueosSettings,
                "settings page POST /settings/reset deletes service config while preserving bootstrap",
            ),
            Rationaled::new(
                JourneyId::RunHostCommand,
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
        dangerous_operations: AssertedSet::established(&[
            Rationaled::new(
                DangerousOperation::Reboot,
                "POST /shutdown with reboot type schedules sudo reboot after a five-second delay",
            ),
            Rationaled::new(
                DangerousOperation::Other("shutdown_poweroff"),
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
                DangerousOperation::Other("arbitrary_host_command"),
                "POST /command/host executes untrusted caller-supplied shell via run_command as root",
            ),
        ]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "dangerous_operations non-empty; REST routes gate destructive ops behind i_know_what_i_am_doing=true",
        ),
        capabilities: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::RebootOnboardComputer,
                "POST /shutdown with ShutdownType.REBOOT schedules companion-computer reboot",
            ),
            Rationaled::new(
                CapabilityId::ShutdownOnboardComputer,
                "POST /shutdown with ShutdownType.POWEROFF schedules companion-computer poweroff",
            ),
            Rationaled::new(
                CapabilityId::SyncSystemTime,
                "POST /set_time adjusts system clock via timedatectl when browser time drifts beyond five minutes",
            ),
            Rationaled::new(
                CapabilityId::ConfigureLegacyCamera,
                "GET/POST /raspi_config/camera_legacy reads and sets raspi-config legacy camera mode",
            ),
            Rationaled::new(
                CapabilityId::InspectRaspberryEeprom,
                "GET /raspi/vcgencmd and GET /raspi/eeprom_update report Pi firmware, bootloader, and EEPROM update status",
            ),
            Rationaled::new(
                CapabilityId::UpdateRaspberryEeprom,
                "POST /raspi/eeprom_update applies available Pi EEPROM and USB-controller firmware updates",
            ),
            Rationaled::new(
                CapabilityId::ResetBlueosSettings,
                "POST /settings/reset deletes BlueOS service settings while preserving bootstrap and ardupilot-manager state",
            ),
            Rationaled::new(
                CapabilityId::RunHostCommand,
                "POST /command/host runs a caller-supplied shell command and returns stdout, stderr, and return code",
            ),
            Rationaled::new(
                CapabilityId::SetupSsh,
                "startup setup_ssh generates /root/.config/.ssh keys and appends the public key to the SSH user's authorized_keys",
            ),
        ]),
        authorities: AssertedSet::established(&[
            Rationaled::new(
                Authority::Other("host_power_controller"),
                "sole REST surface for sudo reboot and shutdown --poweroff; no other cataloged service exposes onboard power control",
            ),
            Rationaled::new(
                Authority::Other("host_command_executor"),
                "sole POST /command/host arbitrary shell executor consumed by the frontend commander store",
            ),
        ]),
        states: AssertedSet::unknown(
            "no cataloged state machine; shutdown scheduling and SSH key setup are one-shot request or boot-time side effects",
        ),
        edges: AssertedSet::established(&[]),
        resources: AssertedSet::established(&[
            Rationaled::new(
                Resource {
                    path: PathRef("/var/logs/blueos"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "POST /services/remove_log and remove_log_stream delete files under the BlueOS log folder",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/shortcuts/ardupilot_logs/logs/"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "POST /services/remove_mavlink_log deletes MAVLink log files under the mavlink log folder",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/.ssh"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "setup_ssh at boot creates and stores the container SSH key pair",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/home/{user}/.ssh/authorized_keys"),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "setup_ssh appends the generated public key to the SSH user's authorized_keys",
            ),
        ]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                &["start-blueos-core create_service"],
                "observed lifecycle trigger: tmux creation at boot in SERVICES tier",
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
                ],
                "observed ordered_after in start-blueos-core SERVICES block",
            ),
            ordered_before: Asserted::established(
                &[
                    ServiceId::NmeaInjector,
                    ServiceId::Helper,
                    ServiceId::Iperf3,
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
                "observed ordered_before lists commander before helper and remaining SERVICES-tier peers",
            ),
            shutdown: Asserted::established(
                "uvicorn server exit on process termination",
                "main.py awaits server.serve with no explicit shutdown hook beyond uvicorn exit",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade semantics for in-flight reboot scheduling and SSH key persistence not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST GET / returns HTML title; setup_ssh runs once at boot",
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
            "no separate permissions manifest; destructive REST routes require i_know_what_i_am_doing=true",
            "check_what_i_am_doing rejects requests without explicit operator acknowledgment",
        ),
        failure_modes: AssertedSet::established(&[
            Rationaled::new(
                "i_know_what_i_am_doing_rejected",
                "check_what_i_am_doing returns HTTP 400 when the acknowledgment query param is false",
            ),
            Rationaled::new(
                "host_command_subprocess_failure",
                "command_host returns non-zero return_code and stderr when run_command fails",
            ),
            Rationaled::new(
                "raspi_config_legacy_failure",
                "raspi_config_camera_legacy routes return HTTP 400 when raspi-config subprocess exits non-zero",
            ),
            Rationaled::new(
                "ssh_setup_failure",
                "setup_ssh logs errors and continues when key generation or authorized_keys write fails",
            ),
            Rationaled::new(
                "settings_reset_partial_failure",
                "delete_everything during settings reset may leave some config paths if deletion raises",
            ),
        ]),
        blast_radius: Asserted::established(
            "reboot or poweroff takes down the entire companion computer and all BlueOS services; MAVLink on the FC may continue but onboard UI, logging, and routing stop",
            "host power control can remove every onboard service at once; not vehicle-critical for live FC control but high operational impact",
        ),
        compatibility_policy: Asserted::unknown(
            "API deprecation policy and shutdown hold-time semantics not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    };

pub const SERVICE: Service = Service {
    id: ServiceId::Commander,
    observed: OBSERVED_FACTS,
    definition: SERVICE_DEFINITION,
    runtime: RUNTIME_FACTS,
};
