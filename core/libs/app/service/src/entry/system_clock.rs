//! Wall and monotonic time for a shipped Service.

use core::time::Duration;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use blueos_domain::Now;

use crate::Clock;

/// Reads the system clock once per Command.
pub(crate) struct SystemClock {
    wall_at_start: SystemTime,
    monotonic_at_start: Instant,
}

impl Clock for SystemClock {
    fn now(&self) -> Now {
        let monotonic = self.monotonic_at_start.elapsed();
        let wall = self
            .wall_at_start
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            + monotonic;
        Now { wall, monotonic }
    }
}

impl SystemClock {
    /// Captures the time at process start.
    pub(crate) fn new() -> Self {
        Self {
            wall_at_start: SystemTime::now(),
            monotonic_at_start: Instant::now(),
        }
    }
}
