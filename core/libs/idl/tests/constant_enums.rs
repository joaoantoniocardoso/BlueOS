//! Constant-family enum round-trip (D-05).

use blueos_idl::{
    Message,
    msg::{
        blueos_msgs::{JobStatus, JobStatusStatus},
        blueos_recorder_msgs::{
            RecordingFileState, RecordingOperation, RecordingOperationOperation,
        },
    },
};

#[test]
fn unknown_constant_decodes_and_reencodes_unchanged() {
    let raw = 99u8;
    let message = JobStatus {
        job_id: 1,
        parent_job_id: 0,
        status: JobStatusStatus::from_raw(raw),
        name: String::new(),
    };
    let payload = message.encode().expect("encode");
    let decoded = JobStatus::decode(&payload).expect("decode");
    assert_eq!(decoded.status, JobStatusStatus::Unknown(raw));
    assert_eq!(decoded.encode().expect("re-encode"), payload);
}

#[test]
fn known_status_variant_round_trips() {
    let message = JobStatus {
        job_id: 0,
        parent_job_id: 0,
        status: JobStatusStatus::Queued,
        name: String::new(),
    };
    let payload = message.encode().expect("encode");
    let decoded = JobStatus::decode(&payload).expect("decode");
    assert_eq!(decoded.status, JobStatusStatus::Queued);
    assert_eq!(decoded.encode().expect("re-encode"), payload);
}

#[test]
fn recorder_operation_and_state_enums_encode_scalars() {
    let operation = RecordingOperation {
        operation: RecordingOperationOperation::Repair,
        path: String::new(),
        output_path: String::new(),
        succeeded: false,
        cancelled: false,
        error: String::new(),
    };
    let payload = operation.encode().expect("encode operation");
    assert_eq!(payload.last(), Some(&0));

    let state_message = blueos_idl::msg::blueos_recorder_msgs::RecordingFile {
        path: String::new(),
        name: String::new(),
        size_bytes: 0,
        created: blueos_idl::msg::builtin_interfaces::Time::default(),
        state: RecordingFileState::Ready,
        repair_bytes_processed: 0,
        repair_total_bytes: 0,
        repair_bytes_per_second: 0.0,
        repair_error: String::new(),
        allowed_operations: Vec::new(),
    };
    let state_payload = state_message.encode().expect("encode state");
    let decoded = blueos_idl::msg::blueos_recorder_msgs::RecordingFile::decode(&state_payload)
        .expect("decode state");
    assert_eq!(decoded.state, RecordingFileState::Ready);
}
