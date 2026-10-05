//! Partial file reads for growing MCAP recordings.

use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom},
    path::Path,
};

use thiserror::Error;

/// Why a byte range of a recording could not be read. Its text is the reason a client gets.
#[derive(Debug, Error)]
pub enum ReadRangeError {
    /// The range starts after the last byte of the file.
    #[error("Offset {offset} is past the end of the recording ({size} bytes).")]
    OffsetPastEnd {
        /// The requested start of the range.
        offset: u64,
        /// The size of the file when it was read.
        size: u64,
    },
    /// Opening or reading the file failed.
    #[error("Failed to read the recording: {0}")]
    Io(#[from] io::Error),
}

/// Where a range read by [`read_range`] starts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeStart {
    /// At this byte of the file.
    Offset(u64),
    /// As many bytes before the end of the file as the range is long, or at its first byte.
    FromEnd,
}

/// A byte range of a recording, read by [`read_range`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingRange {
    /// The size of the file when it was read; it grows while the recording is written.
    pub size: u64,
    /// The bytes of the range, fewer than asked for at the end of the file.
    pub data: Vec<u8>,
}

/// Reads up to `length` bytes of the recording at `path` from `start`, never past the size it reports, so the
/// range agrees with that size while the file grows.
pub fn read_range(
    path: &Path,
    start: RangeStart,
    length: u64,
) -> Result<RecordingRange, ReadRangeError> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    let offset = match start {
        RangeStart::Offset(offset) => offset,
        RangeStart::FromEnd => size.saturating_sub(length),
    };
    if offset > size {
        return Err(ReadRangeError::OffsetPastEnd { offset, size });
    }
    file.seek(SeekFrom::Start(offset))?;
    let mut data = Vec::new();
    file.take(length.min(size - offset))
        .read_to_end(&mut data)?;
    Ok(RecordingRange { size, data })
}
