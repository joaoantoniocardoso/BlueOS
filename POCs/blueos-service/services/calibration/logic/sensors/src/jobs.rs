use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::model::{Action, IoRequest, Sensor};
use blueos_jobs::{JobGraph, JobId, JobSpec, JobStatus, Jobs};

pub(crate) fn sensor_payload(sensor: Sensor) -> Vec<u8> {
    match sensor {
        Sensor::Gyro => b"gyro".to_vec(),
        Sensor::Baro => b"baro".to_vec(),
    }
}

pub(crate) fn parse_sensor(payload: &[u8]) -> Option<Sensor> {
    match payload {
        b"gyro" => Some(Sensor::Gyro),
        b"baro" => Some(Sensor::Baro),
        _ => None,
    }
}

pub(crate) fn leaf(action: Action, sensor: Sensor) -> JobGraph {
    JobGraph::Leaf(JobSpec {
        name: action.name().to_string(),
        payload: sensor_payload(sensor),
    })
}

pub(crate) fn preflight_sequence(sensor: Sensor) -> JobGraph {
    JobGraph::Sequence(vec![
        leaf(Action::StartPreflightCalibration, sensor),
        leaf(Action::AwaitCommandAck, sensor),
        leaf(Action::ReadOffsets, sensor),
    ])
}

pub(crate) fn job_spec_of(jobs: &Jobs, job_id: JobId) -> Option<JobSpec> {
    jobs.snapshot()
        .jobs
        .into_iter()
        .find(|job| job.job_id == job_id)
        .and_then(|job| job.job_spec)
}

pub(crate) fn running_named(jobs: &Jobs, action: Action, sensor: Sensor) -> Option<JobId> {
    jobs.snapshot().jobs.into_iter().find_map(|job| {
        if job.status != JobStatus::Running {
            return None;
        }
        let job_spec = job.job_spec.as_ref()?;
        if job_spec.name == action.name() && parse_sensor(&job_spec.payload) == Some(sensor) {
            Some(job.job_id)
        } else {
            None
        }
    })
}

pub(crate) fn cancellable_roots(jobs: &Jobs) -> Vec<JobId> {
    jobs.snapshot()
        .jobs
        .into_iter()
        .filter(|job| {
            job.parent.is_none()
                && matches!(
                    job.status,
                    JobStatus::Queued | JobStatus::Running | JobStatus::Cancelling
                )
        })
        .map(|job| job.job_id)
        .collect()
}

pub(crate) fn io_from_job(job_id: JobId, job_spec: &JobSpec) -> IoRequest {
    let sensor = parse_sensor(&job_spec.payload).unwrap_or(Sensor::Gyro);
    if job_spec.name == Action::StartPreflightCalibration.name() {
        IoRequest::PreflightCalibration { job_id, sensor }
    } else if job_spec.name == Action::ReadOffsets.name() {
        IoRequest::ReadOffsets { job_id, sensor }
    } else if job_spec.name == Action::CancelCalibration.name() {
        IoRequest::CancelAutopilot { job_id }
    } else {
        // AwaitCommandAck has no IoRequest; it waits on autopilot/state/command_ack (a state read).
        IoRequest::ReadOffsets { job_id, sensor }
    }
}
