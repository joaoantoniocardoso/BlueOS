//! The recordings folder: validated paths and wall-clock file names.

use core::time::Duration;
use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};

use thiserror::Error;

const RECORDING_SUFFIX: &str = ".mcap";

/// Errors from the recordings folder adapter.
#[derive(Debug, Error)]
pub enum StorageError {
    /// The path is not a valid recording relative path.
    #[error("invalid recording path")]
    InvalidPath,
    /// A filesystem operation failed.
    #[error("filesystem operation failed")]
    Io(#[from] io::Error),
    /// No unused file name was found.
    #[error("could not allocate a unique recording file name")]
    NameCollision,
}

/// Where MCAP files are stored on disk.
pub struct RecordingsFolder {
    base: PathBuf,
}

impl RecordingsFolder {
    /// Creates `base` when missing and canonicalizes it.
    pub fn new(base: PathBuf) -> Result<Self, StorageError> {
        fs::create_dir_all(&base)?;
        Ok(Self {
            base: base.canonicalize()?,
        })
    }

    /// Picks a new `recorder_YYYYMMDD_HHMMSS.mcap` path that does not exist yet.
    pub fn allocate_new_recording(
        &self,
        wall_clock: Duration,
    ) -> Result<(PathBuf, String), StorageError> {
        let stamp = format_wall_timestamp(wall_clock);
        for suffix in 0u32..100 {
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
        validate_relative_path(relative)?;
        let path = self.base.join(relative);
        if !path.starts_with(&self.base) {
            return Err(StorageError::InvalidPath);
        }
        Ok(path)
    }

    /// Base directory path.
    pub fn base(&self) -> &Path {
        &self.base
    }
}

fn validate_relative_path(relative: &str) -> Result<(), StorageError> {
    if relative.is_empty() || relative.starts_with('/') || Path::new(relative).is_absolute() {
        return Err(StorageError::InvalidPath);
    }
    if !relative.to_ascii_lowercase().ends_with(RECORDING_SUFFIX) {
        return Err(StorageError::InvalidPath);
    }
    for component in Path::new(relative).components() {
        if matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        ) {
            return Err(StorageError::InvalidPath);
        }
    }
    Ok(())
}

/// Formats wall-clock seconds as `YYYYMMDD_HHMMSS` (UTC).
fn format_wall_timestamp(wall_clock: Duration) -> String {
    let seconds = wall_clock.as_secs();
    let time_of_day = seconds % 86_400;
    let hour = time_of_day / 3_600;
    let minute = (time_of_day % 3_600) / 60;
    let second = time_of_day % 60;
    let days = seconds / 86_400;
    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}{month:02}{day:02}_{hour:02}{minute:02}{second:02}")
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = era * 400 + yoe as i64 + (yoe == 4) as i64;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp as i64 + if mp < 10 { 3 } else { -9 };
    let year = year + (month <= 2) as i64;
    (year, month, day as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recorder_prefix_matches_library_parser() {
        let stamp = format_wall_timestamp(Duration::from_secs(1_767_225_600));
        assert_eq!(stamp, "20260101_000000");
    }
}
