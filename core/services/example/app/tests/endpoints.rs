//! `example-minimal` through the real `build` and the generated `register` (layer L3).

use blueos_example_app::{cli::ExampleArguments, service::ExampleService};
use blueos_example_domain::MAX_LEVEL;
use blueos_idl::msg::blueos_example_msgs::{
    EmptyRequest, LevelQueryResponse, PumpState, SetLevelRequest,
};
use blueos_service::testing::Harness;

#[tokio::test(start_paused = true)]
async fn set_level_updates_the_pump_state() {
    let harness = Harness::<ExampleService>::start(ExampleArguments::default())
        .await
        .unwrap();

    let ack = harness
        .send("SetLevel", &SetLevelRequest { level: 55 })
        .await;
    assert!(ack.accepted);

    let pump = harness.state::<PumpState>("pump").await;
    assert_eq!(pump.level, 55);
    assert_eq!(pump.max_level, MAX_LEVEL);
    assert!(!pump.self_test_active);
}

#[tokio::test(start_paused = true)]
async fn set_level_refuses_a_level_above_the_maximum() {
    let harness = Harness::<ExampleService>::start(ExampleArguments::default())
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelRequest { level: 55 })
        .await;

    let ack = harness
        .send("SetLevel", &SetLevelRequest { level: 150 })
        .await;

    assert!(!ack.accepted);
    assert_eq!(ack.reason, "150 is above the maximum level of 100");
    assert_eq!(harness.state::<PumpState>("pump").await.level, 55);
}

#[tokio::test(start_paused = true)]
async fn level_query_reads_the_snapshot() {
    let harness = Harness::<ExampleService>::start(ExampleArguments::default())
        .await
        .unwrap();
    harness
        .send("SetLevel", &SetLevelRequest { level: 12 })
        .await;

    let answer = harness
        .query::<_, LevelQueryResponse>("Level", &EmptyRequest::default())
        .await
        .unwrap();

    assert_eq!(answer.level, 12);
    assert_eq!(answer.max_level, MAX_LEVEL);
}
