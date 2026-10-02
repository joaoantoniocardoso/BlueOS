//! MCAP footer read for library indexing checks.

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
};

const MCAP_MAGIC: [u8; 8] = [137, 77, 67, 65, 80, 48, 13, 10];
const MAGIC_SIZE: usize = MCAP_MAGIC.len();
const RECORD_HEADER_SIZE: usize = 9;
const FOOTER_RECORD_SIZE: usize = RECORD_HEADER_SIZE + 20;
const FOOTER_OPCODE: u8 = 0x02;

/// Parsed MCAP footer fields used to detect a seekable summary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Footer {
    /// Byte offset of the summary section; zero when missing.
    pub summary_start: u64,
    /// Byte offset of the summary-offset section.
    pub summary_offset_start: u64,
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
    if payload.len() < 20 {
        return Ok(None);
    }
    let summary_start = u64::from_le_bytes(payload[0..8].try_into().expect("summary_start"));
    let summary_offset_start =
        u64::from_le_bytes(payload[8..16].try_into().expect("summary_offset_start"));
    Ok(Some(Footer {
        summary_start,
        summary_offset_start,
    }))
}
