//! Graceful shutdown: signals, a test trigger, and in-flight IO draining (D-29).

use core::{
    future::pending,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use std::sync::Arc;

use tokio::sync::watch;

/// How long shutdown waits for in-flight IO after `on_shutdown` runs (spec startup/shutdown).
pub(crate) const SHUTDOWN_IO_DRAIN_TIMEOUT: Duration = Duration::from_secs(5);

/// Test hook to request graceful Kernel shutdown without sending OS signals.
///
/// Obtain from [`ServiceBuilder::shutdown_handle`](crate::ServiceBuilder::shutdown_handle) before
/// [`Kernel::start`](crate::Kernel::start).
#[derive(Clone)]
pub struct ShutdownHandle {
    sender: watch::Sender<bool>,
}

/// Counts IO chains the Kernel spawned and has not finished yet.
#[derive(Clone)]
pub(crate) struct IoInflight {
    count: Arc<AtomicUsize>,
}

pub(crate) struct IoInflightGuard {
    count: Arc<AtomicUsize>,
}

impl ShutdownHandle {
    pub(crate) fn new(sender: watch::Sender<bool>) -> Self {
        Self { sender }
    }

    pub(crate) fn sender(&self) -> watch::Sender<bool> {
        self.sender.clone()
    }

    /// Requests the same shutdown path as `SIGINT` / `SIGTERM`.
    pub fn trigger(&self) {
        let _ = self.sender.send(true);
    }
}

impl IoInflight {
    pub(crate) fn new() -> Self {
        Self {
            count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub(crate) fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }

    pub(crate) fn track(&self) -> IoInflightGuard {
        self.count.fetch_add(1, Ordering::SeqCst);
        IoInflightGuard {
            count: Arc::clone(&self.count),
        }
    }
}

impl Drop for IoInflightGuard {
    fn drop(&mut self) {
        self.count.fetch_sub(1, Ordering::SeqCst);
    }
}

pub(crate) fn new_shutdown_channel() -> (ShutdownHandle, watch::Receiver<bool>) {
    let (sender, receiver) = watch::channel(false);
    (ShutdownHandle::new(sender), receiver)
}

/// Waits until the process should shut down: `SIGINT`, `SIGTERM`, or the test [`ShutdownHandle`].
pub(crate) async fn wait_for_shutdown_signal(
    shutdown_receiver: &mut Option<watch::Receiver<bool>>,
) {
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::pin!(ctrl_c);

    #[cfg(unix)]
    let mut sigterm =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).ok();

    tokio::select! {
        _ = async {
            if let Err(error) = ctrl_c.as_mut().await {
                tracing::warn!(%error, "SIGINT listener failed");
                pending::<()>().await;
            }
        } => {}
        _ = async {
            if let Some(ref mut sigterm) = sigterm
                && sigterm.recv().await.is_some()
            {
                return;
            }
            pending::<()>().await;
        }, if sigterm.is_some() => {}
        _ = async {
            if let Some(receiver) = shutdown_receiver {
                while !*receiver.borrow_and_update() {
                    if receiver.changed().await.is_err() {
                        pending::<()>().await;
                    }
                }
            } else {
                pending::<()>().await;
            }
        }, if shutdown_receiver.is_some() => {}
    }
}
