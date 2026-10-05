//! Inbox loop and Task panic recovery (layer L3, D-29).

mod recovery_fixture;

use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use blueos_api::{Message, cdr_encoding, command_key, status_state_key};
use blueos_comms::{CommsBackend, QueryBody};
use blueos_idl::msg::{
    blueos_example_msgs::{PumpState, SetLevelGoal},
    blueos_msgs::ServiceStatusStatus,
};
use blueos_jobs::JobId;
use blueos_service::{
    Backoff, Clock, Kernel, RestartPolicy, RunOutcome, Service, ServiceBuilder, ServiceContext,
    TaskFailed,
    testing::{Harness, PausedClock},
};
use tokio::time::timeout;

use recovery_fixture::{
    LEVEL_THAT_PANICS_IN_HANDLE, RECV_TIMEOUT, TankArguments, TankService, TasksDomain,
    TasksService, TasksSnapshot, next_status,
};

#[tokio::test(start_paused = true)]
async fn inbox_loop_panic_marks_status_degraded_and_keeps_running() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .expect("harness starts");
    let backend = harness.backend();
    let mut status_subscriber = backend
        .subscribe(&status_state_key(TankService::NAME))
        .await
        .expect("status key subscribes");

    harness
        .send("SetLevel", &SetLevelGoal { level: 1 })
        .await
        .unwrap();

    let ack = harness
        .send(
            "SetLevel",
            &SetLevelGoal {
                level: LEVEL_THAT_PANICS_IN_HANDLE,
            },
        )
        .await
        .unwrap();
    assert!(!ack.accepted);

    let degraded = next_status(&mut status_subscriber).await;
    assert_eq!(degraded.status, ServiceStatusStatus::Degraded);
    assert_eq!(degraded.detail, "inbox");

    harness
        .send("SetLevel", &SetLevelGoal { level: 2 })
        .await
        .unwrap();
    let ready = next_status(&mut status_subscriber).await;
    assert_eq!(ready.status, ServiceStatusStatus::Ready);
    assert_eq!(harness.state::<PumpState>("tank").await.unwrap().level, 2);
}

#[tokio::test(start_paused = true)]
async fn three_inbox_loop_panics_within_one_minute_exit_non_zero() {
    let backend: Arc<dyn CommsBackend> = Arc::new(blueos_comms::channel::ChannelBackend::default());
    let clock: Arc<dyn Clock> = Arc::new(PausedClock::start());
    let builder = TankService::build(
        &ServiceContext::new(
            TankArguments { capacity: 100 },
            blueos_service::testing::channel_session(),
        ),
        &(),
    )
    .expect("build");
    let kernel = Kernel::start(
        TankService::NAME,
        builder,
        (),
        Arc::clone(&backend),
        Arc::clone(&clock),
    )
    .await
    .expect("kernel starts");
    let run = tokio::spawn(kernel.run());

    let client = tokio::spawn({
        let backend = Arc::clone(&backend);
        async move {
            for attempt in 0..3 {
                let body = QueryBody::new(
                    SetLevelGoal {
                        level: LEVEL_THAT_PANICS_IN_HANDLE,
                    }
                    .encode()
                    .expect("encode"),
                    cdr_encoding(SetLevelGoal::SCHEMA_NAME),
                )
                .with_attachment(JobId::from_u128(attempt).to_string().into_bytes());
                backend
                    .get(
                        &command_key(TankService::NAME, "SetLevel"),
                        Some(body),
                        RECV_TIMEOUT,
                    )
                    .await
                    .expect("command key is valid");
            }
        }
    });

    assert_eq!(
        timeout(RECV_TIMEOUT, run)
            .await
            .expect("kernel finishes")
            .expect("join"),
        RunOutcome::RepeatedInboxPanics
    );
    client.abort();
}

#[tokio::test(start_paused = true)]
async fn poisoned_lock_does_not_stop_another_task() {
    let shared = Arc::new(Mutex::new(0u32));
    let survivor_finished = Arc::new(tokio::sync::Notify::new());
    let shared_for_survivor = Arc::clone(&shared);
    let successes_for_survivor = Arc::new(AtomicUsize::new(0));
    let survivor_finished_for_task = Arc::clone(&survivor_finished);
    let shared_for_poisoner = Arc::clone(&shared);
    let poisoner = std::thread::spawn(move || {
        let _guard = shared_for_poisoner.lock().unwrap();
        panic!("poisoner panicked while holding the lock");
    });
    assert!(poisoner.join().is_err());
    assert!(shared.is_poisoned());

    let mut builder = ServiceBuilder::<TasksDomain>::new(TasksSnapshot);
    builder = builder.task(
        "survivor",
        RestartPolicy::Always {
            backoff: Backoff::default(),
        },
        move |_task_context| {
            let shared_for_survivor = Arc::clone(&shared_for_survivor);
            let successes_for_survivor = Arc::clone(&successes_for_survivor);
            let survivor_finished_for_task = Arc::clone(&survivor_finished_for_task);
            async move {
                let mut guard = shared_for_survivor
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *guard += 1;
                if successes_for_survivor.fetch_add(1, Ordering::SeqCst) + 1 >= 2 {
                    survivor_finished_for_task.notify_one();
                }
                Err(TaskFailed)
            }
        },
    );

    let backend: Arc<dyn CommsBackend> = Arc::new(blueos_comms::channel::ChannelBackend::default());
    let clock: Arc<dyn Clock> = Arc::new(PausedClock::start());
    let kernel = Kernel::start(
        TasksService::NAME,
        builder,
        (),
        Arc::clone(&backend),
        Arc::clone(&clock),
    )
    .await
    .expect("kernel starts");
    let run = tokio::spawn(kernel.run());

    timeout(RECV_TIMEOUT, survivor_finished.notified())
        .await
        .expect("survivor restarts before timeout");

    run.abort();
}
