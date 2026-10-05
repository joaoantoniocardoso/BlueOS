//! Job submit/control acceptance and command rejection replies.

use blueos_domain::{Command, Domain};
use blueos_jobs::{JobControl, JobId, Jobs, JobsError, Submitted};

use crate::{
    builder::InboxCommand,
    command_sender::command_ack,
    inbox::{CommandReply, Input},
};

use super::super::{
    endpoints::complete_command_reply,
    jobs::{jobs_in, jobs_in_mut},
    types::{Kernel, Rejection},
};

pub(super) fn accept<D: Domain, Context: Send + Sync + 'static>(
    kernel: &mut Kernel<D, Context>,
    input: Input<D>,
) -> Result<Option<InboxCommand<D>>, Rejection> {
    match input {
        Input::Command(command) => Ok(Some(command)),
        Input::Submit {
            job_id,
            job_type,
            goal,
            nature,
            request,
        } => match kernel.jobs_mut().submit(job_id, &job_type, &goal, nature)? {
            Submitted::New if !nature.needs_permission => Ok(Some(Command::Request(request))),
            Submitted::New | Submitted::Retry => Ok(None),
        },
        Input::Control { job_id, control } => {
            kernel.jobs_mut().control(job_id, control)?;
            if control != (JobControl::AnswerPermission { granted: true }) {
                return Ok(None);
            }
            let job = kernel
                .jobs()
                .job(job_id)
                .ok_or(JobsError::Unknown(job_id))?;
            let decode = kernel
                .goal_decoders
                .get(&job.job_type)
                .ok_or_else(|| Rejection::UnknownJobType(job.job_type.clone()))?;
            decode(job_id, &job.goal).map(|request| Some(Command::Request(request)))
        }
    }
}

pub(super) async fn reject<D: Domain, Context: Send + Sync + 'static>(
    kernel: &Kernel<D, Context>,
    reply: Option<CommandReply>,
    job_id: Option<JobId>,
    rejection: Rejection,
) {
    let job = match rejection {
        Rejection::Jobs(JobsError::IdReused(_)) => None,
        _ => job_id.and_then(|job_id| kernel.jobs().job(job_id)),
    };
    complete_command_reply(reply, command_ack(job_id, job, Err(rejection))).await;
}

pub(super) fn jobs<D: Domain, Context: Send + Sync + 'static>(
    kernel: &Kernel<D, Context>,
) -> &Jobs {
    jobs_in::<D>(kernel.jobs_access, &kernel.snapshot, &kernel.own_jobs)
}

pub(super) fn jobs_mut<D: Domain, Context: Send + Sync + 'static>(
    kernel: &mut Kernel<D, Context>,
) -> &mut Jobs {
    jobs_in_mut::<D>(
        kernel.jobs_access,
        &mut kernel.snapshot,
        &mut kernel.own_jobs,
    )
}
