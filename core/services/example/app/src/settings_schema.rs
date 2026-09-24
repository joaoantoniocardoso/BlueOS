use blueos_settings::{SettingsError, SettingsSchema};
use example_pump_logic::{ExampleSettings, restart_required_field_names};
use serde::{Deserialize, Serialize};

/// JSON on-disk view (`VERSION` key) of [`ExampleSettings`].
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
#[allow(non_snake_case)]
struct ExampleSettingsJson {
    VERSION: u32,
    max_level: u8,
    self_test_timeout_seconds: u32,
    device_model: String,
}

/// Adapter-side [`SettingsSchema`] impl so the kernel can load Python-compatible JSON (D-11).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(transparent, deny_unknown_fields)]
pub struct ExampleSettingsSchema(ExampleSettingsJson);

impl ExampleSettingsSchema {
    pub fn inner(&self) -> ExampleSettings {
        ExampleSettings {
            version: self.0.VERSION,
            max_level: self.0.max_level,
            self_test_timeout_seconds: self.0.self_test_timeout_seconds,
            device_model: self.0.device_model.clone(),
        }
    }
}

impl From<ExampleSettings> for ExampleSettingsSchema {
    fn from(settings: ExampleSettings) -> Self {
        Self(ExampleSettingsJson {
            VERSION: settings.version,
            max_level: settings.max_level,
            self_test_timeout_seconds: settings.self_test_timeout_seconds,
            device_model: settings.device_model,
        })
    }
}

impl SettingsSchema for ExampleSettingsSchema {
    const VERSION: std::num::NonZeroU32 = std::num::NonZeroU32::new(1).unwrap();

    fn migrate(_data: &mut serde_json::Value) -> Result<(), SettingsError> {
        Ok(())
    }

    fn restart_required_fields() -> &'static [&'static str] {
        restart_required_field_names()
    }
}

impl Default for ExampleSettingsSchema {
    fn default() -> Self {
        Self::from(ExampleSettings::with_defaults())
    }
}

pub fn settings_from_envelope_json(document_json: &str) -> Result<ExampleSettings, String> {
    let json: ExampleSettingsJson =
        serde_json::from_str(document_json).map_err(|error| error.to_string())?;
    Ok(ExampleSettings {
        version: json.VERSION,
        max_level: json.max_level,
        self_test_timeout_seconds: json.self_test_timeout_seconds,
        device_model: json.device_model,
    })
}
