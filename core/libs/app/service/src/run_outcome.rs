//! What [`Kernel::run`](crate::Kernel::run) returns to the entry layer (D-29).

/// Why the Kernel stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunOutcome {
    /// Every endpoint closed, or shutdown finished within the IO drain budget.
    Stopped,
}
