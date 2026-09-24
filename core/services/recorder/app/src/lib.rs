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
mod inject;
mod schema;
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
use blueos_idl::msg::blueos_msgs::{JobList, ServiceInfo, ServiceStatus, SettingsEnvelope};
use blueos_idl::msg::blueos_recorder_msgs::{
    RecordingState, SetPolicyCommand, StartRecordingCommand,
};
use blueos_recorder_mavlink::{
    MavlinkFact, RAW_MAVLINK_IN_TOPIC, SystemAndComponent, build_camera_capture_status,
    build_command_ack, default_discovery_source, discovery_requests_for_camera,
};
use blueos_recorder_mcap::{McapSession, McapWriteConfig, McapWriterHandle};
use blueos_recorder_policy::{
    RecorderCommand, RecorderDomain, RecorderIo, RecorderSnapshot, RecordingPolicy, TapPolicy,
};
use blueos_service::ServiceBuilder;
use blueos_settings::{SettingsError, SettingsSchema};
use bytes::Bytes;
use tokio::sync::{mpsc, watch};
use tracing::{error, info};

use crate::cli::{mcap_write_config, parse_cli, recorder_directory, schema_directory};
use crate::inject::{decode_injected, encode_injected, fact_to_injected_command};

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

pub fn run(arguments: impl IntoIterator<Item = OsString>) {
    let arguments: Vec<String> = arguments
        .into_iter()
        .map(|argument| argument.to_string_lossy().into_owned())
        .collect();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    if let Err(error) = runtime.block_on(run_async(arguments)) {
        error!("recorder: {error}");
    }
}

async fn run_async(arguments: Vec<String>) -> Result<(), anyhow::Error> {
    let cli = parse_cli(&arguments);
    let verbosity = if cli.verbose { 1 } else { 0 };
    let recorder_path = recorder_directory(&cli);
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

    let io_context = Arc::new(IoContext {
        recorder_path,
        mcap_config,
        session: Mutex::new(None),
        writer_watch: writer_watch_sender,
        mavlink_sequence: Mutex::new(0),
        publish_session,
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
        .status(|application| ServiceStatus {
            status: 2,
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
        .command("Internal", decode_injected)
        .state(
            "recording",
            {
                let policy_watch_for_state = policy_watch_for_state.clone();
                move |application| {
                    let policy = TapPolicy::from_snapshot(&application.snapshot);
                    let _ = policy_watch_for_state.send(policy);
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
        .io({
            let io_context = io_context.clone();
            move |_application, request| {
                let io_context = io_context.clone();
                async move {
                    match request {
                        RecorderIo::OpenSession => {
                            let path = io_context.recorder_path.join(generate_filename());
                            info!(path = %path.display(), "Opening recording session");
                            let session = McapSession::open(&path, io_context.mcap_config)
                                .map_err(|_error| RecorderCommand::Ack)?;
                            let writer = session.writer();
                            let _ = io_context.writer_watch.send(Some(writer));
                            *io_context.session.lock().expect("session lock") = Some(session);
                            Ok(RecorderCommand::SessionOpened {
                                file_name: path
                                    .file_name()
                                    .map(|name| name.to_string_lossy().into_owned())
                                    .unwrap_or_default(),
                            })
                        }
                        RecorderIo::FinishSession => {
                            let mut guard = io_context.session.lock().expect("session lock");
                            if let Some(session) = guard.take() {
                                session.finish().map_err(|_error| RecorderCommand::Ack)?;
                            }
                            let _ = io_context.writer_watch.send(None);
                            Ok(RecorderCommand::SessionFinished)
                        }
                        RecorderIo::PublishMavlink(frame) => {
                            publish_mavlink(&io_context, frame)
                                .await
                                .map_err(|_error| RecorderCommand::Ack)?;
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
                            publish_mavlink(&io_context, frame)
                                .await
                                .map_err(|_error| RecorderCommand::Ack)?;
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
                            publish_mavlink(&io_context, frame)
                                .await
                                .map_err(|_error| RecorderCommand::Ack)?;
                            Ok(RecorderCommand::Ack)
                        }
                    }
                }
            }
        });

    builder = builder
        .settings(
            None,
            |envelope| Ok(RecorderCommand::SetPolicy(settings_policy(&envelope)?)),
            |application| settings_from_snapshot(&application.snapshot),
        )
        .map_err(anyhow::Error::from)?;

    let tap_session = fact_session.clone();
    let tap_schema = schema_path.clone();
    let tap_policy = policy_watch_receiver.clone();
    let mut tap_writer_watch = writer_watch_receiver.clone();

    tokio::spawn(async move {
        loop {
            tap_writer_watch.changed().await.ok();
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
            }
        }
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
                        None => break,
                    }
                }
            }
        }
    });

    if auto_start_recording {
        let start_session = fact_session.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            inject_start_recording(&start_session).await;
        });
    }

    builder.run().await.map_err(anyhow::Error::from)
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

async fn inject_start_recording(session: &Session) {
    let message = StartRecordingCommand {
        rotate_if_active: false,
    };
    let bytes = message.encode().expect("cdr");
    if let Err(error) = session
        .query(
            &command_key(SERVICE_NAME, "StartRecording"),
            Payload::from_bytes(Bytes::from(bytes)),
            &cdr_encoding(StartRecordingCommand::SCHEMA_NAME),
            Duration::from_secs(2),
        )
        .await
    {
        error!(%error, "Failed to auto-start recording");
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
