//! Shared helpers for recorder app integration tests (paused clock, State waits).

#![expect(
    dead_code,
    reason = "each integration test binary compiles this module separately"
)]

pub(crate) mod harness;
pub(crate) mod mcap_fixtures;
pub(crate) mod repair_rewrite;

use core::time::Duration;
use std::path::Path;

use blueos_recorder_app::{IndexWalker, RecorderArguments, RecorderContext};

pub(crate) const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

/// Upper bound for waiting on pushed State while blocking IO runs on wall time.
pub(crate) const STATE_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) fn recorder_arguments(path: &Path) -> RecorderArguments {
    RecorderArguments {
        recorder_path: path.to_path_buf(),
    }
}

pub(crate) fn install_index_walker(
    context: &mut RecorderContext,
    walk_timeout: Duration,
    walker: IndexWalker,
) {
    context.index_walk_timeout = walk_timeout;
    context.index_walker = walker;
}
