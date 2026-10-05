//! Library scan and footer summary cache.

use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use tracing::warn;
use walkdir::WalkDir;

use blueos_recorder_mcap::{read_footer_at, read_recording_contents};

use super::{
    FooterReading, LIBRARY_SCAN_MAX_DEPTH, LibraryFooterCache, RECORDING_SUFFIX, RecordingsFolder,
    ScannedRecordingFile, StorageError, relative_path_string,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct FooterCacheKey {
    inode: u64,
    size: u64,
    modified_unix_seconds: i64,
}

pub(crate) fn scan_library(
    folder: &RecordingsFolder,
    active_recording_relative_path: Option<&str>,
    footer_cache: &mut LibraryFooterCache,
) -> Result<Vec<ScannedRecordingFile>, StorageError> {
    let mut recordings = Vec::new();
    let mut listed_paths = Vec::new();
    for entry in WalkDir::new(folder.base())
        .follow_links(false)
        .max_depth(LIBRARY_SCAN_MAX_DEPTH)
        .into_iter()
    {
        let entry = match entry {
            Ok(value) => value,
            Err(error) => {
                warn!(%error, base = %folder.base().display(), "Library walk failed");
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
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        let relative_path = path
            .strip_prefix(folder.base())
            .map(relative_path_string)
            .unwrap_or_else(|_| name.to_string());
        listed_paths.push(path.to_path_buf());
        let skip_footer = active_recording_relative_path == Some(relative_path.as_str())
            || active_recording_relative_path == Some(name);
        let FooterReading { indexed, contents } =
            read_footer_cached(path, &metadata, skip_footer, footer_cache)?;
        recordings.push(ScannedRecordingFile {
            relative_path,
            name: name.to_string(),
            size_bytes: metadata.len(),
            modified_unix_seconds: modified_unix_seconds(&metadata),
            indexed,
            contents,
        });
    }
    prune_footer_cache(&mut footer_cache.entries, &listed_paths);
    Ok(recordings)
}

fn read_footer_cached(
    path: &Path,
    metadata: &fs::Metadata,
    skip_footer: bool,
    footer_cache: &mut LibraryFooterCache,
) -> Result<FooterReading, StorageError> {
    if skip_footer {
        return Ok(FooterReading::default());
    }
    let cache_key = FooterCacheKey {
        inode: inode(metadata),
        size: metadata.len(),
        modified_unix_seconds: modified_unix_seconds(metadata),
    };
    let cache_path = path.to_path_buf();
    if let Some((cached_key, reading)) = footer_cache.entries.get(&cache_path)
        && *cached_key == cache_key
    {
        return Ok(reading.clone());
    }
    let indexed = read_footer_at(path, metadata.len())
        .map_err(StorageError::Io)?
        .is_some_and(|footer| footer.summary_start > 0);
    let contents = indexed_recording_contents(path, indexed);
    let reading = FooterReading { indexed, contents };
    footer_cache
        .entries
        .insert(cache_path, (cache_key, reading.clone()));
    Ok(reading)
}

fn indexed_recording_contents(path: &Path, indexed: bool) -> Option<super::RecordingContents> {
    if !indexed {
        return None;
    }
    read_recording_contents(path).unwrap_or_else(|error| {
        warn!(%error, path = %path.display(), "Failed to read the recording summary");
        None
    })
}

fn prune_footer_cache(
    cache: &mut HashMap<PathBuf, (FooterCacheKey, FooterReading)>,
    keep: &[PathBuf],
) {
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
