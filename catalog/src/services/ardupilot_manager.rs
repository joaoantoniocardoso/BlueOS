use crate::criticality::CriticalityTier;
use crate::edge::{Bus, Edge, FailureImpact, SyncMode};
use crate::id::{CapabilityId, JourneyId, PathRef, PortRef, ServiceId};
use crate::interface::{FileAccessMode, Interface, MavlinkRole};
use crate::journey::{HttpMethod, RouteRef};
use crate::lifecycle::{Lifecycle, ObservedLifecycle};
use crate::observed::{ObservedFacts, ResourceLimits, ServiceKind, StartupTier};
use crate::provenance::GroundedSet;
use crate::provenance::{
    Asserted, AssertedSet, Evidence, Evidenced, GroundedItem, Observed, ObservedSet, Provenance,
    Rationaled,
};
use crate::resource::{Resource, ResourceOwnership};
use crate::runtime::{
    Distribution, PlatformBehavior, ResourceUsage, RuntimeFacts, SettingsMutation, SloBaseline,
    StateContract,
};
use crate::service::{Authority, ServiceDefinition};
use crate::state::StateMachine;
use crate::trust::{DangerousOperation, PrivilegeLevel, UserConfirmation};

const RUNTIME_CAPTURE: &str = "runtime-captures/ardupilot_manager__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator, ArduSub 4.5.3 STABLE";

pub fn runtime_facts() -> RuntimeFacts {
    RuntimeFacts {
        service: ServiceId("ardupilot_manager".into()),
        state_contracts: GroundedSet::known(vec![
            runtime_state_contract(
                "running",
                HttpMethod::Get,
                "/vehicle_type",
                200,
                Some("Submarine"),
            ),
            runtime_state_contract(
                "stopped",
                HttpMethod::Get,
                "/vehicle_type",
                500,
                Some("Did not receive an updated HEARTBEAT before timeout"),
            ),
            runtime_state_contract(
                "running",
                HttpMethod::Get,
                "/firmware_vehicle_type",
                200,
                Some("ArduSub"),
            ),
            runtime_state_contract(
                "stopped",
                HttpMethod::Get,
                "/firmware_vehicle_type",
                500,
                Some("Did not receive an updated HEARTBEAT before timeout"),
            ),
            runtime_state_contract("stopped", HttpMethod::Get, "/board", 200, None),
            runtime_state_contract("stopped", HttpMethod::Get, "/firmware_info", 200, None),
        ]),
        slo_baselines: GroundedSet::known(vec![
            runtime_slo(HttpMethod::Get, "/board", 7.3, 13.6, 15.8),
            runtime_slo(HttpMethod::Get, "/vehicle_type", 1038.5, 1518.9, 1627.9),
            runtime_slo(HttpMethod::Get, "/firmware_info", 29.3, 62.7, 102.1),
            runtime_slo(HttpMethod::Get, "/sitl_frame", 5.9, 10.2, 14.3),
            runtime_slo(HttpMethod::Get, "/preferred_router", 6.2, 13.0, 15.5),
            runtime_slo(HttpMethod::Get, "/available_boards", 15.8, 26.8, 28.5),
            runtime_slo(HttpMethod::Get, "/available_firmwares", 6.6, 13.2, 18.1),
            runtime_slo(HttpMethod::Get, "/endpoints/", 11.7, 27.0, 39.3),
            runtime_slo(HttpMethod::Get, "/serials", 694.6, 1384.8, 1407.7),
        ]),
        resource_usage: GroundedSet::known(vec![
            runtime_resource(
                "running_navigator",
                Distribution {
                    mean: 8.74,
                    median: 1.11,
                    p95: 51.65,
                    min: 0.0,
                    max: 57.45,
                    sd: 17.16,
                },
                flat_rss(46.0),
                60,
            ),
            runtime_resource(
                "stopped_navigator",
                Distribution {
                    mean: 0.29,
                    median: 0.0,
                    p95: 0.98,
                    min: 0.0,
                    max: 0.99,
                    sd: 0.45,
                },
                flat_rss(46.0),
                20,
            ),
            runtime_resource(
                "sitl",
                Distribution {
                    mean: 7.34,
                    median: 0.0,
                    p95: 43.80,
                    min: 0.0,
                    max: 46.94,
                    sd: 14.77,
                },
                flat_rss(114.2),
                20,
            ),
        ]),
        platform_matrix: GroundedSet::known(vec![
            GroundedItem::new(
                PlatformBehavior {
                    platform: "navigator".into(),
                    firmware: Some("4.5.3 STABLE".into()),
                    notes: vec![
                        "GET /serials -> 200 (hardware serial /dev/ttyS0)".into(),
                        "manager RSS ~46 MB".into(),
                    ],
                },
                runtime_prov("#platform_matrix"),
            ),
            GroundedItem::new(
                PlatformBehavior {
                    platform: "SITL_arm_linux_gnueabihf".into(),
                    firmware: Some("4.7.0 BETA".into()),
                    notes: vec![
                        "GET /serials -> 500 {detail:''}".into(),
                        "manager RSS ~114 MB".into(),
                        "software autopilot".into(),
                    ],
                },
                runtime_prov("#platform_matrix"),
            ),
            GroundedItem::new(
                PlatformBehavior {
                    platform: "Manual".into(),
                    firmware: Some("4.5.7 STABLE".into()),
                    notes: vec![
                        "GET /vehicle_type -> 500 (no HEARTBEAT; no auto-detected autopilot)"
                            .into(),
                        "GET /serials -> 500".into(),
                        "manager RSS ~55 MB, CPU ~0.85% idle".into(),
                        "CAVEAT: captured with Manual selected but no external autopilot attached to \
                         the manual master endpoint (udpin 0.0.0.0:14551); 500s reflect the \
                         unconfigured/no-source sub-state, not a correctly set up Manual vehicle \
                         (outcomes likely differ once a real autopilot streams MAVLink)"
                            .into(),
                        "DOC GAP: Manual board is undocumented in ../BlueOS-docs (only Navigator \
                         and SITL board options are covered; cf. serial-autopilot support #2722)"
                            .into(),
                    ],
                },
                runtime_prov("#platform_matrix"),
            ),
        ]),
        settings_mutations: GroundedSet::known(vec![
            runtime_settings_mutation("POST /sitl_frame", vec!["content.sitl_frame"]),
            runtime_settings_mutation("POST /board", vec!["content.preferred_board"]),
            runtime_settings_mutation("POST /start or /stop", vec!["content.start_on_boot"]),
            runtime_settings_mutation("POST /preferred_router", vec!["content.preferred_router"]),
        ]),
    }
}

fn runtime_prov(key: &str) -> Provenance {
    Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV)
}

fn runtime_route(method: HttpMethod, path: &str) -> RouteRef {
    RouteRef {
        service: ServiceId("ardupilot_manager".into()),
        method,
        path: path.into(),
        version: None,
    }
}

fn runtime_state_contract(
    state: &str,
    method: HttpMethod,
    path: &str,
    status: u16,
    body_predicate: Option<&str>,
) -> GroundedItem<StateContract> {
    GroundedItem::new(
        StateContract {
            machine: "autopilot_lifecycle".into(),
            state: state.into(),
            route: runtime_route(method, path),
            status,
            body_predicate: body_predicate.map(str::to_string),
        },
        runtime_prov("#state_contracts"),
    )
}

fn runtime_slo(
    method: HttpMethod,
    path: &str,
    p50: f64,
    p95: f64,
    p99: f64,
) -> GroundedItem<SloBaseline> {
    GroundedItem::new(
        SloBaseline {
            route: runtime_route(method, path),
            latency_p50_ms: p50,
            latency_p95_ms: p95,
            latency_p99_ms: p99,
            sample_size: 60,
        },
        runtime_prov("#slo_running_navigator"),
    )
}

fn flat_rss(mb: f64) -> Distribution {
    Distribution {
        mean: mb,
        median: mb,
        p95: mb,
        min: mb,
        max: mb,
        sd: 0.0,
    }
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
        runtime_prov("#resource_by_state"),
    )
}

fn runtime_settings_mutation(
    trigger: &str,
    keys_changed: Vec<&str>,
) -> GroundedItem<SettingsMutation> {
    GroundedItem::new(
        SettingsMutation {
            trigger: trigger.into(),
            keys_changed: keys_changed.into_iter().map(str::to_string).collect(),
        },
        runtime_prov("#settings_mutations"),
    )
}

pub fn observed_facts() -> ObservedFacts {
    ObservedFacts {
        id: ServiceId("ardupilot_manager".to_string()),
        aliases: ObservedSet::known(vec![
            Evidenced::new(
                "autopilot".to_string(),
                Evidence {
                    file: "core/start-blueos-core".to_string(),
                    line: 118,
                },
            ),
            Evidenced::new(
                "ardupilot-manager".to_string(),
                Evidence {
                    file: "core/services/ardupilot_manager/settings.py".to_string(),
                    line: 10,
                },
            ),
        ]),
        kind: Observed::known(
            ServiceKind::PythonService,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 118,
            },
        ),
        entrypoint: Observed::known(
            "nice --19 $SERVICES_PATH/ardupilot_manager/main.py".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 118,
            },
        ),
        tmux_name: Observed::known(
            "autopilot".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 118,
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
                line: 118,
            },
        ),
        nice: Observed::known(
            19,
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 118,
            },
        ),
        run_as: Observed::known(
            "root".to_string(),
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 118,
            },
        ),
        nginx_prefixes: ObservedSet::known(vec![
            Evidenced::new(
                PathRef("/ardupilot-manager/".to_string()),
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 76,
                },
            ),
            Evidenced::new(
                PathRef("/autopilot-manager/".to_string()),
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 81,
                },
            ),
        ]),
        listen: ObservedSet::known(vec![Evidenced::new(
            PortRef::Literal(8000),
            Evidence {
                file: "core/services/ardupilot_manager/args.py".to_string(),
                line: 28,
            },
        )]),
        git_path: Observed::known(
            PathRef("core/services/ardupilot_manager".to_string()),
            Evidence {
                file: "core/services/ardupilot_manager/main.py".to_string(),
                line: 1,
            },
        ),
        interfaces: ObservedSet::known(vec![
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/ardupilot-manager/".to_string()),
                    port: PortRef::Literal(8000),
                    versions: vec!["v1.0".to_string(), "v2.0".to_string()],
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 76,
                },
            ),
            Evidenced::new(
                Interface::Rest {
                    path_prefix: PathRef("/autopilot-manager/".to_string()),
                    port: PortRef::Literal(8000),
                    versions: vec!["v1.0".to_string(), "v2.0".to_string()],
                },
                Evidence {
                    file: "core/tools/nginx/nginx.conf".to_string(),
                    line: 81,
                },
            ),
            Evidenced::new(
                Interface::Mavlink {
                    role: MavlinkRole::Endpoint,
                    connect: "udpin:0.0.0.0:14550".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/autopilot_manager.py".to_string(),
                    line: 56,
                },
            ),
            Evidenced::new(
                Interface::Mavlink {
                    role: MavlinkRole::Consumer,
                    connect: "udpout:192.168.2.1:14550".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/autopilot_manager.py".to_string(),
                    line: 65,
                },
            ),
            Evidenced::new(
                Interface::Mavlink {
                    role: MavlinkRole::Endpoint,
                    connect: "udpin:127.0.0.1:14001".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/autopilot_manager.py".to_string(),
                    line: 74,
                },
            ),
            Evidenced::new(
                Interface::Mavlink {
                    role: MavlinkRole::Consumer,
                    connect: "udpout:127.0.0.1:14000".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/autopilot_manager.py".to_string(),
                    line: 83,
                },
            ),
            Evidenced::new(
                Interface::Mavlink {
                    role: MavlinkRole::Bridge,
                    connect: "zenoh:0.0.0.0:7117".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/autopilot_manager.py".to_string(),
                    line: 93,
                },
            ),
            Evidenced::new(
                Interface::Mavlink {
                    role: MavlinkRole::Bridge,
                    connect: "zenohraw:0.0.0.0:7117".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/autopilot_manager.py".to_string(),
                    line: 102,
                },
            ),
            Evidenced::new(
                Interface::Mavlink {
                    role: MavlinkRole::Endpoint,
                    connect: "tcpin:127.0.0.1:5777".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/autopilot_manager.py".to_string(),
                    line: 111,
                },
            ),
            Evidenced::new(
                Interface::Mavlink {
                    role: MavlinkRole::Endpoint,
                    connect: "udpin:0.0.0.0:14660".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/autopilot_manager.py".to_string(),
                    line: 121,
                },
            ),
            Evidenced::new(
                Interface::Mavlink {
                    role: MavlinkRole::Endpoint,
                    connect: "udpin:127.0.0.1:8852".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/autopilot_manager.py".to_string(),
                    line: 331,
                },
            ),
            Evidenced::new(
                Interface::Mavlink {
                    role: MavlinkRole::Consumer,
                    connect: "tcpout:127.0.0.1:5760".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/autopilot_manager.py".to_string(),
                    line: 464,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "mavlink-routerd".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/mavlink_proxy/MAVLinkRouter.py"
                        .to_string(),
                    line: 74,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "mavlink-server".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/mavlink_proxy/MAVLinkServer.py"
                        .to_string(),
                    line: 65,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "mavproxy.py".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/mavlink_proxy/MAVProxy.py".to_string(),
                    line: 54,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "mavp2p".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/mavlink_proxy/MAVP2P.py".to_string(),
                    line: 49,
                },
            ),
            Evidenced::new(
                Interface::Subprocess {
                    command: "ardupilot_fw_uploader.py".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/firmware/FirmwareUpload.py".to_string(),
                    line: 25,
                },
            ),
            Evidenced::new(
                Interface::OutboundHttp {
                    url: "https://firmware.ardupilot.org/manifest.json.gz".to_string(),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/firmware/FirmwareDownload.py"
                        .to_string(),
                    line: 26,
                },
            ),
            Evidenced::new(
                Interface::Settings {
                    path: PathRef("/root/.config/ardupilot-manager/settings.json".to_string()),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/settings.py".to_string(),
                    line: 16,
                },
            ),
            Evidenced::new(
                Interface::File {
                    path: PathRef("/usr/blueos/userdata/firmware".to_string()),
                    mode: FileAccessMode::ReadWrite,
                },
                Evidence {
                    file: "core/services/ardupilot_manager/settings.py".to_string(),
                    line: 18,
                },
            ),
            Evidenced::new(
                Interface::Hardware {
                    device: PathRef("/dev/autopilot".to_string()),
                },
                Evidence {
                    file: "core/services/ardupilot_manager/firmware/FirmwareUpload.py".to_string(),
                    line: 12,
                },
            ),
            Evidenced::new(
                Interface::Zenoh {
                    topics_produced: vec!["services/ardupilot-manager/log".to_string()],
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
                    path: PathRef("/dev/autopilot".to_string()),
                    ownership: ResourceOwnership::Exclusive,
                },
                Evidence {
                    file: "core/services/ardupilot_manager/firmware/FirmwareUpload.py".to_string(),
                    line: 12,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/usr/blueos/userdata/firmware".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/ardupilot_manager/settings.py".to_string(),
                    line: 18,
                },
            ),
            Evidenced::new(
                Resource {
                    path: PathRef("/root/.config/ardupilot-manager".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                Evidence {
                    file: "core/services/ardupilot_manager/settings.py".to_string(),
                    line: 15,
                },
            ),
        ]),
        lifecycle: Observed::known(
            ObservedLifecycle {
                triggers: vec![
                    "start-blueos-core create_service".to_string(),
                    "start_on_boot setting".to_string(),
                ],
                ordered_after: vec![],
                ordered_before: vec![
                    ServiceId("cable_guy".to_string()),
                    ServiceId("video".to_string()),
                    ServiceId("mavlink2rest".to_string()),
                ],
            },
            Evidence {
                file: "core/start-blueos-core".to_string(),
                line: 318,
            },
        ),
        logs_path: Observed::unknown(
            "log_path is appdirs.user_config_dir('ardupilot-manager')/logs; resolved at runtime",
        ),
        zenoh_log_topic: Observed::known(
            "services/ardupilot-manager/log".to_string(),
            Evidence {
                file: "core/libs/commonwealth/src/commonwealth/utils/logs.py".to_string(),
                line: 78,
            },
        ),
        sentry: Observed::known(
            true,
            Evidence {
                file: "core/services/ardupilot_manager/main.py".to_string(),
                line: 28,
            },
        ),
        openapi_refs: ObservedSet::unknown("not yet extracted"),
    }
}

pub fn service_definition() -> ServiceDefinition {
    ServiceDefinition {
        id: ServiceId("ardupilot_manager".to_string()),
        singleton: Asserted::established(
            true,
            "Singleton metaclass and single priority-tier tmux instance; no second autopilot manager process",
        ),
        bounded_context: Asserted::established(
            "flight-controller-and-mavlink-routing".to_string(),
            "provisional 2.0 domain: owns FC process, firmware, board selection, and MAVLink router endpoints",
        ),
        journey_refs: AssertedSet::established(vec![
            Rationaled::new(
                JourneyId("vehicle_first_boot".into()),
                "wizard-driven first boot downloads firmware and arms autopilot lifecycle",
            ),
            Rationaled::new(
                JourneyId("change_board".into()),
                "POST /board selects connected FC or virtual SITL board",
            ),
            Rationaled::new(
                JourneyId("run_sitl_simulation".into()),
                "SITL board selection and sitl_frame POST configure simulated vehicle",
            ),
            Rationaled::new(
                JourneyId("start_autopilot".into()),
                "POST /start transitions autopilot_lifecycle stopped to running",
            ),
            Rationaled::new(
                JourneyId("stop_autopilot".into()),
                "POST /stop transitions autopilot_lifecycle running to stopped",
            ),
            Rationaled::new(
                JourneyId("restart_autopilot".into()),
                "POST /restart kills and relaunches the FC process",
            ),
            Rationaled::new(
                JourneyId("update_firmware_online".into()),
                "POST /install_firmware_from_url flashes firmware from ArduPilot repo",
            ),
            Rationaled::new(
                JourneyId("upload_custom_firmware".into()),
                "POST /install_firmware_from_file flashes user-supplied firmware image",
            ),
            Rationaled::new(
                JourneyId("restore_default_firmware".into()),
                "POST /restore_default_firmware reverts to factory ArduSub image",
            ),
        ]),
        tier: Asserted::established(
            CriticalityTier::VehicleCritical,
            "sole MAVLink router owner and FC manager; vehicle loses control path if unavailable",
        ),
        offline_required: Asserted::established(
            true,
            "FC connection, MAVLink routing, and local firmware cache must work without internet",
        ),
        privilege_level: Asserted::established(
            PrivilegeLevel::Root,
            "observed run_as root; main.py aborts when not running as root",
        ),
        dangerous_operations: AssertedSet::established(vec![
            Rationaled::new(
                DangerousOperation::FirmwareFlash,
                "install_firmware REST routes and ardupilot_fw_uploader subprocess rewrite FC firmware",
            ),
            Rationaled::new(
                DangerousOperation::Reboot,
                "restart and kill/start cycles stop and relaunch the flight-controller process",
            ),
            Rationaled::new(
                DangerousOperation::Upgrade,
                "install_firmware_from_url replaces running firmware from remote URL",
            ),
            Rationaled::new(
                DangerousOperation::SettingsReset,
                "restore_default_firmware reverts FC firmware to factory default",
            ),
        ]),
        user_confirmation: Asserted::established(
            UserConfirmation::Required,
            "firmware flash, FC restart, and restore-default are irreversible vehicle-safety operations",
        ),
        capabilities: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("manage_autopilot_lifecycle".to_string()),
                "REST start/stop/restart routes and auto_restart_ardupilot watchdog",
            ),
            Rationaled::new(
                CapabilityId("manage_mavlink_endpoints".to_string()),
                "REST /endpoints CRUD and default endpoint table in autopilot_manager",
            ),
            Rationaled::new(
                CapabilityId("manage_mavlink_router".to_string()),
                "spawns mavlink-routerd/mavproxy/mavp2p/mavlink-server subprocesses; preferred_router API",
            ),
            Rationaled::new(
                CapabilityId("flash_firmware".to_string()),
                "install_firmware_from_url/file and FirmwareUpload subprocess",
            ),
            Rationaled::new(
                CapabilityId("select_flight_controller_board".to_string()),
                "board/available_boards REST and change_board logic",
            ),
            Rationaled::new(
                CapabilityId("detect_flight_controllers".to_string()),
                "BoardDetector used at startup and available_boards endpoint",
            ),
            Rationaled::new(
                CapabilityId("manage_serial_ports".to_string()),
                "serials GET/PUT routes map ArduPilot serial ports to Linux devices",
            ),
            Rationaled::new(
                CapabilityId("query_vehicle_firmware_info".to_string()),
                "firmware_info and vehicle_type REST via VehicleManager MAVLink queries",
            ),
            Rationaled::new(
                CapabilityId("configure_sitl_frame".to_string()),
                "sitl_frame GET/POST for SITL simulation frame selection",
            ),
        ]),
        authorities: AssertedSet::established(vec![
            Rationaled::new(
                Authority::MavlinkRouterOwner,
                "sole spawner of mavlink-router subprocesses and creator of all router endpoints including the mavlink2rest udpin listener",
            ),
            Rationaled::new(
                Authority::HardwareExclusive(PathRef("/dev/autopilot".to_string())),
                "exclusive /dev/autopilot access for firmware uploader; no other service claims this device",
            ),
        ]),
        states: AssertedSet::established(vec![
            Rationaled::new(
                StateMachine {
                    name: "autopilot_lifecycle".to_string(),
                    states: vec![
                        "stopped".to_string(),
                        "running".to_string(),
                        "degraded".to_string(),
                    ],
                    boot_state: "stopped".to_string(),
                    degraded_when: vec![
                        "heartbeat_failures".to_string(),
                        "subprocess_crash".to_string(),
                    ],
                },
                "should_be_running flag, is_running check, and heartbeat watchdog drive FC lifecycle",
            ),
            Rationaled::new(
                StateMachine {
                    name: "board_selection".to_string(),
                    states: vec![
                        "none".to_string(),
                        "selected".to_string(),
                        "running".to_string(),
                    ],
                    boot_state: "none".to_string(),
                    degraded_when: vec!["board_disconnected".to_string()],
                },
                "current_board property and change_board/start_ardupilot transition connected boards to running",
            ),
        ]),
        edges: AssertedSet::established(vec![Rationaled::new(
            Edge {
                from: ServiceId("ardupilot_manager".to_string()),
                to: ServiceId("zenohd".to_string()),
                via: Bus::Zenoh,
                sync: SyncMode::Async,
                endpoint: "zenoh:0.0.0.0:7117".to_string(),
                purpose: "bridge the vehicle MAVLink stream onto the zenoh bus for pub/sub subscribers"
                    .to_string(),
                required_at_boot: false,
                failure_impact: FailureImpact::Degraded,
            },
            "observed Mavlink Bridge connect zenoh:0.0.0.0:7117 + zenohraw:0.0.0.0:7117 target zenohd's zenoh port 7117; the mavlink-router bridges vehicle telemetry onto the bus. Flight control via the MAVLink router is independent of zenohd, so failure is Degraded (subscribers lose vehicle data) and not required at boot (ardupilot_manager is Priority tier and starts before zenohd; the bridge attaches when zenohd is up). Other outbound interfaces are external (firmware.ardupilot.org), flight-controller hardware (/dev/autopilot), local subprocesses, and inbound router endpoints (udpin/tcpin, connected TO by mavlink2rest/video/etc.)",
        )]),
        resources: AssertedSet::established(vec![
            Rationaled::new(
                Resource {
                    path: PathRef("/dev/autopilot".to_string()),
                    ownership: ResourceOwnership::Exclusive,
                },
                "firmware uploader opens /dev/autopilot for exclusive bootloader access",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/usr/blueos/userdata/firmware".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "user firmware drop directory; primary writer but path is user-accessible",
            ),
            Rationaled::new(
                Resource {
                    path: PathRef("/root/.config/ardupilot-manager".to_string()),
                    ownership: ResourceOwnership::SharedWrite,
                },
                "service-owned settings tree; other tools may read but this service is the writer",
            ),
        ]),
        lifecycle: Lifecycle {
            triggers: Asserted::established(
                vec![
                    "start-blueos-core create_service".to_string(),
                    "start_on_boot setting".to_string(),
                ],
                "observed lifecycle triggers: tmux creation at boot and user start_on_boot preference",
            ),
            ordered_after: Asserted::established(
                vec![],
                "observed lifecycle has no explicit ordered_after dependencies",
            ),
            ordered_before: Asserted::established(
                vec![
                    ServiceId("cable_guy".to_string()),
                    ServiceId("video".to_string()),
                    ServiceId("mavlink2rest".to_string()),
                ],
                "priority startup tier lists this service before cable_guy, video, and mavlink2rest",
            ),
            shutdown: Asserted::established(
                "kill_ardupilot on uvicorn server exit".to_string(),
                "main.py awaits server shutdown then calls kill_ardupilot",
            ),
            upgrade_behavior: Asserted::unknown(
                "BlueOS upgrade restart semantics for in-flight FC process not traced in service source",
            ),
        },
        health: Asserted::established(
            "implicit: process liveness via tmux; REST root returns 200; auto_restart watchdogs for FC and router"
                .to_string(),
            "no dedicated /health route; watchdog tasks and REST availability serve as health signals",
        ),
        is_platform: Asserted::established(
            false,
            "core vehicle service with extension endpoint API, not a Kraken-style platform host",
        ),
        api_stable: Asserted::established(
            true,
            "versioned FastAPI v1 routes with broad REST surface; v2 adds only a stub root",
        ),
        permissions_model: Asserted::unknown(
            "no auth middleware or permission checks observed in API app; LAN trust model not cataloged elsewhere",
        ),
        failure_modes: AssertedSet::established(vec![
            Rationaled::new(
                "mavlink_router_crash".to_string(),
                "router subprocess exit breaks all MAVLink fan-out until auto_restart_router recovers",
            ),
            Rationaled::new(
                "flight_controller_heartbeat_loss".to_string(),
                "consecutive heartbeat failures trigger FC restart loop",
            ),
            Rationaled::new(
                "firmware_flash_failure".to_string(),
                "bad image or uploader error can brick FC until manual recovery",
            ),
            Rationaled::new(
                "autopilot_subprocess_crash".to_string(),
                "SITL/Linux FC process exit triggers auto_restart_ardupilot",
            ),
            Rationaled::new(
                "hardware_device_unavailable".to_string(),
                "/dev/autopilot missing blocks firmware upload and serial FC access",
            ),
        ]),
        blast_radius: Asserted::established(
            "vehicle loses MAVLink connectivity and FC control; GCS link, mavlink2rest consumers, Zenoh bridge, and camera MAVLink client isolated".to_string(),
            "owns MAVLink router and FC process; downstream priority-tier services depend on its endpoints",
        ),
        compatibility_policy: Asserted::unknown(
            "firmware/board compatibility matrix and API deprecation policy not established from source",
        ),
        team: Asserted::unknown("no CODEOWNERS or team metadata in observed artifact"),
        adr_refs: AssertedSet::unknown("no ADR references found in service source tree"),
    }
}
