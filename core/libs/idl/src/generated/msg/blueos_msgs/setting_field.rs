#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use alloc::string::String;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SettingField {
    pub path: String,
    pub restart_required: bool,
}
impl CdrStruct for SettingField {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            path: if reader.is_exhausted() {
                String::new()
            } else {
                reader.read_string()?
            },
            restart_required: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.path.as_str())?;
        writer.write_bool(self.restart_required)?;
        Ok(())
    }
}
impl Message for SettingField {
    const SCHEMA: &'static str = "# blueos_msgs/msg/SettingField\n# Restart hint for one settings field (D-11).\n\nstring path\nbool restart_required";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/SettingField";
    const TYPE_HASH: &'static str =
        "a98023d1b48baf939b89899f41763baca4da7726858b4e5fd110a0b1b0eb08fe";
}
impl SettingField {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
