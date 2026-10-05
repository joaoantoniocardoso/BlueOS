//! Kernel integration tests.

mod kernel_fixture;

use core::sync::atomic::Ordering;
use core::time::Duration;
use std::sync::Arc;

use tokio::task::JoinSet;
use tokio::time::timeout;

use blueos_api::{
    Message, cdr_encoding, command_key, event_key, job_result_key, jobs_key, query_key, state_key,
};
use blueos_comms::{CommsBackend, QueryBody, channel::ChannelBackend};
use blueos_idl::msg::{
    blueos_example_msgs::{LevelResponse, PumpState, SetLevelGoal},
    blueos_msgs::{CommandAckStatus, JobStatusStatus},
    std_msgs::Empty,
};
use blueos_service::{
    Kernel, Service, ServiceBuilder, ServiceContext, ServiceError,
    testing::{Harness, PausedClock},
};

use kernel_fixture::*;

#[tokio::test(start_paused = true)]
async fn a_build_that_refuses_its_arguments_stops_startup() {
    let started = Harness::<TankService>::start(TankArguments { capacity: 0 }).await;

    let Err(ServiceError::Build(reason)) = started else {
        panic!("a tank with no capacity must not start");
    };
    assert!(reason.downcast_ref::<NoCapacity>().is_some());
}

#[tokio::test(start_paused = true)]
async fn an_endpoint_the_backbone_refuses_stops_startup() {
    let started = Harness::<MisnamedTankService>::start(TankArguments { capacity: 100 }).await;

    let Err(ServiceError::DeclareEndpoint { key, .. }) = started else {
        panic!("a Service with an invalid key must not start");
    };
    assert_eq!(key, command_key(MisnamedTankService::NAME, "Set#Level"));
}

#[tokio::test(start_paused = true)]
async fn a_state_that_never_reached_the_backbone_has_no_value_to_read() {
    let backend = Arc::new(RecordingBackend::default());
    backend.publishes_fail.store(true, Ordering::SeqCst);
    let harness = Harness::<TankService>::start_on(
        Arc::<RecordingBackend>::clone(&backend),
        TankArguments { capacity: 100 },
    )
    .await
    .unwrap();

    let replies = harness
        .backend()
        .get(
            &state_key(TankService::NAME, "tank"),
            None,
            Duration::from_secs(10),
        )
        .await
        .unwrap();

    assert!(replies.is_empty(), "{replies:?}");
}

#[tokio::test(start_paused = true)]
async fn each_event_endpoint_publishes_only_the_domain_events_it_selects() {
    let backend = Arc::new(RecordingBackend::default());
    let harness = Harness::<TankService>::start_on(
        Arc::<RecordingBackend>::clone(&backend),
        TankArguments { capacity: 100 },
    )
    .await
    .unwrap();
    harness
        .send("SetLevel", &SetLevelGoal { level: 3 })
        .await
        .unwrap();
    backend.take_journal();

    harness
        .send("SetLevel", &SetLevelGoal { level: 0 })
        .await
        .unwrap();

    // `level_set_at` is not published again: the time did not move, so its value did not change.
    assert_eq!(
        backend.take_journal(),
        [
            format!("publish {}", state_key(TankService::NAME, "tank")),
            format!("publish {}", jobs_key(TankService::NAME)),
            format!("reply {}", command_key(TankService::NAME, "SetLevel")),
            format!("publish {}", event_key(TankService::NAME, "LevelChanged")),
            format!("publish {}", event_key(TankService::NAME, "Emptied")),
            format!("publish {}", job_result_key(TankService::NAME, "SetLevel")),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn a_kernel_with_only_io_queries_keeps_answering_them() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let builder = ServiceBuilder::<Tank>::new(TankSnapshot::empty(100)).io_query(
        "Probe",
        |request: SetLevelGoal| {
            Box::pin(async move {
                Ok(LevelResponse {
                    level: request.level,
                    max_level: 100,
                })
            })
        },
    );
    let kernel = Kernel::start(
        "sensor",
        builder,
        (),
        Arc::clone(&backend),
        Arc::new(PausedClock::start()),
    )
    .await
    .unwrap();
    let mut running = JoinSet::new();
    running.spawn(kernel.run());
    // Lets `run` go as far as it can before the IO query arrives.
    tokio::task::yield_now().await;

    let body = QueryBody::new(
        SetLevelGoal { level: 4 }.encode().unwrap(),
        cdr_encoding(SetLevelGoal::SCHEMA_NAME),
    );
    let replies = backend
        .get(
            &query_key("sensor", "Probe"),
            Some(body),
            Duration::from_secs(10),
        )
        .await
        .unwrap();

    let [Ok(reply)] = replies.as_slice() else {
        panic!("expected one answer, got {replies:?}");
    };
    assert_eq!(
        LevelResponse::decode(&reply.payload().to_bytes())
            .unwrap()
            .level,
        4
    );
}

#[tokio::test(start_paused = true)]
async fn the_kernel_stops_once_the_backbone_closes_every_endpoint() {
    let builder = TankService::build(
        &ServiceContext::new(
            TankArguments { capacity: 100 },
            blueos_service::testing::channel_session(),
        ),
        &(),
    )
    .unwrap()
    .io_query("Probe", |_request: Empty| {
        Box::pin(async { Ok(Empty::default()) })
    });
    let kernel = Kernel::start(
        TankService::NAME,
        builder,
        (),
        Arc::new(ClosedBackend),
        Arc::new(PausedClock::start()),
    )
    .await
    .unwrap();

    timeout(Duration::from_secs(10), kernel.run())
        .await
        .expect("the Kernel stops");
}

#[tokio::test(start_paused = true)]
async fn the_harness_errors_when_a_command_gets_no_ack() {
    use blueos_service::testing::HarnessError;

    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();

    assert!(matches!(
        harness.send("Missing", &SetLevelGoal { level: 1 }).await,
        Err(HarnessError::CommandAck { .. })
    ));
}

#[tokio::test(start_paused = true)]
async fn the_harness_errors_when_a_state_has_no_value() {
    use blueos_service::testing::HarnessError;

    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();

    assert!(matches!(
        harness.state::<PumpState>("missing").await,
        Err(HarnessError::ReplyCount { .. })
    ));
}

#[tokio::test(start_paused = true)]
async fn a_domain_without_jobs_still_lists_the_instant_jobs_it_ran() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();

    let ack = harness
        .send("SetLevel", &SetLevelGoal { level: 42 })
        .await
        .unwrap();

    assert_eq!(ack.status, CommandAckStatus::Succeeded);
    let jobs = harness.jobs().await.unwrap();
    let [job] = jobs.jobs.as_slice() else {
        panic!("expected one Job, got {jobs:?}");
    };
    assert_eq!(
        (job.job_id.as_str(), job.job_type.as_str(), job.status),
        (ack.job_id.as_str(), "SetLevel", JobStatusStatus::Succeeded)
    );
}

#[tokio::test(start_paused = true)]
async fn a_request_its_conversion_refuses_is_rejected_before_the_domain() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 10 })
        .await
        .unwrap();

    let refused = harness
        .send("SetPercent", &SetLevelGoal { level: 150 })
        .await
        .unwrap();
    let applied = harness
        .send("SetPercent", &SetLevelGoal { level: 50 })
        .await
        .unwrap();

    assert!(!refused.accepted);
    assert_eq!(refused.reason, "150 is not a percentage");
    assert!(applied.accepted, "{}", applied.reason);
    assert_eq!(harness.state::<PumpState>("tank").await.unwrap().level, 5);
}

#[tokio::test(start_paused = true)]
async fn a_query_is_answered_from_the_snapshot_left_by_the_commands_before_it() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelGoal { level: 42 })
        .await
        .unwrap();

    let answer = harness
        .query::<_, LevelResponse>("Level", &Empty::default())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(answer.level, 42);
}
