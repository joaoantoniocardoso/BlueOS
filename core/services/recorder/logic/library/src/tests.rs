//! L1 tests for the recording library Block acceptance criteria.

use core::time::Duration;

use alloc::string::ToString;

use blueos_domain::{Effect, Now, Outcome};

use super::{
    Library, LibraryIoRequest, LibraryIoResult, LibraryRequest, LibraryTick, ScannedRecording,
};

const NOW: Now = Now {
    wall: Duration::from_secs(1_700_000_000),
    monotonic: Duration::from_secs(1),
};

fn scan_snapshot(paths: &[(&str, bool)]) -> Library {
    let mut library = Library::default();
    let recordings = paths
        .iter()
        .map(|(path, indexed)| ScannedRecording {
            relative_path: (*path).into(),
            name: path.rsplit('/').next().unwrap_or(path).into(),
            size_bytes: 100,
            modified_unix_seconds: 1_000,
            indexed: *indexed,
        })
        .collect();
    let outcome =
        library.handle_io_result(LibraryIoResult::ScanCompleted { recordings }, None, NOW);
    assert!(matches!(outcome, Outcome::Applied { .. }));
    library
}

#[test]
fn delete_rejects_active_recording_file() {
    let mut library = scan_snapshot(&[("live.mcap", true)]);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("live.mcap").expect("path");
    let outcome = library.handle_request(
        LibraryRequest::DeleteRecording { path },
        Some("live.mcap"),
        NOW,
    );
    let Outcome::Rejected { reason } = outcome else {
        panic!("delete must be rejected while recording");
    };
    assert_eq!(reason.to_string(), "This recording is still being written.");
}

#[test]
fn scan_failed_keeps_previous_catalog() {
    let mut library = scan_snapshot(&[("keep.mcap", true)]);
    let before = library.entries().to_vec();
    let outcome = library.handle_io_result(LibraryIoResult::ScanFailed, None, NOW);
    assert!(matches!(outcome, Outcome::Applied { .. }));
    assert_eq!(library.entries(), before.as_slice());
}

#[test]
fn rescan_tick_requests_scan_io() {
    let outcome = Library::handle_tick(LibraryTick::Rescan);
    let Outcome::Applied { effects, .. } = outcome else {
        panic!("tick must apply");
    };
    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, Effect::Io(LibraryIoRequest::Scan)))
    );
    assert!(
        !effects
            .iter()
            .any(|effect| matches!(effect, Effect::Schedule { .. }))
    );
}
