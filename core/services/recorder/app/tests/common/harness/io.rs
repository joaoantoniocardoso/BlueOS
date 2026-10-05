//! Blocking IO drain and filesystem helpers for integration tests.

use core::time::Duration;
use std::{fs, path::Path, process::Command};

use tokio::time::sleep;

use blueos_service::testing::WALL_CLOCK_AT_START;

pub(crate) async fn assert_mcap_readable(path: &Path) {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let bytes = fs::read(&path).expect("read mcap");
        mcap::Summary::read(&bytes)
            .expect("parse mcap")
            .expect("mcap summary");
    })
    .await
    .expect("read task");
}

/// Waits until outstanding kernel [`spawn_blocking`] IO has finished on a paused runtime.
///
/// Auto-advance is inhibited while a blocking task runs, so a tiny [`sleep`] does not complete
/// until the runtime is idle and no blocking work remains (for example after a library rescan
/// triggered by a preceding [`advance`]).
pub(crate) async fn drain_blocking_io() {
    sleep(Duration::from_millis(1)).await;
}

pub(crate) fn set_modified_seconds_ago(path: &Path, seconds_ago: u64) {
    let stamp = WALL_CLOCK_AT_START.as_secs().saturating_sub(seconds_ago);
    let status = Command::new("touch")
        .args([
            "-d",
            &format!("@{stamp}"),
            path.to_str().expect("utf8 path"),
        ])
        .status()
        .expect("touch");
    assert!(
        status.success(),
        "touch must set an old mtime for the 10 s repair rule"
    );
}
