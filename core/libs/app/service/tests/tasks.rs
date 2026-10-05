//! Supervised Tasks: restart, `status` health and shutdown (layer L3).

mod tasks_fixture;

use core::{
    future::pending,
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use std::sync::{Arc, Mutex};

use blueos_idl::msg::blueos_msgs::ServiceStatusStatus;
use blueos_service::{
    Backoff, Clock, Kernel, RestartPolicy, RunOutcome, Service, ServiceBuilder, TaskFailed,
    testing::{Harness, PausedClock, WALL_CLOCK_AT_START, lock_unpoisoned},
};
use tokio::time::timeout;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tasks_fixture::*;

#[tokio::test(start_paused = true)]
async fn always_restarts_after_the_backoff_the_task_declares() {
    let backoff = Backoff {
        minimum_delay: Duration::from_secs(1),
        maximum_delay: Duration::from_secs(1),
    };

    let delay = delay_between_first_two_attempts(RestartPolicy::Always { backoff }).await;

    assert!(delay >= Duration::from_secs(1), "restarted after {delay:?}");
}

#[test]
fn the_default_backoff_runs_from_100_milliseconds_to_30_seconds() {
    assert_eq!(
        Backoff::default(),
        Backoff {
            minimum_delay: Duration::from_millis(100),
            maximum_delay: Duration::from_secs(30),
        }
    );
}

#[tokio::test(start_paused = true)]
async fn on_failure_restarts_after_the_backoff_the_task_declares() {
    let backoff = Backoff {
        minimum_delay: Duration::from_secs(1),
        maximum_delay: Duration::from_secs(1),
    };

    let delay = delay_between_first_two_attempts(RestartPolicy::OnFailure {
        backoff,
        max_attempts: 3,
    })
    .await;

    assert!(delay >= Duration::from_secs(1), "restarted after {delay:?}");
}

#[tokio::test(start_paused = true)]
async fn always_failing_task_restarts_with_backoff_and_degrades_status() {
    let attempts = Arc::new(AtomicUsize::new(0));
    let attempts_for_task = Arc::clone(&attempts);
    let mut builder = ServiceBuilder::<TasksDomain>::new(TasksSnapshot);
    builder = builder.task(
        "flaky",
        RestartPolicy::Always {
            backoff: Backoff::default(),
        },
        move |_context| {
            let attempt_counter = Arc::clone(&attempts_for_task);
            async move {
                attempt_counter.fetch_add(1, Ordering::SeqCst);
                Err(TaskFailed)
            }
        },
    );
    let (backend, run, mut status_subscriber) = start_tasks_kernel(builder).await;

    let mut degraded = next_status(&mut status_subscriber).await;
    if degraded.status != ServiceStatusStatus::Degraded {
        degraded = next_status(&mut status_subscriber).await;
    }
    assert_eq!(degraded.status, ServiceStatusStatus::Degraded);
    assert_eq!(degraded.detail, "flaky");

    tokio::time::advance(Duration::from_millis(100)).await;
    let ready_again = next_status(&mut status_subscriber).await;
    assert_eq!(ready_again.status, ServiceStatusStatus::Ready);

    assert!(attempts.load(Ordering::SeqCst) >= 2);

    run.abort();
    drop(backend);
}

#[tokio::test(start_paused = true)]
async fn status_names_remaining_task_while_the_other_restarts() {
    let beta_gate = Arc::new(tokio::sync::Notify::new());
    let beta_gate_for_task = Arc::clone(&beta_gate);
    let mut builder = ServiceBuilder::<TasksDomain>::new(TasksSnapshot);
    builder = builder
        .task(
            "alpha",
            RestartPolicy::Always {
                backoff: Backoff::default(),
            },
            |_context| async move { Err(TaskFailed) },
        )
        .task(
            "beta",
            RestartPolicy::Always {
                backoff: Backoff::default(),
            },
            move |_context| {
                let beta_notify = Arc::clone(&beta_gate_for_task);
                async move {
                    beta_notify.notified().await;
                    Err(TaskFailed)
                }
            },
        );
    let (_backend, run, mut status_subscriber) = start_tasks_kernel(builder).await;

    let deadline = tokio::time::Instant::now() + RECV_TIMEOUT;
    loop {
        let status = next_status(&mut status_subscriber).await;
        if status.status == ServiceStatusStatus::Degraded && status.detail == "alpha" {
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for status degraded with only alpha"
        );
    }

    beta_gate.notify_waiters();
    run.abort();
}

#[tokio::test(start_paused = true)]
async fn harness_build_can_declare_tasks() {
    let _harness = Harness::<TasksService>::start(TasksArguments {})
        .await
        .expect("harness starts");
    assert_eq!(PausedClock::start().now().wall, WALL_CLOCK_AT_START);
}

#[tokio::test(start_paused = true)]
async fn shutdown_leaves_no_tasks_running() {
    static RUNNING: AtomicUsize = AtomicUsize::new(0);
    let (started_sender, started_receiver) = tokio::sync::oneshot::channel();
    let started_sender = Arc::new(Mutex::new(Some(started_sender)));
    let started_for_task = Arc::clone(&started_sender);
    let mut builder = ServiceBuilder::<TasksDomain>::new(TasksSnapshot);
    builder = builder.task("worker", RestartPolicy::Never, move |context| {
        let started_for_task = Arc::clone(&started_for_task);
        async move {
            RUNNING.fetch_add(1, Ordering::SeqCst);
            if let Some(sender) = lock_unpoisoned(&started_for_task).take() {
                let _ = sender.send(());
            }
            context.shutdown.cancelled().await;
            RUNNING.fetch_sub(1, Ordering::SeqCst);
            Ok(())
        }
    });
    let shutdown = builder.shutdown_handle();
    let backend: Arc<dyn blueos_comms::CommsBackend> =
        Arc::new(blueos_comms::channel::ChannelBackend::default());
    let clock: Arc<dyn Clock> = Arc::new(PausedClock::start());
    let kernel = Kernel::start(TasksService::NAME, builder, (), Arc::clone(&backend), clock)
        .await
        .expect("kernel starts");
    let run = tokio::spawn(async move { kernel.run().await });
    timeout(RECV_TIMEOUT, started_receiver)
        .await
        .expect("worker starts before timeout")
        .expect("worker signals start");
    assert_eq!(RUNNING.load(Ordering::SeqCst), 1);
    shutdown.trigger();
    tokio::time::advance(Duration::from_secs(5)).await;
    assert_eq!(run.await.expect("kernel run finishes"), RunOutcome::Stopped);
    assert_eq!(RUNNING.load(Ordering::SeqCst), 0);
}

#[tokio::test(start_paused = true)]
async fn straggler_is_aborted_and_named_in_warning() {
    let warnings = Arc::new(Mutex::new(Vec::new()));
    let _subscriber = tracing_subscriber::registry()
        .with(WarningCapture(Arc::clone(&warnings)))
        .set_default();

    let mut builder = ServiceBuilder::<TasksDomain>::new(TasksSnapshot);
    builder = builder.task("straggler", RestartPolicy::Never, |_| async {
        pending::<()>().await;
        Ok(())
    });
    let shutdown = builder.shutdown_handle();
    let backend: Arc<dyn blueos_comms::CommsBackend> =
        Arc::new(blueos_comms::channel::ChannelBackend::default());
    let clock: Arc<dyn Clock> = Arc::new(PausedClock::start());
    let kernel = Kernel::start(TasksService::NAME, builder, (), Arc::clone(&backend), clock)
        .await
        .expect("kernel starts");
    let run = tokio::spawn(async move { kernel.run().await });
    shutdown.trigger();
    tokio::time::advance(Duration::from_secs(5)).await;
    assert_eq!(run.await.expect("kernel run finishes"), RunOutcome::Stopped);
    let captured = lock_unpoisoned(&warnings).clone();
    assert!(
        captured.iter().any(|message| message.contains("straggler")),
        "expected a warning naming the straggler, got {captured:?}"
    );
}
