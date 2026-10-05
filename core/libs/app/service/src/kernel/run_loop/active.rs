//! Active service loop: metrics, inbox, timers, and endpoint tasks.

use core::{
    future::pending,
    sync::atomic::{AtomicBool, Ordering},
};

use tokio::{sync::watch, time::Interval};

use blueos_domain::{Command, Domain};

use crate::{
    inbox::{Delivery, Input},
    run_outcome::RunOutcome,
};

use super::super::types::Kernel;

pub(super) enum ActiveStep {
    Continue,
    Break,
    Stop(RunOutcome),
}

pub(super) async fn run_one_select<D: Domain, Context: Send + Sync + 'static>(
    kernel: &mut Kernel<D, Context>,
    shutdown_receiver: &mut Option<watch::Receiver<bool>>,
    stop_requested: &AtomicBool,
    metrics_interval: &mut Interval,
) -> ActiveStep {
    tokio::select! {
        biased;
        _ = async {
            let Some(receiver) = shutdown_receiver else {
                pending::<()>().await;
                return;
            };
            while !*receiver.borrow_and_update() {
                if receiver.changed().await.is_err() {
                    pending::<()>().await;
                }
            }
        }, if shutdown_receiver.is_some() => {
            stop_requested.store(true, Ordering::SeqCst);
            ActiveStep::Continue
        }
        _ = metrics_interval.tick() => {
            kernel.publish_metrics().await;
            ActiveStep::Continue
        }
        delivery = kernel.inbox.recv() => {
            match delivery {
                Some(delivery) => {
                    if let Some(outcome) = kernel.dispatch(delivery).await {
                        ActiveStep::Stop(outcome)
                    } else {
                        ActiveStep::Continue
                    }
                }
                None if kernel.endpoints.is_empty() => ActiveStep::Break,
                None => ActiveStep::Continue,
            }
        }
        tick = kernel.timers.next_tick(), if kernel.timers.waiting() => {
            if let Some(tick) = tick
                && let Some(outcome) = kernel.dispatch(Delivery {
                    input: Input::Command(Command::Tick(tick)),
                    reply: None,
                    persist_settings: false,
                })
                .await
            {
                ActiveStep::Stop(outcome)
            } else {
                ActiveStep::Continue
            }
        }
        _ = kernel.endpoints.join_next(), if !kernel.endpoints.is_empty() => ActiveStep::Continue,
    }
}
