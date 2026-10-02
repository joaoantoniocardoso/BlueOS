//! Injected time for the Kernel and supervised Tasks.

use blueos_domain::Now;

/// Where the Kernel reads the time, once per Command, to give the Domain its [`Now`]. A shipped Service reads the
/// system clock; the test harness injects one that follows the paused tokio clock.
pub trait Clock: Send + Sync {
    /// The current time.
    fn now(&self) -> Now;
}
