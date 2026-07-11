use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, NetworkState, Precondition, RouteRef, StateTransition,
    StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const GS: &str = "content/usage/getting-started/index.md";
const INSTALL: &str = "content/usage/installation.md";
const APM_ROUTER: &str = "core/services/ardupilot_manager/api/v1/routers/index.py";
const RUNTIME_CAPTURE: &str = "runtime-captures/ardupilot_manager__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e), Raspberry Pi 4, Navigator, ArduSub 4.5.3 STABLE";

pub fn journeys() -> Vec<UserJourney> {
    vec![
        vehicle_first_boot(),
        change_board(),
        run_sitl_simulation(),
        start_autopilot(),
        stop_autopilot(),
        restart_autopilot(),
        update_firmware_online(),
        upload_custom_firmware(),
        restore_default_firmware(),
    ]
}

fn vehicle_first_boot() -> UserJourney {
    UserJourney {
        id: JourneyId("vehicle_first_boot".into()),
        summary: Grounded::known(
            "On first boot the configuration wizard downloads and installs up-to-date autopilot firmware".into(),
            Provenance::doc(GS, 52),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 256)),
        services: ardupilot_manager_services(),
        capability_refs: GroundedSet::known(vec![
            cap(
                "detect_flight_controllers",
                "wizard discovers connected boards before firmware install",
            ),
            cap(
                "flash_firmware",
                "install_firmware_from_url during first-boot wizard",
            ),
            cap(
                "manage_autopilot_lifecycle",
                "first boot completes with a running autopilot process",
            ),
            cap(
                "query_vehicle_firmware_info",
                "first-boot flow reads firmware_info to confirm the active autopilot",
            ),
        ]),
        preconditions: GroundedSet::known(vec![
            GroundedItem::new(
                Precondition::Other("Cold start after power-on".into()),
                Provenance::doc(INSTALL, 59),
            ),
            GroundedItem::new(
                Precondition::Other("BlueOS is newly installed and the configuration wizard is available".into()),
                Provenance::doc(GS, 38),
            ),
        ]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Power on the vehicle and wait for the first boot to complete",
                None,
                Provenance::doc(INSTALL, 59),
                None,
            ),
            service_step(
                "Download and install up-to-date autopilot firmware for the selected vehicle type",
                Some(sourced_route(HttpMethod::Post, "/install_firmware_from_url", None, 163)),
                Provenance::doc(GS, 52),
                Some(runtime_outcome(200, None, None, "#firmware_operations")),
            ),
            operator_step(
                "Open the Autopilot Firmware page to view basic information about the active autopilot",
                Some(sourced_route(HttpMethod::Get, "/firmware_info", None, 103)),
                Provenance::doc(ADV, 259),
                Some(runtime_outcome(
                    200,
                    Some("\"version\": \"4.5.3\"".into()),
                    None,
                    "#running_navigator",
                )),
            ),
        ]),
        chains_from: None,
    }
}

fn change_board() -> UserJourney {
    UserJourney {
        id: JourneyId("change_board".into()),
        summary: Grounded::known(
            "Select a connected flight controller board or switch to the virtual SITL board".into(),
            Provenance::doc(ADV, 262),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 261)),
        services: ardupilot_manager_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "select_flight_controller_board",
            "POST /board switches between connected boards and SITL",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other(
                "At least one flight controller board is connected or SITL simulation is available"
                    .into(),
            ),
            Provenance::doc(ADV, 262),
        )]),
        steps: GroundedSet::known(vec![operator_step(
            "Select a connected board or the SITL simulation board on the Autopilot Firmware page",
            Some(sourced_route(HttpMethod::Post, "/board", None, 226)),
            Provenance::doc(ADV, 262),
            Some(runtime_outcome(200, None, None, "#transitions")),
        )]),
        chains_from: None,
    }
}

fn run_sitl_simulation() -> UserJourney {
    UserJourney {
        id: JourneyId("run_sitl_simulation".into()),
        summary: Grounded::known(
            "Run ArduPilot SITL simulation by selecting the virtual board and configuring the vehicle frame"
                .into(),
            Provenance::doc(ADV, 305),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 303)),
        services: ardupilot_manager_services(),
        capability_refs: GroundedSet::known(vec![
            cap(
                "select_flight_controller_board",
                "journey begins by selecting the virtual SITL board",
            ),
            cap(
                "configure_sitl_frame",
                "POST /sitl_frame sets simulated vehicle frame",
            ),
        ]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("Virtual SITL flight controller board is selected".into()),
            Provenance::doc(ADV, 262),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Select the virtual SITL flight controller board",
                Some(sourced_route(HttpMethod::Post, "/board", None, 226)),
                Provenance::doc(ADV, 262),
                Some(runtime_outcome(200, None, None, "#transitions")),
            ),
            operator_step(
                "Set the SITL vehicle frame in the POST /sitl_frame endpoint",
                Some(sourced_route(HttpMethod::Post, "/sitl_frame", Some("v2.0"), 133)),
                Provenance::doc(ADV, 319),
                None,
            ),
        ]),
        chains_from: Some(JourneyId("change_board".into())),
    }
}

fn start_autopilot() -> UserJourney {
    UserJourney {
        id: JourneyId("start_autopilot".into()),
        summary: Grounded::known(
            "Start the autopilot process".into(),
            Provenance::doc(ADV, 263),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 261)),
        services: ardupilot_manager_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "manage_autopilot_lifecycle",
            "POST /start launches the FC subprocess",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("A flight controller board is selected".into()),
            Provenance::doc(ADV, 262),
        )]),
        steps: GroundedSet::known(vec![operator_step(
            "Start the autopilot from the Autopilot Firmware page",
            Some(sourced_route(HttpMethod::Post, "/start", None, 241)),
            Provenance::doc(ADV, 263),
            Some(runtime_outcome(
                200,
                None,
                Some(autopilot_transition("stopped", "running")),
                "#transitions",
            )),
        )]),
        chains_from: Some(JourneyId("change_board".into())),
    }
}

fn stop_autopilot() -> UserJourney {
    UserJourney {
        id: JourneyId("stop_autopilot".into()),
        summary: Grounded::known(
            "Stop the autopilot process".into(),
            Provenance::doc(ADV, 264),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 261)),
        services: ardupilot_manager_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "manage_autopilot_lifecycle",
            "POST /stop terminates the FC subprocess",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("An autopilot is available to stop".into()),
            Provenance::doc(ADV, 264),
        )]),
        steps: GroundedSet::known(vec![operator_step(
            "Stop the autopilot from the Autopilot Firmware page",
            Some(sourced_route(HttpMethod::Post, "/stop", None, 270)),
            Provenance::doc(ADV, 264),
            Some(runtime_outcome(
                200,
                None,
                Some(autopilot_transition("running", "stopped")),
                "#transitions",
            )),
        )]),
        chains_from: None,
    }
}

fn restart_autopilot() -> UserJourney {
    UserJourney {
        id: JourneyId("restart_autopilot".into()),
        summary: Grounded::known("Restart the autopilot".into(), Provenance::doc(ADV, 267)),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 267)),
        services: ardupilot_manager_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "manage_autopilot_lifecycle",
            "POST /restart stops and relaunches the FC process",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("A flight controller board is selected".into()),
            Provenance::doc(ADV, 283),
        )]),
        steps: GroundedSet::known(vec![operator_step(
            "Restart the autopilot from the Autopilot Firmware page",
            Some(sourced_route(HttpMethod::Post, "/restart", None, 233)),
            Provenance::doc(ADV, 267),
            Some(runtime_outcome(200, None, None, "#transitions")),
        )]),
        chains_from: None,
    }
}

fn update_firmware_online() -> UserJourney {
    UserJourney {
        id: JourneyId("update_firmware_online".into()),
        summary: Grounded::known(
            "Update flight-controller firmware from the online ArduPilot repository".into(),
            Provenance::doc(ADV, 268),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 268)),
        services: ardupilot_manager_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "flash_firmware",
            "POST /install_firmware_from_url downloads and flashes remote firmware",
        )]),
        preconditions: GroundedSet::known(vec![
            GroundedItem::new(
                Precondition::Network(NetworkState::Online),
                Provenance::doc(ADV, 271),
            ),
            GroundedItem::new(
                Precondition::Other("A compatible flight controller board is connected".into()),
                Provenance::doc(ADV, 283),
            ),
        ]),
        steps: GroundedSet::known(vec![
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
                    None,
                    163,
                )),
                Provenance::doc(ADV, 271),
                Some(runtime_outcome(200, None, None, "#firmware_operations")),
            ),
        ]),
        chains_from: Some(JourneyId("change_board".into())),
    }
}

fn upload_custom_firmware() -> UserJourney {
    UserJourney {
        id: JourneyId("upload_custom_firmware".into()),
        summary: Grounded::known(
            "Upload and flash a custom ArduPilot firmware file from the surface computer".into(),
            Provenance::doc(ADV, 281),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 268)),
        services: ardupilot_manager_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "flash_firmware",
            "POST /install_firmware_from_file flashes uploaded firmware image",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("A compatible flight controller board is connected".into()),
            Provenance::doc(ADV, 283),
        )]),
        steps: GroundedSet::known(vec![operator_step(
            "Upload a custom firmware file from the surface computer and flash it onto the board",
            Some(sourced_route(
                HttpMethod::Post,
                "/install_firmware_from_file",
                None,
                193,
            )),
            Provenance::doc(ADV, 281),
            Some(runtime_outcome(200, None, None, "#firmware_operations")),
        )]),
        chains_from: Some(JourneyId("change_board".into())),
    }
}

fn restore_default_firmware() -> UserJourney {
    UserJourney {
        id: JourneyId("restore_default_firmware".into()),
        summary: Grounded::known(
            "Restore the default ArduSub firmware for the connected flight controller".into(),
            Provenance::doc(ADV, 282),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 268)),
        services: ardupilot_manager_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "flash_firmware",
            "POST /restore_default_firmware flashes factory default firmware",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("A flight controller board is connected".into()),
            Provenance::doc(ADV, 282),
        )]),
        steps: GroundedSet::known(vec![operator_step(
            "Restore the default ArduSub firmware for the connected flight controller",
            Some(sourced_route(
                HttpMethod::Post,
                "/restore_default_firmware",
                None,
                279,
            )),
            Provenance::doc(ADV, 282),
            Some(runtime_outcome(200, None, None, "#firmware_operations")),
        )]),
        chains_from: Some(JourneyId("change_board".into())),
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn ardupilot_manager_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId("ardupilot_manager".into()),
        Provenance::doc(ADV, 257),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId("ardupilot_manager".into()),
        method,
        path: path.into(),
        version: version.map(str::to_string),
    }
}

fn sourced_route(
    method: HttpMethod,
    path: &str,
    version: Option<&str>,
    line: u32,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(APM_ROUTER, line),
    )
}

fn operator_step(
    description: &str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description: description.into(),
            route,
            outcome,
        },
        provenance,
    )
}

fn service_step(
    description: &str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Service(ServiceId("ardupilot_manager".into())),
            description: description.into(),
            route,
            outcome,
        },
        provenance,
    )
}

fn runtime_outcome(
    status: u16,
    body: Option<String>,
    transition: Option<StateTransition>,
    key: &str,
) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            transition,
        },
        Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV),
    )
}

fn autopilot_transition(from: &str, to: &str) -> StateTransition {
    StateTransition {
        machine: "autopilot_lifecycle".into(),
        from: from.into(),
        to: to.into(),
    }
}
