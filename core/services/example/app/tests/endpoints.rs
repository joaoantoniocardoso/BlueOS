//! `example-minimal` through the real `build` and the generated `register` (layer L3).

use core::time::Duration;
use std::{collections::BTreeMap, sync::Arc};

use tokio::time::timeout;

use serde::Deserialize;

use blueos_api::{Message, status_state_key};
use blueos_comms::{CommsBackend, channel::ChannelBackend};
use blueos_example_app::{cli::ExampleArguments, service::ExampleService};
use blueos_example_domain::MAX_LEVEL;
use blueos_idl::msg::{
    blueos_example_msgs::{EmptyRequest, LevelQueryResponse, PumpState, SetLevelRequest},
    blueos_msgs::{CommandAck, ServiceInfo, ServiceStatus, ServiceStatusStatus},
};
use blueos_service::{Service, testing::Harness};

struct ManifestEndpoint {
    kind: &'static str,
    name: String,
    request_schema: String,
    response_schema: String,
}

#[derive(Deserialize)]
struct Manifest {
    service: String,
    #[serde(default)]
    command: BTreeMap<String, CommandEntry>,
    #[serde(default)]
    query: BTreeMap<String, QueryEntry>,
    #[serde(default)]
    state: BTreeMap<String, StateEntry>,
}

#[derive(Deserialize)]
struct CommandEntry {
    request: String,
}

#[derive(Deserialize)]
struct QueryEntry {
    request: String,
    response: String,
}

#[derive(Deserialize)]
struct StateEntry {
    message: String,
}

fn manifest_endpoints_from_toml() -> BTreeMap<String, ManifestEndpoint> {
    let manifest: Manifest =
        toml::from_str(include_str!("../endpoints.toml")).expect("endpoints.toml parses");
    let service = manifest.service;
    let mut expected = BTreeMap::new();
    for (name, entry) in manifest.command {
        let (request_schema, response_schema) = match name.as_str() {
            "SetLevel" => {
                assert_eq!(entry.request, SetLevelRequest::SCHEMA_NAME);
                (
                    SetLevelRequest::SCHEMA_NAME.to_owned(),
                    CommandAck::SCHEMA_NAME.to_owned(),
                )
            }
            other => panic!("unexpected command {other} in endpoints.toml"),
        };
        let key = format!("blueos/v1/{service}/command/{name}");
        expected.insert(
            key,
            ManifestEndpoint {
                kind: "command",
                name,
                request_schema,
                response_schema,
            },
        );
    }
    for (name, entry) in manifest.query {
        let (request_schema, response_schema) = match name.as_str() {
            "Level" => {
                assert_eq!(entry.request, EmptyRequest::SCHEMA_NAME);
                assert_eq!(entry.response, LevelQueryResponse::SCHEMA_NAME);
                (
                    EmptyRequest::SCHEMA_NAME.to_owned(),
                    LevelQueryResponse::SCHEMA_NAME.to_owned(),
                )
            }
            other => panic!("unexpected query {other} in endpoints.toml"),
        };
        let key = format!("blueos/v1/{service}/query/{name}");
        expected.insert(
            key,
            ManifestEndpoint {
                kind: "query",
                name,
                request_schema,
                response_schema,
            },
        );
    }
    for (name, entry) in manifest.state {
        let response_schema = match name.as_str() {
            "pump" => {
                assert_eq!(entry.message, PumpState::SCHEMA_NAME);
                PumpState::SCHEMA_NAME.to_owned()
            }
            other => panic!("unexpected state {other} in endpoints.toml"),
        };
        let key = format!("blueos/v1/{service}/state/{name}");
        expected.insert(
            key,
            ManifestEndpoint {
                kind: "state",
                name,
                request_schema: String::new(),
                response_schema,
            },
        );
    }
    expected
}

#[tokio::test(start_paused = true)]
async fn info_lists_every_manifest_endpoint_with_schemas() {
    let harness = Harness::<ExampleService>::start(ExampleArguments::default())
        .await
        .unwrap();
    let info = harness
        .query::<EmptyRequest, ServiceInfo>("info", &EmptyRequest::default())
        .await
        .expect("the info query answers");
    let expected = manifest_endpoints_from_toml();
    for (key, manifest) in &expected {
        let endpoint = info
            .endpoints
            .iter()
            .find(|endpoint| endpoint.key == *key)
            .unwrap_or_else(|| panic!("info must list manifest endpoint {key}"));
        assert_eq!(endpoint.kind, manifest.kind);
        assert_eq!(endpoint.name, manifest.name);
        assert_eq!(endpoint.request_schema, manifest.request_schema);
        assert_eq!(endpoint.response_schema, manifest.response_schema);
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
