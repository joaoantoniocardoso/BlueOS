//! Service Context: shared adapters and tunables, built in `RecorderService::context`.

use core::{
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
    time::Duration,
};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc;

use blueos_idl::msg::blueos_recorder_msgs::RecordingIndex;
use blueos_recorder_domain::RecorderObservedFact;
use blueos_recorder_mcap::{IndexError, RewriteError, RewriteSummary};
use blueos_recorder_storage::{LibraryFooterCache, RecordingsFolder};
use blueos_service::Session;

/// Default MCAP writer queue depth for production wiring.
pub(crate) const DEFAULT_MCAP_WRITER_QUEUE_CAPACITY: usize = 4096;

/// Runs one recording index walk outside the Inbox (production default: MCAP `walk_index`).
pub type IndexWalker =
    Arc<dyn Fn(&Path, u64, u32, &AtomicBool) -> Result<RecordingIndex, IndexError> + Send + Sync>;

/// Rewrites a recording into a new file on a blocking thread: `(source, output, progress, cancel)` (production
/// default: MCAP `rewrite`). Repair and snapshot both run through it.
pub type Rewriter = Arc<
    dyn Fn(
            &Path,
            &Path,
            &mut dyn FnMut(u64, u64),
            &AtomicBool,
        ) -> Result<RewriteSummary, RewriteError>
        + Send
        + Sync,
>;

/// Context built in `RecorderService::context` and shared with Tasks and IO executors.
#[derive(Clone)]
pub struct RecorderContext {
    /// Recordings folder on disk (immutable; library scan uses a separate footer cache).
    pub recordings_folder: Arc<RecordingsFolder>,
    /// Footer cache used only on the library blocking IO path.
    pub library_footer_cache: Arc<Mutex<LibraryFooterCache>>,
    /// Bounded queue between the data plane Task and the MCAP writer thread.
    pub mcap_writer_queue_capacity: usize,
    /// Backbone session for IO executors (available before Tasks start).
    pub session: Session,
    /// MAVLink v2 sequence counter for egress frames.
    pub mavlink_sequence: Arc<AtomicU8>,
    /// Sends library repair progress Observed facts to the Inbox bridge Task.
    pub library_observed_sender: mpsc::Sender<RecorderObservedFact>,
    /// Receiver shared by the library Observed-fact bridge Task (async lock while running).
    pub library_observed_receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<RecorderObservedFact>>>,
    /// Per-path cancel flags for in-flight repairs.
    pub repair_cancel_flags: Arc<Mutex<BTreeMap<String, Arc<AtomicBool>>>>,
    /// Rewrite invoked by repair and snapshot IO.
    pub rewriter: Rewriter,
    /// Wall-clock budget for one `index` walk.
    pub index_walk_timeout: Duration,
    /// Index walk invoked from the `index` IO query handler.
    pub index_walker: IndexWalker,
}

impl RecorderContext {
    /// Returns the cancel flag for `relative_path`, creating it when missing.
    pub(crate) fn repair_cancel_flag(
        flags: &Arc<Mutex<BTreeMap<String, Arc<AtomicBool>>>>,
        relative_path: &str,
    ) -> Arc<AtomicBool> {
        let mut flags = flags
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Arc::clone(
            flags
                .entry(relative_path.to_string())
                .or_insert_with(|| Arc::new(AtomicBool::new(false))),
        )
    }

    /// Clears a cancel flag after repair IO ends.
    pub(crate) fn clear_repair_cancel_flag(
        flags: &Arc<Mutex<BTreeMap<String, Arc<AtomicBool>>>>,
        relative_path: &str,
    ) {
        let mut flags = flags
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        flags.remove(relative_path);
    }

    /// Sets the cancel flag for a running repair.
    pub(crate) fn request_repair_cancel(
        flags: &Arc<Mutex<BTreeMap<String, Arc<AtomicBool>>>>,
        relative_path: &str,
    ) {
        Self::repair_cancel_flag(flags, relative_path).store(true, Ordering::Relaxed);
    }
}
