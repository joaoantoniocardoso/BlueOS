//! Repair rejection for files that are not MCAP.

#[path = "support/library_common.rs"]
mod common;
#[path = "support/library_file_fixture.rs"]
mod file_fixture;

use blueos_domain::Outcome;
use blueos_recorder_library::{
    Library, LibraryIoResult, LibraryObservedFact, LibraryRepairOutcome, REPAIR_RECORDING,
    RepairFailure, ScannedRecording, handle_io_result, handle_observed_fact, start_repair,
};

use common::{NOW, job_id, repair_spec};
use file_fixture::default_file_library;

#[test]
fn a_file_that_is_not_an_mcap_is_not_offered_repair_again_until_it_changes() {
    let (mut library, path) = default_file_library();
    start_repair(&mut library, repair_spec(path.clone(), job_id(1), None));
    handle_observed_fact(
        &mut library,
        LibraryObservedFact::RepairFinished {
            path: path.clone(),
            outcome: LibraryRepairOutcome::Failed(RepairFailure::NotMcap),
        },
        None,
        NOW,
    );
    let offers_repair = |snapshot: &Library| {
        snapshot.catalog().entries()[0]
            .allowed_operations
            .iter()
            .any(|operation| operation == REPAIR_RECORDING)
    };
    assert_eq!(
        library.catalog().entries()[0].state,
        blueos_recorder_library::RecordingFileState::NeedsRepair
    );
    assert_eq!(
        library.catalog().entries()[0].repair_error,
        "This is not an MCAP file."
    );
    assert!(!offers_repair(&library));
    let Outcome::Rejected { reason } =
        start_repair(&mut library, repair_spec(path.clone(), job_id(2), None))
    else {
        panic!("a repair that cannot succeed must be rejected");
    };
    assert_eq!(reason.to_string(), "This recording is not an MCAP file.");

    rescan_recording(&mut library, 100, 1_000);
    assert!(
        !offers_repair(&library),
        "a rescan of the same file keeps it"
    );
    rescan_recording(&mut library, 150, 1_000);
    assert!(offers_repair(&library), "a new size offers repair again");

    handle_observed_fact(
        &mut library,
        LibraryObservedFact::RepairFinished {
            path,
            outcome: LibraryRepairOutcome::Failed(RepairFailure::NotMcap),
        },
        None,
        NOW,
    );
    assert!(!offers_repair(&library));
    rescan_recording(&mut library, 150, 1_500);
    assert!(
        offers_repair(&library),
        "a new modification time offers repair again"
    );
}

fn rescan_recording(library: &mut Library, size_bytes: u64, modified_unix_seconds: i64) {
    handle_io_result(
        library,
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
}
