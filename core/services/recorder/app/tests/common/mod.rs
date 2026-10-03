//! Shared helpers for recorder app integration tests (paused clock, State waits).

#![expect(
    dead_code,
    reason = "each integration test binary compiles this module separately"
)]

use core::time::Duration;
use std::{fs, path::Path, path::PathBuf, sync::Arc};

use tokio::time::{advance, sleep, timeout};

use bytes::Bytes;

use blueos_api::{CommandAck, Message, cdr_encoding, command_key, query_key, state_key};
use blueos_comms::{CommsBackend, Payload, QueryBody, ReplyError, Sample};
use blueos_idl::msg::{
    blueos_example_msgs::PumpState,
    blueos_recorder_msgs::{
        RecordingFileState, RecordingLibrary, RecordingState, StartRecordingCommand,
        StopRecordingCommand,
    },
};
use blueos_recorder_app::{
    IndexQuerySetup, RecorderArguments, RecorderService, RepairIoSetup,
    build_with_record_gate_index_and_repair,
};
use blueos_recorder_library::RESCAN_INTERVAL;
use blueos_service::{
    Kernel, Service, ServiceContext,
    testing::{Harness, PausedClock},
};

pub(crate) const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

/// Upper bound for waiting on pushed State while blocking IO runs on wall time.
pub(crate) const STATE_WAIT_TIMEOUT: Duration = Duration::from_secs(30);

/// Recorder harness with a custom index walk (the stock [`Harness`] always uses production wiring).
pub(crate) struct RecorderTestHarness {
    pub backend: Arc<dyn CommsBackend>,
    kernel: tokio::task::JoinSet<()>,
}

impl RecorderTestHarness {
    pub(crate) async fn query<Q: Message, R: Message>(
        &self,
        query: &str,
        request: &Q,
    ) -> Result<R, ReplyError> {
        let body = QueryBody::new(
            request.encode().expect("the request encodes"),
            cdr_encoding(Q::SCHEMA_NAME),
        );
        let replies = self
            .backend
            .get(
                &query_key(RecorderService::NAME, query),
                Some(body),
                REPLY_TIMEOUT,
            )
            .await
            .expect("the query key is valid");
        let [reply] = replies.as_slice() else {
            panic!("expected one reply from {query:?}, got {replies:?}");
        };
        reply
            .clone()
            .map(|sample| R::decode(&sample.payload().to_bytes()).expect("the reply is an R"))
    }

    pub(crate) async fn send<M: Message>(&self, command: &str, request: &M) -> CommandAck {
        let body = QueryBody::new(
            request.encode().expect("the request encodes"),
            cdr_encoding(M::SCHEMA_NAME),
        );
        let replies = self
            .backend
            .get(
                &command_key(RecorderService::NAME, command),
                Some(body),
                REPLY_TIMEOUT,
            )
            .await
            .expect("the command key is valid");
        let [Ok(reply)] = replies.as_slice() else {
            panic!("expected one ack from {command:?}, got {replies:?}");
        };
        CommandAck::decode(&reply.payload().to_bytes()).expect("the reply is a CommandAck")
    }
}

pub(crate) fn recorder_arguments(path: &Path) -> RecorderArguments {
    RecorderArguments {
        recorder_path: path.to_path_buf(),
    }
}

pub(crate) async fn start_harness(path: &Path) -> Harness<RecorderService> {
    Harness::start(recorder_arguments(path))
        .await
        .expect("harness")
}

pub(crate) async fn start_recorder_test_harness(
    path: &Path,
    index: IndexQuerySetup,
) -> RecorderTestHarness {
    start_recorder_test_harness_with(path, index, RepairIoSetup::default()).await
}

pub(crate) async fn start_recorder_test_harness_with(
    path: &Path,
    index: IndexQuerySetup,
    repair: RepairIoSetup,
) -> RecorderTestHarness {
    let backend: Arc<dyn CommsBackend> = Arc::new(blueos_comms::channel::ChannelBackend::default());
    let context = ServiceContext::new(recorder_arguments(path), Arc::clone(&backend));
    let (builder, recorder_context, _) =
        build_with_record_gate_index_and_repair(&context, index, repair, 4096)
            .expect("build recorder service");
    let clock = Arc::new(PausedClock::start());
    let kernel = Kernel::start(
        RecorderService::NAME,
        builder,
        recorder_context,
        Arc::clone(&backend),
        clock,
    )
    .await
    .expect("kernel");
    let mut kernel_tasks = tokio::task::JoinSet::new();
    kernel_tasks.spawn(async move {
        kernel.run().await;
    });
    RecorderTestHarness {
        backend,
        kernel: kernel_tasks,
    }
}

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

/// Waits until outstanding kernel [`spawn_blocking`] IO has finished on a paused runtime.
///
/// Auto-advance is inhibited while a blocking task runs, so a tiny [`sleep`] does not complete
/// until the runtime is idle and no blocking work remains (for example after a library rescan
/// triggered by a preceding [`advance`]).
pub(crate) async fn drain_blocking_io() {
    sleep(Duration::from_millis(1)).await;
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

async fn wait_for_recorder_state<M: Message>(
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
