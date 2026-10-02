//! The recordings folder: validated paths, scan, delete, and wall-clock file names.

#![expect(
    clippy::std_instead_of_core,
    reason = "filesystem and wall-clock adapters use the standard library"
)]

use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs, io,
    path::{Component, Path, PathBuf},
    time::{Duration, UNIX_EPOCH},
};

use thiserror::Error;
use tracing::warn;
use walkdir::WalkDir;

use blueos_recorder_mcap::read_footer_at;

const RECORDING_SUFFIX: &str = ".mcap";
const RECOVER_SUFFIX: &str = ".recover";
const LIBRARY_SCAN_MAX_DEPTH: usize = 8;

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
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct FooterCacheKey {
    inode: u64,
    size: u64,
    modified_unix_seconds: i64,
}

/// Footer read cache for library scans (not shared with the data plane).
#[derive(Clone, Debug, Default)]
pub struct LibraryFooterCache {
    entries: HashMap<PathBuf, (FooterCacheKey, bool)>,
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
        folder.discard_recover_files();
        Ok(folder)
    }

    /// Removes leftover `.recover` files under the recordings folder, including nested ones.
    pub fn discard_recover_files(&self) {
        let mut removed = Vec::new();
        self.discard_recover_files_in(&self.base, &mut removed);
        for (relative, size_bytes) in removed {
            tracing::info!(
                path = %relative,
                size_bytes,
                "Discarded leftover repair temporary file"
            );
        }
    }

    /// Path for a repair rewrite next to `relative` (`<stem>.recover`).
    pub fn recover_temporary_path(&self, relative: &str) -> PathBuf {
        let path = Path::new(relative);
        let parent = path.parent().unwrap_or_else(|| Path::new(""));
        let stem = path
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("recording");
        let temporary_name = format!("{stem}{RECOVER_SUFFIX}");
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

    /// Lists `.mcap` files under the folder, skipping footer reads for the active relative path.
    pub fn scan_library(
        &self,
        active_recording_relative_path: Option<&str>,
        footer_cache: &mut LibraryFooterCache,
    ) -> Result<Vec<ScannedRecordingFile>, StorageError> {
        let mut recordings = Vec::new();
        let mut listed_paths = Vec::new();
        for entry in WalkDir::new(&self.base)
            .follow_links(false)
            .max_depth(LIBRARY_SCAN_MAX_DEPTH)
            .into_iter()
        {
            let entry = match entry {
                Ok(value) => value,
                Err(error) => {
                    warn!(%error, base = %self.base.display(), "Library walk failed");
                    continue;
                }
            };
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let name = path.file_name().and_then(OsStr::to_str).unwrap_or("");
            if !name.to_ascii_lowercase().ends_with(RECORDING_SUFFIX) {
                continue;
            }
            let metadata = match fs::metadata(path) {
                Ok(value) => value,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error.into()),
            };
            let relative_path = path
                .strip_prefix(&self.base)
                .map(relative_path_string)
                .unwrap_or_else(|_| name.to_string());
            listed_paths.push(path.to_path_buf());
            let skip_footer = active_recording_relative_path == Some(relative_path.as_str())
                || active_recording_relative_path == Some(name);
            let indexed = footer_indexed(path, &metadata, skip_footer, footer_cache)?;
            recordings.push(ScannedRecordingFile {
                relative_path,
                name: name.to_string(),
                size_bytes: metadata.len(),
                modified_unix_seconds: modified_unix_seconds(&metadata),
                indexed,
            });
        }
        prune_footer_cache(&mut footer_cache.entries, &listed_paths);
        Ok(recordings)
    }

    /// Resolves a relative recording path under `base`.
    pub fn resolve(&self, relative: &str) -> Result<PathBuf, StorageError> {
        validate_relative_recording_path(relative)?;
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

    fn discard_recover_files_in(&self, directory: &Path, removed: &mut Vec<(String, u64)>) {
        let entries = match fs::read_dir(directory) {
            Ok(value) => value,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.discard_recover_files_in(&path, removed);
                continue;
            }
            let name = entry.file_name();
            if !name.to_string_lossy().ends_with(RECOVER_SUFFIX) {
                continue;
            }
            let relative = path
                .strip_prefix(&self.base)
                .map(relative_path_string)
                .unwrap_or_else(|_| name.to_string_lossy().into_owned());
            let size_bytes = entry.metadata().map(|metadata| metadata.len()).unwrap_or(0);
            if fs::remove_file(&path).is_ok() {
                removed.push((relative, size_bytes));
            }
        }
    }
}

fn validate_relative_recording_path(relative: &str) -> Result<(), StorageError> {
    if relative.is_empty() || relative.starts_with('/') || Path::new(relative).is_absolute() {
        return Err(StorageError::InvalidPath);
    }
    if relative.contains('\0') {
        return Err(StorageError::InvalidPath);
    }
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(segment) => {
                if segment.is_empty() || segment == "." || segment == ".." {
                    return Err(StorageError::InvalidPath);
                }
            }
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(StorageError::InvalidPath);
            }
            Component::CurDir => return Err(StorageError::InvalidPath),
        }
    }
    if !relative.to_ascii_lowercase().ends_with(RECORDING_SUFFIX) {
        return Err(StorageError::InvalidPath);
    }
    Ok(())
}

fn footer_indexed(
    path: &Path,
    metadata: &fs::Metadata,
    skip_footer: bool,
    footer_cache: &mut LibraryFooterCache,
) -> Result<bool, StorageError> {
    if skip_footer {
        return Ok(false);
    }
    let cache_key = FooterCacheKey {
        inode: inode(metadata),
        size: metadata.len(),
        modified_unix_seconds: modified_unix_seconds(metadata),
    };
    let cache_path = path.to_path_buf();
    if let Some((cached_key, indexed)) = footer_cache.entries.get(&cache_path)
        && *cached_key == cache_key
    {
        return Ok(*indexed);
    }
    let indexed = read_footer_at(path, metadata.len())
        .map_err(StorageError::Io)?
        .is_some_and(|footer| footer.summary_start > 0);
    footer_cache
        .entries
        .insert(cache_path, (cache_key, indexed));
    Ok(indexed)
}

fn relative_path_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn prune_footer_cache(cache: &mut HashMap<PathBuf, (FooterCacheKey, bool)>, keep: &[PathBuf]) {
    let keep_set: HashSet<&PathBuf> = keep.iter().collect();
    cache.retain(|path, _| keep_set.contains(path));
}

#[cfg(unix)]
fn inode(metadata: &fs::Metadata) -> u64 {
    use std::os::unix::fs::MetadataExt;
    metadata.ino()
}

#[cfg(not(unix))]
fn inode(_metadata: &fs::Metadata) -> u64 {
    0
}

fn modified_unix_seconds(metadata: &fs::Metadata) -> i64 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .and_then(|duration| i64::try_from(duration.as_secs()).ok())
        .unwrap_or(0)
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

    #[test]
    fn resolve_rejects_parent_dir() {
        let temporary = tempfile::tempdir().expect("tempdir");
        let folder = RecordingsFolder::new(temporary.path().to_path_buf()).expect("folder");
        assert!(matches!(
            folder.resolve("../outside.mcap"),
            Err(StorageError::InvalidPath)
        ));
    }
}
