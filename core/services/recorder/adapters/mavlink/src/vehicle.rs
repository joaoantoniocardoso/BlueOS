//! Vehicle heartbeat helpers.

use mavlink::dialects::ardupilotmega::{HEARTBEAT_DATA, MavModeFlag};

/// Reads the armed flag from an autopilot heartbeat.
pub fn armed_from_heartbeat(data: &HEARTBEAT_DATA) -> bool {
    data.base_mode
        .contains(MavModeFlag::MAV_MODE_FLAG_SAFETY_ARMED)
}
