#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::string::String;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DiskNode {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub is_dir: bool,
}
impl CdrStruct for DiskNode {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            name: reader.read_or_default(|reader| reader.read_string())?,
            path: reader.read_or_default(|reader| reader.read_string())?,
            size_bytes: reader.read_or_default(|reader| reader.read_u64())?,
            is_dir: reader.read_or_default(|reader| reader.read_bool())?,
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_string(self.name.as_str())?;
        writer.write_string(self.path.as_str())?;
        writer.write_u64(self.size_bytes)?;
        writer.write_bool(self.is_dir)?;
        Ok(())
    }
}
impl Message for DiskNode {
    const SCHEMA: &'static str = "# blueos_disk_usage_msgs/msg/DiskNode\n# One file or directory in a usage tree.\n\nstring name\nstring path\nuint64 size_bytes\nbool is_dir";
    const SCHEMA_NAME: &'static str = "blueos_disk_usage_msgs/msg/DiskNode";
    const TYPE_HASH: &'static str =
        "a0a71f5542d65d859fddfdb3673aac2aa5b0f1c0ee75269c7e3b7483bb9d8ca6";
}
impl DiskNode {
    pub const KNOWN_FIELD_COUNT: usize = 4usize;
}
