#![no_std]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use blueos_jobs::{JobId, JobSpec, Jobs};
use thiserror::Error;

pub trait Domain {
    type Command;
    type Event;
    type Query;
    type View;
    type Snapshot;
    type IoRequest;

    fn handle_command(
        snapshot: &mut Self::Snapshot,
        jobs: &mut Jobs,
        command: Self::Command,
    ) -> (Vec<Self::Event>, Vec<Effect<Self::IoRequest>>);

    fn handle_query(snapshot: &Self::Snapshot, jobs: &Jobs, query: Self::Query) -> Self::View;

    fn io_from_job(job_id: JobId, job_spec: &JobSpec) -> Self::IoRequest;
}

pub enum Effect<IoRequest> {
    None,
    Io(IoRequest),
    Publish { key: String, payload: Vec<u8> },
    Reply { correlation: u32, payload: Vec<u8> },
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Jobs(#[from] blueos_jobs::JobsError),
}

pub struct App<D: Domain> {
    pub snapshot: D::Snapshot,
    pub jobs: Jobs,
}

impl<D: Domain> App<D> {
    pub fn new(snapshot: D::Snapshot) -> Self {
        Self {
            snapshot,
            jobs: Jobs::new(),
        }
    }

    pub fn handle(&mut self, command: D::Command) -> (Vec<D::Event>, Vec<Effect<D::IoRequest>>) {
        let (events, mut effects) = D::handle_command(&mut self.snapshot, &mut self.jobs, command);
        for (job_id, job_spec) in self.jobs.poll_runnable() {
            effects.push(Effect::Io(D::io_from_job(job_id, &job_spec)));
        }
        (events, effects)
    }

    pub fn query(&self, query: D::Query) -> D::View {
        D::handle_query(&self.snapshot, &self.jobs, query)
    }

    pub fn complete_job(&mut self, job_id: JobId, succeeded: bool) -> Result<(), AppError> {
        self.jobs.complete(job_id, succeeded)?;
        Ok(())
    }
}
