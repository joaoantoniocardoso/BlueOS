#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SettingsEnvelope {
    pub document_json: String,
    pub fields: Vec<crate::msg::blueos_msgs::SettingField>,
}
impl CdrStruct for SettingsEnvelope {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            document_json: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            fields: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values.push(<crate::msg::blueos_msgs::SettingField>::cdr_decode_fields(
                            reader,
                        )?);
                    }
                    values
                }
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.document_json.as_str())?;
        writer.write_u32(self.fields.len() as u32)?;
        for element in self.fields.iter() {
            <crate::msg::blueos_msgs::SettingField>::cdr_encode_fields(element, writer)?;
        }
        Ok(())
    }
}
impl Message for SettingsEnvelope {
    const SCHEMA: &'static str = "# blueos_msgs/msg/SettingsEnvelope\n# JSON settings document plus per-field restart flags (D-11).\n\nstring document_json\nblueos_msgs/SettingField[] fields\n================================================================================\nMSG: blueos_msgs/SettingField\n# blueos_msgs/msg/SettingField\n# Restart hint for one settings field (D-11).\n\nstring path\nbool restart_required";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/SettingsEnvelope";
    const TYPE_HASH: &'static str =
        "28eb1777c86372a0328bdcb1f3227df554309405b46fff0a1e24d8bfa6f64317";
}
impl SettingsEnvelope {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
