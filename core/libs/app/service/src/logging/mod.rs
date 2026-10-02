//! Service logging: console from the first line of entry and Foxglove `Log` on the `log` key (D-13).

use std::sync::Arc;

use blueos_api::log_key;
use blueos_comms::CommsBackend;
use blueos_logging::{LogPublisher, attach, init};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

use crate::shutdown::SHUTDOWN_IO_DRAIN_TIMEOUT;

/// Installs tracing from the first line of [`crate::entry::run`], using the raw `-v` count.
pub(crate) fn init_from_verbosity(verbosity: u8) {
    init(verbosity);
}

/// Connects the backbone layer and returns a publisher the caller runs on a [`TaskTracker`].
pub(crate) async fn attach_backbone(
    service_name: &str,
    backend: Arc<dyn CommsBackend>,
) -> LogPublisher {
    attach(backend, log_key(service_name)).await
}

/// Runs the log publisher on a tracker until [`LogPublisherRuntime::shutdown_and_wait`].
pub(crate) struct LogPublisherRuntime {
    tracker: TaskTracker,
    shutdown: CancellationToken,
}

impl LogPublisherRuntime {
    /// Spawns `publisher` on a dedicated tracker (no detached [`tokio::spawn`]).
    pub(crate) fn start(publisher: LogPublisher) -> Self {
        let tracker = TaskTracker::new();
        let shutdown = CancellationToken::new();
        tracker.spawn(publisher.run(shutdown.clone()));
        Self { tracker, shutdown }
    }

    /// Cancels the publisher, drains queued records within the service shutdown budget, then joins.
    pub(crate) async fn shutdown_and_wait(self) {
        self.shutdown.cancel();
        self.tracker.close();
        let _ = tokio::time::timeout(SHUTDOWN_IO_DRAIN_TIMEOUT, self.tracker.wait()).await;
    }
}
