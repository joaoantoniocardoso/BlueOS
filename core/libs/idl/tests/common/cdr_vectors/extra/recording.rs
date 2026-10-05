use blueos_idl::{
    Message,
    msg::{
        blueos_recorder_msgs::{
            RecordingContents, RecordingFile, RecordingFileState, RecordingLibrary,
        },
        builtin_interfaces::{Duration, Time},
    },
};

use super::super::fixture::{CdrVector, encode_hex};

pub(crate) fn vectors() -> Vec<CdrVector> {
    let recording_file = RecordingFile {
        path: "session/recorder_20260925.mcap".into(),
        name: "recorder_20260925.mcap".into(),
        size_bytes: 187_315,
        created: Time {
            sec: 1_790_306_134,
            nanosec: 0,
        },
        state: RecordingFileState::Ready,
        repair_bytes_processed: 0,
        repair_total_bytes: 0,
        repair_bytes_per_second: 0.0,
        repair_error: String::new(),
        allowed_operations: Vec::new(),
        repair_job_id: String::new(),
    };
    let recording_library = RecordingLibrary {
        files: vec![recording_file.clone()],
        contents: vec![RecordingContents {
            path: recording_file.path.clone(),
            duration: Duration {
                sec: 2,
                nanosec: 500_000_000,
            },
            video_topics: vec!["video/camera/stream".into()],
            other_topic_count: 3,
        }],
    };

    vec![
        CdrVector {
            schema_name: RecordingFile::SCHEMA_NAME.to_string(),
            hex: encode_hex(&recording_file.encode().expect("encode RecordingFile")),
            decoded: serde_json::to_value(recording_file).expect("recording file json"),
            category: "example".to_string(),
            skip_encode_round_trip: false,
            layout_note: None,
        },
        CdrVector {
            schema_name: RecordingLibrary::SCHEMA_NAME.to_string(),
            hex: encode_hex(&recording_library.encode().expect("encode RecordingLibrary")),
            decoded: serde_json::to_value(recording_library).expect("recording library json"),
            category: "example".to_string(),
            skip_encode_round_trip: false,
            layout_note: None,
        },
    ]
}
