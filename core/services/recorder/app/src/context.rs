//! Service Context: Projections and shared adapters.

use std::sync::{Arc, Mutex};

use blueos_recorder_capture::RecordGate;
use blueos_recorder_storage::{LibraryFooterCache, RecordingsFolder};
use blueos_service::Projection;

/// Context built in `RecorderService::build` and shared with Tasks.
pub struct RecorderContext {
    /// Projection the data plane reconciles against.
    pub record_gate: Projection<RecordGate>,
    /// Recordings folder on disk (immutable; library scan uses a separate footer cache).
    pub recordings_folder: Arc<RecordingsFolder>,
    /// Footer cache used only on the library blocking IO path.
    pub library_footer_cache: Arc<Mutex<LibraryFooterCache>>,
    /// Bounded queue between the data plane Task and the MCAP writer thread.
    pub mcap_writer_queue_capacity: usize,
}
