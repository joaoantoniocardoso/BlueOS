//! Paged MCAP chunk-index walk for the recording `index` IO query.

mod records;
mod stream;
mod walker;

use core::sync::atomic::{AtomicBool, Ordering};
use std::{
    fs::File,
    io::{Read, Seek},
    path::Path,
};

use thiserror::Error;

use blueos_idl::msg::blueos_recorder_msgs::RecordingIndexResponse;

use walker::McapIndexWalker;

const MIN_LIMIT: u32 = 1;
const MAX_LIMIT: u32 = 20_000;

/// Why an index walk failed.
#[derive(Debug, Error)]
pub enum IndexError {
    /// The file does not start with MCAP magic.
    #[error("Invalid MCAP magic.")]
    InvalidMagic,
    /// `limit` is outside the allowed page size range.
    #[error("Index page limit must be between {MIN_LIMIT} and {MAX_LIMIT}.")]
    LimitOutOfRange,
    /// A read or seek on the recording failed.
    #[error("{0}")]
    Io(#[from] std::io::Error),
    /// The walk was cancelled before it finished.
    #[error("Recording index walk cancelled.")]
    Cancelled,
}

fn validate_page_limit(limit: u32) -> Result<(), IndexError> {
    if (MIN_LIMIT..=MAX_LIMIT).contains(&limit) {
        Ok(())
    } else {
        Err(IndexError::LimitOutOfRange)
    }
}

fn walk_until_stopped<R: Read + Seek>(
    walker: &mut McapIndexWalker,
    recording: &mut R,
    limit: u32,
    cancel: &AtomicBool,
) -> Result<(), IndexError> {
    while !cancel.load(Ordering::Relaxed) {
        if !walker.walk_record(recording, limit)? {
            break;
        }
    }
    if cancel.load(Ordering::Relaxed) {
        return Err(IndexError::Cancelled);
    }
    Ok(())
}

/// Walks one page of the MCAP index for `path`, using the file size seen at the start of the walk.
pub fn walk_index(
    path: &Path,
    from_offset: u64,
    limit: u32,
    cancel: &AtomicBool,
) -> Result<RecordingIndexResponse, IndexError> {
    validate_page_limit(limit)?;
    let size = std::fs::metadata(path)?.len();
    let mut recording = File::open(path)?;
    walk_index_reader(&mut recording, size, from_offset, limit, cancel)
}

/// Walks one page over `recording` with a caller-supplied `size` (bytes visible at walk start).
pub fn walk_index_reader<R: Read + Seek>(
    recording: &mut R,
    size: u64,
    from_offset: u64,
    limit: u32,
    cancel: &AtomicBool,
) -> Result<RecordingIndexResponse, IndexError> {
    validate_page_limit(limit)?;
    let mut walker = McapIndexWalker::new(size, from_offset);
    if from_offset == 0 {
        walker.validate_magic(recording)?;
    }
    walk_until_stopped(&mut walker, recording, limit, cancel)?;
    Ok(walker.into_index())
}
