use alloc::string::ToString;
use alloc::vec::Vec;

use crate::model::Sensor;

pub const RPC_PREFLIGHT: &str = "autopilot/rpc/preflight_calibration";
pub const STATE_ACK: &str = "autopilot/state/command_ack";
pub const STATE_OFFSETS: &str = "autopilot/state/sensor_offsets";

// RPC body: b"gyro" | b"baro". Reply: b"IN_PROGRESS" | b"FAILED".
// STATE_ACK: b"<sensor>:<ack>". STATE_OFFSETS: b"<sensor>:<i32>" (gyro_offset / ground_pressure).

pub(crate) fn sensor_wire(sensor: Sensor) -> &'static [u8] {
    match sensor {
        Sensor::Gyro => b"gyro",
        Sensor::Baro => b"baro",
    }
}

pub(crate) fn ack_wire(sensor: Sensor, ack: &str) -> Vec<u8> {
    let mut out = sensor_wire(sensor).to_vec();
    out.push(b':');
    out.extend_from_slice(ack.as_bytes());
    out
}

pub(crate) fn offset_wire(sensor: Sensor, value: i32) -> Vec<u8> {
    let mut out = sensor_wire(sensor).to_vec();
    out.push(b':');
    out.extend_from_slice(value.to_string().as_bytes());
    out
}
