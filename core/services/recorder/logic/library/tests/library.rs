//! L1 tests for the recording library Block acceptance criteria.

use core::time::Duration;

use blueos_domain::{Effect, Now, Outcome};
use blueos_jobs::JobId;
use blueos_recorder_library::{
    CANCEL_JOB, Library, LibraryIoRequest, LibraryIoResult, LibraryObservedFact, LibraryOperation,
    LibraryRepairOutcome, LibraryRepairProgress, LibraryRequest, LibrarySnapshotOutcome,
    REPAIR_PROGRESS_PUBLISH_INTERVAL, REPAIR_RECORDING, RecordingContents, RecordingFileState,
    RepairFailure, RepairProgress, SNAPSHOT_RECORDING, ScannedRecording,
    derive_recording_file_state, snapshot_output_relative_path,
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
            contents: None,
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
        LibraryRequest::DeleteRecording {
            path,
            job_id: JobId::from_u128(1),
        },
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
            progress: RepairProgress {
                bytes_processed: 0,
                total_bytes: 100,
                started_monotonic: NOW.monotonic,
            },
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
fn a_repair_reports_its_read_offset_to_its_job_until_it_ends() {
    let mut library = scan_snapshot(&[("file.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
    assert!(matches!(
        library.start_repair(path.clone(), job_id(1), None, NOW),
        Outcome::Applied { .. }
    ));
    library.handle_observed_fact(
        LibraryObservedFact::RepairProgress(LibraryRepairProgress {
            path: path.clone(),
            bytes_processed: 40,
            total_bytes: 100,
        }),
        None,
        NOW,
    );
    let progress = library.repair_progress(job_id(1)).expect("progress");
    assert_eq!((progress.bytes_processed, progress.total_bytes), (40, 100));
    assert!(library.repair_progress(job_id(2)).is_none());

    library.handle_observed_fact(
        LibraryObservedFact::RepairFinished {
            path: path.clone(),
            outcome: LibraryRepairOutcome::Cancelled,
        },
        None,
        NOW,
    );
    assert!(library.repair_progress(job_id(1)).is_none());
    assert_eq!(
        library.ended_operation(),
        Some(&LibraryOperation::Repair {
            path,
            job_id: job_id(1),
            progress: RepairProgress {
                bytes_processed: 40,
                total_bytes: 100,
                started_monotonic: NOW.monotonic,
            },
        })
    );
    assert_eq!(library.entries()[0].repair_error, "");
}

#[test]
fn the_library_takes_repair_progress_once_per_publish_interval_and_always_its_end() {
    let mut library = scan_snapshot(&[("file.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
    assert!(matches!(
        library.start_repair(path.clone(), job_id(1), None, NOW),
        Outcome::Applied { .. }
    ));
    let mut report = |bytes_processed: u64, after: Duration| {
        library.handle_observed_fact(
            LibraryObservedFact::RepairProgress(LibraryRepairProgress {
                path: path.clone(),
                bytes_processed,
                total_bytes: 100,
            }),
            None,
            Now {
                wall: NOW.wall + after,
                monotonic: NOW.monotonic + after,
            },
        );
        let published = library.entries()[0].repair_bytes_processed;
        let latest = library
            .repair_progress(job_id(1))
            .expect("progress")
            .bytes_processed;
        (published, latest)
    };

    assert_eq!(
        report(10, REPAIR_PROGRESS_PUBLISH_INTERVAL / 5),
        (0, 10),
        "progress right after the repair started waits for the publish interval"
    );
    assert_eq!(report(20, REPAIR_PROGRESS_PUBLISH_INTERVAL), (20, 20));
    assert_eq!(
        report(30, REPAIR_PROGRESS_PUBLISH_INTERVAL * 6 / 5),
        (20, 30),
        "progress within the publish interval waits"
    );
    assert_eq!(
        report(100, REPAIR_PROGRESS_PUBLISH_INTERVAL * 7 / 5),
        (100, 100),
        "the end of the read is published at once"
    );
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
fn a_repair_that_succeeds_takes_the_row_from_repairing_straight_to_ready() {
    let mut library = scan_snapshot(&[("file.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
    library.start_repair(path.clone(), job_id(1), None, NOW);
    assert_eq!(library.entries()[0].state, RecordingFileState::Repairing);

    library.handle_observed_fact(
        LibraryObservedFact::RepairFinished {
            path,
            outcome: LibraryRepairOutcome::Succeeded,
        },
        None,
        NOW,
    );

    assert_eq!(library.entries()[0].state, RecordingFileState::Ready);
}

#[test]
fn a_file_that_is_not_an_mcap_is_not_offered_repair_again_until_it_changes() {
    let mut library = scan_snapshot(&[("file.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
    library.start_repair(path.clone(), job_id(1), None, NOW);
    library.handle_observed_fact(
        LibraryObservedFact::RepairFinished {
            path: path.clone(),
            outcome: LibraryRepairOutcome::Failed(RepairFailure::NotMcap),
        },
        None,
        NOW,
    );
    let offers_repair = |catalog: &Library| {
        catalog.entries()[0]
            .allowed_operations
            .iter()
            .any(|operation| operation == REPAIR_RECORDING)
    };
    assert_eq!(library.entries()[0].state, RecordingFileState::NeedsRepair);
    assert_eq!(
        library.entries()[0].repair_error,
        "This is not an MCAP file."
    );
    assert!(!offers_repair(&library));
    let Outcome::Rejected { reason } = library.start_repair(path.clone(), job_id(2), None, NOW)
    else {
        panic!("a repair that cannot succeed must be rejected");
    };
    assert_eq!(reason.to_string(), "This recording is not an MCAP file.");

    let rescan = |catalog: &mut Library, size_bytes: u64, modified_unix_seconds: i64| {
        catalog.handle_io_result(
            LibraryIoResult::ScanCompleted {
                recordings: vec![ScannedRecording {
                    relative_path: "file.mcap".into(),
                    name: "file.mcap".into(),
                    size_bytes,
                    modified_unix_seconds,
                    indexed: false,
                    contents: None,
                }],
            },
            None,
            NOW,
        );
    };
    rescan(&mut library, 100, 1_000);
    assert!(
        !offers_repair(&library),
        "a rescan of the same file keeps it"
    );
    rescan(&mut library, 150, 1_000);
    assert!(offers_repair(&library), "a new size offers repair again");

    library.handle_observed_fact(
        LibraryObservedFact::RepairFinished {
            path,
            outcome: LibraryRepairOutcome::Failed(RepairFailure::NotMcap),
        },
        None,
        NOW,
    );
    assert!(!offers_repair(&library));
    rescan(&mut library, 150, 1_500);
    assert!(
        offers_repair(&library),
        "a new modification time offers repair again"
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
        library.start_repair(path.clone(), job_id(1), None, NOW);
        library.handle_observed_fact(
            LibraryObservedFact::RepairFinished {
                path,
                outcome: LibraryRepairOutcome::Failed(failure),
            },
            None,
            NOW,
        );
        assert!(
            library.entries()[0]
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
        library.start_snapshot(
            path.clone(),
            output_path.clone(),
            job_id(4),
            Some("live.mcap"),
            NOW
        ),
        Outcome::Applied { .. }
    ));
    let snapshot = LibraryOperation::Snapshot {
        path: path.clone(),
        output_path: output_path.clone(),
        job_id: job_id(4),
    };
    assert_eq!(library.operations(), core::slice::from_ref(&snapshot));

    library.handle_observed_fact(
        LibraryObservedFact::SnapshotFinished {
            path,
            output_path,
            outcome: LibrarySnapshotOutcome::Succeeded,
        },
        Some("live.mcap"),
        NOW,
    );
    assert!(library.operations().is_empty());
    assert_eq!(library.ended_operation(), Some(&snapshot));
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
    library.handle_io_result(LibraryIoResult::ScanCompleted { recordings }, None, NOW);

    assert_eq!(
        library.contents("dive.mcap"),
        Some(&RecordingContents {
            duration: Duration::from_secs(90),
            video_topics: vec!["video/camera/stream".into()],
            other_topic_count: 2,
        })
    );
    assert_eq!(library.contents("missing.mcap"), None);
}
