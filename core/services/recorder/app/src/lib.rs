//! BlueOS recorder service (D-15).
//!
//! ## Control plane vs data plane (D-03)
//!
//! The CQRS domain owns recording policy (armed gating, per-video-stream gating, session lifecycle,
//! MAVLink capture decisions). The data plane is a Zenoh tap on `**` that reads a [`TapPolicy`] snapshot
//! from a watch channel and writes allowed samples to MCAP without routing payloads through the inbox.
//!
//! ## Zero-copy (D-09)
//!
//! Samples are handed to the MCAP writer as [`blueos_comms::Payload`] clones; bytes are materialized on
//! the background writer thread only. Extensions need Docker `IpcMode: host` (or `/dev/shm`) for SHM.

mod cli;
mod error;
mod inject;
pub mod library_io;
mod tap;

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use blueos_api::{cdr_encoding, command_key};
use blueos_comms::{Endpoint, Payload, Session};
use blueos_cqrs::App;
use blueos_idl::Message;
use blueos_idl::msg::blueos_msgs::{
    JobList, ServiceInfo, ServiceStatus, SettingsEnvelope,
    constants_service_status as service_status_constants,
};
use blueos_idl::msg::blueos_recorder_msgs::{
    CancelRepairCommand, DeleteRecordingCommand, RecordingFile, RecordingLibrary,
    RecordingOperation, RecordingState, RepairRecordingCommand, SetPolicyCommand,
    SnapshotRecordingCommand, StartRecordingCommand,
    constants_recording_file as recording_file_constants,
    constants_recording_operation as recording_operation_constants,
};
use blueos_idl::msg::builtin_interfaces::Time;
use blueos_recorder_mavlink::{
    MavlinkFact, SystemAndComponent, build_camera_capture_status, build_command_ack,
    default_discovery_source, discovery_requests_for_camera,
};
use blueos_recorder_mcap::{McapSession, McapWriteConfig, McapWriterHandle};
use blueos_recorder_policy::{
    LibraryRecordingState, RAW_MAVLINK_IN_TOPIC, RecorderCommand, RecorderDomain, RecorderEvent,
    RecorderIo, RecorderSnapshot, RecordingPolicy, TapPolicy,
};
use blueos_service::ServiceBuilder;
use blueos_settings::{SettingsError, SettingsSchema};
use bytes::Bytes;
use tokio::sync::{mpsc, watch};
use tracing::{error, info};

use crate::cli::{mcap_write_config, parse_cli, recorder_directory, schema_directory};
use crate::error::RecorderRunError;
use crate::inject::{decode_injected, encode_injected, fact_to_injected_command};
use crate::library_io::{LibraryIoContext, run_index_query};

const SERVICE_NAME: &str = "recorder";

#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[allow(non_snake_case)]
struct RecorderSettings {
    VERSION: u32,
    record_mavlink_only_when_armed: bool,
    auto_start_recording: bool,
}

impl Default for RecorderSettings {
    fn default() -> Self {
        Self {
            VERSION: 1,
            record_mavlink_only_when_armed: true,
            auto_start_recording: true,
        }
    }
}

impl SettingsSchema for RecorderSettings {
    const VERSION: std::num::NonZeroU32 = std::num::NonZeroU32::new(1).unwrap();

    fn migrate(_data: &mut serde_json::Value) -> Result<(), SettingsError> {
        Ok(())
    }

    fn restart_required_fields() -> &'static [&'static str] {
        &[]
    }
}

struct IoContext {
    recorder_path: PathBuf,
    mcap_config: McapWriteConfig,
    session: Mutex<Option<McapSession>>,
    writer_watch: watch::Sender<Option<Arc<McapWriterHandle>>>,
    mavlink_sequence: Mutex<u8>,
    publish_session: Session,
    library: Arc<LibraryIoContext>,
}

impl Drop for IoContext {
    fn drop(&mut self) {
        let mut guard = match self.session.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };
        if let Some(session) = guard.take()
            && let Err(error) = session.finish()
        {
            error!(%error, "Failed to finish MCAP session on recorder shutdown");
        }
    }
}

fn generate_filename() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_secs() as i64;
    let datetime =
        chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp, 0).expect("timestamp");
    format!("recorder_{}.mcap", datetime.format("%Y%m%d_%H%M%S"))
}

pub fn run(arguments: impl IntoIterator<Item = OsString>) -> std::process::ExitCode {
    let arguments: Vec<String> = arguments
        .into_iter()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect();
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            error!(%error, "Failed to build tokio runtime");
            return std::process::ExitCode::FAILURE;
        }
    };
    match runtime.block_on(run_async(arguments)) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            error!("recorder: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

async fn run_async(arguments: Vec<String>) -> Result<(), RecorderRunError> {
    let cli = parse_cli(&arguments);
    let verbosity = if cli.verbose { 1 } else { 0 };
    let recorder_path = recorder_directory(&cli);
    tokio::fs::create_dir_all(&recorder_path).await?;
    let schema_path = schema_directory(&cli);
    let mcap_config = mcap_write_config(&cli);

    let session = Session::open(SERVICE_NAME, Endpoint::Local).await?;
    let publish_session = session.clone();

    let (policy_watch_sender, policy_watch_receiver) = watch::channel(TapPolicy {
        session_active: false,
        armed: false,
        record_mavlink_only_when_armed: true,
        recording_video_topics: BTreeSet::new(),
    });

    let (writer_watch_sender, writer_watch_receiver) = watch::channel(None);

    let (library_progress_sender, mut library_progress_receiver) = mpsc::channel(256);

    let library_io_context = Arc::new(
        LibraryIoContext::new(recorder_path.clone(), library_progress_sender)
            .map_err(RecorderRunError::RecorderPath)?,
    );
    library_io_context.discard_leftovers();

    let io_context = Arc::new(IoContext {
        recorder_path,
        mcap_config,
        session: Mutex::new(None),
        writer_watch: writer_watch_sender,
        mavlink_sequence: Mutex::new(0),
        publish_session,
        library: library_io_context.clone(),
    });

    let (fact_sender, mut fact_receiver) = mpsc::channel(256);
    let fact_session = session.clone();

    let initial_policy = RecordingPolicy::default();
    let auto_start_recording = initial_policy.auto_start_recording;
    let initial_snapshot = RecorderSnapshot {
        policy: initial_policy,
        ..RecorderSnapshot::default()
    };

    let policy_watch_for_state = policy_watch_sender.clone();

    let mut builder = ServiceBuilder::<RecorderDomain>::new(SERVICE_NAME)
        .app(App::new(initial_snapshot))
        .service_info(ServiceInfo {
            name: SERVICE_NAME.into(),
            version: env!("CARGO_PKG_VERSION").into(),
            build: String::new(),
            capabilities: Vec::new(),
        })
        .verbosity(verbosity)
        .session(session)
        .on_start(RecorderCommand::InitializeLibrary)
        .on_shutdown(RecorderCommand::StopRecording)
        .status(|application| ServiceStatus {
            status: service_status_constants::STATUS_READY,
            detail: application
                .snapshot
                .session
                .as_ref()
                .map(|session| session.file_name.clone())
                .unwrap_or_default(),
        })
        .jobs(|_application| JobList { jobs: Vec::new() })
        .command("SetPolicy", |payload| {
            let message = SetPolicyCommand::decode(payload).map_err(|error| error.to_string())?;
            Ok(RecorderCommand::SetPolicy(RecordingPolicy {
                record_mavlink_only_when_armed: message.policy.record_mavlink_only_when_armed,
                auto_start_recording: message.policy.auto_start_recording,
            }))
        })
        .command("StartRecording", |payload| {
            let message =
                StartRecordingCommand::decode(payload).map_err(|error| error.to_string())?;
            Ok(RecorderCommand::StartRecording {
                rotate_if_active: message.rotate_if_active,
            })
        })
        .command("StopRecording", |_payload| {
            Ok(RecorderCommand::StopRecording)
        })
        .command("RepairRecording", |payload| {
            let message =
                RepairRecordingCommand::decode(payload).map_err(|error| error.to_string())?;
            Ok(RecorderCommand::RepairRecording {
                path: message.path,
                now_unix_seconds: unix_time_now(),
            })
        })
        .command("CancelRepair", |payload| {
            let message =
                CancelRepairCommand::decode(payload).map_err(|error| error.to_string())?;
            Ok(RecorderCommand::CancelRepair { path: message.path })
        })
        .command("DeleteRecording", |payload| {
            let message =
                DeleteRecordingCommand::decode(payload).map_err(|error| error.to_string())?;
            Ok(RecorderCommand::DeleteRecording { path: message.path })
        })
        .command("SnapshotRecording", |payload| {
            let message =
                SnapshotRecordingCommand::decode(payload).map_err(|error| error.to_string())?;
            Ok(RecorderCommand::SnapshotRecording {
                path: message.path,
                now_unix_seconds: unix_time_now(),
            })
        })
        .command("Internal", decode_injected)
        .state(
            "recording",
            {
                let policy_watch_for_state = policy_watch_for_state.clone();
                move |application| {
                    let policy = TapPolicy::from_snapshot(&application.snapshot);
                    if let Err(error) = policy_watch_for_state.send(policy) {
                        error!(%error, "Tap policy watch channel closed");
                    }
                    recording_state_from_snapshot(&application.snapshot)
                }
            },
            |state| {
                let bytes = state.encode().map_err(|error| error.to_string())?;
                Ok((
                    Payload::from_bytes(Bytes::from(bytes)),
                    cdr_encoding(RecordingState::SCHEMA_NAME),
                ))
            },
        )
        .state(
            "library",
            |application| recording_library_from_snapshot(&application.snapshot),
            |state| {
                let bytes = state.encode().map_err(|error| error.to_string())?;
                Ok((
                    Payload::from_bytes(Bytes::from(bytes)),
                    cdr_encoding(RecordingLibrary::SCHEMA_NAME),
                ))
            },
        )
        .event(
            "operation",
            |event| matches!(event, RecorderEvent::RecordingOperation(_)),
            |event| {
                let RecorderEvent::RecordingOperation(operation) = event else {
                    return Err("event filter mismatch".into());
                };
                let message = recording_operation_from_event(operation.clone());
                let bytes = message.encode().map_err(|error| error.to_string())?;
                Ok((
                    Payload::from_bytes(Bytes::from(bytes)),
                    cdr_encoding(RecordingOperation::SCHEMA_NAME),
                ))
            },
        )
        .io_query("index", {
            let folder = library_io_context.folder();
            move |payload| {
                let folder = folder.clone();
                let bytes = payload.to_vec();
                async move { run_index_query(folder, &bytes).await }
            }
        })
        .io({
            let io_context = io_context.clone();
            move |application, request| {
                let io_context = io_context.clone();
                let active_session = application
                    .snapshot
                    .session
                    .as_ref()
                    .map(|session| session.file_name.clone());
                async move {
                    match request {
                        RecorderIo::OpenSession => {
                            let path = io_context.recorder_path.join(generate_filename());
                            let path_for_log = path.display().to_string();
                            let file_name = path
                                .file_name()
                                .map(|name| name.to_string_lossy().into_owned())
                                .unwrap_or_default();
                            let config = io_context.mcap_config;
                            info!(path = %path_for_log, "Opening recording session");
                            let session = tokio::task::spawn_blocking(move || {
                                McapSession::open(&path, config)
                            })
                            .await
                            .map_err(|error| {
                                error!(%error, "MCAP open task join failed");
                                RecorderCommand::IoFailed
                            })?
                            .map_err(|error| {
                                error!(%error, path = %path_for_log, "Failed to open MCAP session");
                                RecorderCommand::IoFailed
                            })?;
                            let writer = session.writer();
                            if let Err(error) = io_context.writer_watch.send(Some(writer)) {
                                error!(%error, "Recorder writer watch channel closed");
                            }
                            *io_context.session.lock().expect("session lock") = Some(session);
                            Ok(RecorderCommand::SessionOpened { file_name })
                        }
                        RecorderIo::FinishSession => {
                            let session = io_context.session.lock().expect("session lock").take();
                            if let Some(session) = session {
                                tokio::task::spawn_blocking(move || session.finish())
                                    .await
                                    .map_err(|error| {
                                        error!(%error, "MCAP finish task join failed");
                                        RecorderCommand::IoFailed
                                    })?
                                    .map_err(|error| {
                                        error!(%error, "Failed to finish MCAP session");
                                        RecorderCommand::IoFailed
                                    })?;
                            }
                            if let Err(error) = io_context.writer_watch.send(None) {
                                error!(%error, "Recorder writer watch channel closed");
                            }
                            Ok(RecorderCommand::SessionFinished)
                        }
                        RecorderIo::PublishMavlink(frame) => {
                            publish_mavlink(&io_context, frame).await.map_err(|error| {
                                error!(%error, "MAVLink publish failed");
                                RecorderCommand::IoFailed
                            })?;
                            Ok(RecorderCommand::Ack)
                        }
                        RecorderIo::MavlinkCommandAck {
                            camera,
                            command,
                            accepted,
                        } => {
                            let sequence = next_sequence(&io_context);
                            let frame = build_command_ack(
                                to_mavlink_system(camera),
                                sequence,
                                command,
                                accepted,
                            );
                            publish_mavlink(&io_context, frame).await.map_err(|error| {
                                error!(%error, "MAVLink command ack publish failed");
                                RecorderCommand::IoFailed
                            })?;
                            Ok(RecorderCommand::Ack)
                        }
                        RecorderIo::MavlinkCaptureStatus {
                            camera,
                            video_status,
                            recording_time_ms,
                        } => {
                            let sequence = next_sequence(&io_context);
                            let frame = build_camera_capture_status(
                                to_mavlink_system(camera),
                                sequence,
                                video_status,
                                recording_time_ms,
                            );
                            publish_mavlink(&io_context, frame).await.map_err(|error| {
                                error!(%error, "MAVLink capture status publish failed");
                                RecorderCommand::IoFailed
                            })?;
                            Ok(RecorderCommand::Ack)
                        }
                        RecorderIo::Library(request) => {
                            Ok(io_context.library.handle(request, active_session).await)
                        }
                    }
                }
            }
        });

    builder = builder.settings(
        None,
        |envelope| Ok(RecorderCommand::SetPolicy(settings_policy(&envelope)?)),
        |application| settings_from_snapshot(&application.snapshot),
    )?;

    let tap_session = fact_session.clone();
    let tap_schema = schema_path.clone();
    let tap_policy = policy_watch_receiver.clone();
    let mut tap_writer_watch = writer_watch_receiver.clone();

    tokio::spawn(async move {
        loop {
            if tap_writer_watch.changed().await.is_err() {
                error!("Recorder tap writer watch closed");
                break;
            }
            let writer = tap_writer_watch.borrow_and_update().clone();
            if let Some(writer) = writer {
                tap::run_data_plane(
                    tap_session.clone(),
                    tap_policy.clone(),
                    writer,
                    tap_schema.clone(),
                    fact_sender.clone(),
                )
                .await;
                error!("Recorder data-plane tap ended unexpectedly");
            }
        }
        error!("Recorder tap supervisor exited");
    });

    let discovery_session = fact_session.clone();
    tokio::spawn(async move {
        let mut discovery_cameras = BTreeSet::new();
        let mut discovery_sequence = 0u8;
        let discovery_source = default_discovery_source();
        let mut discovery_interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            tokio::select! {
                _ = discovery_interval.tick() => {
                    for camera in &discovery_cameras {
                        publish_discovery(
                            &discovery_session,
                            &discovery_source,
                            &mut discovery_sequence,
                            *camera,
                        ).await;
                    }
                }
                facts = fact_receiver.recv() => {
                    match facts {
                        Some(facts) => {
                            for fact in facts {
                                if let MavlinkFact::CameraHeartbeat(camera) = fact {
                                    discovery_cameras.insert(camera);
                                    publish_discovery(
                                        &discovery_session,
                                        &discovery_source,
                                        &mut discovery_sequence,
                                        camera,
                                    ).await;
                                } else if let Some(command) = fact_to_injected_command(fact) {
                                    inject_internal_command(&discovery_session, command).await;
                                }
                            }
                        }
                        None => {
                            error!("Recorder discovery fact channel closed");
                            break;
                        }
                    }
                }
            }
        }
        error!("Recorder discovery loop exited");
    });

    let library_progress_session = fact_session.clone();
    tokio::spawn(async move {
        while let Some(command) = library_progress_receiver.recv().await {
            inject_internal_command(&library_progress_session, command).await;
        }
    });

    if auto_start_recording {
        builder = builder.on_start(RecorderCommand::StartRecording {
            rotate_if_active: false,
        });
    }

    builder.run().await?;
    Ok(())
}

async fn publish_mavlink(io_context: &IoContext, frame: Vec<u8>) -> Result<(), String> {
    io_context
        .publish_session
        .publish(
            RAW_MAVLINK_IN_TOPIC,
            Payload::from_bytes(Bytes::from(frame)),
            "application/octet-stream;mavlink",
            None,
        )
        .await
        .map_err(|error| error.to_string())
}

fn next_sequence(io_context: &IoContext) -> u8 {
    let mut guard = io_context.mavlink_sequence.lock().expect("sequence");
    let value = *guard;
    *guard = guard.wrapping_add(1);
    value
}

async fn inject_internal_command(session: &Session, command: RecorderCommand) {
    let payload = match encode_injected(command) {
        Ok(payload) => payload,
        Err(_) => return,
    };
    if let Err(error) = session
        .query(
            &command_key(SERVICE_NAME, "Internal"),
            Payload::from_bytes(Bytes::from(payload)),
            "application/json",
            Duration::from_secs(2),
        )
        .await
    {
        error!(%error, "Failed to inject internal recorder command");
    }
}

async fn publish_discovery(
    session: &Session,
    source: &SystemAndComponent,
    sequence: &mut u8,
    camera: SystemAndComponent,
) {
    for frame in discovery_requests_for_camera(*source, sequence, camera) {
        if let Err(error) = session
            .publish(
                RAW_MAVLINK_IN_TOPIC,
                Payload::from_bytes(Bytes::from(frame)),
                "application/octet-stream;mavlink",
                None,
            )
            .await
        {
            error!(%error, "Failed to publish MAVLink discovery command");
        }
    }
}

fn to_mavlink_system(system: blueos_recorder_policy::SystemAndComponent) -> SystemAndComponent {
    SystemAndComponent {
        system_id: system.system_id,
        component_id: system.component_id,
    }
}

fn recording_state_from_snapshot(snapshot: &RecorderSnapshot) -> RecordingState {
    RecordingState {
        armed: snapshot.armed,
        session_active: snapshot.session_active,
        current_file: snapshot
            .session
            .as_ref()
            .map(|session| session.file_name.clone())
            .unwrap_or_default(),
        session_bytes_written: snapshot
            .session
            .as_ref()
            .map(|session| session.bytes_written)
            .unwrap_or(0),
        recording_video_topics: snapshot
            .video_streams
            .iter()
            .filter(|(_, stream)| stream.is_recording)
            .map(|(topic, _)| topic.clone())
            .collect(),
    }
}

fn settings_policy(envelope: &SettingsEnvelope) -> Result<RecordingPolicy, String> {
    let parsed: RecorderSettings =
        serde_json::from_str(&envelope.document_json).map_err(|error| error.to_string())?;
    Ok(RecordingPolicy {
        record_mavlink_only_when_armed: parsed.record_mavlink_only_when_armed,
        auto_start_recording: parsed.auto_start_recording,
    })
}

fn unix_time_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn time_from_unix_seconds(seconds: i64) -> Time {
    Time {
        sec: seconds as i32,
        nanosec: 0,
    }
}

fn recording_library_from_snapshot(snapshot: &RecorderSnapshot) -> RecordingLibrary {
    RecordingLibrary {
        files: snapshot
            .library
            .entries
            .iter()
            .map(|entry| RecordingFile {
                path: entry.path.clone(),
                name: entry.name.clone(),
                size_bytes: entry.size_bytes,
                created: time_from_unix_seconds(entry.created_unix_seconds),
                state: library_state_to_idl(entry.state),
                repair_bytes_processed: entry.repair_bytes_processed,
                repair_total_bytes: entry.repair_total_bytes,
                repair_bytes_per_second: entry.repair_bytes_per_second,
                repair_error: entry.repair_error.clone(),
            })
            .collect(),
    }
}

fn library_state_to_idl(state: LibraryRecordingState) -> u8 {
    match state {
        LibraryRecordingState::Recording => recording_file_constants::STATE_RECORDING,
        LibraryRecordingState::Ready => recording_file_constants::STATE_READY,
        LibraryRecordingState::NeedsRepair => recording_file_constants::STATE_NEEDS_REPAIR,
        LibraryRecordingState::Repairing => recording_file_constants::STATE_REPAIRING,
    }
}

fn recording_operation_from_event(
    operation: blueos_recorder_policy::RecordingOperationEvent,
) -> RecordingOperation {
    use blueos_recorder_policy::RecordingOperationKind;
    RecordingOperation {
        operation: match operation.operation {
            RecordingOperationKind::Repair => recording_operation_constants::OPERATION_REPAIR,
            RecordingOperationKind::Snapshot => recording_operation_constants::OPERATION_SNAPSHOT,
            RecordingOperationKind::Delete => recording_operation_constants::OPERATION_DELETE,
        },
        path: operation.path,
        output_path: operation.output_path,
        succeeded: operation.succeeded,
        cancelled: operation.cancelled,
        error: operation.error,
    }
}

fn settings_from_snapshot(snapshot: &RecorderSnapshot) -> RecorderSettings {
    RecorderSettings {
        VERSION: 1,
        record_mavlink_only_when_armed: snapshot.policy.record_mavlink_only_when_armed,
        auto_start_recording: snapshot.policy.auto_start_recording,
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use blueos_comms::ChannelBackend;
    use blueos_recorder_mcap::McapWriteConfig;
    use tokio::time;

    use super::*;

    #[tokio::test]
    async fn tap_records_when_policy_allows() {
        let (service_backend, client_backend) = ChannelBackend::pair();
        let service_session = Session::with_channel(service_backend);
        let client_session = Session::with_channel(client_backend);

        let directory = tempfile::tempdir().expect("tempdir");
        let (_policy_sender, policy_receiver) = watch::channel(TapPolicy {
            session_active: true,
            armed: true,
            record_mavlink_only_when_armed: true,
            recording_video_topics: BTreeSet::new(),
        });

        let path = directory.path().join("test.mcap");
        let mcap_session = McapSession::open(&path, McapWriteConfig::default()).expect("mcap");
        let writer = mcap_session.writer();
        let (fact_sender, _fact_receiver) = mpsc::channel(8);

        let tap_session = service_session.clone();
        let tap_task = tokio::spawn(async move {
            tap::run_data_plane(tap_session, policy_receiver, writer, None, fact_sender).await;
        });

        time::sleep(Duration::from_millis(50)).await;
        client_session
            .publish(
                "blueos/v1/example/state/ping",
                Payload::from_bytes(Bytes::from_static(b"hello")),
                "application/octet-stream",
                None,
            )
            .await
            .expect("publish");

        time::sleep(Duration::from_millis(150)).await;
        mcap_session.finish().expect("finish");
        tap_task.abort();
        assert!(path.metadata().expect("metadata").len() > 0);
    }
}
