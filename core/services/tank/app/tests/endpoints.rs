//! Every endpoint of the tank, through the generated `register`, as a client sees it (layer L3).

use core::time::Duration;

use tokio::time::timeout;

use blueos_api::{Message, event_key};
use blueos_comms::ReplyError;
use blueos_idl::msg::blueos_example_msgs::{EmptyRequest, LevelQueryResponse, SetLevelRequest};
use blueos_service::{Service, testing::Harness};
use blueos_tank_app::{cli::TankArguments, service::TankService};

#[tokio::test(start_paused = true)]
async fn set_level_changes_the_state_and_publishes_the_event() {
    let harness = start(0).await;
    let mut events = harness
        .backend()
        .subscribe(&event_key(TankService::NAME, "LevelChanged"))
        .await
        .unwrap();

    let ack = harness
        .send("SetLevel", &SetLevelRequest { level: 40 })
        .await;

    assert!(ack.accepted);
    assert_eq!(harness.state::<LevelQueryResponse>("tank").await, level(40));
    let event = timeout(Duration::from_secs(10), events.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        LevelQueryResponse::decode(&event.payload().to_bytes()).unwrap(),
        level(40)
    );
}

#[tokio::test(start_paused = true)]
async fn set_level_refuses_a_level_that_is_not_a_percentage() {
    let harness = start(0).await;

    let ack = harness
        .send("SetLevel", &SetLevelRequest { level: 150 })
        .await;

    assert!(!ack.accepted);
    assert_eq!(ack.reason, "150 is not a percentage");
}

#[tokio::test(start_paused = true)]
async fn drain_empties_the_tank() {
    let harness = start(0).await;
    harness
        .send("SetLevel", &SetLevelRequest { level: 40 })
        .await;

    let ack = harness.send("Drain", &EmptyRequest::default()).await;

    assert!(ack.accepted);
    assert_eq!(harness.state::<LevelQueryResponse>("tank").await, level(0));
}

#[tokio::test(start_paused = true)]
async fn level_answers_how_full_the_tank_is() {
    let harness = start(0).await;
    harness
        .send("SetLevel", &SetLevelRequest { level: 40 })
        .await;

    let answer = harness
        .query::<_, LevelQueryResponse>("Level", &EmptyRequest::default())
        .await;

    assert_eq!(answer.unwrap(), level(40));
}

#[tokio::test(start_paused = true)]
async fn level_after_fill_stops_at_full_and_refuses_what_is_not_a_percentage() {
    let harness = start(0).await;
    harness
        .send("SetLevel", &SetLevelRequest { level: 70 })
        .await;

    let full = harness
        .query::<_, LevelQueryResponse>("LevelAfterFill", &SetLevelRequest { level: 50 })
        .await;
    let refused = harness
        .query::<_, LevelQueryResponse>("LevelAfterFill", &SetLevelRequest { level: 101 })
        .await;

    assert_eq!(full.unwrap(), level(100));
    assert_eq!(reason(refused), "101 is not a percentage");
}

#[tokio::test(start_paused = true)]
async fn probe_reads_the_sensor_outside_the_inbox() {
    let harness = start(30).await;

    let answer = harness
        .query::<_, LevelQueryResponse>("Probe", &EmptyRequest::default())
        .await;

    assert_eq!(answer.unwrap(), level(30));
}

#[tokio::test(start_paused = true)]
async fn probe_refuses_a_sensor_reading_that_is_not_a_percentage() {
    let harness = start(130).await;

    let answer = harness
        .query::<_, LevelQueryResponse>("Probe", &EmptyRequest::default())
        .await;

    assert_eq!(reason(answer), "130 is not a percentage");
}

async fn start(sensor_level: u8) -> Harness<TankService> {
    Harness::start(TankArguments { sensor_level })
        .await
        .unwrap()
}

fn level(level: u8) -> LevelQueryResponse {
    LevelQueryResponse {
        level,
        max_level: 100,
    }
}

fn reason(answer: Result<LevelQueryResponse, ReplyError>) -> String {
    let error = answer.expect_err("the query is refused");
    String::from_utf8(error.payload().to_bytes().into_owned()).unwrap()
}
