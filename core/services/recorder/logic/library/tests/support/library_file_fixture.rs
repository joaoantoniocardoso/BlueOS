//! Single-file library rows for repair-focused tests.

use blueos_recorder_paths::RecordingRelativePath;

use blueos_recorder_library::Library;

use crate::common::scan_snapshot;

/// Default unscanned `file.mcap` library row for repair tests.
pub(crate) fn default_file_library() -> (Library, RecordingRelativePath) {
    let library = scan_snapshot(&[("file.mcap", false)], 1_000);
    let path = RecordingRelativePath::parse("file.mcap").expect("path");
    (library, path)
}
