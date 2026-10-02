//! Inbox loop panic recovery: degraded `status` and repeated-panic exit (D-29).

use core::{any::Any, time::Duration};

use std::sync::Mutex;

use tracing::error;

use crate::sync::lock_unpoisoned;

/// Name of the Inbox loop in the shared degraded-task set (D-12, D-29).
pub(crate) const INBOX_LOOP_NAME: &str = "inbox";

/// How many Inbox loop panics within [`LOOP_PANIC_WINDOW`] make the process exit.
const LOOP_PANIC_EXIT_THRESHOLD: usize = 3;
/// Rolling window for counting Inbox loop panics (monotonic time).
const LOOP_PANIC_WINDOW: Duration = Duration::from_secs(60);

/// Timestamps of recent Inbox loop recoveries on the injected [`Clock`](crate::Clock).
pub(crate) struct LoopPanicTracker {
    recent: Mutex<Vec<Duration>>,
}

impl LoopPanicTracker {
    pub(crate) fn new() -> Self {
        Self {
            recent: Mutex::new(Vec::new()),
        }
    }

    /// Records one recovery at `monotonic_now`. Returns true when the Kernel should exit non-zero.
    pub(crate) fn record_recovery(&self, monotonic_now: Duration) -> bool {
        let mut recent = lock_unpoisoned(&self.recent);
        recent.push(monotonic_now);
        recent.retain(|timestamp| monotonic_now.saturating_sub(*timestamp) <= LOOP_PANIC_WINDOW);
        recent.len() >= LOOP_PANIC_EXIT_THRESHOLD
    }
}

/// Formats a caught panic payload the same way the logging panic hook does (D-13).
pub(crate) fn panic_message(panic: &dyn Any) -> String {
    if let Some(text) = panic.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = panic.downcast_ref::<String>() {
        text.clone()
    } else {
        "panic".to_string()
    }
}

/// Logs a caught panic through `tracing` so it reaches the `log` key (D-29).
pub(crate) fn log_caught_panic(service: &str, task: Option<&str>, panic: Box<dyn Any + Send>) {
    let message = panic_message(panic.as_ref());
    if let Some(task) = task {
        error!(service, task, %message, "panic");
    } else {
        error!(service, %message, "panic");
    }
}
