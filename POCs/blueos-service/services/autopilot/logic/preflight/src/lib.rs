#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod keys;
mod model;
mod preflight;

pub use keys::{RPC_PREFLIGHT, STATE_ACK, STATE_OFFSETS};
pub use model::{Command, Event, IoRequest, Query, Sensor, Snapshot, View};

use alloc::vec::Vec;

use blueos_cqrs::{Domain, Effect};
use blueos_jobs::{JobId, JobSpec, Jobs};

pub struct Autopilot;

impl Domain for Autopilot {
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
        match command {
            Command::SetMoving(moving) => {
                snapshot.moving = moving;
                (Vec::new(), Vec::new())
            }
            Command::Preflight {
                correlation,
                sensor,
            } => preflight::preflight(snapshot, jobs, correlation, sensor),
            Command::AdvanceAck { job_id, sensor } => {
                preflight::advance_ack(snapshot, jobs, job_id, sensor)
            }
        }
    }

    fn handle_query(snapshot: &Self::Snapshot, jobs: &Jobs, query: Self::Query) -> Self::View {
        let _ = (jobs, query);
        View {
            snapshot: snapshot.clone(),
        }
    }

    fn io_from_job(job_id: JobId, job_spec: &JobSpec) -> Self::IoRequest {
        let _ = job_spec; // tick jobs carry the sensor in payload; the host maps job_id to AdvanceAck
        IoRequest::Tick { job_id }
    }
}

#[cfg(test)]
mod tests;
