use thiserror::Error;

/// Errors from the service kernel or its adapters.
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Comms(#[from] blueos_comms::CommsError),
    #[error(transparent)]
    App(#[from] blueos_cqrs::AppError),
    #[error(transparent)]
    Settings(#[from] blueos_settings::SettingsError),
    #[error(transparent)]
    Idl(#[from] blueos_idl::Error),
    #[error(transparent)]
    Log(#[from] blueos_logging::LogError),
    #[error("{0}")]
    Message(String),
}
