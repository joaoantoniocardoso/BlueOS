//! Reader that patches an MCAP chunk header still open on disk.

use std::io::{self, Read};

use mcap::{MAGIC, records::op};

use super::{CHUNK_HEADER_BEFORE_COMPRESSION_BYTES, OPEN_CHUNK_LENGTH, RECORD_PREFIX_BYTES};

/// MCAP record length field (`uint64` LE) after the opcode byte.
const MCAP_RECORD_LENGTH_FIELD_BYTES: usize = 8;
/// MCAP Chunk record compression-name length field (`uint32` LE).
const MCAP_COMPRESSION_NAME_LENGTH_FIELD_BYTES: usize = 4;
/// MCAP Chunk record compressed-size field (`uint64` LE).
const MCAP_CHUNK_COMPRESSED_SIZE_FIELD_BYTES: usize = 8;

/// Passes a recording through, but gives the chunk its writer left open a compressed length that runs to the end
/// of the source. The reader then decompresses the part of that chunk that reached the disk, instead of rejecting
/// a header whose lengths are still [`OPEN_CHUNK_LENGTH`].
pub(crate) struct OpenChunkReader<R> {
    source: R,
    /// Bytes until the next record starts: the start magic at first.
    record_remaining: u64,
    /// Bytes read ahead to inspect a record header, served before the source.
    read_ahead: Vec<u8>,
    /// Set once nothing more needs inspecting: after an open chunk header or the footer.
    passing_through: bool,
}

impl<R: Read> Read for OpenChunkReader<R> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.read_ahead.is_empty() && self.record_remaining == 0 && !self.passing_through {
            self.read_record_header()?;
        }
        if !self.read_ahead.is_empty() {
            let served = buffer.len().min(self.read_ahead.len());
            buffer[..served].copy_from_slice(&self.read_ahead[..served]);
            self.read_ahead.drain(..served);
            return Ok(served);
        }
        if self.passing_through {
            return self.source.read(buffer);
        }
        let limit = usize::try_from(self.record_remaining)
            .map_or(buffer.len(), |remaining| remaining.min(buffer.len()));
        let read = self.source.read(&mut buffer[..limit])?;
        self.record_remaining -= read as u64;
        Ok(read)
    }
}

impl<R: Read> OpenChunkReader<R> {
    pub(super) fn new(source: R) -> Self {
        Self {
            source,
            record_remaining: MAGIC.len() as u64,
            read_ahead: Vec::new(),
            passing_through: false,
        }
    }

    /// Reads the next record's opcode and length ahead, and the chunk header too when the chunk is still open.
    fn read_record_header(&mut self) -> io::Result<()> {
        if !self.read_ahead_to(RECORD_PREFIX_BYTES)? {
            self.passing_through = true;
            return Ok(());
        }
        let opcode = self.read_ahead[0];
        let length = le_u64_from_slice(&self.read_ahead[1..RECORD_PREFIX_BYTES]);
        if opcode == op::FOOTER {
            self.passing_through = true;
            return Ok(());
        }
        if opcode != op::CHUNK || length != OPEN_CHUNK_LENGTH {
            self.record_remaining = length;
            return Ok(());
        }
        self.passing_through = true;
        let compression_end = RECORD_PREFIX_BYTES + CHUNK_HEADER_BEFORE_COMPRESSION_BYTES;
        if !self.read_ahead_to(compression_end)? {
            return Ok(());
        }
        let compression_length = u32::from_le_bytes(le_u32_bytes(
            &self.read_ahead[compression_end - 4..compression_end],
        ));
        let compressed_size_start = compression_end + compression_length as usize;
        let header_end = compressed_size_start + MCAP_CHUNK_COMPRESSED_SIZE_FIELD_BYTES;
        if !self.read_ahead_to(header_end)? {
            return Ok(());
        }
        let header_bytes = (header_end - RECORD_PREFIX_BYTES) as u64;
        self.read_ahead[compressed_size_start..header_end]
            .copy_from_slice(&(OPEN_CHUNK_LENGTH - header_bytes).to_le_bytes());
        Ok(())
    }

    /// Reads ahead until `length` bytes are held; `false` when the source ends first.
    fn read_ahead_to(&mut self, length: usize) -> io::Result<bool> {
        let missing = length.saturating_sub(self.read_ahead.len());
        (&mut self.source)
            .take(missing as u64)
            .read_to_end(&mut self.read_ahead)?;
        Ok(self.read_ahead.len() >= length)
    }
}

fn le_u64_from_slice(slice: &[u8]) -> u64 {
    let mut length = [0_u8; MCAP_RECORD_LENGTH_FIELD_BYTES];
    length.copy_from_slice(slice);
    u64::from_le_bytes(length)
}

fn le_u32_bytes(slice: &[u8]) -> [u8; MCAP_COMPRESSION_NAME_LENGTH_FIELD_BYTES] {
    let mut bytes = [0_u8; MCAP_COMPRESSION_NAME_LENGTH_FIELD_BYTES];
    bytes.copy_from_slice(slice);
    bytes
}
