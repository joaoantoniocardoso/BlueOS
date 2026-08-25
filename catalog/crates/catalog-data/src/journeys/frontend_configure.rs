use crate::journey_presence::{
    PRESENCE_CONFIGURE_BATTERY_MONITOR, PRESENCE_CONFIGURE_VEHICLE_BODY,
};
use catalog_kernel::id::capability::CapabilityId;
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::page::PageId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use catalog_model::journey::{
    Actor, BlastRadius, JourneyStep, Precondition, StepOutcome, UseCase, Visibility,
};

const ADV: &str = "content/usage/advanced/index.md";
const POWER_SENSOR: &str = "content/integrations/hardware/required/power-sensor/index.md";
const BODY: &str =
    "core/frontend/src/components/vehiclesetup/configuration/ArdupilotVehicleBodySetup.vue";
const ORIENT: &str = "core/frontend/src/components/vehiclesetup/OrientationPicker.vue";
const POWER: &str =
    "core/frontend/src/components/vehiclesetup/configuration/power/PowerConfiguration.vue";
const BATTERY: &str =
    "core/frontend/src/components/vehiclesetup/configuration/power/BatteryCard.vue";

const BR_PARAMS: Grounded<BlastRadius> = Grounded::known(
    BlastRadius::Reversible,
    Provenance::asserted(
        "FRAME_*, AHRS_ORIENTATION, and BATT_* writes are autopilot parameters restorable by re-setting or restoring defaults",
    ),
);

pub const JOURNEYS: &[UseCase] = &[CONFIGURE_VEHICLE_BODY, CONFIGURE_BATTERY_MONITOR];

const CONFIGURE_VEHICLE_BODY: UseCase = UseCase {
    id: JourneyId::ConfigureVehicleBody,
    summary: Grounded::known(
        "Operator selects vehicle frame and board orientation with 3D previews in Vehicle Setup Configure",
        Provenance::doc(
            ADV,
            681,
            "The Configure tab provides configuration and calibration options for the vehicle sensors",
        ),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 642, "### Vehicle Setup")),
    services: SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConfigureVehicleBody,
        "ArdupilotVehicleBodySetup.vue writes FRAME_* via FrameSelector and AHRS_ORIENTATION via OrientationPicker",
    )]),
    preconditions: GroundedSet::known(&[precond(
        Precondition::Other("autopilot parameters have finished loading"),
        Provenance::source(BODY, 62, "return autopilot_data.finished_loading"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Vehicle Setup and select the Configure tab, then Vehicle Body",
            Provenance::doc(
                ADV,
                681,
                "The Configure tab provides configuration and calibration options for the vehicle sensors",
            ),
        ),
        frontend_step(
            "FrameSelector emits the chosen FRAME_CONFIG/FRAME_CLASS; ArdupilotVehicleBodySetup writes it via mavlink2rest.setParam",
            Provenance::source(BODY, 86, "mavlink2rest.setParam("),
            None,
        ),
        frontend_step(
            "OrientationPicker writes AHRS_ORIENTATION via mavlink2rest.setParam and marks reboot required",
            Provenance::source(ORIENT, 185, "mavlink2rest.setParam("),
            None,
        ),
    ]),
    availability: PRESENCE_CONFIGURE_VEHICLE_BODY,
    blast_radius: BR_PARAMS,
    chains_from: None,
};

const CONFIGURE_BATTERY_MONITOR: UseCase = UseCase {
    id: JourneyId::ConfigureBatteryMonitor,
    summary: Grounded::known(
        "Operator configures battery monitor presets, scaling, and dual-battery params on the Power Configure tab",
        Provenance::doc(
            POWER_SENSOR,
            15,
            "A power sensing module provides analog current and voltage sensing to an autopilot onboard",
        ),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(ADV, 642, "### Vehicle Setup")),
    services: SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::ConfigureBatteryMonitor,
        "BatteryCard applies power-sensor presets and BATT_* scaling through mavlink2rest.setParam",
    )]),
    preconditions: GroundedSet::known(&[precond(
        Precondition::Other("autopilot parameters have finished loading"),
        Provenance::source(POWER, 48, "return autopilot_data.finished_loading"),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Vehicle Setup > Configure and select the Power tab",
            Provenance::doc(
                POWER_SENSOR,
                15,
                "A power sensing module provides analog current and voltage sensing to an autopilot onboard",
            ),
        ),
        operator_step(
            "Choose a Battery Monitor type and optional power-sensor preset on Battery 1 (and Battery 2 when BATT2_MONITOR exists)",
            Provenance::source(BATTERY, 16, "<parameter-label label=\"Battery Monitor\" :param=\"monitorPara"),
        ),
        frontend_step(
            "BatteryCard.applySensorPreset writes voltage/current pin and scaling parameters via mavlink2rest.setParam",
            Provenance::source(BATTERY, 423, "if (this.voltPinParam) mavlink2rest.setParam(this.voltPinPar"),
            None,
        ),
    ]),
    availability: PRESENCE_CONFIGURE_BATTERY_MONITOR,
    blast_radius: BR_PARAMS,
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const fn precond(value: Precondition, provenance: Provenance) -> GroundedItem<Precondition> {
    GroundedItem::new(value, provenance)
}

const SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Mavlink2rest,
    Provenance::source(BODY, 86, "mavlink2rest.setParam("),
)]);

const fn operator_step(
    description: &'static str,
    provenance: Provenance,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description,
            route: None,
            outcome: None,
        },
        provenance,
    )
}

const fn frontend_step(
    description: &'static str,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Frontend(PageId::VehicleSetup),
            description,
            route: None,
            outcome,
        },
        provenance,
    )
}
