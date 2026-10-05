use blueos_idl::{
    Message,
    message::CdrStruct,
    msg::{
        blueos_msgs::ServiceInfo,
        blueos_recorder_msgs::{RecordingFile, RecordingFileState, RecordingLibrary},
        builtin_interfaces::Time,
    },
};

use super::super::fixture::{CdrVector, encode_hex};

pub(crate) fn vectors() -> Vec<CdrVector> {
    vec![
        recording_library_old_writer_vector(),
        command_ack_old_writer_vector(),
        service_info_old_writer_vector(),
    ]
}

fn recording_library_old_writer_vector() -> CdrVector {
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
    let mut library_writer = blueos_idl::cdr::Writer::new();
    library_writer.write_u32(1).expect("files length");
    recording_file
        .cdr_encode_fields(&mut library_writer)
        .expect("recording file");
    let payload = library_writer.finish_with_encapsulation();
    CdrVector {
        schema_name: RecordingLibrary::SCHEMA_NAME.to_string(),
        hex: encode_hex(&payload),
        decoded: serde_json::json!({
            "files": [serde_json::to_value(&recording_file).expect("recording file json")],
            "contents": []
        }),
        category: "old_writer".to_string(),
        skip_encode_round_trip: true,
        layout_note: Some("a recorder from before contents was appended".to_string()),
    }
}

fn command_ack_old_writer_vector() -> CdrVector {
    let mut writer = blueos_idl::cdr::Writer::new();
    writer.write_bool(true).expect("bool");
    writer
        .write_string("0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10")
        .expect("job id");
    let payload = writer.finish_with_encapsulation();
    CdrVector {
        schema_name: blueos_idl::msg::blueos_msgs::CommandAck::SCHEMA_NAME.to_string(),
        hex: encode_hex(&payload),
        decoded: serde_json::json!({
            "accepted": true,
            "job_id": "0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10",
            "status": 0,
            "reason": ""
        }),
        category: "old_writer".to_string(),
        skip_encode_round_trip: true,
        layout_note: None,
    }
}

fn service_info_old_writer_vector() -> CdrVector {
    let mut service_info_writer = blueos_idl::cdr::Writer::new();
    service_info_writer.write_string("recorder").expect("name");
    service_info_writer.write_string("1.0.0").expect("version");
    service_info_writer.write_string("dev").expect("build");
    service_info_writer
        .write_u32(1)
        .expect("capabilities length");
    service_info_writer
        .write_string("record")
        .expect("capability");
    let payload = service_info_writer.finish_with_encapsulation();
    CdrVector {
        schema_name: ServiceInfo::SCHEMA_NAME.to_string(),
        hex: encode_hex(&payload),
        decoded: serde_json::json!({
            "name": "recorder",
            "version": "1.0.0",
            "build": "dev",
            "capabilities": ["record"],
            "endpoints": []
        }),
        category: "old_writer".to_string(),
        skip_encode_round_trip: true,
        layout_note: None,
    }
}
