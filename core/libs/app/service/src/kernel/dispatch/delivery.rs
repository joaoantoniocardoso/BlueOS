//! Applies one Inbox delivery: decide, commit, or roll back.

use blueos_domain::Domain;

use crate::{builder::InboxCommand, inbox::Delivery, run_outcome::RunOutcome};

use super::{
    super::{
        endpoints::complete_command_reply,
        types::{Kernel, Rejection},
    },
    accept,
    commit::AppliedDeliveryCommit,
    decide::{DecideCommandContext, decide_command},
};

struct PendingDecidedDelivery<D: Domain> {
    command: Option<InboxCommand<D>>,
    now: blueos_domain::Now,
    backup: D::Snapshot,
    jobs_backup: blueos_jobs::Jobs,
    reply: Option<crate::inbox::CommandReply>,
    job_id: Option<blueos_jobs::JobId>,
    persist_settings: bool,
}

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    pub(super) async fn dispatch_delivery(&mut self, delivery: Delivery<D>) -> Option<RunOutcome> {
        let Delivery {
            input,
            reply,
            persist_settings,
        } = delivery;
        let job_id = input.job_id();
        let mut reply = reply;
        if reply_shutting_down(self.shutting_down, &mut reply, job_id).await {
            return None;
        }
        let now = self.clock.now();
        let backup = self.snapshot.clone();
        let jobs_backup = self.own_jobs.clone();
        let command = match accept::accept(self, input) {
            Ok(command) => command,
            Err(rejection) => {
                self.snapshot = backup;
                self.own_jobs = jobs_backup;
                accept::reject(self, reply, job_id, rejection).await;
                return None;
            }
        };
        self.finish_decided_delivery(PendingDecidedDelivery {
            command,
            now,
            backup,
            jobs_backup,
            reply,
            job_id,
            persist_settings,
        })
        .await
    }

    async fn finish_decided_delivery(
        &mut self,
        pending: PendingDecidedDelivery<D>,
    ) -> Option<RunOutcome> {
        let PendingDecidedDelivery {
            command,
            now,
            backup,
            jobs_backup,
            reply,
            job_id,
            persist_settings,
        } = pending;
        let persist_settings = persist_settings && command.is_some();
        #[cfg(feature = "testing")]
        let run_effects = self.effect_log.is_none();
        #[cfg(not(feature = "testing"))]
        let run_effects = true;
        let decided = decide_command(DecideCommandContext {
            service: self.service,
            command,
            now,
            backup: &backup,
            jobs_backup: &jobs_backup,
            snapshot: &mut self.snapshot,
            own_jobs: &mut self.own_jobs,
            jobs_access: self.jobs_access,
            states: &self.states,
            job_outputs: &self.job_outputs,
            timers: &mut self.timers,
            io: &self.io,
            run_effects,
            job_id,
        });
        match decided {
            Ok((events, effects, encoded_states, encoded_job_outputs)) => {
                self.commit_applied_delivery(AppliedDeliveryCommit {
                    backup,
                    jobs_backup,
                    reply,
                    job_id,
                    persist_settings,
                    run_effects,
                    events,
                    effects,
                    encoded_states,
                    encoded_job_outputs,
                })
                .await
            }
            Err(rejection) => {
                self.rollback_applied_delivery(backup, jobs_backup, reply, job_id, rejection)
                    .await
            }
        }
    }

    async fn rollback_applied_delivery(
        &mut self,
        backup: D::Snapshot,
        jobs_backup: blueos_jobs::Jobs,
        reply: Option<crate::inbox::CommandReply>,
        job_id: Option<blueos_jobs::JobId>,
        rejection: Rejection,
    ) -> Option<RunOutcome> {
        self.snapshot = backup;
        self.own_jobs = jobs_backup;
        let stop = if matches!(rejection, Rejection::Panicked) {
            self.tasks
                .record_inbox_loop_panic(self.clock.now().monotonic)
                .await
        } else {
            false
        };
        accept::reject(self, reply, job_id, rejection).await;
        if stop {
            Some(RunOutcome::RepeatedInboxPanics)
        } else {
            None
        }
    }
}

async fn reply_shutting_down(
    shutting_down: bool,
    reply: &mut Option<crate::inbox::CommandReply>,
    job_id: Option<blueos_jobs::JobId>,
) -> bool {
    if !shutting_down {
        return false;
    }
    let Some(reply) = reply.take() else {
        return false;
    };
    complete_command_reply(
        Some(reply),
        crate::command_sender::command_ack(job_id, None, Err(Rejection::ShuttingDown)),
    )
    .await;
    true
}
