//! Snapshot operations, derive state, and scanned contents.

#[path = "support/library_common.rs"]
mod common;
#[path = "support/library_file_fixture.rs"]
mod file_fixture;

use core::time::Duration;

use blueos_domain::{Effect, Outcome};
use blueos_recorder_library::{
    Library, LibraryIoRequest, LibraryIoResult, LibraryObservedFact, LibraryOperation,
    LibraryRepairOutcome, LibrarySnapshotOutcome, REPAIR_RECORDING, RecordingContents,
    RecordingFileState, RepairFailure, SNAPSHOT_RECORDING, ScannedRecording, SnapshotStartSpec,
    derive_recording_file_state, handle_io_result, handle_observed_fact,
    snapshot_output_relative_path, start_repair, start_snapshot,
};

use common::{NOW, job_id, repair_spec, scan_snapshot};
use file_fixture::default_file_library;

#[test]
fn failed_repair_keeps_error_on_entry() {
    let mut library = scan_snapshot(&[("file.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
    assert!(matches!(
        start_repair(&mut library, repair_spec(path.clone(), job_id(1), None)),
        Outcome::Applied { .. }
    ));
    assert!(matches!(
        handle_observed_fact(
            &mut library,
            LibraryObservedFact::RepairFinished {
                path,
                outcome: LibraryRepairOutcome::Failed(RepairFailure::Rewrite),
            },
            None,
            NOW,
        ),
        Outcome::Applied { .. }
    ));
    let entry = library
        .catalog()
        .entries()
        .iter()
        .find(|entry| entry.path == "file.mcap")
        .expect("entry");
    assert_eq!(entry.repair_error, "MCAP rewrite failed.");
}

#[test]
fn a_repair_that_succeeds_takes_the_row_from_repairing_straight_to_ready() {
    let (mut library, path) = default_file_library();
    start_repair(&mut library, repair_spec(path.clone(), job_id(1), None));
    assert_eq!(
        library.catalog().entries()[0].state,
        RecordingFileState::Repairing
    );

    handle_observed_fact(
        &mut library,
        LibraryObservedFact::RepairFinished {
            path,
            outcome: LibraryRepairOutcome::Succeeded,
        },
        None,
        NOW,
    );

    assert_eq!(
        library.catalog().entries()[0].state,
        RecordingFileState::Ready
    );
}

#[test]
fn a_repair_that_failed_for_a_reason_that_may_pass_is_offered_again() {
    for failure in [
        RepairFailure::Io,
        RepairFailure::Rewrite,
        RepairFailure::Replace,
    ] {
        let mut library = scan_snapshot(&[("file.mcap", false)], 1_000);
        let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
        start_repair(&mut library, repair_spec(path.clone(), job_id(1), None));
        handle_observed_fact(
            &mut library,
            LibraryObservedFact::RepairFinished {
                path,
                outcome: LibraryRepairOutcome::Failed(failure),
            },
            None,
            NOW,
        );
        assert!(
            library.catalog().entries()[0]
                .allowed_operations
                .iter()
                .any(|operation| operation == REPAIR_RECORDING)
        );
    }
}

#[test]
fn recording_state_priority() {
    assert_eq!(
        derive_recording_file_state("live.mcap", Some("live.mcap"), false, true),
        RecordingFileState::Recording
    );
    assert_eq!(
        derive_recording_file_state("live.mcap", None, false, true),
        RecordingFileState::Ready
    );
    assert_eq!(
        derive_recording_file_state("live.mcap", None, false, false),
        RecordingFileState::NeedsRepair
    );
    assert_eq!(
        derive_recording_file_state("repairing.mcap", None, true, true),
        RecordingFileState::Repairing
    );
}

#[test]
fn a_snapshot_that_ends_is_the_ended_operation_with_its_output_path() {
    let mut library = scan_snapshot(&[("live.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("live.mcap").expect("path");
    let output_path = snapshot_output_relative_path("live.mcap", NOW);
    assert!(matches!(
        start_snapshot(
            &mut library,
            SnapshotStartSpec {
                path: path.clone(),
                output_path: output_path.clone(),
                job_id: job_id(4),
                active_recording_relative_path: Some("live.mcap"),
                now: NOW,
            },
        ),
        Outcome::Applied { .. }
    ));
    let snapshot = LibraryOperation::Snapshot {
        path: path.clone(),
        output_path: output_path.clone(),
        job_id: job_id(4),
    };
    assert_eq!(
        library.work().queue().operations(),
        core::slice::from_ref(&snapshot)
    );

    handle_observed_fact(
        &mut library,
        LibraryObservedFact::SnapshotFinished {
            path,
            output_path,
            outcome: LibrarySnapshotOutcome::Succeeded,
        },
        Some("live.mcap"),
        NOW,
    );
    assert!(library.work().queue().operations().is_empty());
    assert_eq!(library.work().queue().ended_operation(), Some(&snapshot));
}

#[test]
fn active_recording_lists_snapshot_in_allowed_operations() {
    let mut library = scan_snapshot(&[("live.mcap", false)], 1_000);
    let recordings = vec![ScannedRecording {
        relative_path: "live.mcap".into(),
        name: "live.mcap".into(),
        size_bytes: 100,
        modified_unix_seconds: 1_000,
        indexed: false,
        contents: None,
    }];
    handle_io_result(
        &mut library,
        LibraryIoResult::ScanCompleted { recordings },
        Some("live.mcap"),
        NOW,
    );
    let entry = library
        .catalog()
        .entries()
        .iter()
        .find(|entry| entry.path == "live.mcap")
        .expect("entry");
    assert!(
        entry
            .allowed_operations
            .iter()
            .any(|operation| operation == SNAPSHOT_RECORDING)
    );
}

#[test]
fn operation_finished_schedules_rescan() {
    let mut library = scan_snapshot(&[("file.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
    assert!(matches!(
        start_repair(&mut library, repair_spec(path.clone(), job_id(1), None)),
        Outcome::Applied { .. }
    ));
    let Outcome::Applied { effects, .. } = handle_observed_fact(
        &mut library,
        LibraryObservedFact::RepairFinished {
            path,
            outcome: LibraryRepairOutcome::Succeeded,
        },
        None,
        NOW,
    ) else {
        panic!("finish must apply");
    };
    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, Effect::Io(LibraryIoRequest::Scan)))
    );
}

#[test]
fn the_library_keeps_the_contents_of_each_scanned_recording() {
    let mut library = Library::default();
    let recordings = vec![ScannedRecording {
        relative_path: "dive.mcap".into(),
        name: "dive.mcap".into(),
        size_bytes: 100,
        modified_unix_seconds: 1_000,
        indexed: true,
        contents: Some(RecordingContents {
            duration: Duration::from_secs(90),
            video_topics: vec!["video/camera/stream".into()],
            other_topic_count: 2,
        }),
    }];
    handle_io_result(
        &mut library,
        LibraryIoResult::ScanCompleted { recordings },
        None,
        NOW,
    );

    assert_eq!(
        library.catalog().contents("dive.mcap"),
        Some(&RecordingContents {
            duration: Duration::from_secs(90),
            video_topics: vec!["video/camera/stream".into()],
            other_topic_count: 2,
        })
    );
    assert_eq!(library.catalog().contents("missing.mcap"), None);
}
