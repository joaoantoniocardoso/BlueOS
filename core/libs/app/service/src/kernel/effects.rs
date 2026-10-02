//! Carrying out the synchronous part of a Decision's [`Effect`]s before States are projected.

use blueos_domain::{Domain, Effect};

use super::timers::TimerWheel;

/// Why the synchronous part of the Effects could not be applied.
#[derive(Debug, thiserror::Error)]
pub(crate) enum SyncEffectError {
    /// The Decision scheduled IO but `build` did not register an executor.
    #[error("the Decision scheduled IO but the Service did not register an IO executor")]
    IoNotRegistered,
}

/// Applies every timer [`Effect`] and checks that IO [`Effect`]s have an executor before the Command is acknowledged.
pub(crate) fn apply_sync_effects<D: Domain>(
    effects: &[Effect<D::Tick, D::IoRequest, D::TimerKey>],
    timers: &mut TimerWheel<D>,
    io_registered: bool,
) -> Result<(), SyncEffectError> {
    let mut needs_io = false;
    for effect in effects {
        match effect {
            Effect::Io(_) => needs_io = true,
            Effect::Schedule { .. } | Effect::Cancel(_) => timers.apply(effect.clone()),
        }
    }
    if needs_io && !io_registered {
        return Err(SyncEffectError::IoNotRegistered);
    }
    Ok(())
}

/// Collects the IO requests from a Decision's Effects, in order.
pub(crate) fn io_requests<D: Domain>(
    effects: &[Effect<D::Tick, D::IoRequest, D::TimerKey>],
) -> Vec<D::IoRequest> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::Io(request) => Some(request.clone()),
            Effect::Schedule { .. } | Effect::Cancel(_) => None,
        })
        .collect()
}
