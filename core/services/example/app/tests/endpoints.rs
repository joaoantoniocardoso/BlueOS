//! `example-minimal` through the real `build` and the generated `register` (layer L3).

use core::time::Duration;
use std::{collections::BTreeMap, sync::Arc};

use tokio::time::timeout;

use serde::Deserialize;

use blueos_api::{
    Message, cdr_encoding, info_query_key, job_feedback_key, job_result_key, status_state_key,
};
use blueos_comms::{CommsBackend, Subscriber, channel::ChannelBackend};
use blueos_example_app::{cli::ExampleArguments, service::ExampleService};
use blueos_example_domain::MAX_LEVEL;
use blueos_idl::msg::{
    blueos_example_msgs::{
        LevelRequest, LevelResponse, PumpState, SetLevelFeedback, SetLevelGoal, SetLevelResult,
    },
    blueos_msgs::{
        CommandAckStatus, JobFeedbackList, JobResult, JobStatusStatus, ServiceInfo, ServiceStatus,
        ServiceStatusStatus,
    },
};
use blueos_service::{Service, new_job_id, testing::Harness};

/// `endpoints.toml`: each table maps an endpoint name to its interface type.
#[derive(Deserialize)]
struct Manifest {
    service: String,
    #[serde(default)]
    job: BTreeMap<String, Entry>,
    #[serde(default)]
    query: BTreeMap<String, Entry>,
    #[serde(default)]
    state: BTreeMap<String, Entry>,
}

#[derive(Deserialize)]
struct Entry {
    #[serde(rename = "type")]
    interface_type: String,
}

/// Each endpoint key of `endpoints.toml`, with its kind, name and interface type.
fn manifest_endpoints() -> BTreeMap<String, (&'static str, String, String)> {
    let manifest: Manifest =
        toml::from_str(include_str!("../endpoints.toml")).expect("endpoints.toml parses");
    let service = manifest.service;
    let tables = [
        ("job", "command", manifest.job),
        ("query", "query", manifest.query),
        ("state", "state", manifest.state),
    ];
    let mut expected = BTreeMap::new();
    for (kind, segment, table) in tables {
        for (name, entry) in table {
            expected.insert(
                format!("blueos/v1/{service}/{segment}/{name}"),
                (kind, name, entry.interface_type),
            );
        }
    }
    expected
}

/// The `example` lines of `api.lock`: each endpoint key with its interface type.
fn locked_endpoints() -> BTreeMap<String, String> {
    let prefix = format!("blueos/v1/{}/", ExampleService::NAME);
    include_str!("../../../../libs/idl/api.lock")
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let key = parts.next()?;
            key.strip_prefix(&prefix)?;
            let interface_type = parts.nth(1)?.strip_prefix("type=")?;
            Some((key.to_owned(), interface_type.to_owned()))
        })
        .collect()
}

#[tokio::test(start_paused = true)]
async fn the_kernel_serves_the_keys_and_messages_of_api_lock() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let harness =
        Harness::<ExampleService>::start_on(Arc::clone(&backend), ExampleArguments::default())
            .await
            .unwrap();
    let locked = locked_endpoints();

    for key in [
        info_query_key(ExampleService::NAME),
        status_state_key(ExampleService::NAME),
    ] {
        let interface_type = locked
            .get(&key)
            .unwrap_or_else(|| panic!("api.lock must record {key}"));
        let replies = backend
            .get(&key, None, Duration::from_secs(10))
            .await
            .expect("the key is valid");
        let [Ok(reply)] = replies.as_slice() else {
            panic!("the Kernel must answer {key} once, got {replies:?}");
        };
        assert_eq!(reply.encoding(), cdr_encoding(interface_type), "{key}");
    }

    let info = harness
        .query::<LevelRequest, ServiceInfo>("info", &LevelRequest::default())
        .await
        .expect("the harness reaches the info query")
        .expect("the info query answers");
    for endpoint in info.endpoints {
        assert_eq!(
            locked.get(&endpoint.key),
            Some(&endpoint.interface_type),
            "api.lock must record {} with the interface type info reports",
            endpoint.key
        );
    }
}

#[tokio::test(start_paused = true)]
async fn info_lists_every_manifest_endpoint_with_its_interface_type_and_schema_text() {
    let harness = Harness::<ExampleService>::start(ExampleArguments::default())
        .await
        .unwrap();
    let info = harness
        .query::<LevelRequest, ServiceInfo>("info", &LevelRequest::default())
        .await
        .expect("the harness reaches the info query")
        .expect("the info query answers");

    for (key, (kind, name, interface_type)) in manifest_endpoints() {
        let endpoint = info
            .endpoints
            .iter()
            .find(|endpoint| endpoint.key == key)
            .unwrap_or_else(|| panic!("info must list manifest endpoint {key}"));
        assert_eq!(endpoint.kind, kind);
        assert_eq!(endpoint.name, name);
        assert_eq!(endpoint.interface_type, interface_type);
        assert_eq!(
            Some(endpoint.schema.as_str()),
            blueos_idl::schema(&interface_type),
            "{key}"
        );
    }
}

#[tokio::test(start_paused = true)]
async fn status_is_ready_after_startup() {
    let backend: Arc<dyn CommsBackend> = Arc::new(ChannelBackend::default());
    let mut status_subscriber = backend
        .subscribe(&status_state_key(ExampleService::NAME))
        .await
        .expect("status subscribes before startup");
    let _harness =
        Harness::<ExampleService>::start_on(Arc::clone(&backend), ExampleArguments::default())
            .await
            .unwrap();
    let sample = timeout(Duration::from_secs(10), status_subscriber.recv())
        .await
        .expect("status publishes after startup")
        .expect("status stream stays open");
    let status =
        ServiceStatus::decode(&sample.payload().to_bytes()).expect("status payload decodes");
    assert_eq!(status.status, ServiceStatusStatus::Ready);
}

#[tokio::test(start_paused = true)]
async fn set_level_fills_the_pump_to_the_level() {
    let harness = start().await;

    let level = fill(&harness, 3).await;

    assert_eq!(level, 3);
    let pump = harness.state::<PumpState>("pump").await.unwrap();
    assert_eq!(pump.level, 3);
    assert_eq!(pump.max_level, MAX_LEVEL);
    assert!(!pump.self_test_active);
}

#[tokio::test(start_paused = true)]
async fn set_level_publishes_its_feedback_and_job_result_on_the_keys_of_its_job_type() {
    let harness = start().await;
    let mut feedback = subscribe(
        &harness,
        &job_feedback_key(ExampleService::NAME, "SetLevel"),
    )
    .await;
    let mut results = subscribe(&harness, &job_result_key(ExampleService::NAME, "SetLevel")).await;
    let job_id = new_job_id();

    let ack = harness
        .submit("SetLevel", job_id, &SetLevelGoal { level: 2 })
        .await
        .unwrap();

    assert_eq!(ack.status, CommandAckStatus::Executing);
    let mut fed_back = Vec::new();
    for _ in 0..3 {
        let list: JobFeedbackList = next(&mut feedback).await;
        fed_back.push(
            list.jobs
                .into_iter()
                .map(|job| {
                    (
                        job.job_id,
                        SetLevelFeedback::decode(&job.feedback).unwrap().level,
                    )
                })
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(
        fed_back,
        [
            vec![(job_id.to_string(), 0)],
            vec![(job_id.to_string(), 1)],
            vec![],
        ],
        "the Feedback of each step, until the Job ends"
    );
    let result: JobResult = next(&mut results).await;
    assert_eq!(result.job.job_id, job_id.to_string());
    assert_eq!(result.job.status, JobStatusStatus::Succeeded);
    assert_eq!(
        SetLevelResult::decode(&result.result).unwrap(),
        SetLevelResult { level: 2 }
    );
}

#[tokio::test(start_paused = true)]
async fn a_goal_the_conversion_rejects_is_rejected_in_the_ack() {
    let harness = start().await;
    fill(&harness, 5).await;

    let ack = harness
        .send("SetLevel", &SetLevelGoal { level: 150 })
        .await
        .unwrap();

    assert!(!ack.accepted);
    assert_eq!(ack.reason, "150 is above the maximum level of 100");
    assert!(
        harness
            .jobs()
            .await
            .unwrap()
            .jobs
            .iter()
            .all(|job| job.status == JobStatusStatus::Succeeded)
    );
    assert_eq!(harness.state::<PumpState>("pump").await.unwrap().level, 5);
}

#[tokio::test(start_paused = true)]
async fn level_query_reads_the_snapshot() {
    let harness = start().await;
    fill(&harness, 4).await;

    let answer = harness
        .query::<_, LevelResponse>("Level", &LevelRequest::default())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(answer.level, 4);
    assert_eq!(answer.max_level, MAX_LEVEL);
}

async fn start() -> Harness<ExampleService> {
    Harness::start(ExampleArguments::default()).await.unwrap()
}

/// Fills the pump to `level` and returns the level of the Job result, once the Job ends.
async fn fill(harness: &Harness<ExampleService>, level: u8) -> u8 {
    let mut results = subscribe(harness, &job_result_key(ExampleService::NAME, "SetLevel")).await;
    let ack = harness
        .send("SetLevel", &SetLevelGoal { level })
        .await
        .unwrap();
    assert!(ack.accepted, "{}", ack.reason);
    let result: JobResult = next(&mut results).await;
    SetLevelResult::decode(&result.result).unwrap().level
}

async fn subscribe(harness: &Harness<ExampleService>, key: &str) -> Subscriber {
    harness.backend().subscribe(key).await.unwrap()
}

/// The next `M` the Service publishes on `subscriber`. Time is paused, so it advances to the next step at once.
async fn next<M: Message>(subscriber: &mut Subscriber) -> M {
    let sample = timeout(Duration::from_secs(600), subscriber.recv())
        .await
        .unwrap()
        .unwrap();
    M::decode(&sample.payload().to_bytes()).unwrap()
}
