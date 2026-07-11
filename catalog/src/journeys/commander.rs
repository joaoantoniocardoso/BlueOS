use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const DEV_CORE: &str = "content/development/core/index.md";
const APP_VUE: &str = "core/frontend/src/App.vue";
const COMMANDER_MAIN: &str = "core/services/commander/main.py";
const COMMANDER_STORE: &str = "core/frontend/src/store/commander.ts";
const FIRMWARE: &str = "core/frontend/src/components/system-information/Firmware.vue";
const POWER_MENU: &str = "core/frontend/src/components/app/PowerMenu.vue";
const SETTINGS_VIEW: &str = "core/frontend/src/views/SettingsView.vue";
const SYSINFO_VIEW: &str = "core/frontend/src/views/SystemInformationView.vue";
const UPDATE_TIME: &str = "core/frontend/src/utils/update_time.ts";
const VIDEO_MANAGER: &str = "core/frontend/src/components/video-manager/VideoManager.vue";

pub fn journeys() -> Vec<UserJourney> {
    vec![
        reboot_onboard_computer(),
        shutdown_onboard_computer(),
        sync_system_time(),
        enable_legacy_camera_support(),
        inspect_raspberry_eeprom_bootloader(),
        update_raspberry_eeprom_bootloader(),
        reset_blueos_settings(),
        run_host_command(),
    ]
}

fn reboot_onboard_computer() -> UserJourney {
    UserJourney {
        id: JourneyId("reboot_onboard_computer".into()),
        summary: Grounded::known(
            "Reboot the onboard computer from the power menu".into(),
            Provenance::doc(ADV, 219),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 211)),
        services: commander_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "reboot_onboard_computer",
            "power menu triggers commander shutdown with reboot type",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("Request must pass i_know_what_i_am_doing=true".into()),
            Provenance::source(COMMANDER_MAIN, 49),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the power menu from the header bar",
                None,
                Provenance::doc(ADV, 213),
                None,
            ),
            operator_step(
                "Select Reboot to restart the onboard computer",
                Some(sourced_route(
                    HttpMethod::Post,
                    "/shutdown",
                    Some("v1.0"),
                    88,
                )),
                Provenance::source(POWER_MENU, 173),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn shutdown_onboard_computer() -> UserJourney {
    UserJourney {
        id: JourneyId("shutdown_onboard_computer".into()),
        summary: Grounded::known(
            "Shut down the onboard computer from the power menu before removing vehicle power"
                .into(),
            Provenance::doc(ADV, 216),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 211)),
        services: commander_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "shutdown_onboard_computer",
            "power menu triggers commander shutdown with poweroff type",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("Request must pass i_know_what_i_am_doing=true".into()),
            Provenance::source(COMMANDER_MAIN, 49),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the power menu from the header bar",
                None,
                Provenance::doc(ADV, 213),
                None,
            ),
            operator_step(
                "Select Power off to shut down the onboard computer",
                Some(sourced_route(
                    HttpMethod::Post,
                    "/shutdown",
                    Some("v1.0"),
                    88,
                )),
                Provenance::source(POWER_MENU, 186),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn sync_system_time() -> UserJourney {
    UserJourney {
        id: JourneyId("sync_system_time".into()),
        summary: Grounded::known(
            "Sync the onboard computer clock with the browser when drift exceeds five minutes"
                .into(),
            Provenance::source(COMMANDER_MAIN, 76),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::source(APP_VUE, 798)),
        services: commander_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "sync_system_time",
            "frontend posts browser unix time to commander set_time on load",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("Request must pass i_know_what_i_am_doing=true".into()),
            Provenance::source(COMMANDER_MAIN, 74),
        )]),
        steps: GroundedSet::known(vec![service_step(
            "On interface load, post the browser unix timestamp to commander",
            Some(sourced_route(
                HttpMethod::Post,
                "/set_time",
                Some("v1.0"),
                72,
            )),
            Provenance::source(UPDATE_TIME, 9),
            None,
        )]),
        chains_from: None,
    }
}

fn enable_legacy_camera_support() -> UserJourney {
    UserJourney {
        id: JourneyId("enable_legacy_camera_support".into()),
        summary: Grounded::known(
            "Enable Raspberry Pi legacy camera support for Pi camera detection".into(),
            Provenance::doc(ADV, 823),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 822)),
        services: commander_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "configure_legacy_camera",
            "video manager toggles raspi-config legacy camera via commander",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open Video Manager settings from the gear button in the bottom right corner",
                None,
                Provenance::doc(ADV, 824),
                None,
            ),
            operator_step(
                "Turn on Raspberry legacy camera support",
                Some(sourced_route(
                    HttpMethod::Post,
                    "/raspi_config/camera_legacy",
                    Some("v1.0"),
                    117,
                )),
                Provenance::source(VIDEO_MANAGER, 66),
                None,
            ),
            operator_step(
                "Reboot the onboard computer to apply legacy camera support",
                None,
                Provenance::doc(ADV, 825),
                None,
            ),
        ]),
        chains_from: Some(JourneyId("reboot_onboard_computer".into())),
    }
}

fn inspect_raspberry_eeprom_bootloader() -> UserJourney {
    UserJourney {
        id: JourneyId("inspect_raspberry_eeprom_bootloader".into()),
        summary: Grounded::known(
            "View Raspberry Pi firmware, bootloader, and EEPROM update availability".into(),
            Provenance::doc(ADV, 610),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 611)),
        services: commander_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "inspect_raspberry_eeprom",
            "system information firmware tab reads vcgencmd and rpi-eeprom-update output",
        )]),
        preconditions: GroundedSet::known(vec![
            GroundedItem::new(
                Precondition::Other("Pirate mode enabled to access the Firmware tab".into()),
                Provenance::source(SYSINFO_VIEW, 85),
            ),
            GroundedItem::new(
                Precondition::Other("Request must pass i_know_what_i_am_doing=true".into()),
                Provenance::source(COMMANDER_MAIN, 49),
            ),
        ]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Firmware tab on the System Information page",
                None,
                Provenance::doc(ADV, 610),
                None,
            ),
            operator_step(
                "Load Raspberry Pi firmware and bootloader versions",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/raspi/vcgencmd",
                    Some("v1.0"),
                    132,
                )),
                Provenance::source(FIRMWARE, 232),
                None,
            ),
            operator_step(
                "Load current and latest EEPROM bootloader and USB controller versions",
                Some(sourced_route(
                    HttpMethod::Get,
                    "/raspi/eeprom_update",
                    Some("v1.0"),
                    150,
                )),
                Provenance::source(FIRMWARE, 233),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn update_raspberry_eeprom_bootloader() -> UserJourney {
    UserJourney {
        id: JourneyId("update_raspberry_eeprom_bootloader".into()),
        summary: Grounded::known(
            "Update Raspberry Pi firmware and USB controller EEPROM to the latest stable versions"
                .into(),
            Provenance::doc(ADV, 612),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 611)),
        services: commander_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "update_raspberry_eeprom",
            "firmware tab applies rpi-eeprom-update when updates are available",
        )]),
        preconditions: GroundedSet::known(vec![
            GroundedItem::new(
                Precondition::Other("Pirate mode enabled to access the Firmware tab".into()),
                Provenance::source(SYSINFO_VIEW, 85),
            ),
            GroundedItem::new(
                Precondition::Other("Request must pass i_know_what_i_am_doing=true".into()),
                Provenance::source(COMMANDER_MAIN, 49),
            ),
        ]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the Firmware tab on the System Information page",
                None,
                Provenance::doc(ADV, 610),
                None,
            ),
            operator_step(
                "Click Update when bootloader or USB controller versions are out of date",
                Some(sourced_route(
                    HttpMethod::Post,
                    "/raspi/eeprom_update",
                    Some("v1.0"),
                    157,
                )),
                Provenance::source(FIRMWARE, 69),
                None,
            ),
        ]),
        chains_from: Some(JourneyId("inspect_raspberry_eeprom_bootloader".into())),
    }
}

fn reset_blueos_settings() -> UserJourney {
    UserJourney {
        id: JourneyId("reset_blueos_settings".into()),
        summary: Grounded::known(
            "Reset BlueOS settings to remove camera, endpoint, and bridge configuration".into(),
            Provenance::doc(ADV, 204),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 200)),
        services: commander_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "reset_blueos_settings",
            "settings page deletes service config while preserving bootstrap state",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("Request must pass i_know_what_i_am_doing=true".into()),
            Provenance::source(COMMANDER_MAIN, 49),
        )]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open BlueOS Settings from the header bar",
                None,
                Provenance::doc(ADV, 202),
                None,
            ),
            operator_step(
                "Confirm Reset Settings to restore BlueOS services to defaults",
                Some(sourced_route(
                    HttpMethod::Post,
                    "/settings/reset",
                    Some("v1.0"),
                    164,
                )),
                Provenance::source(SETTINGS_VIEW, 669),
                None,
            ),
        ]),
        chains_from: None,
    }
}

fn run_host_command() -> UserJourney {
    UserJourney {
        id: JourneyId("run_host_command".into()),
        summary: Grounded::known(
            "Run an arbitrary bash command on the host through commander".into(),
            Provenance::source(DEV_CORE, 74),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::source(DEV_CORE, 74)),
        services: commander_services(),
        capability_refs: GroundedSet::known(vec![cap(
            "run_host_command",
            "commander executes privileged shell commands when explicitly acknowledged",
        )]),
        preconditions: GroundedSet::known(vec![GroundedItem::new(
            Precondition::Other("Request must pass i_know_what_i_am_doing=true".into()),
            Provenance::source(COMMANDER_MAIN, 49),
        )]),
        steps: GroundedSet::known(vec![operator_step(
            "Post a shell command to commander with i_know_what_i_am_doing acknowledged",
            Some(sourced_route(
                HttpMethod::Post,
                "/command/host",
                Some("v1.0"),
                57,
            )),
            Provenance::source(COMMANDER_STORE, 42),
            None,
        )]),
        chains_from: None,
    }
}

fn cap(id: &str, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(CapabilityId(id.into()), Provenance::asserted(rationale))
}

fn commander_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId("commander".into()),
        Provenance::source(DEV_CORE, 74),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId("commander".into()),
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
        Provenance::source(COMMANDER_MAIN, line),
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
            actor: Actor::Service(ServiceId("commander".into())),
            description: description.into(),
            route,
            outcome,
        },
        provenance,
    )
}
