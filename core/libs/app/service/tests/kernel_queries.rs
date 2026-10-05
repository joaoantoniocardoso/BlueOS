//! Kernel integration tests.

mod kernel_fixture;

use blueos_api::{Message, cdr_encoding, query_key};
use blueos_comms::{QueryBody, ReplyError, Sample};
use blueos_idl::msg::{
    blueos_example_msgs::{LevelResponse, SetLevelGoal},
    std_msgs::Empty,
};
use blueos_service::{Service, testing::Harness};
use core::time::Duration;

use kernel_fixture::*;

#[tokio::test(start_paused = true)]
async fn a_query_without_an_answer_replies_why_and_the_inbox_goes_on() {
    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    let empty = Empty::default();

    let other_response = harness
        .query::<_, LevelResponse>("Other", &empty)
        .await
        .unwrap();
    let panicked = harness
        .query::<_, LevelResponse>("Panics", &empty)
        .await
        .unwrap();
    let refused = harness
        .query::<_, LevelResponse>("Refused", &SetLevelGoal { level: 7 })
        .await
        .unwrap();
    let undecodable = raw_query(&harness, TankService::NAME, "Level").await;

    assert_eq!(
        reason(other_response),
        "the Domain's Response does not belong to this Query"
    );
    assert_eq!(reason(panicked), "the Query panicked");
    assert_eq!(reason(refused), "7 is not a percentage");
    assert_eq!(
        reason(undecodable),
        "the Query does not decode: invalid CDR encapsulation header"
    );
    let answer = harness
        .query::<_, LevelResponse>("Level", &empty)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(answer.level, 0);
}

#[tokio::test(start_paused = true)]
async fn a_query_reply_that_fails_to_encode_replies_why() {
    let harness = Harness::<FragileTankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();
    harness
        .send(
            "SetLevel",
            &SetLevelGoal {
                level: LEVEL_THAT_FAILS_TO_ENCODE,
            },
        )
        .await
        .unwrap();

    let answer = harness
        .query::<_, FragileLevel>("Level", &Empty::default())
        .await
        .unwrap();

    assert_eq!(
        reason(answer),
        "the reply does not encode: invalid CDR length prefix"
    );
}

#[tokio::test(start_paused = true)]
async fn an_io_query_is_answered_outside_the_inbox() {
    let arguments = ProbeArguments::default();
    let sensor = arguments.sensor.clone();
    let harness = Harness::<ProbeService>::start(arguments).await.unwrap();

    // The sensor opens only after a Command was acknowledged, so a Probe that held the Inbox would never end.
    let (answer, ack) = tokio::join!(
        async {
            harness
                .query::<_, LevelResponse>("Probe", &SetLevelGoal { level: 7 })
                .await
                .unwrap()
        },
        async {
            let ack = harness
                .send("SetLevel", &SetLevelGoal { level: 1 })
                .await
                .unwrap();
            sensor.0.add_permits(1);
            ack
        }
    );

    assert!(ack.accepted, "{}", ack.reason);
    assert_eq!(answer.unwrap().level, 7);
}

#[tokio::test(start_paused = true)]
async fn an_io_query_without_an_answer_replies_why_and_the_next_one_is_answered() {
    let arguments = ProbeArguments::default();
    arguments.sensor.0.add_permits(3);
    let harness = Harness::<ProbeService>::start(arguments).await.unwrap();

    let refused = harness
        .query::<_, LevelResponse>("Probe", &SetLevelGoal { level: 101 })
        .await
        .unwrap();
    let panicked = harness
        .query::<_, LevelResponse>(
            "Probe",
            &SetLevelGoal {
                level: LEVEL_THAT_PANICS_IN_HANDLE,
            },
        )
        .await
        .unwrap();
    let undecodable = raw_query(&harness, ProbeService::NAME, "Probe").await;
    let unencodable = harness
        .query::<_, FragileLevel>("FragileProbe", &Empty::default())
        .await
        .unwrap();
    let answer = harness
        .query::<_, LevelResponse>("Probe", &SetLevelGoal { level: 3 })
        .await
        .unwrap();

    assert_eq!(reason(refused), "101 is not a percentage");
    assert_eq!(reason(panicked), "the Query panicked");
    assert_eq!(
        reason(undecodable),
        "the Query does not decode: invalid CDR encapsulation header"
    );
    assert_eq!(
        reason(unencodable),
        "the reply does not encode: invalid CDR length prefix"
    );
    assert_eq!(answer.unwrap().level, 3);
}

#[tokio::test(start_paused = true)]
async fn the_harness_errors_when_a_query_gets_no_reply() {
    use blueos_service::testing::HarnessError;

    let harness = Harness::<TankService>::start(TankArguments { capacity: 100 })
        .await
        .unwrap();

    assert!(matches!(
        harness
            .query::<_, LevelResponse>("Missing", &Empty::default())
            .await,
        Err(HarnessError::ReplyCount { .. })
    ));
}

/// Sends a body that is not CDR to the query endpoint `name`.
async fn raw_query<S: Service>(
    harness: &Harness<S>,
    service: &str,
    name: &str,
) -> Result<Sample, ReplyError> {
    let body = QueryBody::new(vec![0xFF], cdr_encoding(Empty::SCHEMA_NAME));
    let replies = harness
        .backend()
        .get(
            &query_key(service, name),
            Some(body),
            Duration::from_secs(10),
        )
        .await
        .unwrap();
    let [reply] = replies.as_slice() else {
        panic!("expected one reply, got {replies:?}");
    };
    reply.clone()
}

/// The reason in an error reply.
fn reason<T: core::fmt::Debug>(answer: Result<T, ReplyError>) -> String {
    let error = answer.expect_err("the query has no answer");
    assert_eq!(error.encoding(), "text/plain");
    String::from_utf8(error.payload().to_bytes().into_owned()).unwrap()
}
