//! Rewrites a truncated MCAP into a seekable file (in-process repair).

mod from_reader;
mod open_chunk_reader;

use core::sync::atomic::AtomicBool;
use std::{fs::File, io::BufReader, path::Path};

use thiserror::Error;

pub use from_reader::rewrite_from_reader;

/// Maximum bytes read from the source per `read` call (repair progress granularity).
pub const SOURCE_READ_BYTES: usize = 1024 * 1024;
/// Bytes of a record's opcode and length.
pub(crate) const RECORD_PREFIX_BYTES: usize = 1 + 8;
/// Bytes of a chunk header before its compression name: start and end time, uncompressed size and CRC, and the
/// name's length.
pub(crate) const CHUNK_HEADER_BEFORE_COMPRESSION_BYTES: usize = 8 + 8 + 8 + 4 + 4;
/// The record and compressed length the `mcap` writer gives a chunk until it closes it.
pub(crate) const OPEN_CHUNK_LENGTH: u64 = u64::MAX;

/// Why a rewrite stopped before producing a file.
#[derive(Debug, Error)]
pub enum RewriteError {
    /// The caller asked to stop.
    #[error("rewrite cancelled")]
    Cancelled,
    /// The source does not start with the MCAP magic, so no repair can recover it.
    #[error("not an MCAP file")]
    NotMcap,
    /// The MCAP library reported an error.
    #[error("{0}")]
    Mcap(#[from] mcap::McapError),
    /// A filesystem operation failed.
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

/// Counts from a successful rewrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RewriteSummary {
    /// Messages copied into the output file.
    pub messages: u64,
    /// Bytes read from the source (the repair progress offset).
    pub bytes_read: u64,
}

/// Rewrites `source` into `output`, reporting the read offset through `progress`.
///
/// It keeps every complete message. A source cut short is not an error, so one cut before its first complete
/// message becomes an empty recording; a source that is not an MCAP recording fails.
pub fn rewrite(
    source: &Path,
    output: &Path,
    progress: &mut dyn FnMut(u64, u64),
    cancel: &AtomicBool,
) -> Result<RewriteSummary, RewriteError> {
    let source_file = File::open(source)?;
    let total_bytes = source_file.metadata()?.len();
    let buffered = BufReader::with_capacity(SOURCE_READ_BYTES, source_file);
    rewrite_from_reader(buffered, total_bytes, output, progress, cancel)
}
