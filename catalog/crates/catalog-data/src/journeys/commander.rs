use crate::journey_presence::{
    PRESENCE_ENABLE_LEGACY_CAMERA_SUPPORT, PRESENCE_INSPECT_RASPBERRY_EEPROM_BOOTLOADER,
    PRESENCE_REBOOT_ONBOARD_COMPUTER, PRESENCE_RESET_BLUEOS_SETTINGS, PRESENCE_RUN_HOST_COMMAND,
    PRESENCE_SHUTDOWN_ONBOARD_COMPUTER, PRESENCE_SYNC_SYSTEM_TIME,
    PRESENCE_UPDATE_RASPBERRY_EEPROM_BOOTLOADER,
};
use catalog_kernel::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use catalog_model::journey::{
    Actor, BlastRadius, BodyKind, HttpMethod, JourneyStep, Precondition, RouteRef,
    SoftwareAssumption, StepOutcome, UseCase, Visibility,
};

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
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR;

const BR_REBOOT_ONBOARD_COMPUTER: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Destructive,
    Provenance::asserted(
        "POST /shutdown with reboot type restarts the onboard computer and drops all services",
    ),
);
const BR_SHUTDOWN_ONBOARD_COMPUTER: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Destructive,
    Provenance::asserted(
        "POST /shutdown with poweroff type halts the onboard computer until manual power cycle",
    ),
);
const BR_SYNC_SYSTEM_TIME: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Disruptive,
    Provenance::asserted(
        "POST /set_time sets the host system clock from the browser unix timestamp",
    ),
);
const BR_ENABLE_LEGACY_CAMERA_SUPPORT: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Disruptive,
    Provenance::asserted(
        "POST /raspi_config/camera_legacy toggles boot config and chains to an onboard reboot",
    ),
);
const BR_INSPECT_RASPBERRY_EEPROM_BOOTLOADER: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Safe,
    Provenance::asserted(
        "GET /raspi/vcgencmd and /raspi/eeprom_update only read Pi firmware and EEPROM state",
    ),
);
const BR_UPDATE_RASPBERRY_EEPROM_BOOTLOADER: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Destructive,
    Provenance::asserted(
        "POST /raspi/eeprom_update flashes Raspberry Pi bootloader and USB controller EEPROM",
    ),
);
const BR_RESET_BLUEOS_SETTINGS: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Destructive,
    Provenance::asserted(
        "POST /settings/reset deletes service configuration and restores BlueOS defaults",
    ),
);
const BR_RUN_HOST_COMMAND: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Unknown {
        reason: "blast radius depends on the arbitrary privileged shell command posted",
    },
    Provenance::asserted(
        "POST /command/host runs operator-supplied bash with no fixed side effect",
    ),
);

pub const JOURNEYS: &[UseCase] = &[
    REBOOT_ONBOARD_COMPUTER,
    SHUTDOWN_ONBOARD_COMPUTER,
    SYNC_SYSTEM_TIME,
    ENABLE_LEGACY_CAMERA_SUPPORT,
    INSPECT_RASPBERRY_EEPROM_BOOTLOADER,
    UPDATE_RASPBERRY_EEPROM_BOOTLOADER,
    RESET_BLUEOS_SETTINGS,
    RUN_HOST_COMMAND,
];

const REBOOT_ONBOARD_COMPUTER: UseCase = UseCase {
    id: JourneyId::RebootOnboardComputer,
    summary: Grounded::known(
        "Reboot the onboard computer from the power menu",
        Provenance::doc(ADV, 219, "- Reboot onboard computer"),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 211, "##### Power"),
    ),
    services: COMMANDER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RebootOnboardComputer,
        "power menu triggers commander shutdown with reboot type",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Software(SoftwareAssumption::ConfirmDangerousOp),
        Provenance::source(
            COMMANDER_MAIN,
            49,
            "def check_what_i_am_doing(i_know_what_i_am_doing: bool = Fal",
        ),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the power menu from the header bar",
            None,
            Provenance::doc(
                ADV,
                213,
                "{{ simple_pirate_image(src=\"power\", width=300, center=true) ",
            ),
            None,
        ),
        operator_step(
            "Select Reboot to restart the onboard computer",
            Some(sourced_route(
                HttpMethod::Post,
                "/shutdown",
                Some("v1.0"),
                88,
                "@app.post(\"/shutdown\", status_code=status.HTTP_200_OK)",
            )),
            Provenance::source(POWER_MENU, 173, "async reboot(): Promise<void> {"),
            Some(source_outcome(
                200,
                88,
                "@app.post(\"/shutdown\", status_code=status.HTTP_200_OK)",
            )),
        ),
    ]),
    availability: PRESENCE_REBOOT_ONBOARD_COMPUTER,
    blast_radius: BR_REBOOT_ONBOARD_COMPUTER,
    chains_from: None,
};

const SHUTDOWN_ONBOARD_COMPUTER: UseCase = UseCase {
    id: JourneyId::ShutdownOnboardComputer,
    summary: Grounded::known(
        "Shut down the onboard computer from the power menu before removing vehicle power",
        Provenance::doc(ADV, 216, "- Shut down onboard computer"),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 211, "##### Power"),
    ),
    services: COMMANDER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ShutdownOnboardComputer,
        "power menu triggers commander shutdown with poweroff type",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Software(SoftwareAssumption::ConfirmDangerousOp),
        Provenance::source(
            COMMANDER_MAIN,
            49,
            "def check_what_i_am_doing(i_know_what_i_am_doing: bool = Fal",
        ),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the power menu from the header bar",
            None,
            Provenance::doc(
                ADV,
                213,
                "{{ simple_pirate_image(src=\"power\", width=300, center=true) ",
            ),
            None,
        ),
        operator_step(
            "Select Power off to shut down the onboard computer",
            Some(sourced_route(
                HttpMethod::Post,
                "/shutdown",
                Some("v1.0"),
                88,
                "@app.post(\"/shutdown\", status_code=status.HTTP_200_OK)",
            )),
            Provenance::source(POWER_MENU, 186, "async poweroff(): Promise<void> {"),
            Some(Grounded::unknown("destructive; not exercised in capture")),
        ),
    ]),
    availability: PRESENCE_SHUTDOWN_ONBOARD_COMPUTER,
    blast_radius: BR_SHUTDOWN_ONBOARD_COMPUTER,
    chains_from: None,
};

const SYNC_SYSTEM_TIME: UseCase = UseCase {
    id: JourneyId::SyncSystemTime,
    summary: Grounded::known(
        "Sync the onboard computer clock with the browser when drift exceeds five minutes",
        Provenance::source(
            COMMANDER_MAIN,
            76,
            "if abs(unix_time_seconds_now - unix_time_seconds) < 5 * 60:",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::source(APP_VUE, 798, "updateTime()"),
    ),
    services: COMMANDER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::SyncSystemTime,
        "frontend posts browser unix time to commander set_time on load",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Software(SoftwareAssumption::ConfirmDangerousOp),
        Provenance::source(
            COMMANDER_MAIN,
            74,
            "async def set_time(unix_time_seconds: int, i_know_what_i_am_",
        ),
    )]),
    steps: GroundedSet::known(&[service_step(
        "On interface load, post the browser unix timestamp to commander",
        Some(sourced_route(
            HttpMethod::Post,
            "/set_time",
            Some("v1.0"),
            72,
            "@app.post(\"/set_time\", status_code=status.HTTP_200_OK)",
        )),
        Provenance::source(UPDATE_TIME, 9, "url: '/commander/v1.0/set_time',"),
        Some(source_outcome(
            200,
            72,
            "@app.post(\"/set_time\", status_code=status.HTTP_200_OK)",
        )),
    )]),
    availability: PRESENCE_SYNC_SYSTEM_TIME,
    blast_radius: BR_SYNC_SYSTEM_TIME,
    chains_from: None,
};

const ENABLE_LEGACY_CAMERA_SUPPORT: UseCase = UseCase {
    id: JourneyId::EnableLegacyCameraSupport,
    summary: Grounded::known(
        "Enable Raspberry Pi legacy camera support for Pi camera detection",
        Provenance::doc(
            ADV,
            823,
            "- Detection requires turning on legacy camera support:",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(
            ADV,
            822,
            "- Raspberry Pi cameras are supported `(New in 1.1)`",
        ),
    ),
    services: COMMANDER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConfigureLegacyCamera,
        "video manager toggles raspi-config legacy camera via commander",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Video Manager settings from the gear button in the bottom right corner",
            None,
            Provenance::doc(
                ADV,
                824,
                "1. turn on via the settings button in the buttom right corne",
            ),
            None,
        ),
        operator_step(
            "Turn on Raspberry legacy camera support",
            Some(sourced_route(
                HttpMethod::Post,
                "/raspi_config/camera_legacy",
                Some("v1.0"),
                117,
                "@app.post(\"/raspi_config/camera_legacy\", status_code=status.",
            )),
            Provenance::source(VIDEO_MANAGER, 66, "<v-switch"),
            Some(source_outcome(
                200,
                117,
                "@app.post(\"/raspi_config/camera_legacy\", status_code=status.",
            )),
        ),
        operator_step(
            "Reboot the onboard computer to apply legacy camera support",
            None,
            Provenance::doc(ADV, 825, "2. reboot the onboard computer to enable"),
            None,
        ),
    ]),
    availability: PRESENCE_ENABLE_LEGACY_CAMERA_SUPPORT,
    blast_radius: BR_ENABLE_LEGACY_CAMERA_SUPPORT,
    chains_from: Some(JourneyId::RebootOnboardComputer),
};

const INSPECT_RASPBERRY_EEPROM_BOOTLOADER: UseCase = UseCase {
    id: JourneyId::InspectRaspberryEepromBootloader,
    summary: Grounded::known(
        "View Raspberry Pi firmware, bootloader, and EEPROM update availability",
        Provenance::doc(
            ADV,
            610,
            "{{ easy_image(src=\"system-info-firmware\", width=600, class=\"",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Advanced,
        Provenance::doc(ADV, 611, "{% pirate() %}"),
    ),
    services: COMMANDER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::InspectRaspberryEeprom,
        "system information firmware tab reads vcgencmd and rpi-eeprom-update output",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Software(SoftwareAssumption::PirateMode),
            Provenance::source(
                SYSINFO_VIEW,
                85,
                "title: 'Firmware', icon: 'mdi-raspberry-pi', value: 'firmwar",
            ),
        ),
        GroundedItem::new(
            Precondition::Software(SoftwareAssumption::ConfirmDangerousOp),
            Provenance::source(
                COMMANDER_MAIN,
                49,
                "def check_what_i_am_doing(i_know_what_i_am_doing: bool = Fal",
            ),
        ),
    ]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Firmware tab on the System Information page",
            None,
            Provenance::doc(
                ADV,
                610,
                "{{ easy_image(src=\"system-info-firmware\", width=600, class=\"",
            ),
            None,
        ),
        operator_step(
            "Load Raspberry Pi firmware and bootloader versions",
            Some(sourced_route(
                HttpMethod::Get,
                "/raspi/vcgencmd",
                Some("v1.0"),
                132,
                "@app.get(\"/raspi/vcgencmd\", status_code=status.HTTP_200_OK)",
            )),
            Provenance::source(
                FIRMWARE,
                232,
                "commander.getVcgencmd().then((vcgencmd) => { this.vcgencmd =",
            ),
            Some(runtime_outcome(
                200,
                Some("\"vl085\""),
                "runtime-captures/commander__pi4_navigator_master.json#running_baseline",
            )),
        ),
        operator_step(
            "Load current and latest EEPROM bootloader and USB controller versions",
            Some(sourced_route(
                HttpMethod::Get,
                "/raspi/eeprom_update",
                Some("v1.0"),
                150,
                "@app.get(\"/raspi/eeprom_update\", status_code=status.HTTP_200",
            )),
            Provenance::source(
                FIRMWARE,
                233,
                "commander.getRaspiEEPROM().then((eeprom_update) => { this.ee",
            ),
            Some(runtime_outcome(
                200,
                Some("\"return_code\":0"),
                "runtime-captures/commander__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    availability: PRESENCE_INSPECT_RASPBERRY_EEPROM_BOOTLOADER,
    blast_radius: BR_INSPECT_RASPBERRY_EEPROM_BOOTLOADER,
    chains_from: None,
};

const UPDATE_RASPBERRY_EEPROM_BOOTLOADER: UseCase = UseCase {
    id: JourneyId::UpdateRaspberryEepromBootloader,
    summary: Grounded::known(
        "Update Raspberry Pi firmware and USB controller EEPROM to the latest stable versions",
        Provenance::doc(
            ADV,
            612,
            "Update buttons are provided if the device is not running the",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Advanced,
        Provenance::doc(ADV, 611, "{% pirate() %}"),
    ),
    services: COMMANDER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::UpdateRaspberryEeprom,
        "firmware tab applies rpi-eeprom-update when updates are available",
    )]),
    preconditions: GroundedSet::known(&[
        GroundedItem::new(
            Precondition::Software(SoftwareAssumption::PirateMode),
            Provenance::source(
                SYSINFO_VIEW,
                85,
                "title: 'Firmware', icon: 'mdi-raspberry-pi', value: 'firmwar",
            ),
        ),
        GroundedItem::new(
            Precondition::Software(SoftwareAssumption::ConfirmDangerousOp),
            Provenance::source(
                COMMANDER_MAIN,
                49,
                "def check_what_i_am_doing(i_know_what_i_am_doing: bool = Fal",
            ),
        ),
    ]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Firmware tab on the System Information page",
            None,
            Provenance::doc(
                ADV,
                610,
                "{{ easy_image(src=\"system-info-firmware\", width=600, class=\"",
            ),
            None,
        ),
        operator_step(
            "Click Update when bootloader or USB controller versions are out of date",
            Some(sourced_route(
                HttpMethod::Post,
                "/raspi/eeprom_update",
                Some("v1.0"),
                157,
                "@app.post(\"/raspi/eeprom_update\", status_code=status.HTTP_20",
            )),
            Provenance::source(FIRMWARE, 69, "@click=\"doRaspiEEPROMUpdate\""),
            Some(source_outcome(
                200,
                157,
                "@app.post(\"/raspi/eeprom_update\", status_code=status.HTTP_20",
            )),
        ),
    ]),
    availability: PRESENCE_UPDATE_RASPBERRY_EEPROM_BOOTLOADER,
    blast_radius: BR_UPDATE_RASPBERRY_EEPROM_BOOTLOADER,
    chains_from: Some(JourneyId::InspectRaspberryEepromBootloader),
};

const RESET_BLUEOS_SETTINGS: UseCase = UseCase {
    id: JourneyId::ResetBlueosSettings,
    summary: Grounded::known(
        "Reset BlueOS settings to remove camera, endpoint, and bridge configuration",
        Provenance::doc(ADV, 204, "- Reset BlueOS settings"),
    ),
    visibility: Grounded::known(
        Visibility::Default,
        Provenance::doc(ADV, 200, "##### BlueOS Settings"),
    ),
    services: COMMANDER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ResetBlueosSettings,
        "settings page deletes service config while preserving bootstrap state",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Software(SoftwareAssumption::ConfirmDangerousOp),
        Provenance::source(
            COMMANDER_MAIN,
            49,
            "def check_what_i_am_doing(i_know_what_i_am_doing: bool = Fal",
        ),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open BlueOS Settings from the header bar",
            None,
            Provenance::doc(
                ADV,
                202,
                "{{ easy_image(src=\"settings\", width=300, center=true) }}",
            ),
            None,
        ),
        operator_step(
            "Confirm Reset Settings to restore BlueOS services to defaults",
            Some(sourced_route(
                HttpMethod::Post,
                "/settings/reset",
                Some("v1.0"),
                164,
                "@app.post(\"/settings/reset\", status_code=status.HTTP_200_OK)",
            )),
            Provenance::source(SETTINGS_VIEW, 669, "url: `${API_URL}/settings/reset`,"),
            Some(source_outcome(
                200,
                164,
                "@app.post(\"/settings/reset\", status_code=status.HTTP_200_OK)",
            )),
        ),
    ]),
    availability: PRESENCE_RESET_BLUEOS_SETTINGS,
    blast_radius: BR_RESET_BLUEOS_SETTINGS,
    chains_from: None,
};

const RUN_HOST_COMMAND: UseCase = UseCase {
    id: JourneyId::RunHostCommand,
    summary: Grounded::known(
        "Run an arbitrary bash command on the host through commander",
        Provenance::doc(
            DEV_CORE,
            74,
            "| [Commander](https://github.com/bluerobotics/BlueOS/tree/ma",
        ),
    ),
    visibility: Grounded::known(
        Visibility::Advanced,
        Provenance::doc(
            DEV_CORE,
            74,
            "| [Commander](https://github.com/bluerobotics/BlueOS/tree/ma",
        ),
    ),
    services: COMMANDER_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::RunHostCommand,
        "commander executes privileged shell commands when explicitly acknowledged",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Software(SoftwareAssumption::ConfirmDangerousOp),
        Provenance::source(
            COMMANDER_MAIN,
            49,
            "def check_what_i_am_doing(i_know_what_i_am_doing: bool = Fal",
        ),
    )]),
    steps: GroundedSet::known(&[operator_step(
        "Post a shell command to commander with i_know_what_i_am_doing acknowledged",
        Some(sourced_route(
            HttpMethod::Post,
            "/command/host",
            Some("v1.0"),
            57,
            "@app.post(\"/command/host\", status_code=status.HTTP_200_OK)",
        )),
        Provenance::source(COMMANDER_STORE, 42, "url: `${this.API_URL}/command/host`,"),
        Some(source_outcome(
            200,
            57,
            "@app.post(\"/command/host\", status_code=status.HTTP_200_OK)",
        )),
    )]),
    availability: PRESENCE_RUN_HOST_COMMAND,
    blast_radius: BR_RUN_HOST_COMMAND,
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const COMMANDER_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Commander,
    Provenance::doc(
        DEV_CORE,
        74,
        "| [Commander](https://github.com/bluerobotics/BlueOS/tree/ma",
    ),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Commander,
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
        Provenance::source(COMMANDER_MAIN, line, anchor),
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
            actor: Actor::Service(ServiceId::Commander),
            description,
            route,
            outcome,
        },
        provenance,
    )
}

const fn runtime_outcome(
    status: u16,
    body: Option<&'static str>,
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
            transition: None,
        },
        Provenance::runtime(key, RUNTIME_ENV),
    )
}

const fn source_outcome(status: u16, line: u32, anchor: &'static str) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            body_kind: BodyKind::Unknown,
            transition: None,
        },
        Provenance::source(COMMANDER_MAIN, line, anchor),
    )
}
