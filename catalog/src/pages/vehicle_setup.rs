use crate::id::{CapabilityId, ServiceId};
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::VehicleSetup,
        route: Observed::known(
            "/vehicle/setup/:tab?/:subtab?",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 22,
            },
        ),
        name: Observed::known(
            "Vehicle Setup",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 23,
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/VehicleSetupView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 24,
            },
        ),
        menu_title: Observed::known(
            "Vehicle Setup",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 121,
            },
        ),
        advanced_only: Observed::known(
            false,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 124,
            },
        ),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "autopilot_data",
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/OnboardSensors.vue",
                    line: 135,
                },
            ),
            Evidenced::new(
                "autopilot",
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/Configure.vue",
                    line: 40,
                },
            ),
            Evidenced::new(
                "ardupilot_sensors",
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/OnboardSensors.vue",
                    line: 134,
                },
            ),
            Evidenced::new(
                "mavlink",
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/GyroCalib.vue",
                    line: 65,
                },
            ),
            Evidenced::new(
                "ping",
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/PingInfo.vue",
                    line: 43,
                },
            ),
            Evidenced::new(
                "video",
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/VideoOverview.vue",
                    line: 57,
                },
            ),
            Evidenced::new(
                "system",
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/VehicleInfo.vue",
                    line: 41,
                },
            ),
            Evidenced::new(
                "customization",
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/viewers/GenericViewer.vue",
                    line: 104,
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest PARAM_VALUE / PARAM_REQUEST_LIST",
                    purpose: "fetch and cache autopilot parameters into autopilot_data store (PARAM_REQUEST_LIST re-request at parameter-fetcher.ts:84)",
                },
                Evidence {
                    file: "core/frontend/src/types/autopilot/parameter-fetcher.ts",
                    line: 12,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest PARAM_SET",
                    purpose: "write autopilot parameters from editors, loaders, and configuration tabs",
                },
                Evidence {
                    file: "core/frontend/src/libs/MAVLink2Rest/index.ts",
                    line: 243,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_PREFLIGHT_CALIBRATION / COMMAND_ACK",
                    purpose: "gyro, baro, and accelerometer preflight calibration via Calibrator singleton",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/calibration.ts",
                    line: 33,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_PREFLIGHT_CALIBRATION (simple accelerometer)",
                    purpose: "quick accelerometer calibration",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/accelerometer/QuickAccelerometerCalibration.vue",
                    line: 75,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_ACCELCAL_VEHICLE_POS / COMMAND_LONG",
                    purpose: "full accelerometer position wizard",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/accelerometer/FullAccelerometerCalibration.vue",
                    line: 170,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_PREFLIGHT_CALIBRATION (level horizon)",
                    purpose: "level-horizon calibration from accelerometer setup tab",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/compass/LevelHorizonCalibration.vue",
                    line: 159,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_DO_START_MAG_CAL / MAG_CAL_PROGRESS / MAG_CAL_REPORT",
                    purpose: "full compass calibration wizard with progress and fitness reports",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/compass/FullCompassCalibrator.vue",
                    line: 229,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_DO_CANCEL_MAG_CAL",
                    purpose: "cancel in-progress compass calibration",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/compass/FullCompassCalibrator.vue",
                    line: 214,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_FIXED_MAG_CAL_YAW",
                    purpose: "large-vehicle compass calibration",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/compass/LargeVehicleCompassCalibrator.vue",
                    line: 109,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest SET_GPS_GLOBAL_ORIGIN / GLOBAL_POSITION_INT",
                    purpose: "auto-detect compass origin coordinates (listens GLOBAL_POSITION_INT at line 139)",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/compass/AutoCoordinateDetector.vue",
                    line: 187,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_PREFLIGHT_STORAGE",
                    purpose: "reset all parameters to firmware defaults (pirate mode)",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/ParamSets.vue",
                    line: 178,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_DO_SET_MODE / MAV_CMD_COMPONENT_ARM_DISARM",
                    purpose: "motor direction detection mode and forced arming",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/MotorDetection.vue",
                    line: 210,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_DO_MOTOR_TEST",
                    purpose: "manual per-motor PWM test from PWM Outputs tab",
                },
                Evidence {
                    file: "core/frontend/src/utils/ardupilot_mavlink.ts",
                    line: 104,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest STATUSTEXT",
                    purpose: "motor detection progress messages from autopilot",
                },
                Evidence {
                    file: "core/frontend/src/components/common/StatusTextWatcher.vue",
                    line: 29,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest telemetry stream (REQUEST_MESSAGE / ws listeners)",
                    purpose: "live IMU, pressure, servo, and attitude data for sensor status displays",
                },
                Evidence {
                    file: "core/frontend/src/store/mavlink.ts",
                    line: 38,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_DO_GIMBAL_MANAGER_TILTPAN / MAV_CMD_DO_MOUNT_CONTROL",
                    purpose: "camera gimbal min/max PWM calibration",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/camera.vue",
                    line: 331,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/vehicle_type",
                    purpose: "poll vehicle type for frame-specific UI",
                },
                Evidence {
                    file: "core/frontend/src/views/VehicleSetupView.vue",
                    line: 81,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/firmware_vehicle_type",
                    purpose: "poll firmware vehicle type for motor detection eligibility",
                },
                Evidence {
                    file: "core/frontend/src/views/VehicleSetupView.vue",
                    line: 82,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/board",
                    purpose: "board name for parameter set matching and vehicle info",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/ParamSets.vue",
                    line: 155,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/firmware_info",
                    purpose: "firmware version for parameter set matching and vehicle info",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/VehicleInfo.vue",
                    line: 71,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "POST /ardupilot-manager/v1.0/restart",
                    purpose: "reboot autopilot after parameter wipe",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/ParamSets.vue",
                    line: 170,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "GET /mavlink-camera-manager/v4l",
                    purpose: "list video devices on overview tab",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/VideoOverview.vue",
                    line: 102,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "GET /mavlink-camera-manager/streams",
                    purpose: "list configured video streams on overview tab",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/VideoOverview.vue",
                    line: 103,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::External,
                    endpoint: "GET https://docs.bluerobotics.com/Blueos-Parameter-Repository/params_v1.json",
                    purpose: "download curated parameter set catalog",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/ParamSets.vue",
                    line: 160,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::External,
                    endpoint: "GET http://ip-api.com/json/",
                    purpose: "geo-IP lookup for compass auto-coordinate detector",
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/compass/AutoCoordinateDetector.vue",
                    line: 162,
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::CalibrateGyroscope,
                "GyroCalib.vue drives Calibrator.calibrate(PreflightCalibration.GYROSCOPE) and displays live gyro offsets",
            ),
            Rationaled::new(
                CapabilityId::CalibrateAccelerometer,
                "QuickAccelerometerCalibration.vue and FullAccelerometerCalibration.vue implement quick and position-wizard accel flows client-side",
            ),
            Rationaled::new(
                CapabilityId::CalibrateCompass,
                "FullCompassCalibrator.vue and LargeVehicleCompassCalibrator.vue orchestrate mag-cal command sequences and progress UI",
            ),
            Rationaled::new(
                CapabilityId::CalibrateBarometer,
                "BaroCalib.vue drives Calibrator.calibrate(PreflightCalibration.PRESSURE) with per-sensor status table",
            ),
            Rationaled::new(
                CapabilityId::LevelHorizon,
                "LevelHorizonCalibration.vue sends MAV_CMD_PREFLIGHT_CALIBRATION board-level and tracks local wizard state",
            ),
            Rationaled::new(
                CapabilityId::DetectMotorDirections,
                "MotorDetection.vue arms vehicle into MOTOR_DETECT mode and parses STATUSTEXT for completion",
            ),
            Rationaled::new(
                CapabilityId::EditAutopilotParameters,
                "InlineParameterEditor, ParameterSwitch, and ServoFunctionEditorDialog write params via mavlink2rest.setParam across configure and PWM tabs",
            ),
            Rationaled::new(
                CapabilityId::ApplyParameterSet,
                "ParamSets.vue filters external curated sets by board/firmware and ParameterLoader.vue batches PARAM_SET writes",
            ),
            Rationaled::new(
                CapabilityId::DeriveSensorCalibrationStatus,
                "ardupilot_sensors store getters (accelerometers_calibrated, compasses_calibrated, etc.) derive health from cached parameters",
            ),
        ]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "calibration progress",
                    store: "components/vehiclesetup/calibration.ts Calibrator singleton",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "calibrating and calibrationStatus fields; ephemeral wizard state not persisted",
                },
                "Calibrator tracks in-flight preflight calibration type and COMMAND_ACK result locally",
            ),
            Rationaled::new(
                ClientState {
                    name: "per-component calibration status text",
                    store: "GyroCalib.vue / BaroCalib.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "calibration_status strings shown during gyro and baro flows",
                },
                "individual calibrator components hold UI status strings independent of backend",
            ),
            Rationaled::new(
                ClientState {
                    name: "motor detection dialog state",
                    store: "MotorDetection.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "IDLE/STARTING/RUNNING/COMPLETE/FAILED enum and timeout timer",
                },
                "motor reversal wizard tracks dialog progress client-side from STATUSTEXT parsing",
            ),
            Rationaled::new(
                ClientState {
                    name: "compass calibration wizard state",
                    store: "FullCompassCalibrator.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "state machine, percent, fitness dict, progress/report listeners",
                },
                "full compass calibrator holds step progress and fitness scores only in the browser",
            ),
            Rationaled::new(
                ClientState {
                    name: "accelerometer calibration wizard state",
                    store: "FullAccelerometerCalibration.vue / QuickAccelerometerCalibration.vue",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "position-wizard CalState enum and quick-cal dialog state",
                },
                "accelerometer wizards track orientation steps and dialog state locally",
            ),
            Rationaled::new(
                ClientState {
                    name: "selected parameter set",
                    store: "ParamSets.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "all_param_sets, selected_paramset, wipe_successful flags",
                },
                "parameter set picker holds downloaded catalog and in-progress apply selection locally",
            ),
            Rationaled::new(
                ClientState {
                    name: "PWM motor test targets",
                    store: "PwmSetup.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "motor_targets, desired_armed_state, highlight selection",
                },
                "PWM outputs tab tracks manual motor test slider values and arm state locally",
            ),
            Rationaled::new(
                ClientState {
                    name: "autopilot parameter cache",
                    store: "store/autopilot (autopilot_data module)",
                    ownership: StateOwnership::Shared,
                    notes: "parameters[] fetched via PARAM_VALUE; mutated client-side via PARAM_SET",
                },
                "parameter cache is synced from vehicle but edited and derived from in the browser",
            ),
            Rationaled::new(
                ClientState {
                    name: "sensor device lists",
                    store: "store/ardupilot_sensors",
                    ownership: StateOwnership::Shared,
                    notes: "accelerometers, compasses, baros getters decode INS/COMPASS/BARO params",
                },
                "sensor enumerations are computed from cached parameters, not a dedicated backend API",
            ),
            Rationaled::new(
                ClientState {
                    name: "sensor calibration health flags",
                    store: "store/ardupilot_sensors getters",
                    ownership: StateOwnership::Shared,
                    notes: "accelerometers_calibrated, compasses_calibrated, baro status derived from offset/scale params",
                },
                "is-calibrated indicators are client-derived from parameter values",
            ),
            Rationaled::new(
                ClientState {
                    name: "MAVLink telemetry cache",
                    store: "store/mavlink",
                    ownership: StateOwnership::Shared,
                    notes: "available_messages keyed by message type; refreshed via ws listeners",
                },
                "live IMU/pressure/servo readings cached client-side for sensor status displays",
            ),
            Rationaled::new(
                ClientState {
                    name: "autopilot manager metadata",
                    store: "store/autopilot_manager (autopilot module)",
                    ownership: StateOwnership::BackendOwned,
                    notes: "vehicle_type, firmware_info, current_board mirrored from ardupilot_manager REST",
                },
                "board/firmware/vehicle type are fetched from backend and displayed without client derivation",
            ),
            Rationaled::new(
                ClientState {
                    name: "video devices and streams",
                    store: "store/video",
                    ownership: StateOwnership::BackendOwned,
                    notes: "available_devices and available_streams mirrored from mavlink_camera_manager",
                },
                "overview video card displays backend-reported device/stream lists",
            ),
        ]),
    };
