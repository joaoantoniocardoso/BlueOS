use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_ARDUSUB;
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, NetworkState, Precondition, RouteRef, StateTransition,
    StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use crate::version::FeatureAvailability;

const ADV: &str = "content/usage/advanced/index.md";
const GS: &str = "content/usage/getting-started/index.md";
const INSTALL: &str = "content/usage/installation.md";
const APM_ROUTER: &str = "core/services/ardupilot_manager/api/v1/routers/index.py";
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_ARDUSUB;

pub const JOURNEYS: &[UserJourney] = &[
    VEHICLE_FIRST_BOOT,
    CHANGE_BOARD,
    RUN_SITL_SIMULATION,
    START_AUTOPILOT,
    STOP_AUTOPILOT,
    RESTART_AUTOPILOT,
    UPDATE_FIRMWARE_ONLINE,
    UPLOAD_CUSTOM_FIRMWARE,
    RESTORE_DEFAULT_FIRMWARE,
];

const VEHICLE_FIRST_BOOT: UserJourney =
    UserJourney {
        id: JourneyId::VehicleFirstBoot,
        summary: Grounded::known(
            "On first boot the configuration wizard downloads and installs up-to-date autopilot firmware",
            Provenance::doc(GS, 52),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 256)),
        services: ARDUPILOT_MANAGER_SERVICES,
        capability_refs: GroundedSet::known(&[
            cap(CapabilityId::DetectFlightControllers,
                "wizard discovers connected boards before firmware install",
            ),
            cap(CapabilityId::FlashFirmware,
                "install_firmware_from_url during first-boot wizard",
            ),
            cap(CapabilityId::ManageAutopilotLifecycle,
                "first boot completes with a running autopilot process",
            ),
            cap(CapabilityId::QueryVehicleFirmwareInfo,
                "first-boot flow reads firmware_info to confirm the active autopilot",
            ),
        ]),
        preconditions: GroundedSet::known(&[
            GroundedItem::new(
                Precondition::Other("Cold start after power-on"),
                Provenance::doc(INSTALL, 59),
            ),
            GroundedItem::new(
                Precondition::Other("BlueOS is newly installed and the configuration wizard is available"),
                Provenance::doc(GS, 38),
            ),
        ]),
        steps: GroundedSet::known(&[
            operator_step(
                "Power on the vehicle and wait for the first boot to complete",
                None,
                Provenance::doc(INSTALL, 59),
                None,
            ),
            service_step(
                "Download and install up-to-date autopilot firmware for the selected vehicle type",
                Some(sourced_route(HttpMethod::Post, "/install_firmware_from_url", Some("v1.0"), 163)),
                Provenance::doc(GS, 52),
                Some(runtime_outcome(200, None, None, "runtime-captures/ardupilot_manager__pi4_navigator_master.json#firmware_operations")),
            ),
            operator_step(
                "Open the Autopilot Firmware page to view basic information about the active autopilot",
                Some(sourced_route(HttpMethod::Get, "/firmware_info", Some("v1.0"), 103)),
                Provenance::doc(ADV, 259),
                Some(runtime_outcome(
                    200,
                    Some("\"version\": \"4.5.3\""),
                    None,
                    "runtime-captures/ardupilot_manager__pi4_navigator_master.json#running_navigator",
                )),
            ),
        ]),
        availability: FeatureAvailability::unknown(),
    chains_from: None,
    };

const CHANGE_BOARD: UserJourney = UserJourney {
    id: JourneyId::ChangeBoard,
    summary: Grounded::known(
        "Select a connected flight controller board or switch to the virtual SITL board",
        Provenance::doc(ADV, 262),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 261)),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::SelectFlightControllerBoard,
        "POST /board switches between connected boards and SITL",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other(
            "At least one flight controller board is connected or SITL simulation is available",
        ),
        Provenance::doc(ADV, 262),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "Select a connected board or the SITL simulation board on the Autopilot Firmware page",
        Some(sourced_route(HttpMethod::Post, "/board", Some("v1.0"), 226)),
        Provenance::doc(ADV, 262),
        Some(runtime_outcome(
            200,
            None,
            None,
            "runtime-captures/ardupilot_manager__pi4_navigator_master.json#transitions",
        )),
    )]),
    availability: FeatureAvailability::unknown(),
    chains_from: None,
};

const RUN_SITL_SIMULATION: UserJourney =
    UserJourney {
        id: JourneyId::RunSitlSimulation,
        summary: Grounded::known(
            "Run ArduPilot SITL simulation by selecting the virtual board and configuring the vehicle frame"
                ,
            Provenance::doc(ADV, 305),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 303)),
        services: ARDUPILOT_MANAGER_SERVICES,
        capability_refs: GroundedSet::known(&[
            cap(CapabilityId::SelectFlightControllerBoard,
                "journey begins by selecting the virtual SITL board",
            ),
            cap(CapabilityId::ConfigureSitlFrame,
                "POST /sitl_frame sets simulated vehicle frame",
            ),
        ]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Other("Virtual SITL flight controller board is selected"),
            Provenance::doc(ADV, 262),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Select the virtual SITL flight controller board",
                Some(sourced_route(HttpMethod::Post, "/board", Some("v1.0"), 226)),
                Provenance::doc(ADV, 262),
                Some(runtime_outcome(200, None, None, "runtime-captures/ardupilot_manager__pi4_navigator_master.json#transitions")),
            ),
            operator_step(
                "Set the SITL vehicle frame in the POST /sitl_frame endpoint",
                Some(sourced_route(HttpMethod::Post, "/sitl_frame", Some("v1.0"), 133)),
                Provenance::doc(ADV, 319),
                Some(source_outcome(200, APM_ROUTER, 135)),
            ),
        ]),
        availability: FeatureAvailability::unknown(),
    chains_from: Some(JourneyId::ChangeBoard),
    };

const START_AUTOPILOT: UserJourney = UserJourney {
    id: JourneyId::StartAutopilot,
    summary: Grounded::known("Start the autopilot process", Provenance::doc(ADV, 263)),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 261)),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ManageAutopilotLifecycle,
        "POST /start launches the FC subprocess",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("A flight controller board is selected"),
        Provenance::doc(ADV, 262),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "Start the autopilot from the Autopilot Firmware page",
        Some(sourced_route(HttpMethod::Post, "/start", Some("v1.0"), 241)),
        Provenance::doc(ADV, 263),
        Some(runtime_outcome(
            200,
            None,
            Some(autopilot_transition("stopped", "running")),
            "runtime-captures/ardupilot_manager__pi4_navigator_master.json#transitions",
        )),
    )]),
    availability: FeatureAvailability::unknown(),
    chains_from: Some(JourneyId::ChangeBoard),
};

const STOP_AUTOPILOT: UserJourney = UserJourney {
    id: JourneyId::StopAutopilot,
    summary: Grounded::known("Stop the autopilot process", Provenance::doc(ADV, 264)),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 261)),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ManageAutopilotLifecycle,
        "POST /stop terminates the FC subprocess",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("An autopilot is available to stop"),
        Provenance::doc(ADV, 264),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "Stop the autopilot from the Autopilot Firmware page",
        Some(sourced_route(HttpMethod::Post, "/stop", Some("v1.0"), 270)),
        Provenance::doc(ADV, 264),
        Some(runtime_outcome(
            200,
            None,
            Some(autopilot_transition("running", "stopped")),
            "runtime-captures/ardupilot_manager__pi4_navigator_master.json#transitions",
        )),
    )]),
    availability: FeatureAvailability::unknown(),
    chains_from: None,
};

const RESTART_AUTOPILOT: UserJourney = UserJourney {
    id: JourneyId::RestartAutopilot,
    summary: Grounded::known("Restart the autopilot", Provenance::doc(ADV, 267)),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 267)),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ManageAutopilotLifecycle,
        "POST /restart stops and relaunches the FC process",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("A flight controller board is selected"),
        Provenance::doc(ADV, 283),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "Restart the autopilot from the Autopilot Firmware page",
        Some(sourced_route(
            HttpMethod::Post,
            "/restart",
            Some("v1.0"),
            233,
        )),
        Provenance::doc(ADV, 267),
        Some(runtime_outcome(
            200,
            None,
            None,
            "runtime-captures/ardupilot_manager__pi4_navigator_master.json#transitions",
        )),
    )]),
    availability: FeatureAvailability::unknown(),
    chains_from: None,
};

const UPDATE_FIRMWARE_ONLINE: UserJourney = UserJourney {
    id: JourneyId::UpdateFirmwareOnline,
    summary: Grounded::known(
        "Update flight-controller firmware from the online ArduPilot repository",
        Provenance::doc(ADV, 268),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 268)),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::FlashFirmware,
        "POST /install_firmware_from_url downloads and flashes remote firmware",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(ADV, 271),
        ),
        GroundedItem::new(
            Precondition::Other("A compatible flight controller board is connected"),
            Provenance::doc(ADV, 283),
        ),
    ]),
    steps: GroundedSet::known(&[
        operator_step(
            "Select vehicle type (Sub, Rover, Plane, or Copter)",
            None,
            Provenance::doc(ADV, 272),
            None,
        ),
        operator_step(
            "Select desired release and stability level (Stable, Beta, or Dev)",
            None,
            Provenance::doc(ADV, 273),
            None,
        ),
        operator_step(
            "Install firmware from the online repository",
            Some(sourced_route(
                HttpMethod::Post,
                "/install_firmware_from_url",
                Some("v1.0"),
                163,
            )),
            Provenance::doc(ADV, 271),
            Some(runtime_outcome(
                200,
                None,
                None,
                "runtime-captures/ardupilot_manager__pi4_navigator_master.json#firmware_operations",
            )),
        ),
    ]),
    availability: FeatureAvailability::unknown(),
    chains_from: Some(JourneyId::ChangeBoard),
};

const UPLOAD_CUSTOM_FIRMWARE: UserJourney = UserJourney {
    id: JourneyId::UploadCustomFirmware,
    summary: Grounded::known(
        "Upload and flash a custom ArduPilot firmware file from the surface computer",
        Provenance::doc(ADV, 281),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 268)),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::FlashFirmware,
        "POST /install_firmware_from_file flashes uploaded firmware image",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("A compatible flight controller board is connected"),
        Provenance::doc(ADV, 283),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "Upload a custom firmware file from the surface computer and flash it onto the board",
        Some(sourced_route(
            HttpMethod::Post,
            "/install_firmware_from_file",
            Some("v1.0"),
            193,
        )),
        Provenance::doc(ADV, 281),
        Some(runtime_outcome(
            200,
            None,
            None,
            "runtime-captures/ardupilot_manager__pi4_navigator_master.json#firmware_operations",
        )),
    )]),
    availability: FeatureAvailability::unknown(),
    chains_from: Some(JourneyId::ChangeBoard),
};

const RESTORE_DEFAULT_FIRMWARE: UserJourney = UserJourney {
    id: JourneyId::RestoreDefaultFirmware,
    summary: Grounded::known(
        "Restore the default ArduSub firmware for the connected flight controller",
        Provenance::doc(ADV, 282),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 268)),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::FlashFirmware,
        "POST /restore_default_firmware flashes factory default firmware",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("A flight controller board is connected"),
        Provenance::doc(ADV, 282),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "Restore the default ArduSub firmware for the connected flight controller",
        Some(sourced_route(
            HttpMethod::Post,
            "/restore_default_firmware",
            Some("v1.0"),
            279,
        )),
        Provenance::doc(ADV, 282),
        Some(runtime_outcome(
            200,
            None,
            None,
            "runtime-captures/ardupilot_manager__pi4_navigator_master.json#firmware_operations",
        )),
    )]),
    availability: FeatureAvailability::unknown(),
    chains_from: Some(JourneyId::ChangeBoard),
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const ARDUPILOT_MANAGER_SERVICES: GroundedSet<ServiceId> =
    GroundedSet::known(&[GroundedItem::new(
        ServiceId::ArdupilotManager,
        Provenance::doc(ADV, 257),
    )]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::ArdupilotManager,
        method,
        path,
        version,
    }
}

const fn sourced_route(
    method: HttpMethod,
    path: &'static str,
    version: Option<&'static str>,
    line: u32,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(APM_ROUTER, line),
    )
}

const fn operator_step(
    description: &'static str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description,
            route,
            outcome,
        },
        provenance,
    )
}

const fn service_step(
    description: &'static str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Service(ServiceId::ArdupilotManager),
            description,
            route,
            outcome,
        },
        provenance,
    )
}

const fn source_outcome(status: u16, file: &'static str, line: u32) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            transition: None,
        },
        Provenance::source(file, line),
    )
}

const fn runtime_outcome(
    status: u16,
    body: Option<&'static str>,
    transition: Option<StateTransition>,
    key: &'static str,
) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            transition,
        },
        Provenance::runtime(key, RUNTIME_ENV),
    )
}

const fn autopilot_transition(from: &'static str, to: &'static str) -> StateTransition {
    StateTransition {
        machine: "autopilot_lifecycle",
        from,
        to,
    }
}
