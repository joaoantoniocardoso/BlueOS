//! Shared helpers for library integration tests.

use core::time::Duration;

use blueos_domain::{Now, Outcome};
use blueos_jobs::JobId;
use blueos_recorder_paths::RecordingRelativePath;

use blueos_recorder_library::{
    Library, LibraryIoResult, RepairStartSpec, ScannedRecording, handle_io_result,
};

/// Fixed clock for deterministic library tests.
pub(crate) const NOW: Now = Now {
    wall: Duration::from_secs(20_000),
    monotonic: Duration::from_secs(100),
};

/// Builds a library whose catalog contains `paths` after a successful scan.
pub(crate) fn scan_snapshot(paths: &[(&str, bool)], modified_unix_seconds: i64) -> Library {
    let mut library = Library::default();
    let recordings = paths
        .iter()
        .map(|(path, indexed)| ScannedRecording {
            relative_path: (*path).into(),
            name: path.rsplit('/').next().unwrap_or(path).into(),
            size_bytes: 100,
            modified_unix_seconds,
            indexed: *indexed,
            contents: None,
        })
        .collect();
    let outcome = handle_io_result(
        &mut library,
        LibraryIoResult::ScanCompleted { recordings },
        None,
        NOW,
    );
    assert!(matches!(outcome, Outcome::Applied { .. }));
    library
}

/// Builds a [`JobId`] from a test-only raw value.
pub(crate) fn job_id(raw: u128) -> JobId {
    JobId::from_u128(raw)
}

/// Repair start inputs using [`NOW`].
pub(crate) fn repair_spec(
    path: RecordingRelativePath,
    job_id: JobId,
    active_recording_relative_path: Option<&str>,
) -> RepairStartSpec<'_> {
    RepairStartSpec {
        path,
        job_id,
        active_recording_relative_path,
        now: NOW,
    }
}
