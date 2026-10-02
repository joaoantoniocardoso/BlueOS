//! Premise check: paused-runtime sleep waits for outstanding `spawn_blocking` work.

use core::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::sync::Arc;

#[tokio::test(start_paused = true)]
async fn paused_sleep_waits_until_spawn_blocking_finishes() {
    let finished = Arc::new(AtomicBool::new(false));
    tokio::task::spawn_blocking({
        let finished = Arc::clone(&finished);
        move || {
            std::thread::sleep(Duration::from_millis(200));
            finished.store(true, Ordering::Release);
        }
    });
    tokio::time::sleep(Duration::from_millis(1)).await;
    assert!(
        finished.load(Ordering::Acquire),
        "1 ms sleep on a paused runtime must not return while spawn_blocking is still running"
    );
}
