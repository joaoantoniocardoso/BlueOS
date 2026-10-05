//! Snapshot-to-wire projections for published recorder State.

use alloc::{string::String, vec::Vec};

use blueos_idl::{
    msg::blueos_recorder_msgs::{
        RecordingContents, RecordingFile, RecordingFileState as WireRecordingFileState,
        RecordingLibrary, RecordingState,
    },
    msg::builtin_interfaces::Duration,
};
use blueos_recorder_capture::{RecordGate, RecordingState as DomainRecordingState};
use blueos_recorder_domain::RecorderSnapshot;
use blueos_recorder_library::RecordingFileEntry;

/// Projects the published `recording` State from the Snapshot (wire names mapped here).
///
/// Wire field names use legacy "session" wording (see GLOSSARY).
struct SessionFields {
    session_active: bool,
    current_file: String,
    session_bytes_written: u64,
    samples_dropped: u64,
}

/// Projects the published `recording` State from the Snapshot.
pub fn recorder_session_state(snapshot: &RecorderSnapshot) -> RecordingState {
    let session = session_fields_from_recording(&snapshot.blocks.capture.recording);
    let gate = snapshot.blocks.capture.record_gate();
    recording_state_from_parts(snapshot.blocks.capture.armed, session, &gate)
}

fn recording_state_from_parts(
    armed: bool,
    session: SessionFields,
    gate: &RecordGate,
) -> RecordingState {
    RecordingState {
        armed,
        session_active: session.session_active,
        current_file: session.current_file,
        session_bytes_written: session.session_bytes_written,
        recording_video_topics: recording_video_topics_from_gate(gate),
        samples_dropped: session.samples_dropped,
    }
}

/// Projects the published `library` State from the Snapshot.
pub fn recording_library(snapshot: &RecorderSnapshot) -> RecordingLibrary {
    let entries = snapshot.blocks.library.catalog().entries();
    RecordingLibrary {
        files: entries
            .iter()
            .cloned()
            .map(recording_file_message)
            .collect(),
        contents: entries
            .iter()
            .filter_map(|entry| recording_contents_message(snapshot, entry))
            .collect(),
    }
}

fn session_fields_from_recording(recording: &DomainRecordingState) -> SessionFields {
    match recording {
        DomainRecordingState::Idle => inactive_session_fields(),
        DomainRecordingState::AwaitingMcapFile { .. } => awaiting_session_fields(),
        DomainRecordingState::Active(active) => active_session_fields(active),
    }
}

fn inactive_session_fields() -> SessionFields {
    SessionFields {
        session_active: false,
        current_file: String::new(),
        session_bytes_written: 0,
        samples_dropped: 0,
    }
}

fn awaiting_session_fields() -> SessionFields {
    SessionFields {
        session_active: true,
        current_file: String::new(),
        session_bytes_written: 0,
        samples_dropped: 0,
    }
}

fn active_session_fields(active: &blueos_recorder_capture::ActiveRecording) -> SessionFields {
    SessionFields {
        session_active: true,
        current_file: active.file_name.clone(),
        session_bytes_written: active.bytes_written,
        samples_dropped: active.samples_dropped,
    }
}

fn recording_video_topics_from_gate(gate: &RecordGate) -> Vec<String> {
    gate.recording_video_topics.iter().cloned().collect()
}

fn recording_contents_message(
    snapshot: &RecorderSnapshot,
    entry: &RecordingFileEntry,
) -> Option<RecordingContents> {
    let contents = snapshot.blocks.library.catalog().contents(&entry.path)?;
    Some(RecordingContents {
        path: entry.path.clone(),
        duration: Duration {
            sec: i32::try_from(contents.duration.as_secs()).unwrap_or(i32::MAX),
            nanosec: contents.duration.subsec_nanos(),
        },
        video_topics: contents.video_topics.clone(),
        other_topic_count: contents.other_topic_count,
    })
}

fn recording_file_message(entry: RecordingFileEntry) -> RecordingFile {
    let (sec, nanosec) = unix_seconds_to_time(entry.created_unix_seconds);
    RecordingFile {
        path: entry.path,
        name: entry.name,
        size_bytes: entry.size_bytes,
        created: blueos_idl::msg::builtin_interfaces::Time { sec, nanosec },
        state: wire_recording_file_state(entry.state),
        repair_bytes_processed: entry.repair_bytes_processed,
        repair_total_bytes: entry.repair_total_bytes,
        repair_bytes_per_second: entry.repair_bytes_per_second,
        repair_error: entry.repair_error,
        allowed_operations: entry.allowed_operations,
        repair_job_id: entry.repair_job_id.map(String::from).unwrap_or_default(),
    }
}

fn wire_recording_file_state(
    state: blueos_recorder_library::RecordingFileState,
) -> WireRecordingFileState {
    match state {
        blueos_recorder_library::RecordingFileState::Recording => WireRecordingFileState::Recording,
        blueos_recorder_library::RecordingFileState::Ready => WireRecordingFileState::Ready,
        blueos_recorder_library::RecordingFileState::NeedsRepair => {
            WireRecordingFileState::NeedsRepair
        }
        blueos_recorder_library::RecordingFileState::Repairing => WireRecordingFileState::Repairing,
    }
}

fn unix_seconds_to_time(seconds: i64) -> (i32, u32) {
    (i32::try_from(seconds).unwrap_or(i32::MAX), 0)
}
