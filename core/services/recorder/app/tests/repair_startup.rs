//! Recorder repair startup cleanup tests.

mod common;

use std::fs;

use tempfile::tempdir;

use common::harness::startup::start_harness;

#[tokio::test(start_paused = true)]
async fn leftover_recover_file_is_removed_at_startup() {
    let directory = tempdir().expect("tempdir");
    fs::write(directory.path().join("stale.recover"), b"leftover").expect("write");

    let _harness = start_harness(directory.path()).await;

    assert!(
        !directory.path().join("stale.recover").exists(),
        "startup must discard leftover recover files"
    );
}
