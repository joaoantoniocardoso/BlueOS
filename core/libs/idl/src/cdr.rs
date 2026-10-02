//! Little-endian CDR writers and readers for ROS 2 message payloads.

#![expect(
    clippy::arbitrary_source_item_ordering,
    reason = "CDR helpers follow wire layout, not alphabetical order"
)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::error::Error;

const ENCAPSULATION_CDR_LE: [u8; 4] = [0x00, 0x01, 0x00, 0x00];

/// Incremental CDR writer for message field bodies (no encapsulation header).
pub struct Writer {
    buffer: Vec<u8>,
}

impl Default for Writer {
    fn default() -> Self {
        Self::new()
    }
}

/// Incremental CDR reader for message field bodies.
pub struct Reader {
    buffer: Vec<u8>,
    position: usize,
}

impl Writer {
    /// Creates an empty writer.
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    /// Prepends the standard little-endian CDR encapsulation header.
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

    /// Writes one byte (no alignment).
    pub fn write_u8(&mut self, value: u8) -> Result<(), Error> {
        self.buffer.push(value);
        Ok(())
    }

    /// Writes a CDR bool as `0` or `1`.
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
    pub fn write_i16(&mut self, value: i16) -> Result<(), Error> {
        self.write_u16(value as u16)
    }

    /// Writes a little-endian `u32` after alignment.
    pub fn write_u32(&mut self, value: u32) -> Result<(), Error> {
        self.align(4);
        self.buffer.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Writes a little-endian `i32` after alignment.
    pub fn write_i32(&mut self, value: i32) -> Result<(), Error> {
        self.write_u32(value as u32)
    }

    /// Writes a little-endian `u64` after alignment.
    pub fn write_u64(&mut self, value: u64) -> Result<(), Error> {
        self.align(8);
        self.buffer.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Writes a little-endian `i64` after alignment.
    pub fn write_i64(&mut self, value: i64) -> Result<(), Error> {
        self.write_u64(value as u64)
    }

    /// Writes a little-endian `i8` (no alignment).
    pub fn write_i8(&mut self, value: i8) -> Result<(), Error> {
        self.write_u8(value as u8)
    }

    /// Writes an IEEE754 `f32` after alignment.
    pub fn write_f32(&mut self, value: f32) -> Result<(), Error> {
        self.write_u32(value.to_bits())
    }

    /// Writes an IEEE754 `f64` after alignment.
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

    /// Reads a field body without an encapsulation header (tests and nested use).
    pub fn new_body(payload: &[u8]) -> Self {
        Self {
            buffer: payload.to_vec(),
            position: 0,
        }
    }

    /// Returns whether every byte of the field body has been consumed.
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.buffer.len()
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

    /// Reads a CDR bool (`0` or `1`).
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
    pub fn read_i16(&mut self) -> Result<i16, Error> {
        Ok(self.read_u16()? as i16)
    }

    /// Reads a little-endian `u32` after alignment.
    pub fn read_u32(&mut self) -> Result<u32, Error> {
        self.align(4)?;
        let bytes = self.read_exact(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Reads a little-endian `i32` after alignment.
    pub fn read_i32(&mut self) -> Result<i32, Error> {
        Ok(self.read_u32()? as i32)
    }

    /// Reads a little-endian `u64` after alignment.
    pub fn read_u64(&mut self) -> Result<u64, Error> {
        self.align(8)?;
        let bytes = self.read_exact(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    /// Reads a little-endian `i64` after alignment.
    pub fn read_i64(&mut self) -> Result<i64, Error> {
        Ok(self.read_u64()? as i64)
    }

    /// Reads a little-endian `i8` (no alignment).
    pub fn read_i8(&mut self) -> Result<i8, Error> {
        Ok(self.read_u8()? as i8)
    }

    /// Reads an IEEE754 `f32` after alignment.
    pub fn read_f32(&mut self) -> Result<f32, Error> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    /// Reads an IEEE754 `f64` after alignment.
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
    pub fn read_string(&mut self) -> Result<String, Error> {
        let length = self.read_u32()?;
        if length == 0 {
            return Ok(String::new());
        }
        let length_bytes = length as usize;
        if length_bytes > self.remaining_body_bytes() {
            return Err(Error::InvalidLength);
        }
        let bytes = self.read_exact(length_bytes)?;
        if bytes.last() != Some(&0) {
            return Err(Error::Utf8);
        }
        let text = core::str::from_utf8(&bytes[..length_bytes - 1]).map_err(|_| Error::Utf8)?;
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
