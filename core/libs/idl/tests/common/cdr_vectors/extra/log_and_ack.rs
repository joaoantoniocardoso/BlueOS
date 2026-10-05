use blueos_idl::{
    Message,
    msg::{
        blueos_msgs::{CommandAck, CommandAckStatus, EndpointInfo, ServiceInfo},
        builtin_interfaces::Time,
        foxglove_msgs::Log,
    },
};

use super::super::fixture::{CdrVector, encode_hex};

struct LogVectorFields<'a> {
    sec: i32,
    nanosec: u32,
    level: u8,
    message: &'a str,
    name: &'a str,
    file: &'a str,
    line: u32,
}

fn log_from_fields(fields: LogVectorFields<'_>) -> Log {
    Log {
        timestamp: Time {
            sec: fields.sec,
            nanosec: fields.nanosec,
        },
        level: fields.level,
        message: fields.message.into(),
        name: fields.name.into(),
        file: fields.file.into(),
        line: fields.line,
    }
}

pub(crate) fn vectors() -> Vec<CdrVector> {
    vec![
        log_example_vector(),
        log_python_producer_vector(),
        command_ack_example_vector(),
        service_info_example_vector(),
    ]
}

fn log_vector(
    log_message: Log,
    category: &str,
    skip_encode_round_trip: bool,
    layout_note: Option<String>,
) -> CdrVector {
    let payload = log_message.encode().expect("encode Log");
    CdrVector {
        schema_name: Log::SCHEMA_NAME.to_string(),
        hex: encode_hex(&payload),
        decoded: serde_json::to_value(&log_message).expect("log json"),
        category: category.to_string(),
        skip_encode_round_trip,
        layout_note,
    }
}

fn log_example_vector() -> CdrVector {
    log_vector(
        log_from_fields(LogVectorFields {
            sec: 1,
            nanosec: 2,
            level: 2,
            message: "hello",
            name: "test",
            file: "f.py",
            line: 42,
        }),
        "example",
        false,
        Some("string file followed by uint32 line".to_string()),
    )
}

fn log_python_producer_vector() -> CdrVector {
    log_vector(
        log_from_fields(LogVectorFields {
            sec: 9,
            nanosec: 8,
            level: 3,
            message: "python commonwealth log",
            name: "wifi-manager",
            file: "main.py",
            line: 17,
        }),
        "python_producer",
        true,
        Some("encoded by commonwealth blueos_idl (Python services log sink)".to_string()),
    )
}

fn command_ack_example_vector() -> CdrVector {
    let command_ack_example = CommandAck {
        accepted: true,
        job_id: "0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10".into(),
        status: CommandAckStatus::WaitingForPermission,
        reason: "queued".into(),
    };
    CdrVector {
        schema_name: CommandAck::SCHEMA_NAME.to_string(),
        hex: encode_hex(&command_ack_example.encode().expect("encode CommandAck")),
        decoded: serde_json::to_value(command_ack_example).expect("command ack json"),
        category: "example".to_string(),
        skip_encode_round_trip: false,
        layout_note: None,
    }
}

fn endpoint_info(
    kind: &str,
    name: &str,
    key: &str,
    interface_type: &str,
    schema: &str,
) -> EndpointInfo {
    EndpointInfo {
        kind: kind.into(),
        name: name.into(),
        key: key.into(),
        interface_type: interface_type.into(),
        schema: schema.into(),
    }
}

fn service_info_example_vector() -> CdrVector {
    let service_info = ServiceInfo {
        name: "recorder".into(),
        version: "1.0.0".into(),
        build: "dev".into(),
        capabilities: vec!["record".into()],
        endpoints: vec![
            endpoint_info(
                "job",
                "Start",
                "blueos/v1/recorder/command/Start",
                "blueos_recorder_msgs/action/StartRecording",
                "bool rotate_if_active\n---\n---",
            ),
            endpoint_info(
                "state",
                "library",
                "blueos/v1/recorder/state/library",
                "blueos_recorder_msgs/msg/RecordingLibrary",
                "",
            ),
        ],
    };
    CdrVector {
        schema_name: ServiceInfo::SCHEMA_NAME.to_string(),
        hex: encode_hex(&service_info.encode().expect("encode ServiceInfo")),
        decoded: serde_json::to_value(service_info).expect("service info json"),
        category: "example".to_string(),
        skip_encode_round_trip: false,
        layout_note: None,
    }
}
