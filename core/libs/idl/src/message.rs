use alloc::vec::Vec;

use crate::cdr::{Reader, Writer};
use crate::error::Error;

pub trait CdrStruct: Sized {
    fn cdr_encode_fields(&self, writer: &mut Writer) -> Result<(), Error>;
    fn cdr_decode_fields(reader: &mut Reader) -> Result<Self, Error>;
}

pub trait Message: CdrStruct {
    const SCHEMA_NAME: &'static str;
    const SCHEMA: &'static str;
    const TYPE_HASH: &'static str;

    fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut writer = Writer::new();
        self.cdr_encode_fields(&mut writer)?;
        Ok(writer.finish_with_encapsulation())
    }

    fn decode(payload: &[u8]) -> Result<Self, Error> {
        let mut reader = Reader::new_with_encapsulation(payload)?;
        Self::cdr_decode_fields(&mut reader)
    }
}
