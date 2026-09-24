use alloc::string::String;
use alloc::vec::Vec;

use crate::model::{Sensor, UseCase};
use blueos_jobs::JobId;

#[derive(Clone, Debug)]
pub enum Command {
    StartGyro,
    StartBaro,
    StartStationary,
    Cancel,
    JobProgress { job_id: JobId, payload: Vec<u8> },
    AckObserved { sensor: Sensor, ack: String },
    OffsetsRead { sensor: Sensor, value: i32 },
}

#[derive(Clone, Debug)]
pub enum Event {
    Requested { use_case: UseCase },
    Progress { sensor: Sensor, detail: String },
    Completed { sensor: Sensor },
    Failed { sensor: Sensor, reason: String },
    Cancelled { sensor: Sensor },
}

#[derive(Clone, Debug)]
pub enum IoRequest {
    PreflightCalibration { job_id: JobId, sensor: Sensor },
    ReadOffsets { job_id: JobId, sensor: Sensor },
    CancelAutopilot { job_id: JobId },
}
