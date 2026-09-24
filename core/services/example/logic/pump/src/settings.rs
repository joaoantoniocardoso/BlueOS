use alloc::string::String;
use core::num::NonZeroU32;

pub const SETTINGS_VERSION: NonZeroU32 = NonZeroU32::new(1).unwrap();

/// On-disk settings shape (Python-compatible JSON, D-11). `max_level` applies immediately to commands;
/// `device_model` is informational and requires a process restart before the simulated device identity changes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ExampleSettings {
    pub version: u32,
    /// Runtime: caps [`PumpCommand::SetLevel`] and is reflected in published state.
    pub max_level: u8,
    /// Runtime: watchdog duration for [`PumpCommand::StartSelfTest`].
    pub self_test_timeout_seconds: u32,
    /// Restart-required: changing it emits [`PumpEvent::RestartRequired`]; the fake device keeps the old model
    /// until the service restarts.
    pub device_model: String,
}

impl ExampleSettings {
    pub fn with_defaults() -> Self {
        Self {
            version: SETTINGS_VERSION.get(),
            max_level: 100,
            self_test_timeout_seconds: 30,
            device_model: "simulated-pump-v1".into(),
        }
    }
}

pub fn settings_version() -> NonZeroU32 {
    SETTINGS_VERSION
}

pub fn restart_required_field_names() -> &'static [&'static str] {
    &["device_model"]
}
