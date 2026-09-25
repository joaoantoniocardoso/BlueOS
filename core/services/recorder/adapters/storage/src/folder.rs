use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::MetadataExt;

use blueos_recorder_mcap::footer::{Footer, read_footer_at};
use thiserror::Error;

const RECORDING_SUFFIX: &str = ".mcap";
const RECOVER_SUFFIX: &str = ".recover";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredRecording {
    pub relative_path: String,
    pub name: String,
    pub size_bytes: u64,
    pub modified_unix_nanos: u64,
    pub changed_unix_nanos: u64,
    pub indexed: bool,
}

#[derive(Debug, Error)]
pub enum PathError {
    #[error("Invalid recording path.")]
    InvalidPath,
    #[error("Recording not found.")]
    NotFound,
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("{0}")]
    Path(#[from] PathError),
    #[error("{0}")]
    Io(#[from] io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FooterCacheKey {
    inode: u64,
    size: u64,
    modified_unix_nanos: u64,
}

pub struct RecordingsFolder {
    base: PathBuf,
    footer_cache: HashMap<PathBuf, (FooterCacheKey, Option<Footer>)>,
}

impl RecordingsFolder {
    pub fn new(base: PathBuf) -> io::Result<Self> {
        fs::create_dir_all(&base)?;
        Ok(Self {
            base: base.canonicalize()?,
            footer_cache: HashMap::new(),
        })
    }

    pub fn scan(&mut self, active: Option<&str>) -> io::Result<Vec<StoredRecording>> {
        let mut recordings = Vec::new();
        let mut listed_paths = Vec::new();
        let base = self.base.clone();
        self.collect_mcap_files(&base, &mut recordings, &mut listed_paths, active)?;
        prune_footer_cache(&mut self.footer_cache, &listed_paths);
        Ok(recordings)
    }

    pub fn resolve(&self, relative: &str) -> Result<PathBuf, PathError> {
        if relative.is_empty() || relative.starts_with('/') || Path::new(relative).is_absolute() {
            return Err(PathError::InvalidPath);
        }
        for component in Path::new(relative).components() {
            if matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            ) {
                return Err(PathError::InvalidPath);
            }
        }
        if !relative.to_ascii_lowercase().ends_with(RECORDING_SUFFIX) {
            return Err(PathError::InvalidPath);
        }
        let candidate = self.base.join(relative);
        let canonical = match candidate.canonicalize() {
            Ok(path) => path,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(PathError::NotFound);
            }
            Err(_) => return Err(PathError::InvalidPath),
        };
        if !canonical.starts_with(&self.base) {
            return Err(PathError::InvalidPath);
        }
        if canonical.is_dir() {
            return Err(PathError::InvalidPath);
        }
        let metadata = match fs::metadata(&canonical) {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(PathError::NotFound);
            }
            Err(_) => return Err(PathError::InvalidPath),
        };
        if !metadata.is_file() {
            return Err(PathError::InvalidPath);
        }
        Ok(canonical)
    }

    pub fn delete(&self, relative: &str) -> Result<(), StorageError> {
        let path = self.resolve(relative)?;
        fs::remove_file(path)?;
        Ok(())
    }

    pub fn discard_leftovers(&self) -> Vec<(String, u64)> {
        let mut removed = Vec::new();
        self.discard_recover_files(&self.base, &mut removed);
        removed
    }

    fn discard_recover_files(&self, directory: &Path, removed: &mut Vec<(String, u64)>) {
        let entries = match fs::read_dir(directory) {
            Ok(value) => value,
            Err(_) => return,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.discard_recover_files(&path, removed);
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
            let size = entry.metadata().map(|metadata| metadata.len()).unwrap_or(0);
            if fs::remove_file(&path).is_ok() {
                removed.push((relative, size));
            }
        }
    }

    pub fn temporary_output(&self, relative: &str) -> PathBuf {
        let path = Path::new(relative);
        let parent = path.parent().unwrap_or_else(|| Path::new(""));
        let stem = path
            .file_stem()
            .and_then(OsStr::to_str)
            .unwrap_or("recording");
        let temporary_name = format!("{stem}{RECOVER_SUFFIX}");
        self.base.join(parent).join(temporary_name)
    }

    pub fn replace(&self, temporary: &Path, relative: &str) -> Result<(), StorageError> {
        let destination = self.resolve(relative)?;
        fs::rename(temporary, destination)?;
        Ok(())
    }

    fn collect_mcap_files(
        &mut self,
        directory: &Path,
        recordings: &mut Vec<StoredRecording>,
        listed_paths: &mut Vec<PathBuf>,
        active: Option<&str>,
    ) -> io::Result<()> {
        for entry in fs::read_dir(directory)? {
            let path = match entry {
                Ok(value) => value.path(),
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            if path.is_dir() {
                self.collect_mcap_files(&path, recordings, listed_paths, active)?;
                continue;
            }
            let name = path.file_name().and_then(OsStr::to_str).unwrap_or("");
            if !name.to_ascii_lowercase().ends_with(RECORDING_SUFFIX) {
                continue;
            }
            let metadata = match fs::metadata(&path) {
                Ok(value) => value,
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(error),
            };
            let relative_path = path
                .strip_prefix(&self.base)
                .map(relative_path_string)
                .unwrap_or_else(|_| name.to_string());
            let skip_footer = active == Some(relative_path.as_str()) || active == Some(name);
            listed_paths.push(path.clone());
            let indexed = self.footer_for_listing(&path, &metadata, skip_footer)?;
            recordings.push(StoredRecording {
                relative_path,
                name: name.to_string(),
                size_bytes: metadata.len(),
                modified_unix_nanos: modified_unix_nanos(&metadata),
                changed_unix_nanos: changed_unix_nanos(&metadata),
                indexed,
            });
        }
        Ok(())
    }

    fn footer_for_listing(
        &mut self,
        path: &Path,
        metadata: &fs::Metadata,
        skip_footer: bool,
    ) -> io::Result<bool> {
        if skip_footer {
            return Ok(false);
        }
        let cache_key = FooterCacheKey {
            inode: inode(metadata),
            size: metadata.len(),
            modified_unix_nanos: modified_unix_nanos(metadata),
        };
        let cache_path = path.to_path_buf();
        if let Some((cached_key, footer)) = self.footer_cache.get(&cache_path)
            && *cached_key == cache_key
        {
            return Ok(footer.is_some_and(|value| value.summary_start > 0));
        }
        let footer = read_footer_at(path, metadata.len())?;
        self.footer_cache.insert(cache_path, (cache_key, footer));
        Ok(footer.is_some_and(|value| value.summary_start > 0))
    }
}

fn relative_path_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn prune_footer_cache(
    cache: &mut HashMap<PathBuf, (FooterCacheKey, Option<Footer>)>,
    keep: &[PathBuf],
) {
    let keep_set: std::collections::HashSet<&PathBuf> = keep.iter().collect();
    cache.retain(|path, _| keep_set.contains(path));
}

#[cfg(unix)]
fn inode(metadata: &fs::Metadata) -> u64 {
    metadata.ino()
}

#[cfg(not(unix))]
fn inode(_metadata: &fs::Metadata) -> u64 {
    0
}

fn modified_unix_nanos(metadata: &fs::Metadata) -> u64 {
    metadata
        .modified()
        .map(|time| {
            time.duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos() as u64)
                .unwrap_or(0)
        })
        .unwrap_or(0)
}

#[cfg(unix)]
fn changed_unix_nanos(metadata: &fs::Metadata) -> u64 {
    metadata.ctime() as u64 * 1_000_000_000 + metadata.ctime_nsec() as u64
}

#[cfg(not(unix))]
fn changed_unix_nanos(metadata: &fs::Metadata) -> u64 {
    modified_unix_nanos(metadata)
}
