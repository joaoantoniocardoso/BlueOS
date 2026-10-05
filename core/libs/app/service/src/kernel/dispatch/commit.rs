//! Publish and persist after a successful Command decision.

use std::sync::Arc;

use tokio::sync::mpsc;

use blueos_domain::Domain;

use crate::{inbox::CommandReply, sync::lock_unpoisoned};

use super::super::{
    effects::io_requests,
    endpoints::complete_command_reply,
    io::{IoChainWork, spawn_io_chain},
    types::{EncodedJobOutput, Kernel, Rejection},
};

pub(super) struct AppliedDeliveryCommit<D: Domain> {
    pub backup: D::Snapshot,
    pub jobs_backup: blueos_jobs::Jobs,
    pub reply: Option<CommandReply>,
    pub job_id: Option<blueos_jobs::JobId>,
    pub persist_settings: bool,
    pub run_effects: bool,
    pub events: Vec<D::Event>,
    pub effects: Vec<blueos_domain::Effect<D::Tick, D::IoRequest, D::TimerKey>>,
    pub encoded_states: Vec<Result<Vec<u8>, blueos_idl::Error>>,
    pub encoded_job_outputs: Vec<EncodedJobOutput>,
}

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    pub(super) async fn commit_applied_delivery(
        &mut self,
        commit: AppliedDeliveryCommit<D>,
    ) -> Option<crate::run_outcome::RunOutcome> {
        let AppliedDeliveryCommit {
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
        } = commit;
        self.sync_query_snapshot().await;
        self.record_effect_log(&effects);
        let mut reply = reply;
        if self
            .persist_settings_after_apply(
                persist_settings,
                &backup,
                &jobs_backup,
                &mut reply,
                job_id,
            )
            .await
        {
            return None;
        }
        self.queue_durable_if_changed(&backup);
        self.publish_states(encoded_states).await;
        self.publish_jobs().await;
        let job_results = self.publish_job_feedback(encoded_job_outputs).await;
        self.publish_settings().await;
        self.projections.refresh(&self.snapshot);
        let job = job_id.and_then(|job_id| self.jobs().job(job_id));
        complete_command_reply(
            reply,
            crate::command_sender::command_ack(job_id, job, Ok(())),
        )
        .await;
        self.publish_events(events).await;
        self.publish_job_results(job_results).await;
        if run_effects {
            self.spawn_io_for_effects(&effects);
        }
        self.tasks.mark_inbox_loop_healthy().await;
        None
    }

    async fn sync_query_snapshot(&self) {
        let mut shared = self.snapshot_for_queries.write().await;
        *shared = self.snapshot.clone();
    }

    #[cfg(feature = "testing")]
    fn record_effect_log(
        &self,
        effects: &[blueos_domain::Effect<D::Tick, D::IoRequest, D::TimerKey>],
    ) {
        if let Some(log) = &self.effect_log {
            lock_unpoisoned(log).push(effects.to_vec());
        }
    }

    #[cfg(not(feature = "testing"))]
    fn record_effect_log(
        &self,
        _effects: &[blueos_domain::Effect<D::Tick, D::IoRequest, D::TimerKey>],
    ) {
    }

    async fn persist_settings_after_apply(
        &mut self,
        persist_settings: bool,
        backup: &D::Snapshot,
        jobs_backup: &blueos_jobs::Jobs,
        reply: &mut Option<CommandReply>,
        job_id: Option<blueos_jobs::JobId>,
    ) -> bool {
        if !persist_settings {
            return false;
        }
        let Some(settings) = &self.settings else {
            return false;
        };
        let persist_result = lock_unpoisoned(&settings.driver).persist(&self.snapshot);
        match persist_result {
            Ok(()) => {
                lock_unpoisoned(&settings.driver).commit_persisted(&self.snapshot);
                false
            }
            Err(error) => {
                self.snapshot = backup.clone();
                self.own_jobs = jobs_backup.clone();
                complete_command_reply(
                    reply.take(),
                    crate::command_sender::command_ack(
                        job_id,
                        None,
                        Err(Rejection::Domain(error.into())),
                    ),
                )
                .await;
                true
            }
        }
    }

    fn queue_durable_if_changed(&self, backup: &D::Snapshot) {
        if let Some(durable) = &self.durable
            && (durable.changed)(backup, &self.snapshot)
        {
            durable
                .persister
                .queue_document((durable.serialize)(&self.snapshot));
        }
    }

    fn spawn_io_for_effects(
        &self,
        effects: &[blueos_domain::Effect<D::Tick, D::IoRequest, D::TimerKey>],
    ) {
        let requests = io_requests::<D>(effects);
        if let Some(inbox_sender) = &self.inbox_sender
            && !requests.is_empty()
        {
            spawn_io_chain(IoChainWork {
                spawner: self.tasks.spawner(),
                executors: self.io.clone(),
                context: Arc::clone(&self.context),
                snapshot: self.snapshot.clone(),
                requests,
                inbox: mpsc::Sender::clone(inbox_sender),
                io_inflight: self.io_inflight.clone(),
            });
        }
    }
}
