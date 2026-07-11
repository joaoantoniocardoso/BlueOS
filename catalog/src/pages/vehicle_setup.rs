use crate::id::{CapabilityId, ServiceId};
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub fn page() -> Page {
    Page {
        id: PageId("vehicle_setup".to_string()),
        route: Observed::known(
            "/vehicle/setup/:tab?/:subtab?".to_string(),
            Evidence {
                file: "core/frontend/src/router/index.ts".to_string(),
                line: 22,
            },
        ),
        name: Observed::known(
            "Vehicle Setup".to_string(),
            Evidence {
                file: "core/frontend/src/router/index.ts".to_string(),
                line: 23,
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/VehicleSetupView.vue".to_string(),
            Evidence {
                file: "core/frontend/src/router/index.ts".to_string(),
                line: 24,
            },
        ),
        menu_title: Observed::known(
            "Vehicle Setup".to_string(),
            Evidence {
                file: "core/frontend/src/menus.ts".to_string(),
                line: 121,
            },
        ),
        advanced_only: Observed::known(
            false,
            Evidence {
                file: "core/frontend/src/menus.ts".to_string(),
                line: 124,
            },
        ),
        stores: ObservedSet::known(vec![
            Evidenced::new(
                "autopilot_data".to_string(),
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/OnboardSensors.vue".to_string(),
                    line: 135,
                },
            ),
            Evidenced::new(
                "autopilot".to_string(),
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/Configure.vue".to_string(),
                    line: 40,
                },
            ),
            Evidenced::new(
                "ardupilot_sensors".to_string(),
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/OnboardSensors.vue".to_string(),
                    line: 134,
                },
            ),
            Evidenced::new(
                "mavlink".to_string(),
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/GyroCalib.vue".to_string(),
                    line: 65,
                },
            ),
            Evidenced::new(
                "ping".to_string(),
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/PingInfo.vue".to_string(),
                    line: 43,
                },
            ),
            Evidenced::new(
                "video".to_string(),
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/VideoOverview.vue".to_string(),
                    line: 57,
                },
            ),
            Evidenced::new(
                "system".to_string(),
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/VehicleInfo.vue".to_string(),
                    line: 41,
                },
            ),
            Evidenced::new(
                "customization".to_string(),
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/viewers/GenericViewer.vue".to_string(),
                    line: 104,
                },
            ),
        ]),
        consumes: ObservedSet::known(vec![
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest PARAM_VALUE / PARAM_REQUEST_LIST".to_string(),
                    purpose: "fetch and cache autopilot parameters into autopilot_data store (PARAM_REQUEST_LIST re-request at parameter-fetcher.ts:84)".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/types/autopilot/parameter-fetcher.ts".to_string(),
                    line: 12,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest PARAM_SET".to_string(),
                    purpose: "write autopilot parameters from editors, loaders, and configuration tabs".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/libs/MAVLink2Rest/index.ts".to_string(),
                    line: 243,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_PREFLIGHT_CALIBRATION / COMMAND_ACK".to_string(),
                    purpose: "gyro, baro, and accelerometer preflight calibration via Calibrator singleton".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/calibration.ts".to_string(),
                    line: 33,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_PREFLIGHT_CALIBRATION (simple accelerometer)".to_string(),
                    purpose: "quick accelerometer calibration".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/accelerometer/QuickAccelerometerCalibration.vue".to_string(),
                    line: 75,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_ACCELCAL_VEHICLE_POS / COMMAND_LONG".to_string(),
                    purpose: "full accelerometer position wizard".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/accelerometer/FullAccelerometerCalibration.vue".to_string(),
                    line: 170,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_PREFLIGHT_CALIBRATION (level horizon)".to_string(),
                    purpose: "level-horizon calibration from accelerometer setup tab".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/compass/LevelHorizonCalibration.vue".to_string(),
                    line: 159,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_DO_START_MAG_CAL / MAG_CAL_PROGRESS / MAG_CAL_REPORT".to_string(),
                    purpose: "full compass calibration wizard with progress and fitness reports".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/compass/FullCompassCalibrator.vue".to_string(),
                    line: 229,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_DO_CANCEL_MAG_CAL".to_string(),
                    purpose: "cancel in-progress compass calibration".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/compass/FullCompassCalibrator.vue".to_string(),
                    line: 214,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_FIXED_MAG_CAL_YAW".to_string(),
                    purpose: "large-vehicle compass calibration".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/compass/LargeVehicleCompassCalibrator.vue".to_string(),
                    line: 109,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest SET_GPS_GLOBAL_ORIGIN / GLOBAL_POSITION_INT".to_string(),
                    purpose: "auto-detect compass origin coordinates (listens GLOBAL_POSITION_INT at line 139)".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/compass/AutoCoordinateDetector.vue".to_string(),
                    line: 187,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_PREFLIGHT_STORAGE".to_string(),
                    purpose: "reset all parameters to firmware defaults (pirate mode)".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/ParamSets.vue".to_string(),
                    line: 178,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_DO_SET_MODE / MAV_CMD_COMPONENT_ARM_DISARM".to_string(),
                    purpose: "motor direction detection mode and forced arming".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/MotorDetection.vue".to_string(),
                    line: 210,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_DO_MOTOR_TEST".to_string(),
                    purpose: "manual per-motor PWM test from PWM Outputs tab".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/utils/ardupilot_mavlink.ts".to_string(),
                    line: 104,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest STATUSTEXT".to_string(),
                    purpose: "motor detection progress messages from autopilot".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/common/StatusTextWatcher.vue".to_string(),
                    line: 29,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest telemetry stream (REQUEST_MESSAGE / ws listeners)".to_string(),
                    purpose: "live IMU, pressure, servo, and attitude data for sensor status displays".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/store/mavlink.ts".to_string(),
                    line: 38,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest MAV_CMD_DO_GIMBAL_MANAGER_TILTPAN / MAV_CMD_DO_MOUNT_CONTROL".to_string(),
                    purpose: "camera gimbal min/max PWM calibration".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/camera.vue".to_string(),
                    line: 331,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/vehicle_type".to_string(),
                    purpose: "poll vehicle type for frame-specific UI".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/views/VehicleSetupView.vue".to_string(),
                    line: 81,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/firmware_vehicle_type".to_string(),
                    purpose: "poll firmware vehicle type for motor detection eligibility".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/views/VehicleSetupView.vue".to_string(),
                    line: 82,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/board".to_string(),
                    purpose: "board name for parameter set matching and vehicle info".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/ParamSets.vue".to_string(),
                    line: 155,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/firmware_info".to_string(),
                    purpose: "firmware version for parameter set matching and vehicle info".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/VehicleInfo.vue".to_string(),
                    line: 71,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "POST /ardupilot-manager/v1.0/restart".to_string(),
                    purpose: "reboot autopilot after parameter wipe".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/ParamSets.vue".to_string(),
                    line: 170,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "GET /mavlink-camera-manager/v4l".to_string(),
                    purpose: "list video devices on overview tab".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/VideoOverview.vue".to_string(),
                    line: 102,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::MavlinkCameraManager),
                    endpoint: "GET /mavlink-camera-manager/streams".to_string(),
                    purpose: "list configured video streams on overview tab".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/VideoOverview.vue".to_string(),
                    line: 103,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::External,
                    endpoint: "GET https://docs.bluerobotics.com/Blueos-Parameter-Repository/params_v1.json".to_string(),
                    purpose: "download curated parameter set catalog".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/overview/ParamSets.vue".to_string(),
                    line: 160,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::External,
                    endpoint: "GET http://ip-api.com/json/".to_string(),
                    purpose: "geo-IP lookup for compass auto-coordinate detector".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/components/vehiclesetup/configuration/compass/AutoCoordinateDetector.vue".to_string(),
                    line: 162,
                },
            ),
        ]),
        frontend_features: AssertedSet::established(vec![
            Rationaled::new(
                CapabilityId("calibrate_gyroscope".to_string()),
                "GyroCalib.vue drives Calibrator.calibrate(PreflightCalibration.GYROSCOPE) and displays live gyro offsets",
            ),
            Rationaled::new(
                CapabilityId("calibrate_accelerometer".to_string()),
                "QuickAccelerometerCalibration.vue and FullAccelerometerCalibration.vue implement quick and position-wizard accel flows client-side",
            ),
            Rationaled::new(
                CapabilityId("calibrate_compass".to_string()),
                "FullCompassCalibrator.vue and LargeVehicleCompassCalibrator.vue orchestrate mag-cal command sequences and progress UI",
            ),
            Rationaled::new(
                CapabilityId("calibrate_barometer".to_string()),
                "BaroCalib.vue drives Calibrator.calibrate(PreflightCalibration.PRESSURE) with per-sensor status table",
            ),
            Rationaled::new(
                CapabilityId("level_horizon".to_string()),
                "LevelHorizonCalibration.vue sends MAV_CMD_PREFLIGHT_CALIBRATION board-level and tracks local wizard state",
            ),
            Rationaled::new(
                CapabilityId("detect_motor_directions".to_string()),
                "MotorDetection.vue arms vehicle into MOTOR_DETECT mode and parses STATUSTEXT for completion",
            ),
            Rationaled::new(
                CapabilityId("edit_autopilot_parameters".to_string()),
                "InlineParameterEditor, ParameterSwitch, and ServoFunctionEditorDialog write params via mavlink2rest.setParam across configure and PWM tabs",
            ),
            Rationaled::new(
                CapabilityId("apply_parameter_set".to_string()),
                "ParamSets.vue filters external curated sets by board/firmware and ParameterLoader.vue batches PARAM_SET writes",
            ),
            Rationaled::new(
                CapabilityId("derive_sensor_calibration_status".to_string()),
                "ardupilot_sensors store getters (accelerometers_calibrated, compasses_calibrated, etc.) derive health from cached parameters",
            ),
        ]),
        client_state: AssertedSet::established(vec![
            Rationaled::new(
                ClientState {
                    name: "calibration progress".to_string(),
                    store: "components/vehiclesetup/calibration.ts Calibrator singleton".to_string(),
                    ownership: StateOwnership::FrontendOwned,
                    notes: "calibrating and calibrationStatus fields; ephemeral wizard state not persisted".to_string(),
                },
                "Calibrator tracks in-flight preflight calibration type and COMMAND_ACK result locally",
            ),
            Rationaled::new(
                ClientState {
                    name: "per-component calibration status text".to_string(),
                    store: "GyroCalib.vue / BaroCalib.vue component data".to_string(),
                    ownership: StateOwnership::FrontendOwned,
                    notes: "calibration_status strings shown during gyro and baro flows".to_string(),
                },
                "individual calibrator components hold UI status strings independent of backend",
            ),
            Rationaled::new(
                ClientState {
                    name: "motor detection dialog state".to_string(),
                    store: "MotorDetection.vue component data".to_string(),
                    ownership: StateOwnership::FrontendOwned,
                    notes: "IDLE/STARTING/RUNNING/COMPLETE/FAILED enum and timeout timer".to_string(),
                },
                "motor reversal wizard tracks dialog progress client-side from STATUSTEXT parsing",
            ),
            Rationaled::new(
                ClientState {
                    name: "compass calibration wizard state".to_string(),
                    store: "FullCompassCalibrator.vue component data".to_string(),
                    ownership: StateOwnership::FrontendOwned,
                    notes: "state machine, percent, fitness dict, progress/report listeners".to_string(),
                },
                "full compass calibrator holds step progress and fitness scores only in the browser",
            ),
            Rationaled::new(
                ClientState {
                    name: "accelerometer calibration wizard state".to_string(),
                    store: "FullAccelerometerCalibration.vue / QuickAccelerometerCalibration.vue".to_string(),
                    ownership: StateOwnership::FrontendOwned,
                    notes: "position-wizard CalState enum and quick-cal dialog state".to_string(),
                },
                "accelerometer wizards track orientation steps and dialog state locally",
            ),
            Rationaled::new(
                ClientState {
                    name: "selected parameter set".to_string(),
                    store: "ParamSets.vue component data".to_string(),
                    ownership: StateOwnership::FrontendOwned,
                    notes: "all_param_sets, selected_paramset, wipe_successful flags".to_string(),
                },
                "parameter set picker holds downloaded catalog and in-progress apply selection locally",
            ),
            Rationaled::new(
                ClientState {
                    name: "PWM motor test targets".to_string(),
                    store: "PwmSetup.vue component data".to_string(),
                    ownership: StateOwnership::FrontendOwned,
                    notes: "motor_targets, desired_armed_state, highlight selection".to_string(),
                },
                "PWM outputs tab tracks manual motor test slider values and arm state locally",
            ),
            Rationaled::new(
                ClientState {
                    name: "autopilot parameter cache".to_string(),
                    store: "store/autopilot (autopilot_data module)".to_string(),
                    ownership: StateOwnership::Shared,
                    notes: "parameters[] fetched via PARAM_VALUE; mutated client-side via PARAM_SET".to_string(),
                },
                "parameter cache is synced from vehicle but edited and derived from in the browser",
            ),
            Rationaled::new(
                ClientState {
                    name: "sensor device lists".to_string(),
                    store: "store/ardupilot_sensors".to_string(),
                    ownership: StateOwnership::Shared,
                    notes: "accelerometers, compasses, baros getters decode INS/COMPASS/BARO params".to_string(),
                },
                "sensor enumerations are computed from cached parameters, not a dedicated backend API",
            ),
            Rationaled::new(
                ClientState {
                    name: "sensor calibration health flags".to_string(),
                    store: "store/ardupilot_sensors getters".to_string(),
                    ownership: StateOwnership::Shared,
                    notes: "accelerometers_calibrated, compasses_calibrated, baro status derived from offset/scale params".to_string(),
                },
                "is-calibrated indicators are client-derived from parameter values",
            ),
            Rationaled::new(
                ClientState {
                    name: "MAVLink telemetry cache".to_string(),
                    store: "store/mavlink".to_string(),
                    ownership: StateOwnership::Shared,
                    notes: "available_messages keyed by message type; refreshed via ws listeners".to_string(),
                },
                "live IMU/pressure/servo readings cached client-side for sensor status displays",
            ),
            Rationaled::new(
                ClientState {
                    name: "autopilot manager metadata".to_string(),
                    store: "store/autopilot_manager (autopilot module)".to_string(),
                    ownership: StateOwnership::BackendOwned,
                    notes: "vehicle_type, firmware_info, current_board mirrored from ardupilot_manager REST".to_string(),
                },
                "board/firmware/vehicle type are fetched from backend and displayed without client derivation",
            ),
            Rationaled::new(
                ClientState {
                    name: "video devices and streams".to_string(),
                    store: "store/video".to_string(),
                    ownership: StateOwnership::BackendOwned,
                    notes: "available_devices and available_streams mirrored from mavlink_camera_manager".to_string(),
                },
                "overview video card displays backend-reported device/stream lists",
            ),
        ]),
    }
}
