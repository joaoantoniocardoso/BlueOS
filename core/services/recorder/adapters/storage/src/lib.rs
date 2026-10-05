//! The recordings folder: validated paths, scan, delete, and wall-clock file names.

#![expect(
    clippy::pub_use,
    reason = "adapter crate re-exports range read types at the root"
)]

mod discard;
mod path_validation;
mod read_range;
mod scan;

use core::sync::atomic::{AtomicU64, Ordering};
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::{self},
    io,
    path::{Path, PathBuf},
};

use thiserror::Error;

use blueos_recorder_mcap::RecordingContents;

pub use read_range::{RangeStart, ReadRangeError, RecordingRange, read_range};

pub(crate) const RECORDING_SUFFIX: &str = ".mcap";
pub(crate) const RECOVER_SUFFIX: &str = ".recover";
pub(crate) const SNAPSHOT_PARTIAL_SUFFIX: &str = ".partial";
pub(crate) const LIBRARY_SCAN_MAX_DEPTH: usize = 8;
/// How many `recorder_<stamp>_<n>.mcap` suffixes to try before giving up.
const RECORDING_NAME_COLLISION_ATTEMPTS: u32 = 100;

/// Numbers each rewrite's temporary file, so two rewrites of one recording never share one, even when a restarted
/// Task starts a rewrite again while the old one still winds down.
static NEXT_REWRITE: AtomicU64 = AtomicU64::new(0);

/// Errors from the recordings folder adapter.
#[derive(Debug, Error)]
pub enum StorageError {
    /// The path is not a valid recording relative path.
    #[error("invalid recording path")]
    InvalidPath,
    /// The recording file is not in the library folder.
    #[error("recording not found")]
    NotFound,
    /// A filesystem operation failed.
    #[error("filesystem operation failed")]
    Io(#[from] io::Error),
    /// No unused file name was found.
    #[error("could not allocate a unique recording file name")]
    NameCollision,
}

/// One MCAP file discovered during a library scan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScannedRecordingFile {
    /// Path relative to the recordings folder, forward slashes.
    pub relative_path: String,
    /// Base file name.
    pub name: String,
    /// Size in bytes at scan time.
    pub size_bytes: u64,
    /// File modification time as Unix seconds.
    pub modified_unix_seconds: i64,
    /// Whether the MCAP summary is present (seekable).
    pub indexed: bool,
    /// What the recording holds, from its summary; `None` when it has none or it could not be read.
    pub contents: Option<RecordingContents>,
}

/// Footer and summary read cache for library scans (not shared with the data plane).
#[derive(Clone, Debug, Default)]
pub struct LibraryFooterCache {
    pub(crate) entries: HashMap<PathBuf, (scan::FooterCacheKey, FooterReading)>,
}

/// What a scan read from one file's footer and summary.
#[derive(Clone, Debug, Default)]
pub(crate) struct FooterReading {
    indexed: bool,
    contents: Option<RecordingContents>,
}

/// Where MCAP files are stored on disk.
pub struct RecordingsFolder {
    base: PathBuf,
}

impl RecordingsFolder {
    /// Creates `base` when missing and canonicalizes it.
    pub fn new(base: PathBuf) -> Result<Self, StorageError> {
        fs::create_dir_all(&base)?;
        let folder = Self {
            base: base.canonicalize()?,
        };
        discard::discard_recover_files(&folder);
        discard::discard_snapshot_partial_files(&folder);
        Ok(folder)
    }

    /// A new path for a repair rewrite next to `relative` (`<stem>.<rewrite>.recover`).
    pub fn recover_temporary_path(&self, relative: &str) -> PathBuf {
        let path = Path::new(relative);
        let parent = path.parent().unwrap_or_else(|| Path::new(""));
        let stem = path
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("recording");
        let rewrite = NEXT_REWRITE.fetch_add(1, Ordering::Relaxed);
        let temporary_name = format!("{stem}.{rewrite}{RECOVER_SUFFIX}");
        self.base.join(parent).join(temporary_name)
    }

    /// Renames a finished repair temporary file over the recording.
    pub fn replace_recording_from_temporary(
        &self,
        temporary: &Path,
        relative: &str,
    ) -> Result<(), StorageError> {
        let destination = self.resolve(relative)?;
        fs::rename(temporary, destination)?;
        Ok(())
    }

    /// A new path for a snapshot rewrite before it is renamed into place (`<output>.<rewrite>.partial`).
    pub fn snapshot_temporary_path(&self, output_relative: &str) -> PathBuf {
        let rewrite = NEXT_REWRITE.fetch_add(1, Ordering::Relaxed);
        self.base.join(format!(
            "{output_relative}.{rewrite}{SNAPSHOT_PARTIAL_SUFFIX}"
        ))
    }

    /// Renames a finished snapshot temporary file into the library folder.
    pub fn finalize_snapshot(
        &self,
        temporary: &Path,
        output_relative: &str,
    ) -> Result<(), StorageError> {
        path_validation::validate_relative_recording_path(output_relative)?;
        let destination = self.base.join(output_relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(temporary, destination)?;
        Ok(())
    }

    /// Picks a new `recorder_YYYYMMDD_HHMMSS.mcap` path that does not exist yet.
    pub fn allocate_new_recording(
        &self,
        wall_clock_stamp: &str,
    ) -> Result<(PathBuf, String), StorageError> {
        let stamp = wall_clock_stamp;
        for suffix in 0u32..RECORDING_NAME_COLLISION_ATTEMPTS {
            let file_name = if suffix == 0 {
                format!("recorder_{stamp}.mcap")
            } else {
                format!("recorder_{stamp}_{suffix}.mcap")
            };
            let path = self.base.join(&file_name);
            if path.exists() {
                continue;
            }
            return Ok((path, file_name));
        }
        Err(StorageError::NameCollision)
    }

    /// Resolves a relative recording path under `base`.
    pub fn resolve(&self, relative: &str) -> Result<PathBuf, StorageError> {
        path_validation::validate_relative_recording_path(relative)?;
        let candidate = self.base.join(relative);
        if !candidate.starts_with(&self.base) {
            return Err(StorageError::InvalidPath);
        }
        let canonical = match candidate.canonicalize() {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(StorageError::NotFound);
            }
            Err(_) => return Err(StorageError::InvalidPath),
        };
        if !canonical.starts_with(&self.base) {
            return Err(StorageError::InvalidPath);
        }
        if canonical.is_dir() {
            return Err(StorageError::InvalidPath);
        }
        let metadata = match fs::metadata(&canonical) {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(StorageError::NotFound);
            }
            Err(_) => return Err(StorageError::InvalidPath),
        };
        if !metadata.is_file() {
            return Err(StorageError::InvalidPath);
        }
        Ok(canonical)
    }

    /// Deletes a recording file when it exists under `base`.
    pub fn delete_recording(&self, relative: &str) -> Result<(), StorageError> {
        let path = self.resolve(relative)?;
        fs::remove_file(path)?;
        Ok(())
    }

    /// Base directory path.
    pub fn base(&self) -> &Path {
        &self.base
    }
}

/// Lists `.mcap` files under the folder, skipping footer reads for the active relative path.
pub fn scan_recordings_library(
    folder: &RecordingsFolder,
    active_recording_relative_path: Option<&str>,
    footer_cache: &mut LibraryFooterCache,
) -> Result<Vec<ScannedRecordingFile>, StorageError> {
    scan::scan_library(folder, active_recording_relative_path, footer_cache)
}

pub(crate) fn relative_path_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
