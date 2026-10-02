//! Premise check: paused-runtime sleep waits for outstanding `spawn_blocking` work.

use core::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use std::sync::{Arc, mpsc};

#[tokio::test(start_paused = true)]
async fn paused_sleep_waits_until_spawn_blocking_finishes() {
    let (release_sender, release_receiver) = mpsc::channel();
    let finished = Arc::new(AtomicBool::new(false));
    tokio::task::spawn_blocking({
        let finished = Arc::clone(&finished);
        move || {
            release_receiver.recv().expect("release");
            finished.store(true, Ordering::Release);
        }
    });

    let sleep_finished = Arc::new(AtomicBool::new(false));
    let sleep_task = tokio::spawn({
        let sleep_finished = Arc::clone(&sleep_finished);
        async move {
            tokio::time::sleep(Duration::from_millis(1)).await;
            sleep_finished.store(true, Ordering::Release);
        }
    });

    for _ in 0..8 {
        tokio::task::yield_now().await;
    }
    assert!(
        !sleep_finished.load(Ordering::Acquire),
        "1 ms sleep on a paused runtime must not return while spawn_blocking is still running"
    );
    assert!(
        !finished.load(Ordering::Acquire),
        "blocking task must stay parked until released"
    );

    release_sender.send(()).expect("release");
    sleep_task.await.expect("sleep task");
    assert!(
        finished.load(Ordering::Acquire),
        "blocking task must finish after release"
    );
    assert!(
        sleep_finished.load(Ordering::Acquire),
        "sleep must finish after spawn_blocking work completes"
    );
}
