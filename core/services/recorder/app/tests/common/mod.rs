//! Shared helpers for recorder app integration tests (paused clock, State waits).

#![expect(
    dead_code,
    reason = "each integration test binary compiles this module separately"
)]

use core::time::Duration;
use std::{fs, path::Path, path::PathBuf, sync::Arc};

use tokio::time::{advance, timeout};

use bytes::Bytes;

use blueos_api::{Message, cdr_encoding, command_key, query_key, state_key};
use blueos_comms::{CommsBackend, Payload, QueryBody, ReplyError, Sample};
use blueos_idl::msg::{
    blueos_example_msgs::PumpState,
    blueos_recorder_msgs::{RecordingState, StartRecordingCommand, StopRecordingCommand},
};
use blueos_recorder_app::{
    IndexQuerySetup, RecorderArguments, RecorderService, build_with_record_gate_and_index,
};
use blueos_service::{
    Kernel, Service, ServiceContext,
    testing::{Harness, PausedClock},
};

pub(crate) const REPLY_TIMEOUT: Duration = Duration::from_secs(10);

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
}

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

pub(crate) async fn start_recorder_test_harness(
    path: &Path,
    index: IndexQuerySetup,
) -> RecorderTestHarness {
    let backend: Arc<dyn CommsBackend> = Arc::new(blueos_comms::channel::ChannelBackend::default());
    let context = ServiceContext::new(recorder_arguments(path, None), Arc::clone(&backend));
    let (builder, _) =
        build_with_record_gate_and_index(&context, index).expect("build recorder service");
    let clock = Arc::new(PausedClock::start());
    let kernel = Kernel::start(RecorderService::NAME, builder, Arc::clone(&backend), clock)
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
