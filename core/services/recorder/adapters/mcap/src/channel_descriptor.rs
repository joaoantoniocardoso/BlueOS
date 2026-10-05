//! One channel descriptor per topic, built once and reused for every sample.

use std::collections::BTreeMap;

use serde_json::{Map, Number, Value, json};
use tracing::{error, warn};

use blueos_comms::Payload;
use blueos_idl::{catalog, schema};

/// Channel metadata for MCAP registration.
#[derive(Clone)]
pub struct ChannelDescriptor {
    /// Zenoh topic key.
    pub topic: String,
    /// Optional schema.
    pub schema: Option<SchemaDescriptor>,
    /// Payload encoding on the channel.
    pub message_encoding: MessageEncoding,
}

/// Registered schema metadata for one channel.
#[derive(Clone)]
pub struct SchemaDescriptor {
    /// Schema encoding in the MCAP file.
    pub encoding: SchemaEncoding,
    /// Schema name and text when known.
    pub content: Option<SchemaDescriptorContent>,
}

/// Schema name and body for MCAP.
#[derive(Clone)]
pub struct SchemaDescriptorContent {
    /// ROS 2 or JSON schema name.
    pub name: String,
    /// Schema text.
    pub data: String,
}

/// How message payloads on the channel are encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageEncoding {
    /// ROS 2 CDR.
    Cdr,
    /// JSON object payloads.
    Json,
    /// Raw bytes.
    OctetStream,
}

/// How the MCAP schema is encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaEncoding {
    /// Embedded ROS 2 `.msg` text.
    Ros2Msg,
    /// JSON Schema inferred from a JSON payload.
    JsonSchema,
}

/// Identifies one MCAP channel route (default topic lane or a typed schema lane).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ChannelRoute {
    /// Zenoh topic key.
    pub topic: String,
    /// ROS 2 type name for a typed lane; none for encoding/default/fallback lanes.
    pub type_name: Option<String>,
}

impl SchemaEncoding {
    /// MCAP schema encoding string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ros2Msg => "ros2msg",
            Self::JsonSchema => "jsonschema",
        }
    }
}

impl MessageEncoding {
    /// MCAP message encoding string.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cdr => "cdr",
            Self::Json => "json",
            Self::OctetStream => "application/octet-stream",
        }
    }
}

impl ChannelRoute {
    /// Default route for a topic (encoding suffix or fallback lane).
    pub fn for_topic(topic: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            type_name: None,
        }
    }

    /// Typed schema lane for a resolved ROS 2 message type.
    pub fn typed(topic: impl Into<String>, type_name: impl Into<String>) -> Self {
        Self {
            topic: topic.into(),
            type_name: Some(type_name.into()),
        }
    }
}

/// Builds a channel descriptor from a backbone sample (D-24 order: IDL then catalog, else JSON fallback).
pub fn channel_descriptor_for_sample(
    topic: &str,
    sample_encoding: &str,
    payload: &Payload,
) -> Option<ChannelDescriptor> {
    let (media_type, schema_name) = parse_sample_encoding(sample_encoding)?;
    descriptor_for_media_type(topic, sample_encoding, media_type, schema_name, payload)
}

fn descriptor_for_media_type(
    topic: &str,
    sample_encoding: &str,
    media_type: &str,
    schema_name: Option<&str>,
    payload: &Payload,
) -> Option<ChannelDescriptor> {
    match media_type {
        "application/cdr" => descriptor_for_cdr_media(topic, sample_encoding, schema_name),
        "application/json" => descriptor_for_json_sample(topic, schema_name, payload),
        "application/octet-stream" => Some(descriptor_octet_stream(topic)),
        "zenoh/bytes" | "" => None,
        _ => unknown_sample_encoding(sample_encoding),
    }
}

fn descriptor_for_cdr_media(
    topic: &str,
    sample_encoding: &str,
    schema_name: Option<&str>,
) -> Option<ChannelDescriptor> {
    match schema_name {
        Some(type_name) => Some(descriptor_for_cdr_type(topic, type_name)),
        None => unknown_sample_encoding(sample_encoding),
    }
}

fn unknown_sample_encoding(sample_encoding: &str) -> Option<ChannelDescriptor> {
    warn!(sample_encoding, "unknown sample encoding");
    None
}

/// Builds a CDR channel for a resolved ROS 2 type name (IDL, catalog, or fallback).
pub fn channel_descriptor_for_ros2_type(topic: &str, type_name: &str) -> ChannelDescriptor {
    descriptor_for_cdr_type(topic, type_name)
}

/// CDR channel without a schema (late-schema seam for #72).
pub fn channel_descriptor_cdr_fallback(topic: &str) -> ChannelDescriptor {
    ChannelDescriptor {
        topic: topic.to_owned(),
        schema: None,
        message_encoding: MessageEncoding::Cdr,
    }
}

fn parse_sample_encoding(sample_encoding: &str) -> Option<(&str, Option<&str>)> {
    let mut parts = sample_encoding.split(';');
    let Some(media_type) = parts.next() else {
        warn!("sample has no encoding string");
        return None;
    };
    Some((media_type, parts.next()))
}

fn descriptor_for_cdr_type(topic: &str, type_name: &str) -> ChannelDescriptor {
    let Some(schema_data) = load_cdr_schema(type_name) else {
        return channel_descriptor_cdr_fallback(topic);
    };
    ChannelDescriptor {
        topic: topic.to_owned(),
        schema: Some(SchemaDescriptor {
            encoding: SchemaEncoding::Ros2Msg,
            content: Some(SchemaDescriptorContent {
                name: type_name.to_owned(),
                data: schema_data,
            }),
        }),
        message_encoding: MessageEncoding::Cdr,
    }
}

fn descriptor_for_json_sample(
    topic: &str,
    schema_name: Option<&str>,
    payload: &Payload,
) -> Option<ChannelDescriptor> {
    let parsed = json_object_from_payload(payload)?;
    Some(descriptor_for_json_object(topic, schema_name, &parsed))
}

fn json_object_from_payload(payload: &Payload) -> Option<Value> {
    let string = String::from_utf8(payload.to_bytes().to_vec()).ok()?;
    let parsed = json5_parse(&string)?;
    parsed.is_object().then_some(parsed)
}

fn descriptor_for_json_object(
    topic: &str,
    schema_name: Option<&str>,
    parsed: &Value,
) -> ChannelDescriptor {
    ChannelDescriptor {
        topic: topic.to_owned(),
        schema: Some(SchemaDescriptor {
            encoding: SchemaEncoding::JsonSchema,
            content: Some(SchemaDescriptorContent {
                name: json_schema_name(topic, schema_name),
                data: create_schema(parsed).to_string(),
            }),
        }),
        message_encoding: MessageEncoding::Json,
    }
}

fn json_schema_name(topic: &str, schema_name: Option<&str>) -> String {
    schema_name
        .map(str::to_owned)
        .unwrap_or_else(|| topic.replace('/', "."))
}

fn descriptor_octet_stream(topic: &str) -> ChannelDescriptor {
    ChannelDescriptor {
        topic: topic.to_owned(),
        schema: None,
        message_encoding: MessageEncoding::OctetStream,
    }
}

fn load_cdr_schema(schema_name: &str) -> Option<String> {
    if let Some(text) = schema(schema_name) {
        return Some(text.to_owned());
    }
    if let Some(text) = catalog::schema(schema_name) {
        return Some(text.to_owned());
    }
    error!(schema_name, "no embedded schema for CDR sample");
    None
}

fn json5_parse(string: &str) -> Option<Value> {
    json5::from_str(string)
        .or_else(|_| serde_json::from_str(string))
        .ok()
}

fn create_schema(value: &Value) -> Value {
    match value {
        Value::Null => schema_type_null(),
        Value::Bool(_) => schema_type_bool(),
        Value::Number(number) => schema_type_number(number),
        Value::String(_) => schema_type_string(),
        Value::Array(array) => schema_for_array(array),
        Value::Object(map) => schema_for_object(map),
    }
}

fn schema_type_null() -> Value {
    json!({ "type": "null" })
}

fn schema_type_bool() -> Value {
    json!({ "type": "boolean" })
}

fn schema_type_number(number: &Number) -> Value {
    if number.is_i64() {
        json!({ "type": "integer" })
    } else {
        json!({ "type": "number" })
    }
}

fn schema_type_string() -> Value {
    json!({ "type": "string" })
}

fn schema_for_array(array: &[Value]) -> Value {
    json!({ "type": "array", "items": array_items_schema(array) })
}

fn array_items_schema(array: &[Value]) -> Value {
    array.first().map_or_else(|| json!({}), create_schema)
}

fn schema_for_object(map: &Map<String, Value>) -> Value {
    json!({ "type": "object", "properties": object_properties_schema(map) })
}

fn object_properties_schema(map: &Map<String, Value>) -> BTreeMap<String, Value> {
    map.iter()
        .map(|(key, field)| (key.clone(), create_schema(field)))
        .collect()
}
