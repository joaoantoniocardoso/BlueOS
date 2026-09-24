//! Python-compatible JSON settings on disk for BlueOS services.
//!
//! File layout, naming (`settings-<VERSION>.json`), the `VERSION` field, migrations, and atomic
//! saves match `commonwealth.settings` so a Rust service can replace a Python one without touching
//! user data. See `doc/architecture/decisions.md` (D-11) for runtime vs restart-required fields.

mod error;
mod manager;
mod restart;
mod schema;

pub use error::SettingsError;
pub use manager::{
    SETTINGS_NAME_PREFIX, SettingsManager, resolve_config_folder, settings_file_name,
};
pub use restart::{SettingsFieldChanges, diff_top_level_settings};
pub use schema::{SettingsSchema, read_version};

#[cfg(test)]
mod tests;
