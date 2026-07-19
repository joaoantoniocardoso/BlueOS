use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{Actor, JourneyStep, Precondition, StepOutcome, UserJourney, Visibility};
use crate::page::PageId;
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
use crate::version::FeatureAvailability;

const ADV: &str = "content/usage/advanced/index.md";
const CALIB_TS: &str = "core/frontend/src/components/vehiclesetup/calibration.ts";
const GYRO: &str = "core/frontend/src/components/vehiclesetup/overview/GyroCalib.vue";
const BARO: &str = "core/frontend/src/components/vehiclesetup/overview/BaroCalib.vue";
const QUICK_ACCEL: &str =
    "core/frontend/src/components/vehiclesetup/configuration/accelerometer/QuickAccelerometerCalibration.vue";
const FULL_ACCEL: &str =
    "core/frontend/src/components/vehiclesetup/configuration/accelerometer/FullAccelerometerCalibration.vue";
const LEVEL: &str =
    "core/frontend/src/components/vehiclesetup/configuration/compass/LevelHorizonCalibration.vue";
const FULL_COMPASS: &str =
    "core/frontend/src/components/vehiclesetup/configuration/compass/FullCompassCalibrator.vue";
const LARGE_COMPASS: &str =
    "core/frontend/src/components/vehiclesetup/configuration/compass/LargeVehicleCompassCalibrator.vue";
const MOTOR: &str = "core/frontend/src/components/vehiclesetup/MotorDetection.vue";
const STATUSTEXT: &str = "core/frontend/src/components/common/StatusTextWatcher.vue";
const SENSORS_STORE: &str = "core/frontend/src/store/ardupilot_sensors.ts";
const MENUS: &str = "core/frontend/src/menus.ts";
const VS_VIEW: &str = "core/frontend/src/views/VehicleSetupView.vue";

pub const JOURNEYS: &[UserJourney] = &[
    CALIBRATE_GYROSCOPE,
    CALIBRATE_ACCELEROMETER,
    CALIBRATE_COMPASS,
    CALIBRATE_BAROMETER,
    LEVEL_HORIZON,
    DETECT_MOTOR_DIRECTIONS,
];

const CALIBRATE_GYROSCOPE: UserJourney = UserJourney {
    id: JourneyId::CalibrateGyroscope,
    summary: Grounded::known(
        "Operator calibrates the gyroscope from the Vehicle Setup > Configure tab; the browser drives the preflight calibration and shows live gyro offsets",
        Provenance::doc(ADV, 695),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::source(MENUS, 121)),
    services: SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::CalibrateGyroscope,
        "GyroCalib.vue drives calibrator.calibrate(PreflightCalibration.GYROSCOPE) entirely in the browser",
    )]),
    preconditions: GroundedSet::known(&[precond(
        Precondition::Other("vehicle is stationary while the gyro is calibrated"),
        Provenance::doc(ADV, 695),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Vehicle Setup and select the Configure tab",
            Provenance::source(MENUS, 121),
        ),
        operator_step(
            "Press the Calibrate button on the gyroscope card",
            Provenance::source(GYRO, 47),
        ),
        frontend_step(
            "Calibrator singleton sends MAV_CMD_PREFLIGHT_CALIBRATION (gyro) via mavlink2rest and awaits COMMAND_ACK",
            Provenance::source(CALIB_TS, 33),
            Some(live_calibration_outcome()),
        ),
        frontend_step(
            "GyroCalib.vue reports the resulting gyro offsets and calibration status",
            Provenance::source(GYRO, 139),
            None,
        ),
    ]),
    availability: FeatureAvailability::unknown(),
    chains_from: None,
};

const CALIBRATE_ACCELEROMETER: UserJourney = UserJourney {
    id: JourneyId::CalibrateAccelerometer,
    summary: Grounded::known(
        "Operator runs a full six-position or quick accelerometer calibration from the Configure tab; the browser sequences the orientation steps",
        Provenance::doc(ADV, 700),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::source(MENUS, 121)),
    services: SERVICES,
    capability_refs: GroundedSet::known(&[
        cap(
            CapabilityId::CalibrateAccelerometer,
            "FullAccelerometerCalibration.vue and QuickAccelerometerCalibration.vue implement the position wizard and quick flow client-side",
        ),
        cap(
            CapabilityId::DeriveSensorCalibrationStatus,
            "ardupilot_sensors getters derive accelerometers_calibrated from cached offset/scale parameters after calibration",
        ),
    ]),
    preconditions: GroundedSet::known(&[precond(
        Precondition::Other("vehicle can be physically rotated through six orientations for full calibration"),
        Provenance::doc(ADV, 702),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Vehicle Setup > Configure and choose the accelerometer setup",
            Provenance::source(MENUS, 121),
        ),
        operator_step(
            "Start the full calibration wizard (or run quick calibration on a level surface)",
            Provenance::source(FULL_ACCEL, 43),
        ),
        frontend_step(
            "Wizard walks the six ACCELCAL_VEHICLE_POS orientations, sending MAV_CMD_ACCELCAL_VEHICLE_POS via mavlink2rest on each Next",
            Provenance::source(FULL_ACCEL, 170),
            Some(live_calibration_outcome()),
        ),
        frontend_step(
            "Quick path sends a simple MAV_CMD_PREFLIGHT_CALIBRATION accelerometer command",
            Provenance::source(QUICK_ACCEL, 75),
            None,
        ),
        frontend_step(
            "ardupilot_sensors store re-derives the accelerometer calibration status from updated parameters",
            Provenance::source(SENSORS_STORE, 40),
            None,
        ),
    ]),
    availability: FeatureAvailability::unknown(),
    chains_from: None,
};

const CALIBRATE_COMPASS: UserJourney = UserJourney {
    id: JourneyId::CalibrateCompass,
    summary: Grounded::known(
        "Operator calibrates the compass with the full onboard rotation wizard or the large-vehicle single-heading method",
        Provenance::doc(ADV, 710),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::source(MENUS, 121)),
    services: SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::CalibrateCompass,
        "FullCompassCalibrator.vue and LargeVehicleCompassCalibrator.vue orchestrate the mag-cal command sequence and progress UI",
    )]),
    preconditions: GroundedSet::known(&[precond(
        Precondition::Other("vehicle can be rotated about all axes for full onboard compass calibration"),
        Provenance::doc(ADV, 723),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Vehicle Setup > Configure and choose the compass setup",
            Provenance::source(MENUS, 121),
        ),
        frontend_step(
            "Full calibrator starts mag-cal (MAV_CMD_DO_START_MAG_CAL) and tracks MAG_CAL_PROGRESS/MAG_CAL_REPORT via mavlink2rest",
            Provenance::source(FULL_COMPASS, 229),
            Some(live_calibration_outcome()),
        ),
        frontend_step(
            "Large-vehicle method sends MAV_CMD_FIXED_MAG_CAL_YAW for a single-heading calibration",
            Provenance::source(LARGE_COMPASS, 109),
            None,
        ),
        frontend_step(
            "Operator can abort with MAV_CMD_DO_CANCEL_MAG_CAL if the fitness is poor",
            Provenance::source(FULL_COMPASS, 214),
            None,
        ),
    ]),
    availability: FeatureAvailability::unknown(),
    chains_from: None,
};

const CALIBRATE_BAROMETER: UserJourney = UserJourney {
    id: JourneyId::CalibrateBarometer,
    summary: Grounded::known(
        "Operator sets the reference pressure by calibrating the barometer at the start of a dive/flight",
        Provenance::doc(ADV, 739),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::source(MENUS, 121)),
    services: SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::CalibrateBarometer,
        "BaroCalib.vue drives calibrator.calibrate(PreflightCalibration.PRESSURE) with a per-sensor status table",
    )]),
    preconditions: GroundedSet::known(&[precond(
        Precondition::Other("barometer calibration is performed at the start of each dive/flight"),
        Provenance::doc(ADV, 741),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Vehicle Setup > Configure and choose the barometer setup",
            Provenance::source(MENUS, 121),
        ),
        operator_step(
            "Press the Calibrate button on the barometer card",
            Provenance::source(BARO, 58),
        ),
        frontend_step(
            "Calibrator sends MAV_CMD_PREFLIGHT_CALIBRATION (pressure) via mavlink2rest and reports per-sensor GND_PRESS values",
            Provenance::source(BARO, 146),
            Some(live_calibration_outcome()),
        ),
    ]),
    availability: FeatureAvailability::unknown(),
    chains_from: None,
};

const LEVEL_HORIZON: UserJourney = UserJourney {
    id: JourneyId::LevelHorizon,
    summary: Grounded::known(
        "Operator levels the horizon by placing the vehicle on a level surface and running board-level calibration",
        Provenance::doc(ADV, 705),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::source(MENUS, 121)),
    services: SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::LevelHorizon,
        "LevelHorizonCalibration.vue sends MAV_CMD_PREFLIGHT_CALIBRATION board-level and tracks local wizard state",
    )]),
    preconditions: GroundedSet::known(&[precond(
        Precondition::Other("vehicle rests on a level surface in its normal operating orientation"),
        Provenance::doc(ADV, 705),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Place the vehicle on a level surface and open the accelerometer/level setup",
            Provenance::source(MENUS, 121),
        ),
        frontend_step(
            "LevelHorizonCalibration.vue sends the board-level MAV_CMD_PREFLIGHT_CALIBRATION via mavlink2rest",
            Provenance::source(LEVEL, 159),
            Some(live_calibration_outcome()),
        ),
    ]),
    availability: FeatureAvailability::unknown(),
    chains_from: None,
};

const DETECT_MOTOR_DIRECTIONS: UserJourney = UserJourney {
    id: JourneyId::DetectMotorDirections,
    summary: Grounded::known(
        "Operator runs the automated check that detects motors spinning backwards and lets them be reversed",
        Provenance::doc(ADV, 664),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::source(MENUS, 121)),
    services: SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::DetectMotorDirections,
        "MotorDetection.vue arms the vehicle into MOTOR_DETECT mode and parses STATUSTEXT for completion",
    )]),
    preconditions: GroundedSet::known(&[precond(
        Precondition::Other("vehicle is safe to arm and briefly spin its motors"),
        Provenance::doc(ADV, 664),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open Vehicle Setup > Configure and start motor direction detection",
            Provenance::source(MOTOR, 136),
        ),
        frontend_step(
            "MotorDetection.vue sets MOTOR_DETECT mode and force-arms the vehicle via mavlink2rest",
            Provenance::source(MOTOR, 210),
            Some(live_calibration_outcome()),
        ),
        frontend_step(
            "StatusTextWatcher parses autopilot STATUSTEXT to track detection progress and completion",
            Provenance::source(STATUSTEXT, 29),
            None,
        ),
    ]),
    availability: FeatureAvailability::unknown(),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const fn precond(value: Precondition, provenance: Provenance) -> GroundedItem<Precondition> {
    GroundedItem::new(value, provenance)
}

const SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[
    GroundedItem::new(ServiceId::Mavlink2rest, Provenance::source(CALIB_TS, 33)),
    GroundedItem::new(ServiceId::ArdupilotManager, Provenance::source(VS_VIEW, 81)),
]);

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

const fn live_calibration_outcome() -> Grounded<StepOutcome> {
    Grounded::unknown(
        "COMMAND_ACK / progress values require runtime capture during a live calibration on the Pi (mutating; not performed)",
    )
}
