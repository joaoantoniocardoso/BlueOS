//! Main Inbox loop and shutdown.

mod active;
mod finalize;
mod shutdown;

use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::{
    inbox::{Delivery, Input},
    run_outcome::RunOutcome,
    shutdown::{SHUTDOWN_IO_DRAIN_TIMEOUT, wait_for_shutdown_signal},
};
use blueos_domain::{Command, Domain};
use tokio::time::{Instant, MissedTickBehavior};

use super::types::{Kernel, METRICS_PERIOD};

struct InboxLoopState {
    repeated_inbox_panics: Option<RunOutcome>,
    shutdown_monotonic_deadline: Option<core::time::Duration>,
    shutdown_receiver: Option<tokio::sync::watch::Receiver<bool>>,
    stop_requested: Arc<AtomicBool>,
    metrics_interval: tokio::time::Interval,
}

impl<D: Domain, Context: Send + Sync + 'static> Kernel<D, Context> {
    /// Handles the Commands in the Inbox one at a time, until shutdown finishes or every endpoint has stopped.
    pub async fn run(mut self) -> RunOutcome {
        let repeated_inbox_panics = self.run_inbox_loop().await;
        finalize::finalize(self, repeated_inbox_panics).await
    }

    async fn run_inbox_loop(&mut self) -> Option<RunOutcome> {
        let mut state = self.prepare_inbox_loop_state();
        loop {
            apply_shutdown_watch(&state.stop_requested, &mut state.shutdown_receiver);
            self.maybe_begin_shutdown(
                &state.stop_requested,
                &mut state.shutdown_monotonic_deadline,
            )
            .await;
            match shutdown_turn(
                self,
                &mut state.shutdown_monotonic_deadline,
                &mut state.repeated_inbox_panics,
            )
            .await
            {
                ShutdownTurn::Return(outcome) => return Some(outcome),
                ShutdownTurn::Break => break,
                ShutdownTurn::Continue => {}
            }
            if state.repeated_inbox_panics.is_some() || self.endpoints.is_empty() {
                break;
            }
            match active::run_one_select(
                self,
                &mut state.shutdown_receiver,
                &state.stop_requested,
                &mut state.metrics_interval,
            )
            .await
            {
                active::ActiveStep::Stop(outcome) => {
                    state.repeated_inbox_panics = Some(outcome);
                    break;
                }
                active::ActiveStep::Break => break,
                active::ActiveStep::Continue => {}
            }
        }
        state.repeated_inbox_panics
    }

    fn prepare_inbox_loop_state(&mut self) -> InboxLoopState {
        let mut shutdown_receiver = None;
        core::mem::swap(&mut self.shutdown_receiver, &mut shutdown_receiver);
        let stop_requested = Arc::new(AtomicBool::new(false));
        let stop_flag_for_signals = Arc::clone(&stop_requested);
        let mut signal_shutdown_receiver = shutdown_receiver.clone();
        self.tasks.spawner().spawn(async move {
            wait_for_shutdown_signal(&mut signal_shutdown_receiver).await;
            stop_flag_for_signals.store(true, Ordering::SeqCst);
        });
        let mut metrics_interval =
            tokio::time::interval_at(Instant::now() + METRICS_PERIOD, METRICS_PERIOD);
        metrics_interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        InboxLoopState {
            repeated_inbox_panics: None,
            shutdown_monotonic_deadline: None,
            shutdown_receiver,
            stop_requested,
            metrics_interval,
        }
    }

    async fn maybe_begin_shutdown(
        &mut self,
        stop_requested: &AtomicBool,
        shutdown_monotonic_deadline: &mut Option<core::time::Duration>,
    ) {
        if stop_requested.load(Ordering::SeqCst) && !self.shutting_down {
            self.begin_shutdown().await;
            *shutdown_monotonic_deadline =
                Some(self.clock.now().monotonic + SHUTDOWN_IO_DRAIN_TIMEOUT);
        }
    }

    async fn begin_shutdown(&mut self) {
        if self.shutting_down {
            return;
        }
        self.shutting_down = true;
        self.tasks.cancel();
        if let Some(request) = self.shutdown_request.lock().await.take() {
            let delivery = Delivery {
                input: Input::Command(Command::Request(request)),
                reply: None,
                persist_settings: false,
            };
            if let Some(sender) = &self.inbox_sender {
                drop(sender.send(delivery).await);
            }
        }
    }
}

enum ShutdownTurn {
    Continue,
    Break,
    Return(RunOutcome),
}

fn apply_shutdown_watch(
    stop_requested: &AtomicBool,
    shutdown_receiver: &mut Option<tokio::sync::watch::Receiver<bool>>,
) {
    if let Some(receiver) = shutdown_receiver
        && *receiver.borrow_and_update()
    {
        stop_requested.store(true, Ordering::SeqCst);
    }
}

async fn shutdown_turn<D: Domain, Context: Send + Sync + 'static>(
    kernel: &mut Kernel<D, Context>,
    shutdown_monotonic_deadline: &mut Option<core::time::Duration>,
    repeated_inbox_panics: &mut Option<RunOutcome>,
) -> ShutdownTurn {
    if !kernel.shutting_down {
        return ShutdownTurn::Continue;
    }
    match shutdown::drain_one_turn(kernel, shutdown_monotonic_deadline, repeated_inbox_panics).await
    {
        shutdown::ShutdownStep::Stop(outcome) => ShutdownTurn::Return(outcome),
        shutdown::ShutdownStep::Finished => ShutdownTurn::Break,
        shutdown::ShutdownStep::KeepDraining => ShutdownTurn::Continue,
    }
}
