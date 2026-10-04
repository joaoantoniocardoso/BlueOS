//! The recordings folder: validated paths, scan, delete, and wall-clock file names.

use core::sync::atomic::{AtomicU64, Ordering};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom},
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

use thiserror::Error;
use tracing::warn;
use walkdir::WalkDir;

use blueos_recorder_mcap::{RecordingContents, read_footer_at, read_recording_contents};

const RECORDING_SUFFIX: &str = ".mcap";
const RECOVER_SUFFIX: &str = ".recover";
const SNAPSHOT_PARTIAL_SUFFIX: &str = ".partial";
const LIBRARY_SCAN_MAX_DEPTH: usize = 8;

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

/// A byte range of a recording, read by [`read_range`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordingRange {
    /// The size of the file when it was read; it grows while the recording is written.
    pub size: u64,
    /// The bytes of the range, fewer than asked for at the end of the file.
    pub data: Vec<u8>,
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct FooterCacheKey {
    inode: u64,
    size: u64,
    modified_unix_seconds: i64,
}

/// Footer and summary read cache for library scans (not shared with the data plane).
#[derive(Clone, Debug, Default)]
pub struct LibraryFooterCache {
    entries: HashMap<PathBuf, (FooterCacheKey, FooterReading)>,
}

/// What a scan read from one file's footer and summary.
#[derive(Clone, Debug, Default)]
struct FooterReading {
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
        folder.discard_recover_files();
        folder.discard_snapshot_partial_files();
        Ok(folder)
    }

    /// Removes leftover `.recover` files under the recordings folder, including nested ones.
    pub fn discard_recover_files(&self) {
        let mut removed = Vec::new();
        self.discard_recover_files_in(&self.base, &mut removed);
        removed.into_iter().for_each(|(relative, size_bytes)| {
            tracing::info!(
                path = %relative,
                size_bytes,
                "Discarded leftover repair temporary file"
            );
        });
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
        validate_relative_recording_path(output_relative)?;
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

    /// Removes leftover snapshot `.partial` files under the recordings folder.
    pub fn discard_snapshot_partial_files(&self) {
        let mut removed = Vec::new();
        self.discard_snapshot_partial_files_in(&self.base, &mut removed);
        for (relative, size_bytes) in removed {
            tracing::info!(
                path = %relative,
                size_bytes,
                "Discarded leftover snapshot temporary file"
            );
        }
    }

    fn discard_snapshot_partial_files_in(
        &self,
        directory: &Path,
        removed: &mut Vec<(String, u64)>,
    ) {
        let entries = match fs::read_dir(directory) {
            Ok(value) => value,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.discard_snapshot_partial_files_in(&path, removed);
                continue;
            }
            let name = entry.file_name();
            if !name.to_string_lossy().ends_with(SNAPSHOT_PARTIAL_SUFFIX) {
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

/// Reads up to `length` bytes of the recording at `path` from `offset`, never past the size it reports, so the
/// range agrees with that size while the file grows.
pub fn read_range(path: &Path, offset: u64, length: u64) -> Result<RecordingRange, ReadRangeError> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    if offset > size {
        return Err(ReadRangeError::OffsetPastEnd { offset, size });
    }
    file.seek(SeekFrom::Start(offset))?;
    let mut data = Vec::new();
    file.take(length.min(size - offset))
        .read_to_end(&mut data)?;
    Ok(RecordingRange { size, data })
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
    let contents = if indexed {
        read_recording_contents(path).unwrap_or_else(|error| {
            warn!(%error, path = %path.display(), "Failed to read the recording summary");
            None
        })
    } else {
        None
    };
    let reading = FooterReading { indexed, contents };
    footer_cache
        .entries
        .insert(cache_path, (cache_key, reading.clone()));
    Ok(reading)
}

fn relative_path_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
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

#[cfg(test)]
mod tests {
    use core::time::Duration;

    use super::*;

    #[test]
    fn resolve_rejects_parent_dir() {
        let temporary = tempfile::tempdir().expect("tempdir");
        let folder = RecordingsFolder::new(temporary.path().to_path_buf()).expect("folder");
        assert!(matches!(
            folder.resolve("../outside.mcap"),
            Err(StorageError::InvalidPath)
        ));
    }

    #[test]
    fn each_rewrite_gets_its_own_temporary_file_and_startup_discards_them_all() {
        let temporary = tempfile::tempdir().expect("tempdir");
        let folder = RecordingsFolder::new(temporary.path().to_path_buf()).expect("folder");
        let repairs = [(); 2].map(|()| folder.recover_temporary_path("dive/one.mcap"));
        let snapshots = [(); 2].map(|()| folder.snapshot_temporary_path("dive/one-indexed.mcap"));
        fs::create_dir_all(folder.base().join("dive")).expect("directory");
        for path in repairs.iter().chain(&snapshots) {
            fs::write(path, b"partial").expect("temporary file");
        }

        RecordingsFolder::new(temporary.path().to_path_buf()).expect("folder");

        assert_ne!(repairs[0], repairs[1]);
        assert_ne!(snapshots[0], snapshots[1]);
        for path in repairs.iter().chain(&snapshots) {
            assert_eq!(path.parent(), Some(folder.base().join("dive").as_path()));
            assert!(!path.exists(), "{}", path.display());
        }
    }

    #[test]
    fn scan_reads_the_duration_and_video_topics_of_an_indexed_recording() {
        let temporary = tempfile::tempdir().expect("tempdir");
        let folder = RecordingsFolder::new(temporary.path().to_path_buf()).expect("folder");
        write_recording(&folder.base().join("dive.mcap"));

        let [scanned] = scan_one(&folder, &mut LibraryFooterCache::default());

        assert!(scanned.indexed);
        assert_eq!(
            scanned.contents,
            Some(RecordingContents {
                duration: Duration::from_millis(2_500),
                video_topics: vec!["video/camera/stream".into()],
                other_topic_count: 1,
            })
        );
    }

    #[test]
    fn scan_knows_no_duration_or_video_of_a_recording_without_summary() {
        let temporary = tempfile::tempdir().expect("tempdir");
        let folder = RecordingsFolder::new(temporary.path().to_path_buf()).expect("folder");
        let path = folder.base().join("cut.mcap");
        write_recording(&path);
        let bytes = fs::read(&path).expect("read");
        fs::write(&path, &bytes[..bytes.len() / 2]).expect("truncate");

        let [scanned] = scan_one(&folder, &mut LibraryFooterCache::default());

        assert!(!scanned.indexed);
        assert_eq!(scanned.contents, None);
    }

    #[test]
    fn scan_reuses_the_summary_of_a_file_whose_size_and_time_are_unchanged() {
        let temporary = tempfile::tempdir().expect("tempdir");
        let folder = RecordingsFolder::new(temporary.path().to_path_buf()).expect("folder");
        let path = folder.base().join("dive.mcap");
        write_recording(&path);
        let mut footer_cache = LibraryFooterCache::default();
        let [first] = scan_one(&folder, &mut footer_cache);

        let metadata = fs::metadata(&path).expect("metadata");
        let mut file = fs::File::options().write(true).open(&path).expect("open");
        io::Write::write_all(&mut file, &vec![0; metadata.len() as usize]).expect("overwrite");
        file.set_modified(metadata.modified().expect("modified"))
            .expect("restore modified time");
        let [second] = scan_one(&folder, &mut footer_cache);

        assert_eq!(second, first);
    }

    fn scan_one(
        folder: &RecordingsFolder,
        footer_cache: &mut LibraryFooterCache,
    ) -> [ScannedRecordingFile; 1] {
        folder
            .scan_library(None, footer_cache)
            .expect("scan")
            .try_into()
            .expect("one recording")
    }

    /// A video message at 1 s and a telemetry message at 3.5 s.
    fn write_recording(path: &Path) {
        let mut writer =
            mcap::Writer::new(fs::File::create(path).expect("create")).expect("writer");
        let video_schema = writer
            .add_schema("foxglove.CompressedVideo", "ros2msg", b"uint8[] data")
            .expect("schema");
        let video = writer
            .add_channel(
                video_schema,
                "video/camera/stream",
                "cdr",
                &Default::default(),
            )
            .expect("video channel");
        let telemetry = writer
            .add_channel(0, "mavlink/heartbeat", "json", &Default::default())
            .expect("telemetry channel");
        for (channel_id, log_time) in [(video, 1_000_000_000), (telemetry, 3_500_000_000)] {
            writer
                .write_to_known_channel(
                    &mcap::records::MessageHeader {
                        channel_id,
                        sequence: 0,
                        log_time,
                        publish_time: log_time,
                    },
                    b"{}",
                )
                .expect("write");
        }
        writer.finish().expect("finish");
    }
}
