//! Kernel Effects: ordered IO, typed timers, and synchronous Effect rollback.

mod effects_fixture;

use core::time::Duration;
use std::sync::{Arc, mpsc};

use blueos_domain::Effect;
use blueos_idl::msg::{blueos_example_msgs::LevelResponse, std_msgs::Empty};
use blueos_service::testing::Harness;
use tokio::time::advance;

use effects_fixture::domain::EffectsWithoutIoService;
use effects_fixture::helpers::{
    BlockingHoldChannels, assert_blocking_active_level, await_blocking_hold_applied,
    await_blocking_hold_started, blocking_hold_channels, expect_no_state_sample, next_state_sample,
    spawn_run_hold_command, subscribe_state,
};
use effects_fixture::service::{EffectsArguments, EffectsService, EffectsTick, EffectsTimerKey};

#[tokio::test(start_paused = true)]
async fn failed_first_io_still_runs_second() {
    let harness = Harness::<EffectsService>::start(EffectsArguments::default())
        .await
        .unwrap();
    let mut io_states = subscribe_state(&harness, "io").await;
    let ack = harness.send("RunIoChain", &Empty::default()).await.unwrap();
    assert!(ack.accepted);
    let after_fail = next_state_sample(&mut io_states).await;
    assert_eq!(after_fail.level, 1);
    assert_eq!(after_fail.max_level, 0);
    let after_success = next_state_sample(&mut io_states).await;
    assert_eq!(after_success.level, 1);
    assert_eq!(after_success.max_level, 1);
}

#[tokio::test(start_paused = true)]
async fn io_panic_reaches_domain_as_io_failed() {
    let harness = Harness::<EffectsService>::start(EffectsArguments::default())
        .await
        .unwrap();
    let mut io_states = subscribe_state(&harness, "io").await;
    let ack = harness.send("RunIoPanic", &Empty::default()).await.unwrap();
    assert!(ack.accepted);
    let after_panic = next_state_sample(&mut io_states).await;
    assert_eq!(after_panic.level, 1);
    assert_eq!(after_panic.max_level, 0);
}

#[tokio::test(start_paused = true)]
async fn rearmed_timer_fires_once_at_new_time() {
    let harness = Harness::<EffectsService>::start(EffectsArguments::default())
        .await
        .unwrap();
    let mut tick_states = subscribe_state(&harness, "ticks").await;
    harness.send("ArmTimer", &Empty::default()).await.unwrap();
    harness.send("ReArmTimer", &Empty::default()).await.unwrap();
    advance(Duration::from_secs(5)).await;
    assert_eq!(next_state_sample(&mut tick_states).await.level, 1);
    advance(Duration::from_secs(10)).await;
    expect_no_state_sample(&mut tick_states).await;
}

#[tokio::test(start_paused = true)]
async fn cancelled_timer_never_fires() {
    let harness = Harness::<EffectsService>::start(EffectsArguments::default())
        .await
        .unwrap();
    let mut tick_states = subscribe_state(&harness, "ticks").await;
    harness
        .send("CancelTimer", &Empty::default())
        .await
        .unwrap();
    advance(Duration::from_secs(20)).await;
    expect_no_state_sample(&mut tick_states).await;
}

#[tokio::test(start_paused = true)]
async fn synchronous_io_effect_failure_rolls_back_command() {
    let harness = Harness::<EffectsWithoutIoService>::start(EffectsArguments::default())
        .await
        .unwrap();
    let ack = harness
        .send("ScheduleIoWithoutExecutor", &Empty::default())
        .await
        .unwrap();
    assert!(!ack.accepted);
    assert!(ack.reason.contains("IO executor"));
    assert_eq!(
        harness.state::<LevelResponse>("level").await.unwrap().level,
        7
    );
}

#[tokio::test(start_paused = true)]
async fn io_executor_receives_context_and_snapshot() {
    let (done_sender, done_receiver) = mpsc::sync_channel(1);
    let wait_for_io = tokio::task::spawn_blocking(move || done_receiver.recv());
    let harness = Harness::<EffectsService>::start(EffectsArguments {
        capacity: 10,
        record_capacity_done: Some(done_sender),
        ..EffectsArguments::default()
    })
    .await
    .unwrap();
    let ack = harness
        .send("RecordCapacity", &Empty::default())
        .await
        .unwrap();
    assert!(ack.accepted);
    wait_for_io
        .await
        .expect("join")
        .expect("RecordCapacity IO should run");
}

#[tokio::test(start_paused = true)]
async fn query_answers_while_blocking_io_is_held() {
    let BlockingHoldChannels {
        latch,
        started_receiver,
        release_sender,
        io_applied_receiver,
    } = blocking_hold_channels();
    let harness = Harness::<EffectsService>::start(EffectsArguments {
        capacity: 10,
        blocking_hold: Some(latch),
        ..EffectsArguments::default()
    })
    .await
    .unwrap();
    let command = spawn_run_hold_command(Arc::clone(harness.backend()), "RunBlockingHold");
    await_blocking_hold_started(started_receiver).await;
    assert_blocking_active_level(&harness, 1).await;
    release_sender
        .send(())
        .expect("the test should release blocking IO");
    await_blocking_hold_applied(io_applied_receiver).await;
    command.await.expect("the Command should finish");
    assert_blocking_active_level(&harness, 0).await;
}

#[tokio::test(start_paused = true)]
async fn effect_recorder_sees_effects_without_running_them() {
    let (harness, log) =
        Harness::<EffectsService>::start_recording_effects(EffectsArguments::default())
            .await
            .unwrap();
    let mut tick_states = subscribe_state(&harness, "ticks").await;
    let ack = harness
        .send("CancelTimer", &Empty::default())
        .await
        .unwrap();
    assert!(ack.accepted);
    let batch = log.last_batch().expect("one Command was applied");
    assert_eq!(
        batch,
        vec![
            Effect::Schedule {
                after: Duration::from_secs(10),
                key: EffectsTimerKey::Alarm,
                command: EffectsTick::Fired,
            },
            Effect::Cancel(EffectsTimerKey::Alarm),
        ]
    );
    advance(Duration::from_secs(20)).await;
    expect_no_state_sample(&mut tick_states).await;
}
