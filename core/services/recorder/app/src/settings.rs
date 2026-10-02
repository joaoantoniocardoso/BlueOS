//! Python-compatible persisted settings for the Recorder (owned by the Kernel).

use core::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use blueos_recorder_capture::CaptureSettings;
use blueos_settings::SettingsSchema;

/// Python-compatible persisted settings for the Recorder (owned by the Kernel).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecorderSettings {
    /// Settings document version.
    #[serde(rename = "VERSION")]
    pub version: NonZeroU32,
    /// When set, MAVLink topics are recorded only while the vehicle is armed.
    pub record_mavlink_only_when_armed: bool,
    /// When set, recording starts as soon as settings allow.
    pub auto_start_recording: bool,
}

impl Default for RecorderSettings {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            record_mavlink_only_when_armed: true,
            auto_start_recording: true,
        }
    }
}

impl SettingsSchema for RecorderSettings {
    const VERSION: NonZeroU32 = NonZeroU32::new(1).unwrap();

    fn migrate(_data: &mut serde_json::Value) -> Result<(), blueos_settings::SettingsError> {
        Ok(())
    }

    fn restart_required_fields() -> &'static [&'static str] {
        &[]
    }
}

impl RecorderSettings {
    /// Maps into the capture Block settings.
    pub(crate) fn into_capture_settings(self) -> CaptureSettings {
        CaptureSettings {
            record_mavlink_only_when_armed: self.record_mavlink_only_when_armed,
            auto_start_recording: self.auto_start_recording,
        }
    }

    /// Builds from the capture Block settings and document version.
    pub(crate) fn from_capture(settings: &CaptureSettings) -> Self {
        Self {
            version: Self::VERSION,
            record_mavlink_only_when_armed: settings.record_mavlink_only_when_armed,
            auto_start_recording: settings.auto_start_recording,
        }
    }
}
