//! Conversions between the Recorder Domain and its public Messages.

#![no_std]

extern crate alloc;

pub mod endpoints;

use alloc::{string::String, vec::Vec};
use core::convert::Infallible;

use blueos_idl::{
    Message,
    msg::blueos_recorder_msgs::{
        DeleteRecordingFeedback, DeleteRecordingGoal, DeleteRecordingResult, RecordingFile,
        RecordingFileState as WireRecordingFileState, RecordingLibrary, RecordingState,
        RepairRecordingFeedback, RepairRecordingGoal, RepairRecordingResult,
        SnapshotRecordingFeedback, SnapshotRecordingGoal, SnapshotRecordingResult,
        StartRecordingFeedback, StartRecordingGoal, StartRecordingResult, StopRecordingFeedback,
        StopRecordingGoal, StopRecordingResult,
    },
};
use blueos_jobs::JobId;
use blueos_recorder_capture::RecordingState as DomainRecordingState;
use blueos_recorder_domain::{RecorderDomain, RecorderRequest, RecorderSnapshot};
use blueos_recorder_library::{LibraryOperation, RecordingFileState};
use blueos_recorder_paths::{RecordingPathError, RecordingRelativePath};

use crate::endpoints::Conversions;

impl Conversions for RecorderDomain {
    type DeleteRecordingError = RecordingPathError;
    type RepairRecordingError = RecordingPathError;
    type SnapshotRecordingError = RecordingPathError;
    type StartError = Infallible;
    type StopError = Infallible;

    fn delete_recording(
        job_id: JobId,
        goal: DeleteRecordingGoal,
    ) -> Result<RecorderRequest, RecordingPathError> {
        RecordingRelativePath::parse(&goal.path)
            .map(|path| RecorderRequest::DeleteRecording { job_id, path })
    }

    fn delete_recording_feedback(
        _snapshot: &RecorderSnapshot,
        _job_id: JobId,
    ) -> Option<DeleteRecordingFeedback> {
        None
    }

    fn delete_recording_result(
        snapshot: &RecorderSnapshot,
        job_id: JobId,
    ) -> DeleteRecordingResult {
        snapshot
            .jobs
            .job(job_id)
            .and_then(|job| DeleteRecordingGoal::decode(&job.goal).ok())
            .map(|goal| DeleteRecordingResult { path: goal.path })
            .unwrap_or_default()
    }

    fn repair_recording(
        job_id: JobId,
        goal: RepairRecordingGoal,
    ) -> Result<RecorderRequest, RecordingPathError> {
        RecordingRelativePath::parse(&goal.path)
            .map(|path| RecorderRequest::RepairRecording { job_id, path })
    }

    fn repair_recording_feedback(
        snapshot: &RecorderSnapshot,
        job_id: JobId,
    ) -> Option<RepairRecordingFeedback> {
        snapshot
            .library
            .repair_progress(job_id)
            .map(|progress| RepairRecordingFeedback {
                bytes_processed: progress.bytes_processed,
                total_bytes: progress.total_bytes,
            })
    }

    fn repair_recording_result(
        snapshot: &RecorderSnapshot,
        job_id: JobId,
    ) -> RepairRecordingResult {
        match snapshot.library.ended_operation() {
            Some(LibraryOperation::Repair {
                path,
                job_id: ended,
                ..
            }) if *ended == job_id => RepairRecordingResult {
                path: path.as_str().into(),
            },
            _ => RepairRecordingResult::default(),
        }
    }

    fn snapshot_recording(
        job_id: JobId,
        goal: SnapshotRecordingGoal,
    ) -> Result<RecorderRequest, RecordingPathError> {
        RecordingRelativePath::parse(&goal.path)
            .map(|path| RecorderRequest::SnapshotRecording { job_id, path })
    }

    fn snapshot_recording_feedback(
        snapshot: &RecorderSnapshot,
        job_id: JobId,
    ) -> Option<SnapshotRecordingFeedback> {
        snapshot
            .library
            .operations()
            .iter()
            .find_map(|operation| match operation {
                LibraryOperation::Snapshot {
                    output_path,
                    job_id: running,
                    ..
                } if *running == job_id => Some(SnapshotRecordingFeedback {
                    output_path: output_path.clone(),
                }),
                _ => None,
            })
    }

    fn snapshot_recording_result(
        snapshot: &RecorderSnapshot,
        job_id: JobId,
    ) -> SnapshotRecordingResult {
        match snapshot.library.ended_operation() {
            Some(LibraryOperation::Snapshot {
                path,
                output_path,
                job_id: ended,
            }) if *ended == job_id => SnapshotRecordingResult {
                path: path.as_str().into(),
                output_path: output_path.clone(),
            },
            _ => SnapshotRecordingResult::default(),
        }
    }

    fn start(goal: StartRecordingGoal) -> Result<RecorderRequest, Infallible> {
        Ok(RecorderRequest::StartRecording {
            rotate_if_active: goal.rotate_if_active,
        })
    }

    fn start_feedback(
        _snapshot: &RecorderSnapshot,
        _job_id: JobId,
    ) -> Option<StartRecordingFeedback> {
        None
    }

    fn start_result(_snapshot: &RecorderSnapshot, _job_id: JobId) -> StartRecordingResult {
        StartRecordingResult::default()
    }

    fn stop(_goal: StopRecordingGoal) -> Result<RecorderRequest, Infallible> {
        Ok(RecorderRequest::StopRecording)
    }

    fn stop_feedback(
        _snapshot: &RecorderSnapshot,
        _job_id: JobId,
    ) -> Option<StopRecordingFeedback> {
        None
    }

    fn stop_result(_snapshot: &RecorderSnapshot, _job_id: JobId) -> StopRecordingResult {
        StopRecordingResult::default()
    }

    fn recording(snapshot: &RecorderSnapshot) -> RecordingState {
        recorder_session_state(snapshot)
    }

    fn library(snapshot: &RecorderSnapshot) -> RecordingLibrary {
        recording_library(snapshot)
    }
}

/// Projects the published `recording` State from the Snapshot (wire names mapped here).
///
/// Wire field names use legacy "session" wording (see GLOSSARY).
pub fn recorder_session_state(snapshot: &RecorderSnapshot) -> RecordingState {
    let gate = snapshot.capture.record_gate();
    let (session_active, current_file, session_bytes_written) = match &snapshot.capture.recording {
        DomainRecordingState::Idle => (false, String::new(), 0),
        DomainRecordingState::AwaitingMcapFile { .. } => (true, String::new(), 0),
        DomainRecordingState::Active(active) => {
            (true, active.file_name.clone(), active.bytes_written)
        }
    };
    RecordingState {
        armed: snapshot.capture.armed,
        session_active,
        current_file,
        session_bytes_written,
        recording_video_topics: gate
            .recording_video_topics
            .iter()
            .cloned()
            .collect::<Vec<_>>(),
    }
}

/// Projects the published `library` State from the Snapshot.
pub fn recording_library(snapshot: &RecorderSnapshot) -> RecordingLibrary {
    RecordingLibrary {
        files: snapshot
            .library
            .entries()
            .iter()
            .map(recording_file_message)
            .collect(),
    }
}

fn recording_file_message(entry: &blueos_recorder_library::RecordingFileEntry) -> RecordingFile {
    let (sec, nanosec) = unix_seconds_to_time(entry.created_unix_seconds);
    RecordingFile {
        path: entry.path.clone(),
        name: entry.name.clone(),
        size_bytes: entry.size_bytes,
        created: blueos_idl::msg::builtin_interfaces::Time { sec, nanosec },
        state: recording_file_state_wire(entry.state),
        repair_bytes_processed: entry.repair_bytes_processed,
        repair_total_bytes: entry.repair_total_bytes,
        repair_bytes_per_second: entry.repair_bytes_per_second,
        repair_error: entry.repair_error.clone(),
        allowed_operations: entry.allowed_operations.clone(),
        repair_job_id: entry.repair_job_id.map(String::from).unwrap_or_default(),
    }
}

fn recording_file_state_wire(state: RecordingFileState) -> WireRecordingFileState {
    match state {
        RecordingFileState::Recording => WireRecordingFileState::Recording,
        RecordingFileState::Ready => WireRecordingFileState::Ready,
        RecordingFileState::NeedsRepair => WireRecordingFileState::NeedsRepair,
        RecordingFileState::Repairing => WireRecordingFileState::Repairing,
    }
}

fn unix_seconds_to_time(seconds: i64) -> (i32, u32) {
    (i32::try_from(seconds).unwrap_or(i32::MAX), 0)
}
