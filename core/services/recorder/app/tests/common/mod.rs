//! Shared helpers for recorder app integration tests (paused clock, State waits).

#![expect(
    dead_code,
    reason = "each integration test binary compiles this module separately"
)]

use core::time::Duration;
use std::{fs, path::Path, path::PathBuf, sync::Arc};

use tokio::time::{advance, timeout};

use blueos_api::{Message, cdr_encoding, command_key, state_key};
use blueos_comms::{CommsBackend, QueryBody};
use blueos_idl::msg::blueos_recorder_msgs::{
    RecordingState, StartRecordingCommand, StopRecordingCommand,
};
use blueos_recorder_app::{RecorderArguments, RecorderService};
use blueos_service::{Service, testing::Harness};

pub(crate) const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

pub(crate) fn recorder_arguments(
    path: &Path,
    mcap_writer_queue_capacity: Option<usize>,
) -> RecorderArguments {
    RecorderArguments {
        recorder_path: path.to_path_buf(),
        mcap_writer_queue_capacity,
    }
}

pub(crate) async fn start_harness(path: &Path) -> Harness<RecorderService> {
    Harness::start(recorder_arguments(path, None))
        .await
        .expect("harness")
}

pub(crate) async fn start_recording(harness: &Harness<RecorderService>) {
    start_recording_on(harness.backend()).await;
}

pub(crate) async fn start_recording_on(backend: &Arc<dyn CommsBackend>) {
    let start = StartRecordingCommand {
        rotate_if_active: false,
    };
    let body = QueryBody::new(
        start.encode().expect("encode"),
        cdr_encoding(StartRecordingCommand::SCHEMA_NAME),
    );
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

/// Stop clears `recording` State before the writer finishes, so no State update marks finalize.
/// Wait for idle, then advance virtual time until the data plane Task completes `finish`.
pub(crate) async fn stop_recording_and_finalize_mcap(backend: &Arc<dyn CommsBackend>, path: &Path) {
    stop_recording_on(backend).await;
    wait_for_recording_idle(backend).await;
    wait_for_mcap_finalized(path).await;
}

pub(crate) async fn stop_recording_on(backend: &Arc<dyn CommsBackend>) {
    let stop = StopRecordingCommand::default();
    let body = QueryBody::new(
        stop.encode().expect("encode"),
        cdr_encoding(StopRecordingCommand::SCHEMA_NAME),
    );
    backend
        .get(
            &command_key(RecorderService::NAME, "Stop"),
            Some(body),
            REPLY_TIMEOUT,
        )
        .await
        .expect("stop");
}

pub(crate) async fn assert_mcap_readable(path: &Path) {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let bytes = fs::read(&path).expect("read mcap");
        mcap::Summary::read(&bytes)
            .expect("parse mcap")
            .expect("mcap summary");
    })
    .await
    .expect("read task");
}

pub(crate) async fn wait_for_mcap_finalized(path: &Path) {
    let path = path.to_path_buf();
    timeout(REPLY_TIMEOUT, async {
        loop {
            let finalized = tokio::task::spawn_blocking({
                let path = path.clone();
                move || -> Option<()> {
                    let bytes = fs::read(&path).ok()?;
                    mcap::Summary::read(&bytes).ok().flatten()?;
                    Some(())
                }
            })
            .await
            .ok()
            .flatten()
            .is_some();
            if finalized {
                return;
            }
            advance(Duration::from_millis(50)).await;
        }
    })
    .await
    .expect("mcap file never finalized");
}

pub(crate) async fn wait_for_recording_idle(backend: &Arc<dyn CommsBackend>) {
    wait_for_recording_state(backend, false, |state| !state.session_active).await;
}

pub(crate) async fn wait_for_active_recording(backend: &Arc<dyn CommsBackend>) {
    wait_for_recording_state(backend, false, |state| !state.current_file.is_empty()).await;
}

pub(crate) async fn wait_for_recording_bytes(harness: &Harness<RecorderService>, minimum: u64) {
    wait_for_recording_bytes_on(harness.backend(), minimum).await;
}

pub(crate) async fn wait_for_recording_bytes_on(backend: &Arc<dyn CommsBackend>, minimum: u64) {
    wait_for_recording_state(backend, true, |state| {
        state.session_bytes_written >= minimum
    })
    .await;
}

pub(crate) async fn recording_state(backend: &Arc<dyn CommsBackend>) -> RecordingState {
    let replies = backend
        .get(
            &state_key(RecorderService::NAME, "recording"),
            None,
            REPLY_TIMEOUT,
        )
        .await
        .expect("state");
    let [Ok(reply)] = replies.as_slice() else {
        panic!("expected one recording state");
    };
    RecordingState::decode(&reply.payload().to_bytes()).expect("decode recording state")
}

pub(crate) async fn wait_for_recording_state(
    backend: &Arc<dyn CommsBackend>,
    advance_bytes_report_interval: bool,
    predicate: impl Fn(&RecordingState) -> bool,
) {
    let key = state_key(RecorderService::NAME, "recording");
    let mut updates = backend
        .subscribe(&key)
        .await
        .expect("subscribe recording state");

    if predicate(&recording_state(backend).await) {
        return;
    }

    if advance_bytes_report_interval {
        advance(Duration::from_secs(1)).await;
        if predicate(&recording_state(backend).await) {
            return;
        }
    }

    timeout(REPLY_TIMEOUT, async {
        while let Some(sample) = updates.recv().await {
            let state = RecordingState::decode(&sample.payload().to_bytes())
                .expect("decode recording state");
            if predicate(&state) {
                return;
            }
        }
        panic!("recording state subscription closed");
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for recording state"));
}

pub(crate) fn recorder_mcaps(directory: &Path) -> Vec<PathBuf> {
    fs::read_dir(directory)
        .expect("read dir")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("recorder_") && name.ends_with(".mcap"))
        })
        .collect()
}

pub(crate) fn recorder_mcap_paths(directory: &Path) -> Vec<PathBuf> {
    let mut paths = recorder_mcaps(directory);
    paths.sort();
    paths
}

pub(crate) fn single_recorder_mcap(directory: &Path) -> PathBuf {
    let paths = recorder_mcap_paths(directory);
    assert_eq!(paths.len(), 1);
    paths[0].clone()
}
