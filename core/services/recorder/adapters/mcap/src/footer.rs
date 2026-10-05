//! MCAP footer read for library indexing checks.

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
};

/// MCAP file magic bytes at the start and end of a file.
pub const MCAP_MAGIC: [u8; 8] = [137, 77, 67, 65, 80, 48, 13, 10];
const MAGIC_SIZE: usize = MCAP_MAGIC.len();
const RECORD_HEADER_SIZE: usize = 9;
/// MCAP Footer record payload: summary start, summary offset start, summary CRC (LE `uint64`, `uint64`, `uint32`).
const FOOTER_RECORD_PAYLOAD_BYTES: usize = 8 + 8 + 4;
const FOOTER_RECORD_SIZE: usize = RECORD_HEADER_SIZE + FOOTER_RECORD_PAYLOAD_BYTES;
const FOOTER_OPCODE: u8 = 0x02;
const MCAP_FOOTER_U64_FIELD_BYTES: usize = 8;

/// Parsed MCAP footer fields used to detect a seekable summary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Footer {
    /// Byte offset of the summary section; zero when missing.
    pub summary_start: u64,
    /// Byte offset of the summary-offset section.
    pub summary_offset_start: u64,
}

/// Returns whether `path` has a non-zero MCAP summary (seekable).
// qual:api
pub fn is_indexed(path: &Path) -> bool {
    let size = match std::fs::metadata(path) {
        Ok(metadata) => metadata.len(),
        Err(_) => return false,
    };
    read_footer_at(path, size)
        .ok()
        .flatten()
        .is_some_and(|footer| footer.summary_start > 0)
}

/// Reads the footer at the end of an MCAP file of `size` bytes.
pub fn read_footer_at(path: &Path, size: u64) -> io::Result<Option<Footer>> {
    if size < (MAGIC_SIZE as u64 * 2) + FOOTER_RECORD_SIZE as u64 {
        return Ok(None);
    }
    let mut recording = File::open(path)?;
    recording.seek(SeekFrom::Start(
        size - MAGIC_SIZE as u64 - FOOTER_RECORD_SIZE as u64,
    ))?;
    let mut footer_bytes = vec![0_u8; FOOTER_RECORD_SIZE + MAGIC_SIZE];
    recording.read_exact(&mut footer_bytes)?;
    parse_footer_bytes(&footer_bytes)
}

fn parse_footer_bytes(footer_bytes: &[u8]) -> io::Result<Option<Footer>> {
    if footer_bytes.len() != FOOTER_RECORD_SIZE + MAGIC_SIZE {
        return Ok(None);
    }
    if footer_bytes[0] != FOOTER_OPCODE {
        return Ok(None);
    }
    if footer_bytes[footer_bytes.len() - MAGIC_SIZE..] != MCAP_MAGIC {
        return Ok(None);
    }
    let payload = &footer_bytes[RECORD_HEADER_SIZE..FOOTER_RECORD_SIZE];
    if payload.len() < FOOTER_RECORD_PAYLOAD_BYTES {
        return Ok(None);
    }
    let summary_start = match le_u64_at(payload, 0) {
        Some(value) => value,
        None => return Ok(None),
    };
    let summary_offset_start = match le_u64_at(payload, MCAP_FOOTER_U64_FIELD_BYTES) {
        Some(value) => value,
        None => return Ok(None),
    };
    Ok(Some(Footer {
        summary_start,
        summary_offset_start,
    }))
}

fn le_u64_at(payload: &[u8], offset: usize) -> Option<u64> {
    let end = offset + MCAP_FOOTER_U64_FIELD_BYTES;
    let slice = payload.get(offset..end)?;
    let bytes: [u8; MCAP_FOOTER_U64_FIELD_BYTES] = slice.try_into().ok()?;
    Some(u64::from_le_bytes(bytes))
}
