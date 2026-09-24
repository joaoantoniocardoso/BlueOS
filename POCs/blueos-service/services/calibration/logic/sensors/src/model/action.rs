#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    StartPreflightCalibration,
    AwaitCommandAck,
    ReadOffsets,
    CancelCalibration,
}

impl Action {
    pub fn name(self) -> &'static str {
        match self {
            Action::StartPreflightCalibration => "start_preflight_calibration",
            Action::AwaitCommandAck => "await_command_ack",
            Action::ReadOffsets => "read_offsets",
            Action::CancelCalibration => "cancel_calibration",
        }
    }
}
