#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Duration {
    pub sec: i32,
    pub nanosec: u32,
}
impl CdrStruct for Duration {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            sec: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_i32()?
            },
            nanosec: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u32()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_i32(self.sec)?;
        writer.write_u32(self.nanosec)?;
        Ok(())
    }
}
impl Message for Duration {
    const SCHEMA: &'static str =
        "# This message communicates ROS Duration.\n\nint32 sec\nuint32 nanosec";
    const SCHEMA_NAME: &'static str = "builtin_interfaces/msg/Duration";
    const TYPE_HASH: &'static str =
        "71bd4207cf712263d6579d8859264a854efa8f393d6e0b8ae9651d1baf691749";
}
impl Duration {
    pub const KNOWN_FIELD_COUNT: usize = 2usize;
}
