//! Pure parsers for rmw_zenoh and zenoh-plugin-ros2dds Zenoh keys and liveliness tokens (D-24).

#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Parsed rmw_zenoh sample key: `<domain>/<topic...>/<dds_type>/<RIHS01 hash>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RmwZenohDataKey {
    pub domain_id: u32,
    /// Fully qualified ROS topic (`/chatter`).
    pub topic: String,
    /// ROS type name (`std_msgs/msg/String`).
    pub type_name: String,
    pub type_hash: String,
}

/// ROS 2 entity metadata from a liveliness token (rmw_zenoh or ros2dds).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ros2LivelinessInfo {
    pub transport: Ros2Transport,
    pub entity_kind: Ros2EntityKind,
    pub topic: String,
    pub type_name: String,
    pub type_hash: Option<String>,
    pub node: Option<String>,
    pub namespace: Option<String>,
    pub domain_id: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ros2Transport {
    RmwZenoh,
    Ros2dds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ros2EntityKind {
    Publisher,
    Subscriber,
    ServiceServer,
    ServiceClient,
    ActionServer,
    ActionClient,
}

/// Parses an rmw_zenoh data key, or returns `None` when the key is not rmw_zenoh traffic.
pub fn parse_rmw_zenoh_data_key(key: &str) -> Option<RmwZenohDataKey> {
    let segments: Vec<&str> = key.split('/').collect();
    if segments.len() < 4 {
        return None;
    }
    let domain_id = segments[0].parse::<u32>().ok()?;
    let type_hash = segments[segments.len() - 1];
    if !type_hash.starts_with("RIHS01_") {
        return None;
    }
    let dds_type = segments[segments.len() - 2];
    let type_name = dds_type_name_to_ros(dds_type)?;
    let topic_segments = &segments[1..segments.len() - 2];
    if topic_segments.is_empty() {
        return None;
    }
    let topic = ros_topic_from_segments(topic_segments);
    Some(RmwZenohDataKey {
        domain_id,
        topic,
        type_name,
        type_hash: type_hash.to_string(),
    })
}

/// Parses an rmw_zenoh `@ros2_lv/...` liveliness token. Node tokens (`NN`) return `None`.
pub fn parse_rmw_zenoh_liveliness_token(key: &str) -> Option<Ros2LivelinessInfo> {
    let rest = key.strip_prefix("@ros2_lv/")?;
    if key.starts_with("@/") {
        return None;
    }
    let segments: Vec<&str> = rest.split('/').collect();
    if segments.len() < 9 {
        return None;
    }
    let domain_id = segments[0].parse::<u32>().ok()?;
    let kind = segments[4];
    if kind == "NN" {
        return None;
    }
    let entity_kind = rmw_entity_kind(kind)?;
    let namespace = unescape_rmw_name(segments[6]);
    let node = segments[7].to_string();
    let (topic, dds_type, type_hash) = parse_rmw_token_tail(&segments[8..])?;
    let type_name = dds_type_name_to_ros(dds_type)?;
    Some(Ros2LivelinessInfo {
        transport: Ros2Transport::RmwZenoh,
        entity_kind,
        topic,
        type_name,
        type_hash: Some(type_hash),
        node: Some(node),
        namespace: Some(namespace),
        domain_id: Some(domain_id),
    })
}

/// Parses a zenoh-plugin-ros2dds liveliness token (`@/<zid>/@ros2_lv/...`).
pub fn parse_ros2dds_liveliness_token(key: &str) -> Option<Ros2LivelinessInfo> {
    let after_at = key.strip_prefix("@/")?;
    let zid_end = after_at.find('/')?;
    if zid_end == 0 {
        return None;
    }
    let rest = after_at[zid_end + 1..].strip_prefix("@ros2_lv/")?;
    let segments: Vec<&str> = rest.split('/').collect();
    if segments.len() < 3 {
        return None;
    }
    let entity_kind = ros2dds_entity_kind(segments[0])?;
    let data_key = ros2dds_unescape_key(segments[1]);
    let topic = ros2dds_data_key_to_topic(&data_key);
    let type_name = ros2dds_unescape_type(segments[2])?;
    Some(Ros2LivelinessInfo {
        transport: Ros2Transport::Ros2dds,
        entity_kind,
        topic,
        type_name,
        type_hash: None,
        node: None,
        namespace: None,
        domain_id: None,
    })
}

/// Converts a DDS type string (`pkg::msg::dds_::Name_`) to ROS form (`pkg/msg/Name`).
pub fn dds_type_name_to_ros(dds: &str) -> Option<String> {
    let segments: Vec<&str> = dds.split("::").collect();
    if segments.len() != 4 {
        return None;
    }
    let package = segments[0];
    let kind = segments[1];
    if segments[2] != "dds_" {
        return None;
    }
    let name = segments[3].strip_suffix('_')?;
    if name.is_empty() {
        return None;
    }
    if kind != "msg" && kind != "srv" && kind != "action" {
        return None;
    }
    Some(format!("{package}/{kind}/{name}"))
}

/// Maps a ros2dds Zenoh data key to the ROS topic name.
pub fn ros2dds_data_key_to_topic(data_key: &str) -> String {
    if data_key.is_empty() {
        return "/".to_string();
    }
    if data_key.starts_with('/') {
        return data_key.to_string();
    }
    format!("/{data_key}")
}

/// Extracts the Zenoh data key a ros2dds token advertises (slashes, no leading slash).
pub fn ros2dds_liveliness_token_to_data_key(key: &str) -> Option<String> {
    let after_at = key.strip_prefix("@/")?;
    let zid_end = after_at.find('/')?;
    let rest = after_at[zid_end + 1..].strip_prefix("@ros2_lv/")?;
    let segments: Vec<&str> = rest.split('/').collect();
    if segments.len() < 2 {
        return None;
    }
    ros2dds_entity_kind(segments[0])?;
    Some(ros2dds_unescape_key(segments[1]))
}

fn ros_topic_from_segments(segments: &[&str]) -> String {
    format!("/{}", segments.join("/"))
}

fn unescape_rmw_name(segment: &str) -> String {
    if segment == "%" {
        return "/".to_string();
    }
    segment.replace('%', "/")
}

fn parse_rmw_token_tail<'a>(segments: &'a [&'a str]) -> Option<(String, &'a str, String)> {
    let type_index = segments.iter().position(|segment| segment.contains("::"))?;
    let dds_type = segments[type_index];
    let hash_index = type_index + 1;
    if hash_index >= segments.len() {
        return None;
    }
    let type_hash = segments[hash_index];
    if !type_hash.starts_with("RIHS01_") {
        return None;
    }
    let topic = if type_index == 1 {
        unescape_rmw_name(segments[0])
    } else {
        let joined = segments[..type_index].join("/");
        unescape_rmw_name(&joined)
    };
    Some((topic, dds_type, type_hash.to_string()))
}

fn rmw_entity_kind(kind: &str) -> Option<Ros2EntityKind> {
    match kind {
        "MP" => Some(Ros2EntityKind::Publisher),
        "MS" => Some(Ros2EntityKind::Subscriber),
        "SS" => Some(Ros2EntityKind::ServiceServer),
        "SC" => Some(Ros2EntityKind::ServiceClient),
        "AS" => Some(Ros2EntityKind::ActionServer),
        "AC" => Some(Ros2EntityKind::ActionClient),
        _ => None,
    }
}

fn ros2dds_entity_kind(kind: &str) -> Option<Ros2EntityKind> {
    rmw_entity_kind(kind)
}

fn ros2dds_unescape_key(segment: &str) -> String {
    segment.replace('\u{a7}', "/")
}

fn ros2dds_unescape_type(segment: &str) -> Option<String> {
    let ros = segment.replace('\u{a7}', "/");
    if ros.contains('/') && ros.split('/').count() == 3 {
        Some(ros)
    } else {
        None
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use serde::Deserialize;

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
        domainId: u32,
        topic: String,
        typeName: String,
        typeHash: String,
    }

    #[derive(Deserialize)]
    struct RmwTokenVector {
        key: String,
        expected: Option<RmwTokenExpected>,
    }

    #[derive(Debug, Deserialize)]
    struct RmwTokenExpected {
        transport: String,
        entityKind: String,
        domainId: u32,
        namespace: String,
        node: String,
        topic: String,
        typeName: String,
        typeHash: String,
    }

    #[derive(Deserialize)]
    struct Ros2ddsTokenVector {
        key: String,
        expected: Option<Ros2ddsTokenExpected>,
    }

    #[derive(Debug, Deserialize)]
    struct Ros2ddsTokenExpected {
        transport: String,
        entityKind: String,
        topic: String,
        typeName: String,
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
        dataKey: String,
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
    fn vectors_from_json() {
        let text = include_str!("../tests/vectors/names.json");
        let vectors: Vectors = serde_json::from_str(text).expect("parse vectors");

        for entry in &vectors.rmw_zenoh_data_keys {
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
                    assert_eq!(domain_id, expected.domainId);
                    assert_eq!(topic, expected.topic);
                    assert_eq!(type_name, expected.typeName);
                    assert_eq!(type_hash, expected.typeHash);
                }
                (expected, parsed) => {
                    panic!(
                        "rmw data key {:?}: expected {:?}, got {:?}",
                        entry.key, expected, parsed
                    );
                }
            }
        }

        for entry in &vectors.rmw_zenoh_tokens {
            let parsed = parse_rmw_zenoh_liveliness_token(&entry.key);
            match (&entry.expected, parsed) {
                (None, None) => {}
                (Some(expected), Some(info)) => {
                    assert_eq!(transport_name(info.transport), expected.transport);
                    assert_eq!(entity_kind_name(info.entity_kind), expected.entityKind);
                    assert_eq!(info.domain_id, Some(expected.domainId));
                    assert_eq!(info.namespace.as_deref(), Some(expected.namespace.as_str()));
                    assert_eq!(info.node.as_deref(), Some(expected.node.as_str()));
                    assert_eq!(info.topic, expected.topic);
                    assert_eq!(info.type_name, expected.typeName);
                    assert_eq!(info.type_hash.as_deref(), Some(expected.typeHash.as_str()));
                }
                (expected, parsed) => {
                    panic!(
                        "rmw token {:?}: expected {:?}, got {:?}",
                        entry.key, expected, parsed
                    );
                }
            }
        }

        for entry in &vectors.ros2dds_tokens {
            let parsed = parse_ros2dds_liveliness_token(&entry.key);
            match (&entry.expected, parsed) {
                (None, None) => {}
                (Some(expected), Some(info)) => {
                    assert_eq!(transport_name(info.transport), expected.transport);
                    assert_eq!(entity_kind_name(info.entity_kind), expected.entityKind);
                    assert_eq!(info.topic, expected.topic);
                    assert_eq!(info.type_name, expected.typeName);
                }
                (expected, parsed) => {
                    panic!(
                        "ros2dds token {:?}: expected {:?}, got {:?}",
                        entry.key, expected, parsed
                    );
                }
            }
        }

        for entry in &vectors.dds_type_names {
            let parsed = dds_type_name_to_ros(&entry.dds);
            assert_eq!(parsed.as_deref(), entry.ros.as_deref(), "dds {}", entry.dds);
        }

        for entry in &vectors.ros2dds_data_key_topics {
            assert_eq!(ros2dds_data_key_to_topic(&entry.key), entry.topic);
        }

        if let Some(entries) = &vectors.ros2dds_token_data_keys {
            for entry in entries {
                assert_eq!(
                    ros2dds_liveliness_token_to_data_key(&entry.key).as_deref(),
                    Some(entry.dataKey.as_str()),
                    "token {}",
                    entry.key
                );
            }
        }
    }
}
