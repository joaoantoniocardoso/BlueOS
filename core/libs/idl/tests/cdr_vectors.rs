//! Integration tests that decode committed CDR hex fixtures for every IDL message.

#![expect(
    clippy::too_many_lines,
    unreachable_pub,
    reason = "CDR vector integration tests"
)]
mod cdr_codec_dispatch {
    include!("generated/cdr_codec_dispatch.rs");
}

use std::fs;
use std::path::PathBuf;

use blueos_idl::Message;
use blueos_idl::msg::blueos_example_msgs::LevelRequest;
use blueos_idl::msg::blueos_msgs::{
    CommandAck, CommandAckStatus, EndpointInfo, MetricCounter, MetricGauge, MetricHistogram,
    MetricLabel, ServiceInfo, ServiceMetrics,
};
use blueos_idl::msg::blueos_recorder_msgs::{RecordingFile, RecordingFileState};
use blueos_idl::msg::builtin_interfaces::Time;
use blueos_idl::msg::foxglove_msgs::Log;
use blueos_idl_codegen::collect_messages_for_test;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CdrVector {
    schema_name: String,
    hex: String,
    decoded: serde_json::Value,
    category: String,
    #[serde(default, skip_serializing_if = "is_false")]
    skip_encode_round_trip: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    layout_note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CdrVectorsFile {
    vectors: Vec<CdrVector>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn encode_hex(payload: &[u8]) -> String {
    payload.iter().map(|byte| format!("{:02x}", byte)).collect()
}

fn decode_hex(hex: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let mut index = 0;
    while index < hex.len() {
        let pair = hex.get(index..index + 2).expect("hex pair");
        bytes.push(u8::from_str_radix(pair, 16).expect("hex digit"));
        index += 2;
    }
    bytes
}

fn interfaces_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("interfaces")
}

fn vectors_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors/cdr.json")
}

fn extra_vectors() -> Vec<CdrVector> {
    let log_message = Log {
        timestamp: Time { sec: 1, nanosec: 2 },
        level: 2,
        message: "hello".into(),
        name: "test".into(),
        file: "f.py".into(),
        line: 42,
    };
    let log_payload = log_message.encode().expect("encode Log");
    let command_ack_example = CommandAck {
        accepted: true,
        job_id: "0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10".into(),
        status: CommandAckStatus::WaitingForPermission,
        reason: "queued".into(),
    };
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
    let service_info = ServiceInfo {
        name: "recorder".into(),
        version: "1.0.0".into(),
        build: "dev".into(),
        capabilities: vec!["record".into()],
        endpoints: vec![
            EndpointInfo {
                kind: "job".into(),
                name: "Start".into(),
                key: "blueos/v1/recorder/command/Start".into(),
                interface_type: "blueos_recorder_msgs/action/StartRecording".into(),
                schema: "bool rotate_if_active\n---\n---".into(),
            },
            EndpointInfo {
                kind: "state".into(),
                name: "library".into(),
                key: "blueos/v1/recorder/state/library".into(),
                interface_type: "blueos_recorder_msgs/msg/RecordingLibrary".into(),
                schema: String::new(),
            },
        ],
    };

    let mut writer = blueos_idl::cdr::Writer::new();
    writer.write_bool(true).expect("bool");
    writer
        .write_string("0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10")
        .expect("job id");
    let command_ack_old_writer = writer.finish_with_encapsulation();

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
    let service_info_old_writer = service_info_writer.finish_with_encapsulation();

    let python_producer_log = Log {
        timestamp: Time { sec: 9, nanosec: 8 },
        level: 3,
        message: "python commonwealth log".into(),
        name: "wifi-manager".into(),
        file: "main.py".into(),
        line: 17,
    };

    let mut vectors = vec![
        CdrVector {
            schema_name: Log::SCHEMA_NAME.to_string(),
            hex: encode_hex(&log_payload),
            decoded: serde_json::to_value(log_message).expect("log json"),
            category: "example".to_string(),
            skip_encode_round_trip: false,
            layout_note: Some("string file followed by uint32 line".to_string()),
        },
        CdrVector {
            schema_name: Log::SCHEMA_NAME.to_string(),
            hex: encode_hex(
                &python_producer_log
                    .encode()
                    .expect("encode python producer Log"),
            ),
            decoded: serde_json::to_value(python_producer_log).expect("python producer log json"),
            category: "python_producer".to_string(),
            skip_encode_round_trip: true,
            layout_note: Some(
                "encoded by commonwealth blueos_idl (Python services log sink)".to_string(),
            ),
        },
        CdrVector {
            schema_name: CommandAck::SCHEMA_NAME.to_string(),
            hex: encode_hex(&command_ack_example.encode().expect("encode CommandAck")),
            decoded: serde_json::to_value(command_ack_example).expect("command ack json"),
            category: "example".to_string(),
            skip_encode_round_trip: false,
            layout_note: None,
        },
        CdrVector {
            schema_name: ServiceInfo::SCHEMA_NAME.to_string(),
            hex: encode_hex(&service_info.encode().expect("encode ServiceInfo")),
            decoded: serde_json::to_value(service_info).expect("service info json"),
            category: "example".to_string(),
            skip_encode_round_trip: false,
            layout_note: None,
        },
        CdrVector {
            schema_name: RecordingFile::SCHEMA_NAME.to_string(),
            hex: encode_hex(&recording_file.encode().expect("encode RecordingFile")),
            decoded: serde_json::to_value(recording_file).expect("recording file json"),
            category: "example".to_string(),
            skip_encode_round_trip: false,
            layout_note: None,
        },
        CdrVector {
            schema_name: CommandAck::SCHEMA_NAME.to_string(),
            hex: encode_hex(&command_ack_old_writer),
            decoded: serde_json::json!({
                "accepted": true,
                "job_id": "0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10",
                "status": 0,
                "reason": ""
            }),
            category: "old_writer".to_string(),
            skip_encode_round_trip: true,
            layout_note: None,
        },
        CdrVector {
            schema_name: ServiceInfo::SCHEMA_NAME.to_string(),
            hex: encode_hex(&service_info_old_writer),
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
        },
    ];

    let setting_field = blueos_idl::msg::blueos_msgs::SettingField {
        path: "network.mode".into(),
        restart_required: true,
    };
    vectors.push(CdrVector {
        schema_name: blueos_idl::msg::blueos_msgs::SettingField::SCHEMA_NAME.to_string(),
        hex: encode_hex(&setting_field.encode().expect("encode SettingField")),
        decoded: serde_json::to_value(setting_field).expect("setting field json"),
        category: "layout".to_string(),
        skip_encode_round_trip: false,
        layout_note: Some("string path followed by bool restart_required".to_string()),
    });

    let service_metrics = ServiceMetrics {
        counters: vec![MetricCounter {
            name: "task_restarts".into(),
            labels: vec![MetricLabel {
                name: "task".into(),
                value: "data_plane".into(),
            }],
            value: 3,
        }],
        gauges: vec![MetricGauge {
            name: "inbox_depth".into(),
            labels: Vec::new(),
            value: 2.0,
        }],
        histograms: vec![MetricHistogram {
            name: "inbox_step_seconds".into(),
            labels: Vec::new(),
            count: 4,
            sum: 0.0255,
            bucket_bounds: vec![0.001, 0.01],
            bucket_counts: vec![1, 2, 1],
        }],
    };
    vectors.push(CdrVector {
        schema_name: ServiceMetrics::SCHEMA_NAME.to_string(),
        hex: encode_hex(&service_metrics.encode().expect("encode ServiceMetrics")),
        decoded: serde_json::to_value(service_metrics).expect("service metrics json"),
        category: "example".to_string(),
        skip_encode_round_trip: false,
        layout_note: Some(
            "one counter with a label, one gauge, one histogram with two bounds and three bucket counts"
                .to_string(),
        ),
    });

    vectors.push(CdrVector {
        schema_name: LevelRequest::SCHEMA_NAME.to_string(),
        hex: "0001000000".to_string(),
        decoded: serde_json::json!({}),
        category: "empty_struct".to_string(),
        skip_encode_round_trip: false,
        layout_note: Some(
            "ROS 2 writes one byte for an empty struct (structure_needs_at_least_one_member)"
                .to_string(),
        ),
    });

    vectors
}

fn build_vectors_file() -> CdrVectorsFile {
    let mut vectors = Vec::new();
    for record in collect_messages_for_test(&interfaces_root()) {
        let schema_name = record.schema_name;
        let payload = cdr_codec_dispatch::encode_default(&schema_name).expect("encode default");
        let decoded =
            cdr_codec_dispatch::decode_to_json(&schema_name, &payload).expect("decode default");
        vectors.push(CdrVector {
            schema_name,
            hex: encode_hex(&payload),
            decoded,
            category: "default".to_string(),
            skip_encode_round_trip: false,
            layout_note: None,
        });
    }
    vectors.extend(extra_vectors());
    CdrVectorsFile { vectors }
}

fn read_vectors_file() -> CdrVectorsFile {
    let content = fs::read_to_string(vectors_path()).expect("read cdr.json");
    serde_json::from_str(&content).expect("parse cdr.json")
}

#[test]
fn cdr_vectors_match_rust_codec() {
    let updating = std::env::var("BLUEOS_IDL_UPDATE_VECTORS").as_deref() == Ok("1");
    let expected = build_vectors_file();

    if updating {
        fs::create_dir_all(vectors_path().parent().expect("parent")).expect("create vectors dir");
        let content = serde_json::to_string_pretty(&expected).expect("serialize vectors");
        fs::write(vectors_path(), format!("{content}\n")).expect("write cdr.json");
        return;
    }

    let on_disk = read_vectors_file();
    assert_eq!(
        serde_json::to_value(&on_disk).expect("serialize on disk"),
        serde_json::to_value(&expected).expect("serialize expected"),
        "cdr.json drift; run BLUEOS_IDL_UPDATE_VECTORS=1 cargo test -p blueos-idl cdr_vectors_match_rust_codec"
    );

    for vector in &on_disk.vectors {
        let payload = decode_hex(&vector.hex);
        let decoded =
            cdr_codec_dispatch::decode_to_json(&vector.schema_name, &payload).expect("rust decode");
        assert_eq!(
            decoded, vector.decoded,
            "decode mismatch for {}",
            vector.schema_name
        );
        if !vector.skip_encode_round_trip {
            let reencoded = cdr_codec_dispatch::encode_default(&vector.schema_name);
            if vector.category == "default" {
                let reencoded = reencoded.expect("encode default");
                assert_eq!(
                    encode_hex(&reencoded),
                    vector.hex,
                    "default re-encode mismatch for {}",
                    vector.schema_name
                );
            } else {
                let message = decoded;
                let reencoded_hex =
                    encode_json_message(&vector.schema_name, &message).expect("re-encode example");
                assert_eq!(
                    reencoded_hex, vector.hex,
                    "example re-encode mismatch for {}",
                    vector.schema_name
                );
            }
        }
    }
}

fn encode_json_message(schema_name: &str, message: &serde_json::Value) -> Option<String> {
    let encoded = match schema_name {
        LevelRequest::SCHEMA_NAME => {
            let message: LevelRequest =
                serde_json::from_value(message.clone()).expect("LevelRequest from json");
            message.encode().ok()
        }
        Log::SCHEMA_NAME => {
            let message: Log = serde_json::from_value(message.clone()).expect("Log from json");
            message.encode().ok()
        }
        CommandAck::SCHEMA_NAME => {
            let message: CommandAck =
                serde_json::from_value(message.clone()).expect("CommandAck from json");
            message.encode().ok()
        }
        ServiceInfo::SCHEMA_NAME => {
            let message: ServiceInfo =
                serde_json::from_value(message.clone()).expect("ServiceInfo from json");
            message.encode().ok()
        }
        ServiceMetrics::SCHEMA_NAME => {
            let message: ServiceMetrics =
                serde_json::from_value(message.clone()).expect("ServiceMetrics from json");
            message.encode().ok()
        }
        RecordingFile::SCHEMA_NAME => {
            let message: RecordingFile =
                serde_json::from_value(message.clone()).expect("RecordingFile from json");
            message.encode().ok()
        }
        blueos_idl::msg::blueos_msgs::SettingField::SCHEMA_NAME => {
            let message: blueos_idl::msg::blueos_msgs::SettingField =
                serde_json::from_value(message.clone()).expect("SettingField from json");
            message.encode().ok()
        }
        _ => None,
    };
    encoded.map(|payload| encode_hex(&payload))
}
