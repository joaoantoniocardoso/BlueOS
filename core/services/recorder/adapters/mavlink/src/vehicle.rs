use mavlink::dialects::ardupilotmega::{HEARTBEAT_DATA, MavModeFlag};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmState {
    Armed,
    Disarmed,
}

pub fn armed_from_heartbeat(data: &HEARTBEAT_DATA) -> Option<bool> {
    let armed = data
        .base_mode
        .contains(MavModeFlag::MAV_MODE_FLAG_SAFETY_ARMED);
    Some(armed)
}
