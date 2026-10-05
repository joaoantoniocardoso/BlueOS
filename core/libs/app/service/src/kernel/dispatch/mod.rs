//! Command delivery through the Inbox.

mod accept;
mod commit;
mod decide;
mod delivery;

use blueos_domain::Domain;

use crate::{inbox::Delivery, inbox_recovery::log_caught_panic, run_outcome::RunOutcome};

use super::types::Kernel;

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    /// Applies one Command as a transaction: if the Domain rejects it, or `handle` or a Projection panics, the
    /// Snapshot is restored from a clone taken first and the domain events are dropped.
    ///
    /// Returns [`RunOutcome::RepeatedInboxPanics`] when the rolling panic budget is exhausted (D-29).
    pub(crate) async fn dispatch(&mut self, delivery: Delivery<D>) -> Option<RunOutcome> {
        self.inbox_depth.set(self.inbox.len() as f64);
        let started = self.clock.now().monotonic;
        let unwound = core::panic::AssertUnwindSafe(self.dispatch_delivery(delivery))
            .catch_unwind()
            .await;
        use futures_util::FutureExt;
        self.inbox_step_time
            .record(self.clock.now().monotonic.saturating_sub(started));
        match unwound {
            Ok(maybe_stop) => maybe_stop,
            Err(panic) => {
                log_caught_panic(self.service, None, panic);
                if self
                    .tasks
                    .record_inbox_loop_panic(self.clock.now().monotonic)
                    .await
                {
                    Some(RunOutcome::RepeatedInboxPanics)
                } else {
                    None
                }
            }
        }
    }

    pub(crate) fn jobs(&self) -> &blueos_jobs::Jobs {
        accept::jobs(self)
    }

    // qual:allow(srp, nms) reason: "jobs_mut forwards to accept so dispatch and accept share one Jobs handle"
    pub(crate) fn jobs_mut(&mut self) -> &mut blueos_jobs::Jobs {
        accept::jobs_mut(self)
    }
}
