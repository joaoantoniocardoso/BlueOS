use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::watch;

/// Test hook to request graceful kernel shutdown without sending OS signals.
///
/// Obtain from [`ServiceBuilder::shutdown_handle`] before [`ServiceBuilder::run`] or
/// [`ServiceBuilder::run_with_session`].
#[derive(Clone)]
pub struct ShutdownHandle {
    sender: watch::Sender<bool>,
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

pub(crate) fn new_shutdown_channel() -> (ShutdownHandle, watch::Receiver<bool>) {
    let (sender, receiver) = watch::channel(false);
    (ShutdownHandle::new(sender), receiver)
}

#[derive(Clone)]
pub(crate) struct IoInflight {
    count: Arc<AtomicUsize>,
}

impl IoInflight {
    pub fn new() -> Self {
        Self {
            count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }

    pub fn track(&self) -> IoInflightGuard {
        self.count.fetch_add(1, Ordering::SeqCst);
        IoInflightGuard {
            count: Arc::clone(&self.count),
        }
    }
}

pub(crate) struct IoInflightGuard {
    count: Arc<AtomicUsize>,
}

impl Drop for IoInflightGuard {
    fn drop(&mut self) {
        self.count.fetch_sub(1, Ordering::SeqCst);
    }
}
