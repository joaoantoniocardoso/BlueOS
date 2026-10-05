//! L1 tests for the recording library Block acceptance criteria.

#[path = "support/library_common.rs"]
mod common;
#[path = "support/repair_rejection_fixture.rs"]
mod repair_rejection_fixture;

use blueos_domain::Outcome;
use blueos_recorder_library::{
    CANCEL_JOB, LibraryObservedFact, LibraryOperation, LibraryRepairOutcome, LibraryRepairProgress,
    LibraryRequest, RepairProgress, handle_library_request, handle_observed_fact, start_repair,
};

use common::{NOW, job_id, repair_spec, scan_snapshot};
use repair_rejection_fixture::repair_rejection_library;

fn assert_repair_rejected(
    library: &mut blueos_recorder_library::Library,
    spec: blueos_recorder_library::RepairStartSpec<'_>,
    expected_message: &str,
) {
    let Outcome::Rejected { reason } = start_repair(library, spec) else {
        panic!("repair must be rejected: {expected_message}");
    };
    assert_eq!(reason.to_string(), expected_message);
}

#[test]
fn delete_rejects_active_recording_file() {
    let mut library = scan_snapshot(&[("live.mcap", true)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("live.mcap").expect("path");
    let outcome = handle_library_request(
        &mut library,
        LibraryRequest::DeleteRecording {
            path,
            job_id: job_id(1),
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
    let (mut library, path) = repair_rejection_library("file.mcap", false, 1_000);
    assert!(matches!(
        start_repair(&mut library, repair_spec(path.clone(), job_id(1), None)),
        Outcome::Applied { .. }
    ));
    assert_repair_rejected(
        &mut library,
        repair_spec(path, job_id(3), None),
        "This recording is already being repaired.",
    );
}

#[test]
fn repair_rejects_indexed_file() {
    let (mut library, path) = repair_rejection_library("file.mcap", true, 1_000);
    assert_repair_rejected(
        &mut library,
        repair_spec(path, job_id(1), None),
        "This recording already has an index.",
    );
}

#[test]
fn repair_rejects_active_recording_file() {
    let (mut library, path) = repair_rejection_library("live.mcap", false, 1_000);
    assert_repair_rejected(
        &mut library,
        repair_spec(path, job_id(1), Some("live.mcap")),
        "This recording is still being written. Try again once it is finished.",
    );
}

#[test]
fn repair_rejects_recently_written_file() {
    let (mut library, path) = repair_rejection_library("recent.mcap", false, 19_995);
    assert_repair_rejected(
        &mut library,
        repair_spec(path, job_id(1), None),
        "This recording is still being written. Try again once it is finished.",
    );
}

#[test]
fn a_repair_is_listed_with_its_job_until_it_ends() {
    let mut library = scan_snapshot(&[("file.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
    assert!(matches!(
        start_repair(&mut library, repair_spec(path.clone(), job_id(7), None)),
        Outcome::Applied { .. }
    ));
    assert_eq!(
        library.work().queue().operations(),
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
    let repairing = &library.catalog().entries()[0];
    assert_eq!(repairing.repair_job_id, Some(job_id(7)));
    assert!(
        repairing
            .allowed_operations
            .iter()
            .any(|operation| operation == CANCEL_JOB)
    );

    handle_observed_fact(
        &mut library,
        LibraryObservedFact::RepairFinished {
            path,
            outcome: LibraryRepairOutcome::Cancelled,
        },
        None,
        NOW,
    );
    assert!(library.work().queue().operations().is_empty());
    let ended = &library.catalog().entries()[0];
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
        start_repair(&mut library, repair_spec(path.clone(), job_id(1), None)),
        Outcome::Applied { .. }
    ));
    handle_observed_fact(
        &mut library,
        LibraryObservedFact::RepairProgress(LibraryRepairProgress {
            path: path.clone(),
            bytes_processed: 40,
            total_bytes: 100,
        }),
        None,
        NOW,
    );
    let progress = library
        .work()
        .queue()
        .repair_progress(job_id(1))
        .expect("progress");
    assert_eq!((progress.bytes_processed, progress.total_bytes), (40, 100));
    assert!(library.work().queue().repair_progress(job_id(2)).is_none());

    handle_observed_fact(
        &mut library,
        LibraryObservedFact::RepairFinished {
            path: path.clone(),
            outcome: LibraryRepairOutcome::Cancelled,
        },
        None,
        NOW,
    );
    assert!(library.work().queue().repair_progress(job_id(1)).is_none());
    assert_eq!(
        library.work().queue().ended_operation(),
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
    assert_eq!(library.catalog().entries()[0].repair_error, "");
}
