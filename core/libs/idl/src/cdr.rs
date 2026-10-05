//! Little-endian CDR writers and readers for ROS 2 message payloads.

use alloc::{string::String, vec::Vec};

use crate::error::Error;

const ENCAPSULATION_CDR_LE: [u8; 4] = [0x00, 0x01, 0x00, 0x00];
const ENCAPSULATION_HEADER_LEN: usize = ENCAPSULATION_CDR_LE.len();
const CDR_ALIGN_U32: usize = 4;
const CDR_ALIGN_U64: usize = 8;
const CDR_U32_BYTES: usize = 4;
const CDR_U64_BYTES: usize = 8;

/// Incremental CDR writer for message field bodies (no encapsulation header).
pub struct Writer {
    buffer: Vec<u8>,
}

/// Incremental CDR reader for message field bodies.
pub struct Reader {
    buffer: Vec<u8>,
    position: usize,
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

impl Writer {
    /// Creates an empty writer.
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Prepends the standard little-endian CDR encapsulation header.
    pub fn finish_with_encapsulation(self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(ENCAPSULATION_HEADER_LEN + self.buffer.len());
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

    /// Writes one byte (no alignment).
    pub fn write_u8(&mut self, value: u8) -> Result<(), Error> {
        self.buffer.push(value);
        Ok(())
    }

    /// Writes bytes as they are (no alignment), such as the elements of a `uint8[]`.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), Error> {
        self.buffer.extend_from_slice(bytes);
        Ok(())
    }

    /// Writes a CDR bool as `0` or `1`.
    // qual:api
    pub fn write_bool(&mut self, value: bool) -> Result<(), Error> {
        self.write_u8(if value { 1 } else { 0 })
    }

    /// Writes a little-endian `u16` after alignment.
    pub fn write_u16(&mut self, value: u16) -> Result<(), Error> {
        self.align(2);
        self.buffer.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Writes a little-endian `i16` after alignment.
    // qual:api
    pub fn write_i16(&mut self, value: i16) -> Result<(), Error> {
        self.write_u16(value as u16)
    }

    /// Writes a little-endian `u32` after alignment.
    pub fn write_u32(&mut self, value: u32) -> Result<(), Error> {
        self.align(CDR_ALIGN_U32);
        self.buffer.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Writes a little-endian `i32` after alignment.
    // qual:api
    pub fn write_i32(&mut self, value: i32) -> Result<(), Error> {
        self.write_u32(value as u32)
    }

    /// Writes a little-endian `u64` after alignment.
    pub fn write_u64(&mut self, value: u64) -> Result<(), Error> {
        self.align(CDR_ALIGN_U64);
        self.buffer.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Writes a little-endian `i64` after alignment.
    // qual:api
    pub fn write_i64(&mut self, value: i64) -> Result<(), Error> {
        self.write_u64(value as u64)
    }

    /// Writes a little-endian `i8` (no alignment).
    // qual:api
    pub fn write_i8(&mut self, value: i8) -> Result<(), Error> {
        self.write_u8(value as u8)
    }

    /// Writes an IEEE754 `f32` after alignment.
    // qual:api
    pub fn write_f32(&mut self, value: f32) -> Result<(), Error> {
        self.write_u32(value.to_bits())
    }

    /// Writes an IEEE754 `f64` after alignment.
    // qual:api
    pub fn write_f64(&mut self, value: f64) -> Result<(), Error> {
        self.write_u64(value.to_bits())
    }

    /// Writes a ROS string: `u32` length including the null terminator, bytes, then `0`.
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
    /// Strips the encapsulation header and positions at the first field.
    pub fn new_with_encapsulation(payload: &[u8]) -> Result<Self, Error> {
        if payload.len() < ENCAPSULATION_HEADER_LEN {
            return Err(Error::InvalidEncapsulation);
        }
        if payload[0..ENCAPSULATION_HEADER_LEN] != ENCAPSULATION_CDR_LE {
            return Err(Error::InvalidEncapsulation);
        }
        Ok(Self {
            buffer: payload[ENCAPSULATION_HEADER_LEN..].to_vec(),
            position: 0,
        })
    }

    /// Returns whether every byte of the field body has been consumed.
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.buffer.len()
    }

    /// Reads one field with `read`, or gives its default when the body ends before it: the sender's version of the
    /// Message predates the field (D-06).
    pub fn read_or_default<T: Default>(
        &mut self,
        read: impl FnOnce(&mut Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        if self.is_exhausted() {
            return Ok(T::default());
        }
        read(self)
    }

    /// Bytes left in the field body after the current read position.
    pub fn remaining_body_bytes(&self) -> usize {
        self.buffer.len().saturating_sub(self.position)
    }

    fn align(&mut self, alignment: usize) -> Result<(), Error> {
        let offset = self.position % alignment;
        if offset == 0 {
            return Ok(());
        }
        let padding = alignment - offset;
        let next_position = self
            .position
            .checked_add(padding)
            .ok_or(Error::UnexpectedEnd)?;
        if next_position > self.buffer.len() {
            return Err(Error::UnexpectedEnd);
        }
        self.position = next_position;
        Ok(())
    }

    fn read_exact(&mut self, count: usize) -> Result<&[u8], Error> {
        let end = self
            .position
            .checked_add(count)
            .ok_or(Error::UnexpectedEnd)?;
        if end > self.buffer.len() {
            return Err(Error::UnexpectedEnd);
        }
        let slice = &self.buffer[self.position..end];
        self.position = end;
        Ok(slice)
    }

    /// Reads one byte (no alignment).
    pub fn read_u8(&mut self) -> Result<u8, Error> {
        Ok(self.read_exact(1)?[0])
    }

    /// Reads `count` bytes as they are (no alignment), such as the elements of a `uint8[]`.
    pub fn read_bytes(&mut self, count: usize) -> Result<&[u8], Error> {
        self.read_exact(count)
    }

    /// Reads a CDR bool (`0` or `1`).
    // qual:api
    pub fn read_bool(&mut self) -> Result<bool, Error> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::InvalidBool),
        }
    }

    /// Reads a little-endian `u16` after alignment.
    pub fn read_u16(&mut self) -> Result<u16, Error> {
        self.align(2)?;
        let bytes = self.read_exact(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Reads a little-endian `i16` after alignment.
    // qual:api
    pub fn read_i16(&mut self) -> Result<i16, Error> {
        Ok(self.read_u16()? as i16)
    }

    /// Reads a little-endian `u32` after alignment.
    pub fn read_u32(&mut self) -> Result<u32, Error> {
        self.align(CDR_ALIGN_U32)?;
        let bytes = self.read_exact(CDR_U32_BYTES)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Reads a little-endian `i32` after alignment.
    // qual:api
    pub fn read_i32(&mut self) -> Result<i32, Error> {
        Ok(self.read_u32()? as i32)
    }

    /// Reads a little-endian `u64` after alignment.
    pub fn read_u64(&mut self) -> Result<u64, Error> {
        self.align(CDR_ALIGN_U64)?;
        let bytes = self.read_exact(CDR_U64_BYTES)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Reads a little-endian `i64` after alignment.
    // qual:api
    pub fn read_i64(&mut self) -> Result<i64, Error> {
        Ok(self.read_u64()? as i64)
    }

    /// Reads a little-endian `i8` (no alignment).
    // qual:api
    pub fn read_i8(&mut self) -> Result<i8, Error> {
        Ok(self.read_u8()? as i8)
    }

    /// Reads an IEEE754 `f32` after alignment.
    // qual:api
    pub fn read_f32(&mut self) -> Result<f32, Error> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    /// Reads an IEEE754 `f64` after alignment.
    // qual:api
    pub fn read_f64(&mut self) -> Result<f64, Error> {
        Ok(f64::from_bits(self.read_u64()?))
    }

    /// Reads a sequence length prefix bounded by the remaining field body bytes.
    pub fn read_bounded_sequence_length(&mut self) -> Result<u32, Error> {
        let length = self.read_u32()?;
        let length_bytes = length as usize;
        if length_bytes > self.remaining_body_bytes() {
            return Err(Error::InvalidLength);
        }
        Ok(length)
    }

    /// Reads a ROS string (length includes the null terminator; no padding after it).
    // qual:api
    pub fn read_string(&mut self) -> Result<String, Error> {
        decode_cdr_string_field(self.read_length_prefixed_field()?)
    }

    fn read_length_prefixed_field(&mut self) -> Result<Vec<u8>, Error> {
        let length = self.read_u32()?;
        if length == 0 {
            return Ok(Vec::new());
        }
        let length_bytes = length as usize;
        if length_bytes > self.remaining_body_bytes() {
            return Err(Error::InvalidLength);
        }
        Ok(self.read_exact(length_bytes)?.to_vec())
    }
}

fn decode_cdr_string_field(field_bytes: Vec<u8>) -> Result<String, Error> {
    match field_bytes.as_slice() {
        [] => Ok(String::new()),
        bytes => Ok(String::from(utf8_without_null_suffix(bytes)?)),
    }
}

fn utf8_without_null_suffix(bytes: &[u8]) -> Result<&str, Error> {
    if bytes.last() != Some(&0) {
        return Err(Error::Utf8);
    }
    core::str::from_utf8(&bytes[..bytes.len() - 1]).map_err(|_| Error::Utf8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_primitives() {
        let mut writer = Writer::new();
        writer.write_bool(true).expect("write bool");
        writer.write_u8(7).expect("write u8");
        writer.write_i8(-3).expect("write i8");
        writer.write_u16(1000).expect("write u16");
        writer.write_i16(-1000).expect("write i16");
        writer.write_u32(42).expect("write u32");
        writer.write_i32(-42).expect("write i32");
        writer.write_u64(9_000_000_000).expect("write u64");
        writer.write_i64(-9_000_000_000).expect("write i64");
        writer.write_f32(1.5).expect("write f32");
        writer.write_f64(-2.5).expect("write f64");
        writer.write_string("hello").expect("write string");
        let mut reader =
            Reader::new_with_encapsulation(&writer.finish_with_encapsulation()).expect("reader");
        assert!(reader.read_bool().expect("read bool"));
        assert_eq!(reader.read_u8().expect("read u8"), 7);
        assert_eq!(reader.read_i8().expect("read i8"), -3);
        assert_eq!(reader.read_u16().expect("read u16"), 1000);
        assert_eq!(reader.read_i16().expect("read i16"), -1000);
        assert_eq!(reader.read_u32().expect("read u32"), 42);
        assert_eq!(reader.read_i32().expect("read i32"), -42);
        assert_eq!(reader.read_u64().expect("read u64"), 9_000_000_000);
        assert_eq!(reader.read_i64().expect("read i64"), -9_000_000_000);
        assert_eq!(reader.read_f32().expect("read f32"), 1.5);
        assert_eq!(reader.read_f64().expect("read f64"), -2.5);
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
    fn bytes_are_written_and_read_whole_without_alignment() {
        let mut writer = Writer::new();
        writer.write_u8(9).expect("write u8");
        writer.write_bytes(&[1, 2, 3]).expect("write bytes");
        let payload = writer.finish_with_encapsulation();
        assert_eq!(payload, [0, 1, 0, 0, 9, 1, 2, 3]);
        let mut reader = Reader::new_with_encapsulation(&payload).expect("reader");
        assert_eq!(reader.read_u8().expect("read u8"), 9);
        assert_eq!(reader.read_bytes(3).expect("read bytes"), [1, 2, 3]);
        assert_eq!(reader.read_bytes(1), Err(Error::UnexpectedEnd));
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

    #[test]
    fn read_string_rejects_length_beyond_remaining() {
        let mut writer = Writer::new();
        writer.write_u32(0xFFFF_FFFF).expect("length");
        let mut reader =
            Reader::new_with_encapsulation(&writer.finish_with_encapsulation()).expect("reader");
        assert_eq!(reader.read_string(), Err(Error::InvalidLength));
    }

    #[test]
    fn read_bounded_sequence_length_rejects_hostile_prefix() {
        let mut writer = Writer::new();
        writer.write_u32(0xFFFF_FFFF).expect("length");
        let mut reader =
            Reader::new_with_encapsulation(&writer.finish_with_encapsulation()).expect("reader");
        assert_eq!(
            reader.read_bounded_sequence_length(),
            Err(Error::InvalidLength)
        );
    }
}
