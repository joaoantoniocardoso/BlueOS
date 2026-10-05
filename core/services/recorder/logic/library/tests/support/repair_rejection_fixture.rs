//! Library rows for repair rejection tests in `library.rs`.

use blueos_recorder_paths::RecordingRelativePath;

use blueos_recorder_library::Library;

use crate::common::scan_snapshot;

/// Scanned library row for repair rejection tests.
pub(crate) fn repair_rejection_library(
    file_name: &str,
    indexed: bool,
    modified_unix_seconds: i64,
) -> (Library, RecordingRelativePath) {
    let library = scan_snapshot(&[(file_name, indexed)], modified_unix_seconds);
    let path = RecordingRelativePath::parse(file_name).expect("path");
    (library, path)
}
