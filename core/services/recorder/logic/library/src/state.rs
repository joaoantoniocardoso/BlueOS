//! [`Library`] and its catalog and work parts.

use alloc::{string::String, vec::Vec};

use blueos_jobs::JobId;

use crate::types::{
    LibraryOperation, RecordingContents, RecordingFileEntry, RepairProgress, ScannedRecording,
};

/// In-memory library Block: published catalog plus in-flight work.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Library {
    pub(crate) catalog: LibraryCatalog,
    pub(crate) work: LibraryWork,
}

/// Scanned files and the published catalog rows derived from them.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LibraryCatalog {
    pub(crate) entries: Vec<RecordingFileEntry>,
    pub(crate) scanned: alloc::collections::BTreeMap<String, ScannedRecording>,
    pub(crate) entries_rebuilt_monotonic: core::time::Duration,
}

/// In-flight repairs and snapshots plus paths marked for delete.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LibraryWork {
    pub(crate) queue: LibraryOperationQueue,
    pub(crate) deleting: alloc::collections::BTreeSet<String>,
}

/// Active and recently ended library operations.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LibraryOperationQueue {
    pub(crate) operations: alloc::vec::Vec<LibraryOperation>,
    pub(crate) ended: Option<LibraryOperation>,
    pub(crate) repair_errors: alloc::collections::BTreeMap<String, String>,
    /// Files whose repair failed because they are not MCAP, with the size and modification time they had.
    pub(crate) not_mcap: alloc::collections::BTreeMap<String, (u64, i64)>,
}

impl Library {
    /// Published catalog State.
    pub fn catalog(&self) -> &LibraryCatalog {
        &self.catalog
    }

    /// In-flight repairs, snapshots, and deletes.
    pub fn work(&self) -> &LibraryWork {
        &self.work
    }
}

impl LibraryCatalog {
    /// Published catalog rows, newest first.
    pub fn entries(&self) -> &[RecordingFileEntry] {
        &self.entries
    }

    /// What the recording at `relative_path` holds; `None` when unknown or not in the library.
    pub fn contents(&self, relative_path: &str) -> Option<&RecordingContents> {
        self.scanned.get(relative_path)?.contents.as_ref()
    }
}

impl LibraryWork {
    /// Repairs, snapshots, and delete markers.
    pub fn queue(&self) -> &LibraryOperationQueue {
        &self.queue
    }
}

impl LibraryOperationQueue {
    /// The repairs and snapshots the library wants done, in the order they started.
    pub fn operations(&self) -> &[LibraryOperation] {
        &self.operations
    }

    /// The repair or snapshot that ended last, kept until the next one ends.
    pub fn ended_operation(&self) -> Option<&LibraryOperation> {
        self.ended.as_ref()
    }

    /// How far the repair the Job `job_id` runs has read, while it runs.
    pub fn repair_progress(&self, job_id: JobId) -> Option<&RepairProgress> {
        self.operations
            .iter()
            .find_map(|operation| match operation {
                LibraryOperation::Repair {
                    job_id: running,
                    progress,
                    ..
                } if *running == job_id => Some(progress),
                LibraryOperation::Repair { .. } | LibraryOperation::Snapshot { .. } => None,
            })
    }
}
