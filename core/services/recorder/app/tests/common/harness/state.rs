//! Recorder State subscription waits.

use core::time::Duration;
use std::sync::Arc;

use tokio::time::{advance, timeout};

use blueos_api::{Message, state_key};
use blueos_comms::CommsBackend;
use blueos_idl::msg::blueos_recorder_msgs::{RecordingFileState, RecordingLibrary, RecordingState};
use blueos_recorder_app::RecorderService;
use blueos_service::{Service, testing::Harness};

use super::super::{REPLY_TIMEOUT, STATE_WAIT_TIMEOUT};

pub(crate) async fn wait_for_recording_idle(backend: &Arc<dyn CommsBackend>) {
    wait_for_recording_state(backend, |state| !state.session_active).await;
}

pub(crate) async fn wait_for_active_recording(backend: &Arc<dyn CommsBackend>) {
    wait_for_recording_state(backend, |state| !state.current_file.is_empty()).await;
}

pub(crate) async fn wait_for_recording_bytes(harness: &Harness<RecorderService>, minimum: u64) {
    wait_for_recording_bytes_on(harness.backend(), minimum).await;
}

pub(crate) async fn wait_for_recording_bytes_on(backend: &Arc<dyn CommsBackend>, minimum: u64) {
    advance(Duration::from_secs(1)).await;
    wait_for_recording_state(backend, |state| state.session_bytes_written >= minimum).await;
}

pub(crate) async fn wait_for_rotated_recording(
    backend: &Arc<dyn CommsBackend>,
    previous_file: &str,
) {
    let previous_file = previous_file.to_owned();
    wait_for_recording_state(backend, |state| {
        state.session_active
            && !state.current_file.is_empty()
            && state.current_file != previous_file
    })
    .await;
}

pub(crate) async fn recording_state(backend: &Arc<dyn CommsBackend>) -> RecordingState {
    fetch_recorder_state(backend, "recording").await
}

pub(crate) async fn library_state(backend: &Arc<dyn CommsBackend>) -> RecordingLibrary {
    fetch_recorder_state(backend, "library").await
}

pub(crate) async fn wait_for_recording_state(
    backend: &Arc<dyn CommsBackend>,
    predicate: impl Fn(&RecordingState) -> bool,
) {
    wait_for_recorder_state(backend, "recording", predicate).await;
}

pub(crate) async fn wait_for_library_state(
    backend: &Arc<dyn CommsBackend>,
    predicate: impl Fn(&RecordingLibrary) -> bool,
) {
    wait_for_recorder_state(backend, "library", predicate).await;
}

pub(crate) async fn wait_for_library_file_listed(
    harness: &Harness<RecorderService>,
    relative_path: &str,
) {
    let path = relative_path.to_owned();
    wait_for_library_state(harness.backend(), move |library| {
        library.files.iter().any(|file| file.path == path)
    })
    .await;
}

pub(crate) async fn wait_for_library_file_ready_by_name(
    backend: &Arc<dyn CommsBackend>,
    file_name: &str,
) {
    let name = file_name.to_owned();
    wait_for_library_state(backend, move |library| {
        library
            .files
            .iter()
            .any(|file| file.name == name && file.state == RecordingFileState::Ready)
    })
    .await;
}

pub(crate) async fn wait_for_library_file_ready(
    harness: &Harness<RecorderService>,
    relative_path: &str,
) {
    let path = relative_path.to_owned();
    wait_for_library_state(harness.backend(), move |library| {
        library
            .files
            .iter()
            .any(|file| file.path == path && file.state == RecordingFileState::Ready)
    })
    .await;
}

pub(crate) async fn wait_for_library_file_not_repairing(
    harness: &Harness<RecorderService>,
    relative_path: &str,
) {
    let path = relative_path.to_owned();
    wait_for_library_state(harness.backend(), move |library| {
        library
            .files
            .iter()
            .all(|file| file.path != path || file.state != RecordingFileState::Repairing)
    })
    .await;
}

async fn fetch_recorder_state<M: Message>(backend: &Arc<dyn CommsBackend>, state: &str) -> M {
    let replies = backend
        .get(
            &state_key(RecorderService::NAME, state),
            None,
            REPLY_TIMEOUT,
        )
        .await
        .expect("state");
    let [Ok(reply)] = replies.as_slice() else {
        panic!("expected one {state} state");
    };
    M::decode(&reply.payload().to_bytes()).expect("decode state")
}

pub(crate) async fn wait_for_recorder_state<M: Message>(
    backend: &Arc<dyn CommsBackend>,
    state: &str,
    predicate: impl Fn(&M) -> bool,
) {
    let key = state_key(RecorderService::NAME, state);
    let mut updates = backend.subscribe(&key).await.expect("subscribe state");
    if predicate(&fetch_recorder_state::<M>(backend, state).await) {
        return;
    }
    timeout(STATE_WAIT_TIMEOUT, async {
        while let Some(sample) = updates.recv().await {
            let value = M::decode(&sample.payload().to_bytes()).expect("decode state");
            if predicate(&value) {
                return;
            }
        }
        panic!("{state} state subscription closed");
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for {state} state"));
}
