use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;

use anyhow::Result;
use blueos_comms::Payload;
use serde_json::{Value, json};
use tracing::{error, instrument, warn};

pub struct ChannelDescriptor {
    pub topic: String,
    pub schema: Option<SchemaDescriptor>,
    pub message_encoding: MessageEncoding,
}

pub struct SchemaDescriptor {
    pub encoding: SchemaEncoding,
    pub content: Option<SchemaDescriptorContent>,
}

pub struct SchemaDescriptorContent {
    pub name: String,
    pub data: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaEncoding {
    Ros2Msg,
    JsonSchema,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageEncoding {
    Cdr,
    Json,
    OctetStream,
}

#[instrument(skip_all)]
pub fn channel_descriptor_for_sample(
    topic: &str,
    encoding: &str,
    payload: &Payload,
    schema_lookup: &dyn Fn(&str) -> Option<String>,
    schema_path: Option<&Path>,
) -> Option<ChannelDescriptor> {
    let (encoding, schema) = {
        let mut encoding_split = encoding.split(';');
        let Some(encoding) = encoding_split.next() else {
            warn!("No encoding string");
            return None;
        };
        let schema = encoding_split.next();
        (encoding, schema)
    };

    match (encoding, schema) {
        ("application/cdr", Some(schema_name)) => {
            let schema_data = match load_cdr_schema(schema_name, schema_lookup, schema_path) {
                Ok(schema_data) => schema_data,
                Err(error) => {
                    error!(%error, "Failed to load schema");
                    return None;
                }
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
        ("application/json", schema) => {
            let string = String::from_utf8(payload.to_vec()).map_err(|_| ()).ok()?;
            let value = json5_parse(&string)?;
            if !value.is_object() {
                return None;
            }
            let schema_name = match schema {
                Some(name) => name.to_owned(),
                None => topic.replace('/', "."),
            };
            let schema_data = create_schema(&value).to_string();
            Some(ChannelDescriptor {
                topic: topic.to_owned(),
                schema: Some(SchemaDescriptor {
                    encoding: SchemaEncoding::JsonSchema,
                    content: Some(SchemaDescriptorContent {
                        name: schema_name,
                        data: schema_data,
                    }),
                }),
                message_encoding: MessageEncoding::Json,
            })
        }
        ("application/octet-stream", _schema) => Some(ChannelDescriptor {
            topic: topic.to_owned(),
            schema: None,
            message_encoding: MessageEncoding::OctetStream,
        }),
        _ => {
            warn!(encoding, "Received unknown encoding");
            None
        }
    }
}

fn json5_parse(string: &str) -> Option<Value> {
    json5::from_str(string)
        .or_else(|_| serde_json::from_str(string))
        .ok()
}

impl SchemaEncoding {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ros2Msg => "ros2msg",
            Self::JsonSchema => "jsonschema",
        }
    }
}

impl MessageEncoding {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cdr => "cdr",
            Self::Json => "json",
            Self::OctetStream => "application/octet-stream",
        }
    }
}

impl fmt::Display for SchemaEncoding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl fmt::Display for MessageEncoding {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[instrument(skip_all)]
fn load_cdr_schema(
    schema: &str,
    schema_lookup: &dyn Fn(&str) -> Option<String>,
    schema_path: Option<&Path>,
) -> Result<String> {
    if let Some(embedded) = schema_lookup(schema) {
        return Ok(embedded);
    }

    let mut schema_splitted = schema.split('.');
    let schema_package = schema_splitted
        .next()
        .ok_or_else(|| anyhow::anyhow!("Failed to get schema package from {schema}"))?;
    let schema_name = schema_splitted
        .next()
        .ok_or_else(|| anyhow::anyhow!("Failed to get schema name from {schema}"))?;

    if let Some(schema_path) = schema_path {
        let schema_path = schema_path.join(format!("{schema_package}/{schema_name}.msg"));
        std::fs::read_to_string(&schema_path)
            .map_err(|error| anyhow::anyhow!("Failed to read schema: {error}, ({schema_path:?})"))
    } else {
        Err(anyhow::anyhow!("No schema source for {schema}"))
    }
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
                .map(|(key, value)| (key.clone(), create_schema(value)))
                .collect();
            json!({ "type": "object", "properties": properties })
        }
    }
}
