//! Service Context: Projections and shared adapters.

use core::sync::atomic::AtomicU8;
use std::sync::{Arc, Mutex};

use blueos_recorder_capture::RecordGate;
use blueos_recorder_storage::{LibraryFooterCache, RecordingsFolder};
use blueos_service::{Projection, Session};

/// Context built in `RecorderService::build` and shared with Tasks and IO executors.
pub struct RecorderContext {
    /// Projection the data plane reconciles against.
    pub record_gate: Projection<RecordGate>,
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
}
