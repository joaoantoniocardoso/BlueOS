use crate::journey_presence::{
    PRESENCE_CHANGE_BOARD, PRESENCE_RESTART_AUTOPILOT, PRESENCE_RESTORE_DEFAULT_FIRMWARE,
    PRESENCE_RUN_SITL_SIMULATION, PRESENCE_START_AUTOPILOT, PRESENCE_STOP_AUTOPILOT,
    PRESENCE_UPDATE_FIRMWARE_ONLINE, PRESENCE_UPLOAD_CUSTOM_FIRMWARE, PRESENCE_VEHICLE_FIRST_BOOT,
};
use catalog_kernel::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_ARDUSUB;
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use catalog_model::journey::{
    Actor, BlastRadius, BodyKind, HttpMethod, JourneyStep, NetworkState, Precondition, RouteRef,
    StateTransition, StepOutcome, UseCase, Visibility,
};

const ADV: &str = "content/usage/advanced/index.md";
const GS: &str = "content/usage/getting-started/index.md";
const INSTALL: &str = "content/usage/installation.md";
const APM_ROUTER: &str = "core/services/ardupilot_manager/api/v1/routers/index.py";
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_ARDUSUB;

const BR_VEHICLE_FIRST_BOOT: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Destructive,
    Provenance::asserted(
        "first-boot wizard downloads and flashes autopilot firmware onto the flight controller",
    ),
);
const BR_CHANGE_BOARD: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Disruptive,
    Provenance::asserted(
        "POST /board switches the active flight controller or SITL target and restarts autopilot",
    ),
);
const BR_RUN_SITL: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Disruptive,
    Provenance::asserted(
        "selecting SITL and setting /sitl_frame stops hardware FC and runs a simulated autopilot",
    ),
);
const BR_START_AUTOPILOT: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Disruptive,
    Provenance::asserted(
        "POST /start launches the flight-controller subprocess and restores MAVLink control",
    ),
);
const BR_STOP_AUTOPILOT: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Disruptive,
    Provenance::asserted(
        "POST /stop terminates the flight-controller subprocess and drops MAVLink vehicle control",
    ),
);
const BR_RESTART_AUTOPILOT: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Disruptive,
    Provenance::asserted(
        "POST /restart cycles the flight-controller subprocess and briefly interrupts MAVLink",
    ),
);
const BR_UPDATE_FIRMWARE_ONLINE: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Destructive,
    Provenance::asserted(
        "POST /install_firmware_from_url downloads and flashes firmware onto the flight controller",
    ),
);
const BR_UPLOAD_CUSTOM_FIRMWARE: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Destructive,
    Provenance::asserted(
        "POST /install_firmware_from_file flashes an uploaded image onto the flight controller",
    ),
);
const BR_RESTORE_DEFAULT_FIRMWARE: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Destructive,
    Provenance::asserted(
        "POST /restore_default_firmware re-flashes factory default firmware onto the flight controller",
    ),
);

pub const JOURNEYS: &[UseCase] = &[
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

const VEHICLE_FIRST_BOOT: UseCase =
    UseCase {
        id: JourneyId::VehicleFirstBoot,
        summary: Grounded::known(
            "On first boot the configuration wizard downloads and installs up-to-date autopilot firmware",
            Provenance::doc(GS, 52, "Progress is displayed for any selected configuration changes"),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 256, "### Autopilot Firmware")),
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
                Provenance::doc(INSTALL, 59, "- The first boot may take a couple of minutes, as it expands"),
            ),
            GroundedItem::new(
                Precondition::Other("BlueOS is newly installed and the configuration wizard is available"),
                Provenance::doc(GS, 38, "When BlueOS is newly installed the interface provides a conf"),
            ),
        ]),
        steps: GroundedSet::known(&[
            operator_step(
                "Power on the vehicle and wait for the first boot to complete",
                None,
                Provenance::doc(INSTALL, 59, "- The first boot may take a couple of minutes, as it expands"),
                None,
            ),
            service_step(
                "Download and install up-to-date autopilot firmware for the selected vehicle type",
                Some(sourced_route(HttpMethod::Post, "/install_firmware_from_url", Some("v1.0"), 163, "@index_router")),
                Provenance::doc(GS, 52, "Progress is displayed for any selected configuration changes"),
                Some(runtime_outcome(200, None, None, "runtime-captures/ardupilot_manager__pi4_navigator_master.json#firmware_operations")),
            ),
            operator_step(
                "Open the Autopilot Firmware page to view basic information about the active autopilot",
                Some(sourced_route(HttpMethod::Get, "/firmware_info", Some("v1.0"), 103, "@index_router_v1.get(\"/f")),
                Provenance::doc(ADV, 259, "The Autopilot Firmware page provides basic information about"),
                Some(runtime_outcome(
                    200,
                    Some("\"version\": \"4.5.3\""),
                    None,
                    "runtime-captures/ardupilot_manager__pi4_navigator_master.json#running_navigator",
                )),
            ),
        ]),
        availability: PRESENCE_VEHICLE_FIRST_BOOT,
    blast_radius: BR_VEHICLE_FIRST_BOOT,
    chains_from: None,
    };

const CHANGE_BOARD: UseCase = UseCase {
    id: JourneyId::ChangeBoard,
    summary: Grounded::known(
        "Select a connected flight controller board or switch to the virtual SITL board",
        Provenance::doc(
            ADV,
            262,
            "- Change board (select a connected board, or run a [SITL sim",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Advanced,
        Provenance::doc(ADV, 261, "{% pirate() %}"),
    ),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::SelectFlightControllerBoard,
        "POST /board switches between connected boards and SITL",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other(
            "At least one flight controller board is connected or SITL simulation is available",
        ),
        Provenance::doc(
            ADV,
            262,
            "- Change board (select a connected board, or run a [SITL sim",
        ),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "Select a connected board or the SITL simulation board on the Autopilot Firmware page",
        Some(sourced_route(
            HttpMethod::Post,
            "/board",
            Some("v1.0"),
            226,
            "@index_router_v1.post(\"/board\", summary=\"Set board to be use",
        )),
        Provenance::doc(
            ADV,
            262,
            "- Change board (select a connected board, or run a [SITL sim",
        ),
        Some(runtime_outcome(
            200,
            None,
            None,
            "runtime-captures/ardupilot_manager__pi4_navigator_master.json#transitions",
        )),
    )]),
    availability: PRESENCE_CHANGE_BOARD,
    blast_radius: BR_CHANGE_BOARD,
    chains_from: None,
};

const RUN_SITL_SIMULATION: UseCase =
    UseCase {
        id: JourneyId::RunSitlSimulation,
        summary: Grounded::known(
            "Run ArduPilot SITL simulation by selecting the virtual board and configuring the vehicle frame"
                ,
            Provenance::doc(ADV, 305, "If you want to do some testing without needing physical hard"),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 303, "{% pirate() %}")),
        services: ARDUPILOT_MANAGER_SERVICES,
        capability_refs: GroundedSet::known(&[
            cap(CapabilityId::SelectFlightControllerBoard,
                "journey begins by selecting the virtual SITL board",
            ),
            cap(CapabilityId::ConfigureSitlFrame,
                "POST /sitl_frame sets simulated vehicle frame",
            ),
        ]),
        preconditions: GroundedSet::known(&[
            GroundedItem::new(
                Precondition::Other("Virtual SITL flight controller board is selected"),
                Provenance::doc(ADV, 262, "- Change board (select a connected board, or run a [SITL sim"),
            ),
        ]),
        steps: GroundedSet::known(&[
            operator_step(
                "Select the virtual SITL flight controller board",
                Some(sourced_route(HttpMethod::Post, "/board", Some("v1.0"), 226, "@index_router_v1.post(\"/board\",")),
                Provenance::doc(ADV, 262, "- Change board (select a connected board, or run a [SITL sim"),
                Some(runtime_outcome(200, None, None, "runtime-captures/ardupilot_manager__pi4_navigator_master.json#transitions")),
            ),
            operator_step(
                "Set the SITL vehicle frame in the POST /sitl_frame endpoint",
                Some(sourced_route(HttpMethod::Post, "/sitl_frame", Some("v1.0"), 133, "@index_router_v1.post(\"/si")),
                Provenance::doc(ADV, 319, "\"Try it out\" in the `POST: /sitl_frame` endpoint, select the"),
                Some(source_outcome(200, APM_ROUTER, 135, "async def set_sitl_frame(frame: SITLFrame) -> Any:")),
            ),
        ]),
        availability: PRESENCE_RUN_SITL_SIMULATION,
    blast_radius: BR_RUN_SITL,
    chains_from: Some(JourneyId::ChangeBoard),
    };

const START_AUTOPILOT: UseCase = UseCase {
    id: JourneyId::StartAutopilot,
    summary: Grounded::known(
        "Start the autopilot process",
        Provenance::doc(ADV, 263, "- Start the autopilot"),
    ),
    visibility: Grounded::known(
        Visibility::Advanced,
        Provenance::doc(ADV, 261, "{% pirate() %}"),
    ),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ManageAutopilotLifecycle,
        "POST /start launches the FC subprocess",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("A flight controller board is selected"),
        Provenance::doc(
            ADV,
            262,
            "- Change board (select a connected board, or run a [SITL sim",
        ),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "Start the autopilot from the Autopilot Firmware page",
        Some(sourced_route(
            HttpMethod::Post,
            "/start",
            Some("v1.0"),
            241,
            "@index_router_v1.post(\"/start\", summary=\"Start the autopilot",
        )),
        Provenance::doc(ADV, 263, "- Start the autopilot"),
        Some(runtime_outcome(
            200,
            None,
            Some(autopilot_transition("stopped", "running")),
            "runtime-captures/ardupilot_manager__pi4_navigator_master.json#transitions",
        )),
    )]),
    availability: PRESENCE_START_AUTOPILOT,
    blast_radius: BR_START_AUTOPILOT,
    chains_from: Some(JourneyId::ChangeBoard),
};

const STOP_AUTOPILOT: UseCase = UseCase {
    id: JourneyId::StopAutopilot,
    summary: Grounded::known(
        "Stop the autopilot process",
        Provenance::doc(ADV, 264, "- Stop the autopilot"),
    ),
    visibility: Grounded::known(
        Visibility::Advanced,
        Provenance::doc(ADV, 261, "{% pirate() %}"),
    ),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ManageAutopilotLifecycle,
        "POST /stop terminates the FC subprocess",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("An autopilot is available to stop"),
        Provenance::doc(ADV, 264, "- Stop the autopilot"),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "Stop the autopilot from the Autopilot Firmware page",
        Some(sourced_route(
            HttpMethod::Post,
            "/stop",
            Some("v1.0"),
            270,
            "@index_router_v1.post(\"/stop\", summary=\"Stop the autopilot.\"",
        )),
        Provenance::doc(ADV, 264, "- Stop the autopilot"),
        Some(runtime_outcome(
            200,
            None,
            Some(autopilot_transition("running", "stopped")),
            "runtime-captures/ardupilot_manager__pi4_navigator_master.json#transitions",
        )),
    )]),
    availability: PRESENCE_STOP_AUTOPILOT,
    blast_radius: BR_STOP_AUTOPILOT,
    chains_from: None,
};

const RESTART_AUTOPILOT: UseCase = UseCase {
    id: JourneyId::RestartAutopilot,
    summary: Grounded::known(
        "Restart the autopilot",
        Provenance::doc(ADV, 267, "- Restart the autopilot"),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 267, "- Restart the autopilot"),
    ),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ManageAutopilotLifecycle,
        "POST /restart stops and relaunches the FC process",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("A flight controller board is selected"),
        Provenance::doc(ADV, 283, "- Flash firmware onto a connected compatible"),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "Restart the autopilot from the Autopilot Firmware page",
        Some(sourced_route(
            HttpMethod::Post,
            "/restart",
            Some("v1.0"),
            233,
            "@index_router_v1.post(\"/restart\", summary=\"Restart the autop",
        )),
        Provenance::doc(ADV, 267, "- Restart the autopilot"),
        Some(runtime_outcome(
            200,
            None,
            None,
            "runtime-captures/ardupilot_manager__pi4_navigator_master.json#transitions",
        )),
    )]),
    availability: PRESENCE_RESTART_AUTOPILOT,
    blast_radius: BR_RESTART_AUTOPILOT,
    chains_from: None,
};

const UPDATE_FIRMWARE_ONLINE: UseCase = UseCase {
    id: JourneyId::UpdateFirmwareOnline,
    summary: Grounded::known(
        "Update flight-controller firmware from the online ArduPilot repository",
        Provenance::doc(ADV, 268, "- Update the firmware"),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 268, "- Update the firmware"),
    ),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::FlashFirmware,
        "POST /install_firmware_from_url downloads and flashes remote firmware",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Network(NetworkState::Online),
            Provenance::doc(ADV, 271, "- Select from the online repository"),
        ),
        GroundedItem::new(
            Precondition::Other("A compatible flight controller board is connected"),
            Provenance::doc(ADV, 283, "- Flash firmware onto a connected compatible"),
        ),
    ]),
    steps: GroundedSet::known(&[
        operator_step(
            "Select vehicle type (Sub, Rover, Plane, or Copter)",
            None,
            Provenance::doc(
                ADV,
                272,
                "- Select vehicle type (Sub / Rover / Plane / Copter)",
            ),
            None,
        ),
        operator_step(
            "Select desired release and stability level (Stable, Beta, or Dev)",
            None,
            Provenance::doc(ADV, 273, "- Select desired release and stability level"),
            None,
        ),
        operator_step(
            "Install firmware from the online repository",
            Some(sourced_route(
                HttpMethod::Post,
                "/install_firmware_from_url",
                Some("v1.0"),
                163,
                "@index_router_v1.post(\"/install_firmware_from_url\", summary=",
            )),
            Provenance::doc(ADV, 271, "- Select from the online repository"),
            Some(runtime_outcome(
                200,
                None,
                None,
                "runtime-captures/ardupilot_manager__pi4_navigator_master.json#firmware_operations",
            )),
        ),
    ]),
    availability: PRESENCE_UPDATE_FIRMWARE_ONLINE,
    blast_radius: BR_UPDATE_FIRMWARE_ONLINE,
    chains_from: Some(JourneyId::ChangeBoard),
};

const UPLOAD_CUSTOM_FIRMWARE: UseCase = UseCase {
    id: JourneyId::UploadCustomFirmware,
    summary: Grounded::known(
        "Upload and flash a custom ArduPilot firmware file from the surface computer",
        Provenance::doc(
            ADV,
            281,
            "- Upload a custom firmware file from the surface computer",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 268, "- Update the firmware"),
    ),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::FlashFirmware,
        "POST /install_firmware_from_file flashes uploaded firmware image",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("A compatible flight controller board is connected"),
        Provenance::doc(ADV, 283, "- Flash firmware onto a connected compatible"),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "Upload a custom firmware file from the surface computer and flash it onto the board",
        Some(sourced_route(
            HttpMethod::Post,
            "/install_firmware_from_file",
            Some("v1.0"),
            193,
            "@index_router_v1.post(\"/install_firmware_from_file\", summary",
        )),
        Provenance::doc(
            ADV,
            281,
            "- Upload a custom firmware file from the surface computer",
        ),
        Some(runtime_outcome(
            200,
            None,
            None,
            "runtime-captures/ardupilot_manager__pi4_navigator_master.json#firmware_operations",
        )),
    )]),
    availability: PRESENCE_UPLOAD_CUSTOM_FIRMWARE,
    blast_radius: BR_UPLOAD_CUSTOM_FIRMWARE,
    chains_from: Some(JourneyId::ChangeBoard),
};

const RESTORE_DEFAULT_FIRMWARE: UseCase = UseCase {
    id: JourneyId::RestoreDefaultFirmware,
    summary: Grounded::known(
        "Restore the default ArduSub firmware for the connected flight controller",
        Provenance::doc(
            ADV,
            282,
            "- Restore the default (ArduSub) firmware for the connected f",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 268, "- Update the firmware"),
    ),
    services: ARDUPILOT_MANAGER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::FlashFirmware,
        "POST /restore_default_firmware flashes factory default firmware",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Other("A flight controller board is connected"),
        Provenance::doc(
            ADV,
            282,
            "- Restore the default (ArduSub) firmware for the connected f",
        ),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "Restore the default ArduSub firmware for the connected flight controller",
        Some(sourced_route(
            HttpMethod::Post,
            "/restore_default_firmware",
            Some("v1.0"),
            279,
            "@index_router_v1.post(\"/restore_default_firmware\", summary=\"",
        )),
        Provenance::doc(
            ADV,
            282,
            "- Restore the default (ArduSub) firmware for the connected f",
        ),
        Some(runtime_outcome(
            200,
            None,
            None,
            "runtime-captures/ardupilot_manager__pi4_navigator_master.json#firmware_operations",
        )),
    )]),
    availability: PRESENCE_RESTORE_DEFAULT_FIRMWARE,
    blast_radius: BR_RESTORE_DEFAULT_FIRMWARE,
    chains_from: Some(JourneyId::ChangeBoard),
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const ARDUPILOT_MANAGER_SERVICES: GroundedSet<ServiceId> =
    GroundedSet::known(&[GroundedItem::new(
        ServiceId::ArdupilotManager,
        Provenance::doc(
            ADV,
            257,
            "{{ service(service=\"ArduPilot Manager\", port=8000, link=\"/se",
        ),
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
    anchor: &'static str,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(APM_ROUTER, line, anchor),
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

const fn source_outcome(
    status: u16,
    file: &'static str,
    line: u32,
    anchor: &'static str,
) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            body_kind: BodyKind::Unknown,
            transition: None,
        },
        Provenance::source(file, line, anchor),
    )
}

const fn runtime_outcome(
    status: u16,
    body: Option<&'static str>,
    transition: Option<StateTransition>,
    key: &'static str,
) -> Grounded<StepOutcome> {
    let body_kind = if body.is_some() {
        BodyKind::Payload
    } else {
        BodyKind::Unknown
    };
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            body_kind,
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
