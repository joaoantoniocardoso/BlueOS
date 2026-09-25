use alloc::string::String;
use alloc::vec;

use blueos_cqrs::{Domain, Effect};
use blueos_jobs::Jobs;

use super::{
    LibraryCommand, LibraryDomain, LibraryIo, LibrarySnapshot, OperationKind, RecordingState,
    ScannedRecording, derive_recording_state, handle_library_command, snapshot_output_path,
    timestamp::created_unix_seconds_from_filename, validate_relative_recording_path,
};

fn scan_snapshot(paths: &[(&str, bool)]) -> LibrarySnapshot {
    let mut snapshot = LibrarySnapshot::default();
    let recordings = paths
        .iter()
        .map(|(path, indexed)| {
            let name = path.rsplit('/').next().unwrap_or(path);
            ScannedRecording {
                relative_path: (*path).into(),
                name: name.into(),
                size_bytes: 100,
                modified_unix_seconds: 1_000,
                indexed: *indexed,
            }
        })
        .collect();
    handle_library_command(
        &mut snapshot,
        LibraryCommand::ScanCompleted {
            recordings,
            now_unix_seconds: 2_000,
        },
        None,
    );
    snapshot
}

#[test]
fn recording_state_priority() {
    assert_eq!(
        derive_recording_state("live.mcap", Some("live.mcap"), false, true),
        RecordingState::Recording
    );
    assert_eq!(
        derive_recording_state("live.mcap", None, false, true),
        RecordingState::Ready
    );
    assert_eq!(
        derive_recording_state("live.mcap", None, false, false),
        RecordingState::NeedsRepair
    );
    assert_eq!(
        derive_recording_state("repairing.mcap", None, true, true),
        RecordingState::Repairing
    );
}

#[test]
fn created_from_filename_prefers_embedded_timestamp() {
    let created = created_unix_seconds_from_filename("recorder_20240102_030405.mcap", 1_000);
    assert_eq!(created, 1_704_164_645);
    assert_ne!(created, 1_000);
}

#[test]
fn created_from_filename_reads_copy_snapshot_and_legacy_split_names() {
    let expected = created_unix_seconds_from_filename("recorder_20240102_040506.mcap", 0);
    let copy = created_unix_seconds_from_filename(
        "recorder_20240102_030405.copy-2024-01-02T04-05-06Z.mcap",
        0,
    );
    let snapshot = created_unix_seconds_from_filename(
        "recorder_20240102_030405.snapshot-2024-01-02T04-05-06Z.mcap",
        0,
    );
    let legacy = created_unix_seconds_from_filename(
        "recorder_20240102_030405_split_20240102_040506.mcap",
        0,
    );
    assert_eq!(copy, expected);
    assert_eq!(snapshot, expected);
    assert_eq!(legacy, expected);
}

#[test]
fn cancelled_repair_is_not_reported_as_a_failure() {
    let mut snapshot = scan_snapshot(&[("cancelled.mcap", false)]);
    handle_library_command(
        &mut snapshot,
        LibraryCommand::RepairRecording {
            path: "cancelled.mcap".into(),
            now_unix_seconds: 20_000,
        },
        None,
    );
    let decision = handle_library_command(
        &mut snapshot,
        LibraryCommand::OperationFinished {
            operation: OperationKind::Repair,
            path: "cancelled.mcap".into(),
            output_path: String::new(),
            succeeded: false,
            cancelled: true,
            error: "mcap recover exited with -15".into(),
        },
        None,
    );
    assert_eq!(decision.events.len(), 1);
    let event = &decision.events[0];
    assert!(event.operation_event().cancelled);
    assert!(event.operation_event().error.is_empty());
    assert!(
        snapshot
            .entries
            .iter()
            .find(|entry| entry.path == "cancelled.mcap")
            .is_none_or(|entry| entry.repair_error.is_empty())
    );
}

#[test]
fn failed_repair_keeps_error_message() {
    let mut snapshot = scan_snapshot(&[("broken.mcap", false)]);
    handle_library_command(
        &mut snapshot,
        LibraryCommand::RepairRecording {
            path: "broken.mcap".into(),
            now_unix_seconds: 20_000,
        },
        None,
    );
    handle_library_command(
        &mut snapshot,
        LibraryCommand::OperationFinished {
            operation: OperationKind::Repair,
            path: "broken.mcap".into(),
            output_path: String::new(),
            succeeded: false,
            cancelled: false,
            error: "mcap recover exited with -15".into(),
        },
        None,
    );
    let entry = snapshot
        .entries
        .iter()
        .find(|entry| entry.path == "broken.mcap")
        .expect("entry");
    assert_eq!(entry.repair_error, "mcap recover exited with -15");
}

#[test]
fn resolve_recording_rejects_path_traversal() {
    assert_eq!(
        validate_relative_recording_path("../outside.mcap"),
        Err("Invalid recording path.".into())
    );
    assert_eq!(
        validate_relative_recording_path("outside.txt"),
        Err("Only .mcap recordings are supported.".into())
    );
}

#[test]
fn repair_rejects_when_already_repairing() {
    let mut snapshot = scan_snapshot(&[("file.mcap", false)]);
    let first = handle_library_command(
        &mut snapshot,
        LibraryCommand::RepairRecording {
            path: "file.mcap".into(),
            now_unix_seconds: 20_000,
        },
        None,
    );
    assert!(first.rejection.is_none());
    let second = handle_library_command(
        &mut snapshot,
        LibraryCommand::RepairRecording {
            path: "file.mcap".into(),
            now_unix_seconds: 20_001,
        },
        None,
    );
    assert_eq!(
        second.rejection.as_deref(),
        Some("This recording is already being repaired.")
    );
}

#[test]
fn repair_rejects_indexed_file() {
    let mut snapshot = scan_snapshot(&[("file.mcap", true)]);
    let decision = handle_library_command(
        &mut snapshot,
        LibraryCommand::RepairRecording {
            path: "file.mcap".into(),
            now_unix_seconds: 20_000,
        },
        None,
    );
    assert_eq!(
        decision.rejection.as_deref(),
        Some("This recording already has an index.")
    );
}

#[test]
fn repair_rejects_active_session_file() {
    let mut snapshot = scan_snapshot(&[("live.mcap", false)]);
    let decision = handle_library_command(
        &mut snapshot,
        LibraryCommand::RepairRecording {
            path: "live.mcap".into(),
            now_unix_seconds: 20_000,
        },
        Some("live.mcap"),
    );
    assert_eq!(
        decision.rejection.as_deref(),
        Some("This recording is still being written. Try again once it is finished.")
    );
}

#[test]
fn repair_rejects_recently_written_file() {
    let mut snapshot = LibrarySnapshot::default();
    handle_library_command(
        &mut snapshot,
        LibraryCommand::ScanCompleted {
            recordings: vec![ScannedRecording {
                relative_path: "recent.mcap".into(),
                name: "recent.mcap".into(),
                size_bytes: 10,
                modified_unix_seconds: 19_995,
                indexed: false,
            }],
            now_unix_seconds: 20_000,
        },
        None,
    );
    let decision = handle_library_command(
        &mut snapshot,
        LibraryCommand::RepairRecording {
            path: "recent.mcap".into(),
            now_unix_seconds: 20_000,
        },
        None,
    );
    assert_eq!(
        decision.rejection.as_deref(),
        Some("This recording is still being written. Try again once it is finished.")
    );
}

#[test]
fn cancel_repair_rejects_when_not_repairing() {
    let mut snapshot = scan_snapshot(&[("file.mcap", false)]);
    let decision = handle_library_command(
        &mut snapshot,
        LibraryCommand::CancelRepair {
            path: "file.mcap".into(),
        },
        None,
    );
    assert_eq!(
        decision.rejection.as_deref(),
        Some("This recording is not being repaired.")
    );
}

#[test]
fn delete_rejects_active_session_file() {
    let mut snapshot = scan_snapshot(&[("live.mcap", true)]);
    let decision = handle_library_command(
        &mut snapshot,
        LibraryCommand::DeleteRecording {
            path: "live.mcap".into(),
        },
        Some("live.mcap"),
    );
    assert_eq!(
        decision.rejection.as_deref(),
        Some("This recording is still being written.")
    );
}

#[test]
fn snapshot_rejects_missing_file() {
    let mut snapshot = LibrarySnapshot::default();
    let decision = handle_library_command(
        &mut snapshot,
        LibraryCommand::SnapshotRecording {
            path: "missing.mcap".into(),
            now_unix_seconds: 20_000,
        },
        None,
    );
    assert_eq!(decision.rejection.as_deref(), Some("Recording not found."));
}

#[test]
fn snapshot_output_name_uses_utc_timestamp() {
    let path = snapshot_output_path("recorder_20240102_030405.mcap", 1_704_164_645);
    assert_eq!(
        path,
        "recorder_20240102_030405.snapshot-2024-01-02T03-04-05Z.mcap"
    );
}

#[test]
fn operation_finished_schedules_rescan() {
    let mut snapshot = scan_snapshot(&[("file.mcap", false)]);
    handle_library_command(
        &mut snapshot,
        LibraryCommand::RepairRecording {
            path: "file.mcap".into(),
            now_unix_seconds: 20_000,
        },
        None,
    );
    let decision = handle_library_command(
        &mut snapshot,
        LibraryCommand::OperationFinished {
            operation: OperationKind::Repair,
            path: "file.mcap".into(),
            output_path: String::new(),
            succeeded: true,
            cancelled: false,
            error: String::new(),
        },
        None,
    );
    assert!(
        decision
            .effects
            .iter()
            .any(|effect| matches!(effect, Effect::Io(LibraryIo::Scan)))
    );
}

trait OperationEventAccess {
    fn operation_event(&self) -> &super::RecordingOperationEvent;
}

impl OperationEventAccess for super::LibraryEvent {
    fn operation_event(&self) -> &super::RecordingOperationEvent {
        match self {
            super::LibraryEvent::Operation(event) => event,
        }
    }
}

#[test]
fn delete_rejects_while_already_deleting() {
    let mut snapshot = scan_snapshot(&[("file.mcap", true)]);
    let first = handle_library_command(
        &mut snapshot,
        LibraryCommand::DeleteRecording {
            path: "file.mcap".into(),
        },
        None,
    );
    assert!(first.rejection.is_none());
    let second = handle_library_command(
        &mut snapshot,
        LibraryCommand::DeleteRecording {
            path: "file.mcap".into(),
        },
        None,
    );
    assert_eq!(
        second.rejection.as_deref(),
        Some("This recording is being processed.")
    );
}

#[test]
fn failed_operation_clears_in_flight_maps() {
    let mut snapshot = scan_snapshot(&[("file.mcap", true)]);
    handle_library_command(
        &mut snapshot,
        LibraryCommand::DeleteRecording {
            path: "file.mcap".into(),
        },
        None,
    );
    handle_library_command(
        &mut snapshot,
        LibraryCommand::OperationFinished {
            operation: OperationKind::Delete,
            path: "file.mcap".into(),
            output_path: String::new(),
            succeeded: false,
            cancelled: false,
            error: "task join failed".into(),
        },
        None,
    );
    let retry = handle_library_command(
        &mut snapshot,
        LibraryCommand::DeleteRecording {
            path: "file.mcap".into(),
        },
        None,
    );
    assert!(retry.rejection.is_none());
}

#[test]
fn library_domain_delegates_to_handler() {
    let mut snapshot = LibrarySnapshot::default();
    let decision =
        LibraryDomain::handle_command(&mut snapshot, &mut Jobs::new(), LibraryCommand::RescanTick);
    assert!(decision.rejection.is_none());
    assert!(decision.effects.len() >= 2);
}
