//! Service Context: Projections and shared adapters.

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc;

use blueos_recorder_capture::RecordGate;
use blueos_recorder_domain::RecorderObservedFact;
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
    /// Sends library repair progress Observed facts to the Inbox bridge Task.
    pub library_observed_sender: mpsc::Sender<RecorderObservedFact>,
    /// Receiver shared by the library Observed-fact bridge Task (async lock while running).
    pub library_observed_receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<RecorderObservedFact>>>,
    /// Per-path cancel flags for in-flight repairs.
    pub repair_cancel_flags: Arc<Mutex<BTreeMap<String, Arc<AtomicBool>>>>,
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
