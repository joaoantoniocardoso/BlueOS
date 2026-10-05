//! Recorder session command integration tests.

mod common;

use std::collections::BTreeMap;

use tempfile::tempdir;

use blueos_idl::msg::{
    blueos_msgs::CommandAckStatus,
    blueos_recorder_msgs::{StartRecordingGoal, StopRecordingGoal},
};
use blueos_recorder_app::RecorderService;
use blueos_service::Service;

use common::harness::startup::start_harness;
use common::harness::state::wait_for_active_recording;

#[tokio::test(start_paused = true)]
async fn start_and_stop_acks_carry_their_final_status() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;

    let start_ack = harness
        .send(
            "Start",
            &StartRecordingGoal {
                rotate_if_active: true,
            },
        )
        .await
        .unwrap();
    assert!(start_ack.accepted, "start rejected: {}", start_ack.reason);
    assert_eq!(start_ack.status, CommandAckStatus::Succeeded);
    wait_for_active_recording(harness.backend()).await;

    let stop_ack = harness
        .send("Stop", &StopRecordingGoal::default())
        .await
        .unwrap();
    assert!(stop_ack.accepted, "stop rejected: {}", stop_ack.reason);
    assert_eq!(stop_ack.status, CommandAckStatus::Succeeded);
}

#[tokio::test(start_paused = true)]
async fn recorder_service_info_is_published() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    let info = harness.info().await.unwrap();
    assert_eq!(info.name, RecorderService::NAME);
    assert_eq!(info.version, RecorderService::VERSION);
}

#[tokio::test(start_paused = true)]
async fn info_lists_each_endpoint_with_the_interface_type_of_api_lock_and_its_schema_text() {
    let directory = tempdir().expect("tempdir");
    let harness = start_harness(directory.path()).await;
    let prefix = format!("blueos/v1/{}/", RecorderService::NAME);
    let locked: BTreeMap<&str, &str> = include_str!("../../../../libs/idl/api.lock")
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let key = parts.next().filter(|key| key.starts_with(&prefix))?;
            Some((key, parts.nth(1)?.strip_prefix("type=")?))
        })
        .collect();

    let info = harness.info().await.unwrap();

    assert_eq!(
        info.endpoints.len(),
        10 + 1 + 1 + 3 * 6,
        "{:?}",
        info.endpoints
    );
    for endpoint in &info.endpoints {
        assert_eq!(
            locked.get(endpoint.key.as_str()),
            Some(&endpoint.interface_type.as_str()),
            "{}",
            endpoint.key
        );
        assert!(
            endpoint.schema.starts_with(
                blueos_idl::schema(&endpoint.interface_type).expect("a listed type has a schema")
            ),
            "{}",
            endpoint.key
        );
    }
    let index = info
        .endpoints
        .iter()
        .find(|endpoint| endpoint.name == "index")
        .expect("info lists the index Query");
    assert_eq!(index.kind, "query");
    assert_eq!(
        index.interface_type,
        "blueos_recorder_msgs/srv/RecordingIndex"
    );
}
