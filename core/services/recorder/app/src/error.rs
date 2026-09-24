use blueos_comms::CommsError;
use blueos_service::ServiceError;
use blueos_settings::SettingsError;

#[derive(Debug, thiserror::Error)]
pub enum RecorderRunError {
    #[error("zenoh session: {0}")]
    Session(#[from] CommsError),
    #[error("{0}")]
    Service(#[from] ServiceError),
    #[error("recorder path: {0}")]
    RecorderPath(#[from] std::io::Error),
    #[error("settings: {0}")]
    Settings(#[from] SettingsError),
}
