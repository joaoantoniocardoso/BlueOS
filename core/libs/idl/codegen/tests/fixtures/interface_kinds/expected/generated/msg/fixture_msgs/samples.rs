#![expect(missing_docs, reason = "generated from ROS .msg sources")]
use alloc::vec::Vec;

use serde::{Deserialize, Serialize};

use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Samples {
    pub samples: Vec<u8>,
    #[serde(with = "serde_arrays")]
    pub tag: [u8; 4usize],
    pub counts: Vec<u16>,
}
impl CdrStruct for Samples {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            samples: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values.push(reader.read_u8()?);
                    }
                    values
                }
            },
            tag: {
                let mut values = [Default::default(); 4usize];
                if !reader.is_exhausted() {
                    for index in 0..4usize {
                        values[index] = reader.read_u8()?;
                    }
                }
                values
            },
            counts: {
                if reader.is_exhausted() {
                    Vec::new()
                } else {
                    let length = reader.read_bounded_sequence_length()?;
                    let mut values = Vec::with_capacity(length as usize);
                    for _index in 0..length {
                        values.push(reader.read_u16()?);
                    }
                    values
                }
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u32(self.samples.len() as u32)?;
        for element in self.samples.iter() {
            writer.write_u8(*element)?;
        }
        for element in self.tag.iter() {
            writer.write_u8(*element)?;
        }
        writer.write_u32(self.counts.len() as u32)?;
        for element in self.counts.iter() {
            writer.write_u16(*element)?;
        }
        Ok(())
    }
}
impl Message for Samples {
    const SCHEMA: &'static str =
        "# fixture_msgs/msg/Samples\nuint8[] samples\nuint8[4] tag\nuint16[] counts";
    const SCHEMA_NAME: &'static str = "fixture_msgs/msg/Samples";
    const TYPE_HASH: &'static str =
        "7070936f9bbc48b64c5584d5cb98fb301a0114883cf6f5961a5ecd2615c17b1f";
}
impl Samples {
    pub const KNOWN_FIELD_COUNT: usize = 3usize;
}
