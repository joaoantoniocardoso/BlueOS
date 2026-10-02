//! Versioned durable state JSON next to the settings document (D-28).

use core::num::NonZeroU32;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use tracing::warn;

use super::{
    error::SettingsError,
    manager::resolve_config_folder,
    schema::{atomic_write_file, read_version},
};

/// Filename prefix for durable state: `state-<VERSION>.json`.
pub const STATE_NAME_PREFIX: &str = "state-";

/// Opens or creates the durable state file for one service version.
#[derive(Clone)]
pub struct ServiceStateStore {
    path: PathBuf,
    version: NonZeroU32,
}

impl ServiceStateStore {
    /// Resolves the config folder like settings and points at `state-<version>.json` without touching disk.
    #[must_use]
    pub fn open(
        service_name: impl AsRef<str>,
        config_folder: Option<PathBuf>,
        version: NonZeroU32,
    ) -> Self {
        let folder = resolve_config_folder(service_name, config_folder);
        Self {
            path: folder.join(state_file_name(version)),
            version,
        }
    }

    /// Creates the service config folder when the Kernel starts durable state.
    ///
    /// # Errors
    ///
    /// [`SettingsError`] when the folder cannot be created.
    pub fn ensure_directory(&self) -> Result<(), SettingsError> {
        let parent = self.path.parent().ok_or_else(|| {
            SettingsError::BadSettingsFile(format!(
                "invalid durable state path {}",
                self.path.display()
            ))
        })?;
        fs::create_dir_all(parent).map_err(SettingsError::from)
    }

    /// The on-disk path for this service's durable state.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Reads UTF-8 JSON when the file exists, parses, and checks `VERSION`. On failure the file is moved aside.
    pub fn read_document(&self) -> Option<serde_json::Value> {
        let data = match fs::read_to_string(&self.path) {
            Ok(data) => data,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
            Err(error) => {
                warn!(
                    %error,
                    path = %self.path.display(),
                    "Durable state file is unreadable; starting fresh"
                );
                self.move_aside_corrupt();
                return None;
            }
        };
        let value: serde_json::Value = match serde_json::from_str(&data) {
            Ok(value) => value,
            Err(error) => {
                warn!(
                    %error,
                    path = %self.path.display(),
                    "Durable state file is not JSON; starting fresh"
                );
                self.move_aside_corrupt();
                return None;
            }
        };
        match read_version(&value) {
            Ok(version) if version == self.version => Some(value),
            Ok(version) => {
                warn!(
                    path = %self.path.display(),
                    found = version.get(),
                    expected = self.version.get(),
                    "Durable state VERSION does not match; starting fresh"
                );
                self.move_aside_corrupt();
                None
            }
            Err(error) => {
                warn!(
                    %error,
                    path = %self.path.display(),
                    "Durable state file has no VERSION; starting fresh"
                );
                self.move_aside_corrupt();
                None
            }
        }
    }

    /// Writes `bytes` atomically, using the same helper as settings (D-11).
    ///
    /// # Errors
    ///
    /// [`SettingsError`] when the write fails.
    pub fn write_document(&self, bytes: &[u8]) -> Result<(), SettingsError> {
        atomic_write_file(&self.path, bytes)
    }

    /// Moves the current file aside so the next write starts fresh.
    pub fn move_aside_corrupt(&self) {
        self.move_aside_corrupt_at_wall_millis(None);
    }

    /// Like [`Self::move_aside_corrupt`], but names the backup from injected wall time when provided.
    pub fn move_aside_corrupt_at_wall_millis(&self, wall_millis: Option<u128>) {
        if !self.path.is_file() {
            return;
        }
        let millis = wall_millis.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_millis())
                .unwrap_or(0)
        });
        let aside = self.path.with_extension(format!("corrupt.{millis}"));
        if let Err(error) = fs::rename(&self.path, aside) {
            warn!(
                %error,
                path = %self.path.display(),
                "Could not move corrupt durable state aside"
            );
        }
    }
}

/// Path to `state-<version>.json` inside a service config folder.
pub fn state_file_name(version: NonZeroU32) -> String {
    format!("{STATE_NAME_PREFIX}{version}.json")
}
