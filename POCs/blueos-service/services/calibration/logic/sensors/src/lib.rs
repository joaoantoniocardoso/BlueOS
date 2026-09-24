#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod handle;
mod jobs;
mod keys;
mod model;

pub use keys::{
    AUTOPILOT_RPC_PREFLIGHT, AUTOPILOT_STATE_ACK, AUTOPILOT_STATE_OFFSETS,
    CALIBRATION_STATE_SNAPSHOT, CALIBRATION_STREAM_EVENTS,
};
pub use model::{
    Action, CalibrationStatus, Command, Event, IoRequest, Query, Sensor, Snapshot, UseCase, View,
};

use alloc::vec::Vec;

use blueos_cqrs::{Domain, Effect};
use blueos_jobs::{JobId, JobSpec, Jobs};

pub struct Calibration;

impl Domain for Calibration {
    type Command = Command;
    type Event = Event;
    type Query = Query;
    type View = View;
    type Snapshot = Snapshot;
    type IoRequest = IoRequest;

    fn handle_command(
        snapshot: &mut Self::Snapshot,
        jobs: &mut Jobs,
        command: Self::Command,
    ) -> (Vec<Self::Event>, Vec<Effect<Self::IoRequest>>) {
        let events = match command {
            Command::StartGyro => {
                handle::start_sensor(snapshot, jobs, Sensor::Gyro, UseCase::CalibrateGyroscope)
            }
            Command::StartBaro => {
                handle::start_sensor(snapshot, jobs, Sensor::Baro, UseCase::CalibrateBarometer)
            }
            Command::StartStationary => handle::start_stationary(snapshot, jobs),
            Command::Cancel => handle::cancel(snapshot, jobs),
            Command::JobProgress { job_id, payload } => {
                handle::job_progress(snapshot, jobs, job_id, &payload)
            }
            Command::AckObserved { sensor, ack } => {
                handle::ack_observed(snapshot, jobs, sensor, ack)
            }
            Command::OffsetsRead { sensor, value } => {
                handle::offsets_read(snapshot, jobs, sensor, value)
            }
        };
        let effects = handle::publish_effects(snapshot, &events);
        (events, effects)
    }

    fn handle_query(snapshot: &Self::Snapshot, jobs: &Jobs, query: Self::Query) -> Self::View {
        let _ = query;
        View {
            snapshot: snapshot.clone(),
            jobs: jobs.snapshot(),
        }
    }

    fn io_from_job(job_id: JobId, job_spec: &JobSpec) -> Self::IoRequest {
        jobs::io_from_job(job_id, job_spec)
    }
}

#[cfg(test)]
mod tests;
