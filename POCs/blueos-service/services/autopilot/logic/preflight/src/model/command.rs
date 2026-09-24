use alloc::string::String;

use crate::model::Sensor;
use blueos_jobs::JobId;

#[derive(Clone, Debug)]
pub enum Command {
    Preflight { correlation: u32, sensor: Sensor },
    AdvanceAck { job_id: JobId, sensor: Sensor },
    SetMoving(bool),
}

#[derive(Clone, Debug)]
pub enum Event {
    Ack { sensor: Sensor, ack: String },
    OffsetsWritten { sensor: Sensor, value: i32 },
}

#[derive(Clone, Debug)]
pub enum IoRequest {
    Tick { job_id: JobId },
}
