use std::{
    fs::OpenOptions,
    io::Write,
    num::NonZeroU32,
    path::{Path, PathBuf},
};

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Serializer;

use super::error::SettingsError;

/// Contract for a service settings document stored as `settings-<VERSION>.json`.
///
/// Implementations must match Python `commonwealth.settings.bases.PydanticSettings` semantics
/// (see architecture decision D-11).
pub trait SettingsSchema: Serialize + DeserializeOwned + Clone + Default {
    const VERSION: NonZeroU32;

    fn migrate(data: &mut serde_json::Value) -> Result<(), SettingsError>;

    fn legacy_load_paths(_config_folder: &Path) -> Vec<PathBuf> {
        Vec::new()
    }

    fn on_settings_created(&self, _path: &Path) -> Result<(), SettingsError> {
        Ok(())
    }

    fn reset(&mut self) {
        *self = Self::default();
    }

    /// Top-level JSON keys whose change requires a service restart (D-11). Default: none.
    fn restart_required_fields() -> &'static [&'static str] {
        &[]
    }

    fn load_from_value(mut data: serde_json::Value) -> Result<Self, SettingsError> {
        let version = read_version(&data)?;
        if version > Self::VERSION {
            return Err(SettingsError::settings_from_the_future(
                version.get(),
                Self::VERSION.get(),
            ));
        }
        if version < Self::VERSION {
            Self::migrate(&mut data)?;
            let migrated = read_version(&data)?;
            if migrated != Self::VERSION {
                return Err(SettingsError::MigrationFail);
            }
        }
        serde_json::from_value(data).map_err(SettingsError::from)
    }

    fn load(path: &Path) -> Result<Self, SettingsError> {
        if !path.is_file() {
            return Err(SettingsError::BadSettingsFile(format!(
                "Settings file does not exist: {}",
                path.display()
            )));
        }

        let data = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&data)?;
        Self::load_from_value(value)
    }

    /// Atomic write compatible with Python `PydanticSettings.save` (`indent=4`, `os.replace`).
    ///
    /// The service kernel invokes this from a blocking thread; synchronous IO is intentional.
    fn save(&self, path: &Path) -> Result<(), SettingsError> {
        let parent = path.parent().ok_or_else(|| {
            SettingsError::BadSettingsFile(format!("invalid settings path {}", path.display()))
        })?;
        std::fs::create_dir_all(parent)?;

        if !path.is_file() {
            self.on_settings_created(path)?;
        }

        let json = serialize_settings_document(self)?;
        let temp = path.with_extension("tmp");
        {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&temp)?;
            file.write_all(&json)?;
            file.sync_all()?;
        }
        std::fs::rename(temp, path)?;
        Ok(())
    }
}

pub fn read_version(data: &serde_json::Value) -> Result<NonZeroU32, SettingsError> {
    let Some(value) = data.get("VERSION") else {
        return Err(SettingsError::missing_version_field(data));
    };

    let version = match value.as_u64() {
        Some(number) if number <= u32::MAX as u64 => number as u32,
        _ => return Err(SettingsError::BadAttributes),
    };
    NonZeroU32::new(version).ok_or(SettingsError::BadAttributes)
}

pub(crate) fn serialize_settings_document<T: Serialize>(
    value: &T,
) -> Result<Vec<u8>, SettingsError> {
    let mut buffer = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let mut serializer = Serializer::with_formatter(&mut buffer, formatter);
    value.serialize(&mut serializer)?;
    Ok(buffer)
}
