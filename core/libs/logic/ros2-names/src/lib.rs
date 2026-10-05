//! Pure parsers for rmw_zenoh and zenoh-plugin-ros2dds Zenoh keys and liveliness tokens.

#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

const MIN_RMW_DATA_KEY_SEGMENTS: usize = 4;
const MIN_RMW_LIVELINESS_SEGMENTS: usize = 9;
const MIN_ROS2DDS_TOKEN_SEGMENTS: usize = 3;
const DDS_TYPE_NAME_SEGMENTS: usize = 4;
const ROS_TYPE_PATH_SEGMENTS: usize = 3;

/// Parsed rmw_zenoh sample key: `<domain>/<topic...>/<dds_type>/<RIHS01 hash>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RmwZenohDataKey {
    /// ROS domain id from the first path segment.
    pub domain_id: u32,
    /// Fully qualified ROS topic (`/chatter`).
    pub topic: String,
    /// ROS type name (`std_msgs/msg/String`).
    pub type_name: String,
    /// Type hash suffix (`RIHS01_...`).
    pub type_hash: String,
}

/// ROS 2 entity metadata from a liveliness token (rmw_zenoh or ros2dds).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ros2LivelinessInfo {
    /// Which transport produced the token.
    pub transport: Ros2Transport,
    /// Publisher, subscriber, service, or action role.
    pub entity_kind: Ros2EntityKind,
    /// Fully qualified ROS topic.
    pub topic: String,
    /// ROS type name (`pkg/msg/Name`).
    pub type_name: String,
    /// `RIHS01_` hash when present (rmw_zenoh tokens).
    pub type_hash: Option<String>,
    /// Node name when present.
    pub node: Option<String>,
    /// Node namespace when present.
    pub namespace: Option<String>,
    /// Domain id when present (rmw_zenoh tokens).
    pub domain_id: Option<u32>,
}

/// Which ROS 2 stack produced a liveliness token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ros2Transport {
    /// Keys and tokens from `rmw_zenoh`.
    RmwZenoh,
    /// Tokens from `zenoh-plugin-ros2dds`.
    Ros2dds,
}

/// The role of a ROS 2 entity named by a liveliness token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ros2EntityKind {
    /// Data publisher (`MP`).
    Publisher,
    /// Data subscriber (`MS`).
    Subscriber,
    /// Service server (`SS`).
    ServiceServer,
    /// Service client (`SC`).
    ServiceClient,
    /// Action server (`AS`).
    ActionServer,
    /// Action client (`AC`).
    ActionClient,
}

/// Parses an rmw_zenoh data key, or returns `None` when the key is not rmw_zenoh traffic.
pub fn parse_rmw_zenoh_data_key(key: &str) -> Option<RmwZenohDataKey> {
    let segments: Vec<&str> = key.split('/').collect();
    if segments.len() < MIN_RMW_DATA_KEY_SEGMENTS {
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
// qual:api
pub fn parse_rmw_zenoh_liveliness_token(key: &str) -> Option<Ros2LivelinessInfo> {
    let rest = key.strip_prefix("@ros2_lv/")?;
    if key.starts_with("@/") {
        return None;
    }
    let segments: Vec<&str> = rest.split('/').collect();
    if segments.len() < MIN_RMW_LIVELINESS_SEGMENTS {
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
    if segments.len() < MIN_ROS2DDS_TOKEN_SEGMENTS {
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
    if segments.len() != DDS_TYPE_NAME_SEGMENTS {
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

fn parse_rmw_token_tail<'segment>(
    segments: &[&'segment str],
) -> Option<(String, &'segment str, String)> {
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
    if ros.contains('/') && ros.split('/').count() == ROS_TYPE_PATH_SEGMENTS {
        Some(ros)
    } else {
        None
    }
}
