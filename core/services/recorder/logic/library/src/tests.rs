//! L1 tests for the recording library Block acceptance criteria.

use core::time::Duration;

use alloc::{string::ToString, vec};

use blueos_domain::{Effect, Now, Outcome};
use blueos_jobs::JobId;

use super::{
    CANCEL_JOB, Library, LibraryEvent, LibraryIoRequest, LibraryIoResult, LibraryObservedFact,
    LibraryOperation, LibraryRepairOutcome, LibraryRequest, LibrarySnapshotOutcome,
    RecordingFileState, RecordingOperationKind, RepairFailure, SNAPSHOT_RECORDING,
    ScannedRecording, derive_recording_file_state, snapshot_output_relative_path,
};

const NOW: Now = Now {
    wall: Duration::from_secs(20_000),
    monotonic: Duration::from_secs(100),
};

fn scan_snapshot(paths: &[(&str, bool)], modified_unix_seconds: i64) -> Library {
    let mut library = Library::default();
    let recordings = paths
        .iter()
        .map(|(path, indexed)| ScannedRecording {
            relative_path: (*path).into(),
            name: path.rsplit('/').next().unwrap_or(path).into(),
            size_bytes: 100,
            modified_unix_seconds,
            indexed: *indexed,
        })
        .collect();
    let outcome =
        library.handle_io_result(LibraryIoResult::ScanCompleted { recordings }, None, NOW);
    assert!(matches!(outcome, Outcome::Applied { .. }));
    library
}

fn job_id(raw: u128) -> JobId {
    JobId::from_u128(raw)
}

#[test]
fn delete_rejects_active_recording_file() {
    let mut library = scan_snapshot(&[("live.mcap", true)], 1_000);
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
fn repair_rejects_when_already_repairing() {
    let mut library = scan_snapshot(&[("file.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
    assert!(matches!(
        library.start_repair(path.clone(), job_id(1), None, NOW),
        Outcome::Applied { .. }
    ));
    let Outcome::Rejected { reason } = library.start_repair(path, job_id(3), None, NOW) else {
        panic!("second repair must be rejected");
    };
    assert_eq!(
        reason.to_string(),
        "This recording is already being repaired."
    );
}

#[test]
fn repair_rejects_indexed_file() {
    let mut library = scan_snapshot(&[("file.mcap", true)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
    let Outcome::Rejected { reason } = library.start_repair(path, job_id(1), None, NOW) else {
        panic!("repair must be rejected for indexed files");
    };
    assert_eq!(reason.to_string(), "This recording already has an index.");
}

#[test]
fn repair_rejects_active_recording_file() {
    let mut library = scan_snapshot(&[("live.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("live.mcap").expect("path");
    let Outcome::Rejected { reason } =
        library.start_repair(path, job_id(1), Some("live.mcap"), NOW)
    else {
        panic!("repair must be rejected while recording");
    };
    assert_eq!(
        reason.to_string(),
        "This recording is still being written. Try again once it is finished."
    );
}

#[test]
fn repair_rejects_recently_written_file() {
    let mut library = scan_snapshot(&[("recent.mcap", false)], 19_995);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("recent.mcap").expect("path");
    let Outcome::Rejected { reason } = library.start_repair(path, job_id(1), None, NOW) else {
        panic!("repair must be rejected when written less than 10 s ago");
    };
    assert_eq!(
        reason.to_string(),
        "This recording is still being written. Try again once it is finished."
    );
}

#[test]
fn a_repair_is_listed_with_its_job_until_it_ends() {
    let mut library = scan_snapshot(&[("file.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
    assert!(matches!(
        library.start_repair(path.clone(), job_id(7), None, NOW),
        Outcome::Applied { .. }
    ));
    assert_eq!(
        library.operations(),
        [LibraryOperation::Repair {
            path: path.clone(),
            job_id: job_id(7),
        }]
    );
    let repairing = &library.entries()[0];
    assert_eq!(repairing.repair_job_id, Some(job_id(7)));
    assert!(
        repairing
            .allowed_operations
            .iter()
            .any(|operation| operation == CANCEL_JOB)
    );

    library.handle_observed_fact(
        LibraryObservedFact::RepairFinished {
            path,
            outcome: LibraryRepairOutcome::Cancelled,
        },
        None,
        NOW,
    );
    assert!(library.operations().is_empty());
    let ended = &library.entries()[0];
    assert_eq!(ended.repair_job_id, None);
    assert!(
        !ended
            .allowed_operations
            .iter()
            .any(|operation| operation == CANCEL_JOB)
    );
}

#[test]
fn cancelled_repair_operation_event_is_not_a_failure() {
    let mut library = scan_snapshot(&[("file.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
    assert!(matches!(
        library.start_repair(path.clone(), job_id(1), None, NOW),
        Outcome::Applied { .. }
    ));
    let Outcome::Applied { events, .. } = library.handle_observed_fact(
        LibraryObservedFact::RepairFinished {
            path,
            outcome: LibraryRepairOutcome::Cancelled,
        },
        None,
        NOW,
    ) else {
        panic!("repair finish must apply");
    };
    assert_eq!(events.len(), 1);
    let LibraryEvent::Operation(event) = &events[0];
    assert_eq!(event.operation, RecordingOperationKind::Repair);
    assert!(event.cancelled);
    assert!(!event.succeeded);
    assert_eq!(event.failure, RepairFailure::None);
}

#[test]
fn failed_repair_keeps_error_on_entry() {
    let mut library = scan_snapshot(&[("file.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
    assert!(matches!(
        library.start_repair(path.clone(), job_id(1), None, NOW),
        Outcome::Applied { .. }
    ));
    assert!(matches!(
        library.handle_observed_fact(
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
        .entries()
        .iter()
        .find(|entry| entry.path == "file.mcap")
        .expect("entry");
    assert_eq!(entry.repair_error, "MCAP rewrite failed.");
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
fn snapshot_emits_operation_with_output_path() {
    let mut library = scan_snapshot(&[("live.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("live.mcap").expect("path");
    let output_path = snapshot_output_relative_path("live.mcap", NOW);
    assert!(matches!(
        library.start_snapshot(path.clone(), output_path.clone(), Some("live.mcap"), NOW),
        Outcome::Applied { .. }
    ));
    let Outcome::Applied { events, .. } = library.handle_observed_fact(
        LibraryObservedFact::SnapshotFinished {
            path,
            output_path: output_path.clone(),
            outcome: LibrarySnapshotOutcome::Succeeded,
        },
        Some("live.mcap"),
        NOW,
    ) else {
        panic!("snapshot finish must apply");
    };
    let LibraryEvent::Operation(event) = &events[0];
    assert_eq!(event.operation, RecordingOperationKind::Snapshot);
    assert_eq!(event.output_path, output_path);
    assert!(event.succeeded);
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
    }];
    library.handle_io_result(
        LibraryIoResult::ScanCompleted { recordings },
        Some("live.mcap"),
        NOW,
    );
    let entry = library
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
        library.start_repair(path.clone(), job_id(1), None, NOW),
        Outcome::Applied { .. }
    ));
    let Outcome::Applied { effects, .. } = library.handle_observed_fact(
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
