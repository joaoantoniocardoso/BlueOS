use blueos_idl::{
    Message,
    msg::{
        blueos_example_msgs::LevelRequest,
        blueos_msgs::{MetricCounter, MetricGauge, MetricHistogram, MetricLabel, ServiceMetrics},
    },
};

use super::super::fixture::{CdrVector, encode_hex};

pub(crate) fn vectors() -> Vec<CdrVector> {
    let setting_field = blueos_idl::msg::blueos_msgs::SettingField {
        path: "network.mode".into(),
        restart_required: true,
    };
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

    vec![
        CdrVector {
            schema_name: blueos_idl::msg::blueos_msgs::SettingField::SCHEMA_NAME.to_string(),
            hex: encode_hex(&setting_field.encode().expect("encode SettingField")),
            decoded: serde_json::to_value(setting_field).expect("setting field json"),
            category: "layout".to_string(),
            skip_encode_round_trip: false,
            layout_note: Some("string path followed by bool restart_required".to_string()),
        },
        CdrVector {
            schema_name: ServiceMetrics::SCHEMA_NAME.to_string(),
            hex: encode_hex(&service_metrics.encode().expect("encode ServiceMetrics")),
            decoded: serde_json::to_value(service_metrics).expect("service metrics json"),
            category: "example".to_string(),
            skip_encode_round_trip: false,
            layout_note: Some(
                "one counter with a label, one gauge, one histogram with two bounds and three bucket counts"
                    .to_string(),
            ),
        },
        CdrVector {
            schema_name: LevelRequest::SCHEMA_NAME.to_string(),
            hex: "0001000000".to_string(),
            decoded: serde_json::json!({}),
            category: "empty_struct".to_string(),
            skip_encode_round_trip: false,
            layout_note: Some(
                "ROS 2 writes one byte for an empty struct (structure_needs_at_least_one_member)"
                    .to_string(),
            ),
        },
    ]
}
