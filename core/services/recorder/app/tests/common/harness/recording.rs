//! Start, stop, and finalize recording through the harness.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::time::advance;

use bytes::Bytes;

use blueos_api::{Message, cdr_encoding, command_key};
use blueos_comms::{CommsBackend, Payload, QueryBody, Sample};
use blueos_idl::msg::{
    blueos_example_msgs::PumpState,
    blueos_recorder_msgs::{StartRecordingGoal, StopRecordingGoal},
};
use blueos_recorder_app::RecorderService;
use blueos_recorder_library::RESCAN_INTERVAL;
use blueos_service::{Service, new_job_id, testing::Harness};

use super::{
    super::REPLY_TIMEOUT,
    state::{
        recording_state, wait_for_active_recording, wait_for_library_file_ready_by_name,
        wait_for_recording_idle,
    },
};

pub(crate) async fn publish_pump_state(backend: &Arc<dyn CommsBackend>) {
    let message = PumpState::default();
    backend
        .publish(Sample::new(
            "blueos/v1/example/state/pump",
            Payload::new(Bytes::from(message.encode().expect("encode"))),
            cdr_encoding(PumpState::SCHEMA_NAME),
        ))
        .await
        .expect("publish");
}

pub(crate) async fn start_recording(harness: &Harness<RecorderService>) {
    start_recording_on(harness.backend()).await;
}

pub(crate) async fn start_recording_on(backend: &Arc<dyn CommsBackend>) {
    let start = StartRecordingGoal {
        rotate_if_active: false,
    };
    let body = QueryBody::new(
        start.encode().expect("encode"),
        cdr_encoding(StartRecordingGoal::SCHEMA_NAME),
    )
    .with_attachment(new_job_id().to_string().into_bytes());
    backend
        .get(
            &command_key(RecorderService::NAME, "Start"),
            Some(body),
            REPLY_TIMEOUT,
        )
        .await
        .expect("start");
}

pub(crate) async fn stop_recording(harness: &Harness<RecorderService>) {
    stop_recording_on(harness.backend()).await;
}

pub(crate) async fn active_recording_mcap_path(
    harness: &Harness<RecorderService>,
    directory: &Path,
) -> PathBuf {
    active_recording_mcap_path_on(harness.backend(), directory).await
}

pub(crate) async fn active_recording_mcap_path_on(
    backend: &Arc<dyn CommsBackend>,
    directory: &Path,
) -> PathBuf {
    wait_for_active_recording(backend).await;
    directory.join(recording_state(backend).await.current_file)
}

/// Stop clears `recording` State before the writer finishes; wait for idle, rescan, and library Ready.
pub(crate) async fn stop_recording_and_finalize_mcap(backend: &Arc<dyn CommsBackend>, path: &Path) {
    let file_name = path
        .file_name()
        .expect("recording path has a file name")
        .to_string_lossy()
        .into_owned();
    stop_recording_on(backend).await;
    wait_for_recording_idle(backend).await;
    advance(RESCAN_INTERVAL).await;
    wait_for_library_file_ready_by_name(backend, &file_name).await;
}

pub(crate) async fn stop_recording_on(backend: &Arc<dyn CommsBackend>) {
    let stop = StopRecordingGoal::default();
    let body = QueryBody::new(
        stop.encode().expect("encode"),
        cdr_encoding(StopRecordingGoal::SCHEMA_NAME),
    )
    .with_attachment(new_job_id().to_string().into_bytes());
    backend
        .get(
            &command_key(RecorderService::NAME, "Stop"),
            Some(body),
            REPLY_TIMEOUT,
        )
        .await
        .expect("stop");
}
