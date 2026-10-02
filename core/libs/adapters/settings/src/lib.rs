//! Python-compatible JSON settings on disk for BlueOS services.
//!
//! File layout, naming (`settings-<VERSION>.json`), the `VERSION` field, migrations, and atomic
//! saves match `commonwealth.settings` so a Rust service can replace a Python one without touching
//! user data.

mod error;
mod manager;
mod restart;
mod schema;
mod service_state;

pub use error::SettingsError;
pub use manager::{
    SETTINGS_NAME_PREFIX, SettingsManager, resolve_config_folder, settings_file_name,
};
pub use restart::{SettingsFieldChanges, diff_top_level_settings};
pub use schema::{SettingsSchema, atomic_write_file, read_version, serialize_settings_document};
pub use service_state::{STATE_NAME_PREFIX, ServiceStateStore, state_file_name};

#[cfg(test)]
mod tests;
