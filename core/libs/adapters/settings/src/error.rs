use thiserror::Error;

#[derive(Debug, Error)]
pub enum SettingsError {
    #[error("settings file is not valid: {0}")]
    BadSettingsFile(String),
    #[error("settings file version is from a newer service: {0}")]
    SettingsFromTheFuture(String),
    #[error("could not apply migration")]
    MigrationFail,
    #[error("settings file contains invalid version number")]
    BadAttributes,
    #[error("{0} is not a valid settings class name, valid names should contain a number. Eg: V1")]
    BadSettingsClassNaming(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

impl SettingsError {
    pub fn settings_from_the_future(file_version: u32, latest_supported: u32) -> Self {
        Self::SettingsFromTheFuture(format!(
            "Settings file comes from a future settings version: {file_version}, \
             latest supported: {latest_supported}, tomorrow does not exist"
        ))
    }

    pub fn missing_version_field(data: &serde_json::Value) -> Self {
        Self::BadSettingsFile(format!(
            "Settings file does not appears to contain a valid settings format: {data}"
        ))
    }
}
