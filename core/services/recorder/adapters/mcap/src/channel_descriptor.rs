//! One channel descriptor per topic, built once and reused for every sample.

use core::fmt;
use std::collections::BTreeMap;

use serde_json::{Value, json};
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

impl fmt::Display for SchemaEncoding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
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

impl fmt::Display for MessageEncoding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
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
    let (media_type, schema_name) = {
        let mut parts = sample_encoding.split(';');
        let Some(media_type) = parts.next() else {
            warn!("sample has no encoding string");
            return None;
        };
        let schema_name = parts.next();
        (media_type, schema_name)
    };

    match (media_type, schema_name) {
        ("application/cdr", Some(schema_name)) => {
            let Some(schema_data) = load_cdr_schema(schema_name) else {
                return Some(channel_descriptor_cdr_fallback(topic));
            };
            Some(ChannelDescriptor {
                topic: topic.to_owned(),
                schema: Some(SchemaDescriptor {
                    encoding: SchemaEncoding::Ros2Msg,
                    content: Some(SchemaDescriptorContent {
                        name: schema_name.to_owned(),
                        data: schema_data,
                    }),
                }),
                message_encoding: MessageEncoding::Cdr,
            })
        }
        ("application/json", schema_name) => {
            let string = String::from_utf8(payload.to_bytes().to_vec()).ok()?;
            let parsed = json5_parse(&string)?;
            if !parsed.is_object() {
                return None;
            }
            let name = schema_name
                .map(str::to_owned)
                .unwrap_or_else(|| topic.replace('/', "."));
            Some(ChannelDescriptor {
                topic: topic.to_owned(),
                schema: Some(SchemaDescriptor {
                    encoding: SchemaEncoding::JsonSchema,
                    content: Some(SchemaDescriptorContent {
                        name,
                        data: create_schema(&parsed).to_string(),
                    }),
                }),
                message_encoding: MessageEncoding::Json,
            })
        }
        ("application/octet-stream", _) => Some(ChannelDescriptor {
            topic: topic.to_owned(),
            schema: None,
            message_encoding: MessageEncoding::OctetStream,
        }),
        ("zenoh/bytes", _) | ("", _) => None,
        _ => {
            warn!(sample_encoding, "unknown sample encoding");
            None
        }
    }
}

/// Builds a CDR channel for a resolved ROS 2 type name (IDL, catalog, or fallback).
pub fn channel_descriptor_for_ros2_type(topic: &str, type_name: &str) -> ChannelDescriptor {
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

/// CDR channel without a schema (late-schema seam for #72).
pub fn channel_descriptor_cdr_fallback(topic: &str) -> ChannelDescriptor {
    ChannelDescriptor {
        topic: topic.to_owned(),
        schema: None,
        message_encoding: MessageEncoding::Cdr,
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
        Value::Null => json!({ "type": "null" }),
        Value::Bool(_) => json!({ "type": "boolean" }),
        Value::Number(number) if number.is_i64() => json!({ "type": "integer" }),
        Value::Number(_) => json!({ "type": "number" }),
        Value::String(_) => json!({ "type": "string" }),
        Value::Array(array) => {
            let items = if let Some(first) = array.first() {
                create_schema(first)
            } else {
                json!({})
            };
            json!({ "type": "array", "items": items })
        }
        Value::Object(map) => {
            let properties: BTreeMap<_, _> = map
                .iter()
                .map(|(key, field)| (key.clone(), create_schema(field)))
                .collect();
            json!({ "type": "object", "properties": properties })
        }
    }
}
