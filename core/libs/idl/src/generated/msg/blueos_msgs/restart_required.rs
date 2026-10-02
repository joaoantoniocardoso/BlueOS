#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::{string::String, vec::Vec};

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RestartRequired {
    pub fields: Vec<String>,
}
impl CdrStruct for RestartRequired {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            fields: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values.push(reader.read_string()?);
                    }
                    values
                }
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u32(self.fields.len() as u32)?;
        for element in self.fields.iter() {
            writer.write_string(element.as_str())?;
        }
        Ok(())
    }
}
impl Message for RestartRequired {
    const SCHEMA: &'static str = "# blueos_msgs/msg/RestartRequired\n# Event listing settings fields that need a service restart (D-11).\n\nstring[] fields";
    const SCHEMA_NAME: &'static str = "blueos_msgs/msg/RestartRequired";
    const TYPE_HASH: &'static str =
        "0a561257a82d7148488f913ffcd56736d9eae223d9868b69903edac6a14ebcd0";
}
impl RestartRequired {
    pub const KNOWN_FIELD_COUNT: usize = 1usize;
}
