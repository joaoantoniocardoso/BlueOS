//! Synchronous Command decision inside a panic-catching transaction.

use core::panic::AssertUnwindSafe;
use std::panic;

use blueos_domain::{Domain, Outcome};
use blueos_jobs::{JobEnd, JobStatus};

use crate::{
    builder::{InboxCommand, JobsAccess},
    inbox_recovery::{self, log_caught_panic},
};

use super::super::{
    effects::apply_sync_effects,
    jobs::{encode_job_outputs, jobs_in, jobs_in_mut},
    timers::TimerWheel,
    types::{EncodedJobOutput, JobTypeOutput, PublishedState, Rejection},
};

type DecideCommandOutcome<D> = (
    Vec<<D as Domain>::Event>,
    Vec<
        blueos_domain::Effect<
            <D as Domain>::Tick,
            <D as Domain>::IoRequest,
            <D as Domain>::TimerKey,
        >,
    >,
    Vec<Result<Vec<u8>, blueos_idl::Error>>,
    Vec<EncodedJobOutput>,
);

pub(super) struct DecideCommandContext<'a, D: Domain, Context> {
    pub service: &'static str,
    pub command: Option<InboxCommand<D>>,
    pub now: blueos_domain::Now,
    pub backup: &'a D::Snapshot,
    pub jobs_backup: &'a blueos_jobs::Jobs,
    pub snapshot: &'a mut D::Snapshot,
    pub own_jobs: &'a mut blueos_jobs::Jobs,
    pub jobs_access: Option<JobsAccess<D>>,
    pub states: &'a [PublishedState<D>],
    pub job_outputs: &'a [JobTypeOutput<D>],
    pub timers: &'a mut TimerWheel<D>,
    pub io: &'a super::super::io::IoExecutors<D, Context>,
    pub run_effects: bool,
    pub job_id: Option<blueos_jobs::JobId>,
}

pub(super) fn decide_command<D: Domain, Context>(
    context: DecideCommandContext<'_, D, Context>,
) -> Result<DecideCommandOutcome<D>, Rejection> {
    let DecideCommandContext {
        service,
        command,
        now,
        backup,
        jobs_backup,
        snapshot,
        own_jobs,
        jobs_access,
        states,
        job_outputs,
        timers,
        io,
        run_effects,
        job_id,
    } = context;
    panic::catch_unwind(AssertUnwindSafe(|| {
        let outcome = match command {
            Some(command) => D::handle(snapshot, command, now),
            None => Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            },
        };
        match outcome {
            Outcome::Applied { events, effects } => {
                apply_sync_effects(&effects, timers, io, run_effects)
                    .map_err(|error| Rejection::Domain(Box::new(error)))?;
                if let Some(job_id) = job_id {
                    let jobs = jobs_in_mut::<D>(jobs_access, snapshot, own_jobs);
                    if jobs.job(job_id).is_some_and(|job| {
                        !job.nature.lasting && job.status == JobStatus::Executing
                    }) {
                        jobs.end(job_id, JobEnd::Succeeded)?;
                    }
                }
                let encoded_states: Vec<_> = states
                    .iter()
                    .map(|state| (state.endpoint.project)(snapshot))
                    .collect();
                let encoded_job_outputs = encode_job_outputs(
                    job_outputs,
                    snapshot,
                    jobs_in::<D>(jobs_access, snapshot, own_jobs),
                    jobs_in::<D>(jobs_access, backup, jobs_backup),
                );
                Ok((events, effects, encoded_states, encoded_job_outputs))
            }
            Outcome::Rejected { reason } => Err(Rejection::Domain(reason)),
        }
    }))
    .unwrap_or_else(|panic| {
        log_caught_panic(service, Some(inbox_recovery::INBOX_LOOP_NAME), panic);
        Err(Rejection::Panicked)
    })
}
