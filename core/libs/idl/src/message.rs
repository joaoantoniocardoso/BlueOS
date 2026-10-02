//! Traits linking generated message types to the CDR codec and schema metadata.

use alloc::vec::Vec;

use crate::{
    cdr::{Reader, Writer},
    error::Error,
};

/// Encodes and decodes the field body of a ROS 2 message (without the CDR encapsulation header).
pub trait CdrStruct: Sized {
    /// Reads this message's fields in `.msg` order.
    fn cdr_decode_fields(reader: &mut Reader) -> Result<Self, Error>;
    /// Writes this message's fields in `.msg` order.
    fn cdr_encode_fields(&self, writer: &mut Writer) -> Result<(), Error>;
}

/// A BlueOS IDL message: CDR payload plus embedded `ros2msg` schema metadata.
pub trait Message: CdrStruct {
    /// Embedded `ros2msg` schema text (root first, dependencies under `MSG:` headers).
    const SCHEMA: &'static str;
    /// ROS 2 schema name (`package/msg/Name`).
    const SCHEMA_NAME: &'static str;
    /// SHA-256 of schema name and field signature (API lock and diagnostics).
    const TYPE_HASH: &'static str;

    /// Decodes a CDR payload with encapsulation header.
    fn decode(payload: &[u8]) -> Result<Self, Error> {
        let mut reader = Reader::new_with_encapsulation(payload)?;
        Self::cdr_decode_fields(&mut reader)
    }

    /// Encodes this message with the standard CDR encapsulation header.
    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut writer = Writer::new();
        self.cdr_encode_fields(&mut writer)?;
        Ok(writer.finish_with_encapsulation())
    }
}
