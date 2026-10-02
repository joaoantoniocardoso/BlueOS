use thiserror::Error;

/// Why a settings file could not be loaded, migrated or saved.
#[derive(Debug, Error)]
pub enum SettingsError {
    /// The file is not a settings document this service understands.
    #[error("settings file is not valid: {0}")]
    BadSettingsFile(String),
    /// The file was written by a newer version of the service.
    #[error("settings file version is from a newer service: {0}")]
    SettingsFromTheFuture(String),
    /// A migration step between two versions failed.
    #[error("could not apply migration")]
    MigrationFail,
    /// The `VERSION` field is not a positive integer.
    #[error("settings file contains invalid version number")]
    BadAttributes,
    /// A settings type name carries no version number.
    #[error("{0} is not a valid settings class name, valid names should contain a number. Eg: V1")]
    BadSettingsClassNaming(String),
    /// Reading or writing the file failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The file is not valid JSON, or does not match the settings type.
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

impl SettingsError {
    /// The file's version is newer than the latest version this service supports.
    pub fn settings_from_the_future(file_version: u32, latest_supported: u32) -> Self {
        Self::SettingsFromTheFuture(format!(
            "Settings file comes from a future settings version: {file_version}, \
             latest supported: {latest_supported}, tomorrow does not exist"
        ))
    }

    /// The document `data` has no usable `VERSION` field.
    pub fn missing_version_field(data: &serde_json::Value) -> Self {
        Self::BadSettingsFile(format!(
            "Settings file does not appears to contain a valid settings format: {data}"
        ))
    }
}
