use alloc::string::String;
use alloc::vec::Vec;

use crate::error::Error;

const ENCAPSULATION_CDR_LE: [u8; 4] = [0x00, 0x01, 0x00, 0x00];

pub struct Writer {
    buffer: Vec<u8>,
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Reader {
    buffer: Vec<u8>,
    position: usize,
}

impl Writer {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn finish_with_encapsulation(self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(4 + self.buffer.len());
        payload.extend_from_slice(&ENCAPSULATION_CDR_LE);
        payload.extend_from_slice(&self.buffer);
        payload
    }

    fn align(&mut self, alignment: usize) {
        let padding = (alignment - (self.buffer.len() % alignment)) % alignment;
        for _index in 0..padding {
            self.buffer.push(0);
        }
    }

    pub fn write_u8(&mut self, value: u8) -> Result<(), Error> {
        self.buffer.push(value);
        Ok(())
    }

    pub fn write_bool(&mut self, value: bool) -> Result<(), Error> {
        self.write_u8(if value { 1 } else { 0 })
    }

    pub fn write_u16(&mut self, value: u16) -> Result<(), Error> {
        self.align(2);
        self.buffer.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    pub fn write_i16(&mut self, value: i16) -> Result<(), Error> {
        self.write_u16(value as u16)
    }

    pub fn write_u32(&mut self, value: u32) -> Result<(), Error> {
        self.align(4);
        self.buffer.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    pub fn write_i32(&mut self, value: i32) -> Result<(), Error> {
        self.write_u32(value as u32)
    }

    pub fn write_u64(&mut self, value: u64) -> Result<(), Error> {
        self.align(8);
        self.buffer.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    pub fn write_i64(&mut self, value: i64) -> Result<(), Error> {
        self.write_u64(value as u64)
    }

    pub fn write_i8(&mut self, value: i8) -> Result<(), Error> {
        self.write_u8(value as u8)
    }

    pub fn write_f32(&mut self, value: f32) -> Result<(), Error> {
        self.write_u32(value.to_bits())
    }

    pub fn write_f64(&mut self, value: f64) -> Result<(), Error> {
        self.write_u64(value.to_bits())
    }

    pub fn write_string(&mut self, value: &str) -> Result<(), Error> {
        let bytes = value.as_bytes();
        let length = (bytes.len() + 1) as u32;
        self.write_u32(length)?;
        self.buffer.extend_from_slice(bytes);
        self.buffer.push(0);
        Ok(())
    }
}

impl Reader {
    pub fn new_with_encapsulation(payload: &[u8]) -> Result<Self, Error> {
        if payload.len() < 4 {
            return Err(Error::InvalidEncapsulation);
        }
        if payload[0..4] != ENCAPSULATION_CDR_LE {
            return Err(Error::InvalidEncapsulation);
        }
        Ok(Self {
            buffer: payload[4..].to_vec(),
            position: 0,
        })
    }

    pub fn new_body(payload: &[u8]) -> Self {
        Self {
            buffer: payload.to_vec(),
            position: 0,
        }
    }

    pub fn is_exhausted(&self) -> bool {
        self.position >= self.buffer.len()
    }

    fn align(&mut self, alignment: usize) -> Result<(), Error> {
        let offset = self.position % alignment;
        if offset == 0 {
            return Ok(());
        }
        let padding = alignment - offset;
        if self.position + padding > self.buffer.len() {
            return Err(Error::UnexpectedEnd);
        }
        self.position += padding;
        Ok(())
    }

    fn read_exact(&mut self, count: usize) -> Result<&[u8], Error> {
        if self.position + count > self.buffer.len() {
            return Err(Error::UnexpectedEnd);
        }
        let slice = &self.buffer[self.position..self.position + count];
        self.position += count;
        Ok(slice)
    }

    pub fn read_u8(&mut self) -> Result<u8, Error> {
        Ok(self.read_exact(1)?[0])
    }

    pub fn read_bool(&mut self) -> Result<bool, Error> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::InvalidBool),
        }
    }

    pub fn read_u16(&mut self) -> Result<u16, Error> {
        self.align(2)?;
        let bytes = self.read_exact(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub fn read_i16(&mut self) -> Result<i16, Error> {
        Ok(self.read_u16()? as i16)
    }

    pub fn read_u32(&mut self) -> Result<u32, Error> {
        self.align(4)?;
        let bytes = self.read_exact(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    pub fn read_i32(&mut self) -> Result<i32, Error> {
        Ok(self.read_u32()? as i32)
    }

    pub fn read_u64(&mut self) -> Result<u64, Error> {
        self.align(8)?;
        let bytes = self.read_exact(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    pub fn read_i64(&mut self) -> Result<i64, Error> {
        Ok(self.read_u64()? as i64)
    }

    pub fn read_i8(&mut self) -> Result<i8, Error> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_f32(&mut self) -> Result<f32, Error> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    pub fn read_f64(&mut self) -> Result<f64, Error> {
        Ok(f64::from_bits(self.read_u64()?))
    }

    pub fn read_string(&mut self) -> Result<String, Error> {
        let length = self.read_u32()? as usize;
        if length == 0 {
            return Ok(String::new());
        }
        let bytes = self.read_exact(length)?.to_vec();
        if bytes.last() != Some(&0) {
            return Err(Error::Utf8);
        }
        let text = core::str::from_utf8(&bytes[..length - 1]).map_err(|_| Error::Utf8)?;
        Ok(text.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_primitives() {
        let mut writer = Writer::new();
        writer.write_bool(true).expect("write bool");
        writer.write_u32(42).expect("write u32");
        writer.write_string("hello").expect("write string");
        let mut reader =
            Reader::new_with_encapsulation(&writer.finish_with_encapsulation()).expect("reader");
        assert!(reader.read_bool().expect("read bool"));
        assert_eq!(reader.read_u32().expect("read u32"), 42);
        assert_eq!(reader.read_string().expect("read string"), "hello");
    }

    #[test]
    fn string_is_not_padded_before_a_byte_field() {
        let mut writer = Writer::new();
        writer.write_string("ab").expect("write string");
        writer.write_bool(true).expect("write bool");
        let payload = writer.finish_with_encapsulation();
        assert_eq!(payload, [0, 1, 0, 0, 3, 0, 0, 0, b'a', b'b', 0, 1]);
        let mut reader = Reader::new_with_encapsulation(&payload).expect("reader");
        assert_eq!(reader.read_string().expect("read string"), "ab");
        assert!(reader.read_bool().expect("read bool"));
        assert!(reader.is_exhausted());
    }

    #[test]
    fn decode_tolerates_trailing_bytes() {
        let mut writer = Writer::new();
        writer.write_u32(7).expect("write");
        writer.write_u32(99).expect("trailing");
        let mut reader =
            Reader::new_with_encapsulation(&writer.finish_with_encapsulation()).expect("reader");
        assert_eq!(reader.read_u32().expect("read"), 7);
        assert!(!reader.is_exhausted());
    }
}
