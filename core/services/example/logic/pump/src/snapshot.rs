use blueos_jobs::JobId;

use crate::phase;
use crate::settings::ExampleSettings;

/// Mutable domain state. The kernel copies this on persist; adapters never write it directly.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PumpSnapshot {
    pub level: u8,
    pub settings: ExampleSettings,
    pub self_test_phase: u8,
    pub self_test_active: bool,
    pub self_test_root_job: Option<JobId>,
}

impl Default for PumpSnapshot {
    fn default() -> Self {
        Self {
            level: 0,
            settings: ExampleSettings::with_defaults(),
            self_test_phase: phase::SELF_TEST_IDLE,
            self_test_active: false,
            self_test_root_job: None,
        }
    }
}

impl PumpSnapshot {
    pub fn effective_max_level(&self) -> u8 {
        self.settings.max_level
    }

    pub fn clamp_level(&self, requested: u8) -> u8 {
        requested.min(self.effective_max_level())
    }
}
