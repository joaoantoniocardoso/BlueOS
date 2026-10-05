//! Kernel integration tests.

mod kernel_fixture;

use core::sync::atomic::Ordering;
use core::time::Duration;
use std::sync::Arc;

use blueos_api::{
    Message, cdr_encoding, command_key, event_key, job_result_key, jobs_key, state_key,
};
use blueos_comms::{QueryBody, Sample, Subscriber};
use blueos_domain::Command;
use blueos_idl::msg::{
    blueos_example_msgs::{LevelResponse, PumpState, SetLevelGoal},
    blueos_msgs::{CommandAck, CommandAckStatus},
    builtin_interfaces::Time,
};
use blueos_jobs::JobId;
use blueos_service::{Service, testing::Harness, testing::WALL_CLOCK_AT_START};
use tokio::time::timeout;

use kernel_fixture::*;

#[tokio::test(start_paused = true)]
async fn a_client_reads_the_new_state_right_after_the_ack() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    assert_eq!(harness.state::<PumpState>("tank").await.unwrap().level, 0);

    let ack = harness
        .send("SetLevel", &SetLevelGoal { level: 42 })
        .await
        .unwrap();

    assert!(ack.accepted, "{}", ack.reason);
    let state = harness.state::<PumpState>("tank").await.unwrap();
    assert_eq!((state.level, state.max_level), (42, 100));
}

#[tokio::test(start_paused = true)]
async fn a_rejected_request_leaves_the_state_and_tells_the_client_why() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 10 })
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelGoal { level: 4 })
        .await
        .unwrap();

    let ack = harness
        .send("SetLevel", &SetLevelGoal { level: 11 })
        .await
        .unwrap();

    assert!(!ack.accepted);
    assert_eq!(ack.reason, "level 11 is above the capacity 10");
    assert_eq!(ack.status, CommandAckStatus::StatusUnknown);
    assert_eq!(harness.state::<PumpState>("tank").await.unwrap().level, 4);
}

#[tokio::test(start_paused = true)]
async fn states_are_published_before_the_ack_and_events_after_it() {
    let backend = Arc::new(RecordingBackend::default());
    let harness = Harness::<TankService>::start_on(
        Arc::<RecordingBackend>::clone(&backend),
        TankArguments { capacity: 100 },
    )
    .await
    .unwrap();
    backend.take_journal();

    harness
        .send("SetLevel", &SetLevelGoal { level: 3 })
        .await
        .unwrap();

    assert_eq!(
        backend.take_journal(),
        [
            format!("publish {}", state_key(TankService::NAME, "level_set_at")),
            format!("publish {}", state_key(TankService::NAME, "tank")),
            format!("publish {}", jobs_key(TankService::NAME)),
            format!("reply {}", command_key(TankService::NAME, "SetLevel")),
            format!("publish {}", event_key(TankService::NAME, "LevelChanged")),
            format!("publish {}", job_result_key(TankService::NAME, "SetLevel")),
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn send_awaiting_ack_returns_rejection_when_handle_panics() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelGoal { level: 5 })
        .await
        .unwrap();
    let ack = harness
        .command_sender()
        .send_awaiting_ack(Command::Request(TankRequest::SetLevel(
            LEVEL_THAT_PANICS_IN_HANDLE,
        )))
        .await
        .expect("the Inbox stays open");
    assert!(!ack.accepted);
    assert_eq!(ack.reason, "the Command panicked, so nothing changed");
    assert_eq!(harness.state::<PumpState>("tank").await.unwrap().level, 5);
}

#[tokio::test(start_paused = true)]
async fn a_panic_in_handle_restores_the_snapshot_and_rejects_the_ack() {
    assert_a_panic_changes_nothing(LEVEL_THAT_PANICS_IN_HANDLE).await;
}

#[tokio::test(start_paused = true)]
async fn a_panic_in_a_projection_restores_the_snapshot_and_drops_the_domain_events() {
    assert_a_panic_changes_nothing(LEVEL_THAT_PANICS_IN_PROJECTION).await;
}

async fn assert_a_panic_changes_nothing(level_that_panics: u8) {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelGoal { level: 5 })
        .await
        .unwrap();
    let mut events = harness
        .backend()
        .subscribe(&event_key(TankService::NAME, "LevelChanged"))
        .await
        .unwrap();

    let ack = harness
        .send(
            "SetLevel",
            &SetLevelGoal {
                level: level_that_panics,
            },
        )
        .await
        .unwrap();

    assert!(!ack.accepted);
    assert_eq!(ack.reason, "the Command panicked, so nothing changed");
    assert_eq!(harness.state::<PumpState>("tank").await.unwrap().level, 5);
    harness
        .send("SetLevel", &SetLevelGoal { level: 6 })
        .await
        .unwrap();
    let first_event = next_sample(&mut events).await;
    let first_event = LevelResponse::decode(&first_event.payload().to_bytes()).unwrap();
    assert_eq!(first_event.level, 6);
}

#[tokio::test(start_paused = true)]
async fn a_failed_publish_does_not_stop_the_inbox_and_the_state_is_sent_on_the_next_command() {
    let backend = Arc::new(RecordingBackend::default());
    let harness = Harness::<TankService>::start_on(
        Arc::<RecordingBackend>::clone(&backend),
        TankArguments { capacity: 100 },
    )
    .await
    .unwrap();
    let mut states = harness
        .backend()
        .subscribe(&state_key(TankService::NAME, "tank"))
        .await
        .unwrap();
    backend.publishes_fail.store(true, Ordering::SeqCst);

    let ack = harness
        .send("SetLevel", &SetLevelGoal { level: 1 })
        .await
        .unwrap();
    backend.publishes_fail.store(false, Ordering::SeqCst);
    // The same State twice: sent by the first, because the failed publish was never stored, and deduplicated by
    // the second.
    for level in [1, 1, 2] {
        harness
            .send("SetLevel", &SetLevelGoal { level })
            .await
            .unwrap();
    }

    assert!(ack.accepted, "{}", ack.reason);
    let mut published_levels = Vec::new();
    for _sample in 0..2 {
        let sample = next_sample(&mut states).await;
        published_levels.push(
            PumpState::decode(&sample.payload().to_bytes())
                .unwrap()
                .level,
        );
    }
    assert_eq!(published_levels, [1, 2]);
}

#[tokio::test(start_paused = true)]
async fn a_state_that_fails_to_encode_does_not_stop_the_inbox() {
    let harness = Harness::<FragileTankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();

    let failing_ack = harness
        .send(
            "SetLevel",
            &SetLevelGoal {
                level: LEVEL_THAT_FAILS_TO_ENCODE,
            },
        )
        .await
        .unwrap();
    let next_ack = harness
        .send("SetLevel", &SetLevelGoal { level: 2 })
        .await
        .unwrap();

    assert!(failing_ack.accepted, "{}", failing_ack.reason);
    assert!(next_ack.accepted, "{}", next_ack.reason);
    assert_eq!(
        harness.state::<FragileLevel>("tank").await.unwrap().level,
        2
    );
}

/// The next sample, or a failed test when none comes. Time is paused, so the timeout costs no real time.
async fn next_sample(subscriber: &mut Subscriber) -> Sample {
    timeout(Duration::from_secs(10), subscriber.recv())
        .await
        .expect("a sample is published")
        .expect("the subscription is open")
}

#[tokio::test(start_paused = true)]
async fn handle_receives_the_time_of_the_injected_clock() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    tokio::time::advance(Duration::from_millis(5_250)).await;

    harness
        .send("SetLevel", &SetLevelGoal { level: 7 })
        .await
        .unwrap();

    let level_set_at = harness.state::<Time>("level_set_at").await.unwrap();
    let expected = WALL_CLOCK_AT_START + Duration::from_millis(5_250);
    assert_eq!(
        (level_set_at.sec, level_set_at.nanosec),
        (i32::try_from(expected.as_secs()).unwrap(), 250_000_000)
    );
}

#[tokio::test(start_paused = true)]
async fn a_request_that_does_not_decode_is_rejected_before_the_domain() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    let body = QueryBody::new(vec![0xFF], cdr_encoding(SetLevelGoal::SCHEMA_NAME))
        .with_attachment(JobId::from_u128(1).to_string().into_bytes());

    let replies = harness
        .backend()
        .get(
            &command_key(TankService::NAME, "SetLevel"),
            Some(body),
            Duration::from_secs(10),
        )
        .await
        .unwrap();

    let [Ok(reply)] = replies.as_slice() else {
        panic!("expected one ack, got {replies:?}");
    };
    let ack = CommandAck::decode(&reply.payload().to_bytes()).unwrap();
    assert!(!ack.accepted);
    assert_eq!(
        ack.reason,
        "the Request does not decode: invalid CDR encapsulation header"
    );
    assert_eq!(harness.state::<PumpState>("tank").await.unwrap().level, 0);
}
