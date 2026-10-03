//! Service Context: shared adapters and tunables, built in `RecorderService::context`.

use core::{
    sync::atomic::{AtomicBool, AtomicU8},
    time::Duration,
};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use blueos_idl::msg::blueos_recorder_msgs::RecordingIndexResponse;
use blueos_recorder_mcap::{IndexError, RewriteError, RewriteSummary};
use blueos_recorder_storage::{LibraryFooterCache, RecordingsFolder};
use blueos_service::Session;

/// Default MCAP writer queue depth for production wiring.
pub(crate) const DEFAULT_MCAP_WRITER_QUEUE_CAPACITY: usize = 4096;

/// Runs one recording index walk outside the Inbox (production default: MCAP `walk_index`).
pub type IndexWalker = Arc<
    dyn Fn(&Path, u64, u32, &AtomicBool) -> Result<RecordingIndexResponse, IndexError>
        + Send
        + Sync,
>;

/// Rewrites a recording into a new file on a blocking thread: `(source, output, progress, cancel)` (production
/// default: MCAP `rewrite`). The library operations Task runs repair and snapshot through it.
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
    /// Rewrite the library operations Task runs for repair and snapshot.
    pub rewriter: Rewriter,
    /// Wall-clock budget for one `index` walk.
    pub index_walk_timeout: Duration,
    /// Index walk invoked from the `index` IO query handler.
    pub index_walker: IndexWalker,
}
