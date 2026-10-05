//! Integration tests for ROS 2 and Zenoh name parsing against committed JSON vectors.

use serde::Deserialize;

use blueos_ros2_names::{
    RmwZenohDataKey, Ros2EntityKind, Ros2LivelinessInfo, Ros2Transport, dds_type_name_to_ros,
    parse_rmw_zenoh_data_key, parse_rmw_zenoh_liveliness_token, parse_ros2dds_liveliness_token,
    ros2dds_data_key_to_topic, ros2dds_liveliness_token_to_data_key,
};

#[derive(Deserialize)]
struct Vectors {
    rmw_zenoh_data_keys: Vec<RmwDataVector>,
    rmw_zenoh_tokens: Vec<RmwTokenVector>,
    ros2dds_tokens: Vec<Ros2ddsTokenVector>,
    dds_type_names: Vec<DdsTypeVector>,
    ros2dds_data_key_topics: Vec<DataKeyTopicVector>,
    ros2dds_token_data_keys: Option<Vec<TokenDataKeyVector>>,
}

#[derive(Deserialize)]
struct RmwDataVector {
    key: String,
    expected: Option<RmwDataExpected>,
}

#[derive(Debug, Deserialize)]
struct RmwDataExpected {
    #[serde(rename = "domainId")]
    domain_id: u32,
    topic: String,
    #[serde(rename = "typeName")]
    type_name: String,
    #[serde(rename = "typeHash")]
    type_hash: String,
}

#[derive(Deserialize)]
struct RmwTokenVector {
    key: String,
    expected: Option<RmwTokenExpected>,
}

#[derive(Debug, Deserialize)]
struct RmwTokenExpected {
    transport: String,
    #[serde(rename = "entityKind")]
    entity_kind: String,
    #[serde(rename = "domainId")]
    domain_id: u32,
    namespace: String,
    node: String,
    topic: String,
    #[serde(rename = "typeName")]
    type_name: String,
    #[serde(rename = "typeHash")]
    type_hash: String,
}

#[derive(Deserialize)]
struct Ros2ddsTokenVector {
    key: String,
    expected: Option<Ros2ddsTokenExpected>,
}

#[derive(Debug, Deserialize)]
struct Ros2ddsTokenExpected {
    transport: String,
    #[serde(rename = "entityKind")]
    entity_kind: String,
    topic: String,
    #[serde(rename = "typeName")]
    type_name: String,
}

#[derive(Deserialize)]
struct DdsTypeVector {
    dds: String,
    ros: Option<String>,
}

#[derive(Deserialize)]
struct DataKeyTopicVector {
    key: String,
    topic: String,
}

#[derive(Deserialize)]
struct TokenDataKeyVector {
    key: String,
    #[serde(rename = "dataKey")]
    data_key: String,
}

fn entity_kind_name(kind: Ros2EntityKind) -> &'static str {
    match kind {
        Ros2EntityKind::Publisher => "publisher",
        Ros2EntityKind::Subscriber => "subscriber",
        Ros2EntityKind::ServiceServer => "service_server",
        Ros2EntityKind::ServiceClient => "service_client",
        Ros2EntityKind::ActionServer => "action_server",
        Ros2EntityKind::ActionClient => "action_client",
    }
}

fn transport_name(transport: Ros2Transport) -> &'static str {
    match transport {
        Ros2Transport::RmwZenoh => "rmw_zenoh",
        Ros2Transport::Ros2dds => "ros2dds",
    }
}

#[test]
fn names_json_vectors() {
    let text = include_str!("vectors/names.json");
    let vectors: Vectors = serde_json::from_str(text).expect("parse vectors");

    assert_rmw_data_keys(&vectors.rmw_zenoh_data_keys);
    assert_rmw_tokens(&vectors.rmw_zenoh_tokens);
    assert_ros2dds_tokens(&vectors.ros2dds_tokens);
    assert_dds_type_names(&vectors.dds_type_names);
    assert_ros2dds_data_key_topics(&vectors.ros2dds_data_key_topics);
    if let Some(entries) = &vectors.ros2dds_token_data_keys {
        assert_ros2dds_token_data_keys(entries);
    }
}

fn assert_rmw_data_keys(entries: &[RmwDataVector]) {
    for entry in entries {
        let parsed = parse_rmw_zenoh_data_key(&entry.key);
        match (&entry.expected, parsed) {
            (None, None) => {}
            (
                Some(expected),
                Some(RmwZenohDataKey {
                    domain_id,
                    topic,
                    type_name,
                    type_hash,
                }),
            ) => {
                assert_eq!(domain_id, expected.domain_id);
                assert_eq!(topic, expected.topic);
                assert_eq!(type_name, expected.type_name);
                assert_eq!(type_hash, expected.type_hash);
            }
            (expected, parsed) => {
                panic!(
                    "rmw data key {:?}: expected {:?}, got {:?}",
                    entry.key, expected, parsed
                );
            }
        }
    }
}

fn assert_rmw_tokens(entries: &[RmwTokenVector]) {
    for entry in entries {
        let parsed = parse_rmw_zenoh_liveliness_token(&entry.key);
        match (&entry.expected, parsed) {
            (None, None) => {}
            (Some(expected), Some(info)) => assert_rmw_token_matches(expected, &info),
            (expected, parsed) => {
                panic!(
                    "rmw token {:?}: expected {:?}, got {:?}",
                    entry.key, expected, parsed
                );
            }
        }
    }
}

fn assert_rmw_token_matches(expected: &RmwTokenExpected, info: &Ros2LivelinessInfo) {
    assert_eq!(transport_name(info.transport), expected.transport);
    assert_eq!(entity_kind_name(info.entity_kind), expected.entity_kind);
    assert_eq!(info.domain_id, Some(expected.domain_id));
    assert_eq!(info.namespace.as_deref(), Some(expected.namespace.as_str()));
    assert_eq!(info.node.as_deref(), Some(expected.node.as_str()));
    assert_eq!(info.topic, expected.topic);
    assert_eq!(info.type_name, expected.type_name);
    assert_eq!(info.type_hash.as_deref(), Some(expected.type_hash.as_str()));
}

fn assert_ros2dds_tokens(entries: &[Ros2ddsTokenVector]) {
    for entry in entries {
        let parsed = parse_ros2dds_liveliness_token(&entry.key);
        match (&entry.expected, parsed) {
            (None, None) => {}
            (Some(expected), Some(info)) => {
                assert_eq!(transport_name(info.transport), expected.transport);
                assert_eq!(entity_kind_name(info.entity_kind), expected.entity_kind);
                assert_eq!(info.topic, expected.topic);
                assert_eq!(info.type_name, expected.type_name);
            }
            (expected, parsed) => {
                panic!(
                    "ros2dds token {:?}: expected {:?}, got {:?}",
                    entry.key, expected, parsed
                );
            }
        }
    }
}

fn assert_dds_type_names(entries: &[DdsTypeVector]) {
    for entry in entries {
        let parsed = dds_type_name_to_ros(&entry.dds);
        assert_eq!(parsed.as_deref(), entry.ros.as_deref(), "dds {}", entry.dds);
    }
}

fn assert_ros2dds_data_key_topics(entries: &[DataKeyTopicVector]) {
    for entry in entries {
        assert_eq!(ros2dds_data_key_to_topic(&entry.key), entry.topic);
    }
}

fn assert_ros2dds_token_data_keys(entries: &[TokenDataKeyVector]) {
    for entry in entries {
        assert_eq!(
            ros2dds_liveliness_token_to_data_key(&entry.key).as_deref(),
            Some(entry.data_key.as_str())
        );
    }
}
