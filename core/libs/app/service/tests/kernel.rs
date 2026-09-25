use std::num::NonZeroU32;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use blueos_api::{
    cdr_encoding, command_key, event_key, info_query_key, query_key, status_state_key,
};
use blueos_comms::{ChannelBackend, Payload, Session};
use blueos_cqrs::{App, Decision, Domain, Effect, TimerId};
use blueos_idl::Message;
use blueos_idl::msg::blueos_msgs::{
    CommandAck, JobList, RestartRequired, ServiceInfo, ServiceStatus, SettingsEnvelope,
    constants_service_status as service_status_constants,
};
use blueos_jobs::{JobId, Jobs};
use blueos_service::ServiceBuilder;
use blueos_service::ShutdownHandle;
use blueos_settings::{SettingsError, SettingsSchema};
use bytes::Bytes;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use tokio::time;

static KERNEL_TEST_SERVICE_ID: AtomicU64 = AtomicU64::new(0);

fn unique_service_name() -> String {
    format!(
        "kernel_test_{}",
        KERNEL_TEST_SERVICE_ID.fetch_add(1, Ordering::Relaxed)
    )
}

struct KernelTestDomain;

#[derive(Clone, Default, Serialize, Deserialize)]
#[allow(non_snake_case)]
struct KernelTestSettings {
    VERSION: u32,
    live_field: u32,
    restart_field: String,
}

impl SettingsSchema for KernelTestSettings {
    const VERSION: NonZeroU32 = NonZeroU32::new(1).unwrap();

    fn migrate(_data: &mut serde_json::Value) -> Result<(), SettingsError> {
        Ok(())
    }

    fn restart_required_fields() -> &'static [&'static str] {
        &["restart_field"]
    }
}

#[derive(Clone, Default)]
struct KernelTestSnapshot {
    counter: u32,
    settings: KernelTestSettings,
    status_detail: String,
}

#[derive(Clone, Debug, PartialEq)]
enum KernelTestCommand {
    Increment,
    DomainReject,
    SlowIo,
    ArmTimer,
    ArmThenCancel,
    CancelTimer,
    UpdateSettings(SettingsEnvelope),
    IoDone,
    ShutdownMarker,
}

#[derive(Clone, Debug, PartialEq)]
enum KernelTestEvent {
    RestartRequired(RestartRequired),
}

enum KernelTestQuery {
    Counter,
}

#[derive(Clone, PartialEq, Eq)]
struct KernelTestView {
    counter: u32,
}

#[derive(Clone)]
enum KernelTestIo {
    Sleep(Duration),
}

#[derive(Clone)]
enum KernelTestJobSpec {}

impl Domain for KernelTestDomain {
    type Command = KernelTestCommand;
    type Event = KernelTestEvent;
    type Query = KernelTestQuery;
    type View = KernelTestView;
    type Snapshot = KernelTestSnapshot;
    type IoRequest = KernelTestIo;
    type JobSpec = KernelTestJobSpec;

    fn handle_command(
        snapshot: &mut Self::Snapshot,
        _jobs: &mut Jobs<Self::JobSpec>,
        command: Self::Command,
    ) -> Decision<Self> {
        match command {
            KernelTestCommand::Increment => {
                snapshot.counter += 1;
                Decision::new()
            }
            KernelTestCommand::DomainReject => Decision::reject("not allowed in tests"),
            KernelTestCommand::SlowIo => Decision {
                events: Vec::new(),
                effects: vec![Effect::Io(KernelTestIo::Sleep(Duration::from_millis(50)))],
                rejection: None,
            },
            KernelTestCommand::ArmTimer => Decision {
                events: Vec::new(),
                effects: vec![Effect::Schedule {
                    after: Duration::from_millis(50),
                    timer: TimerId(1),
                    command: KernelTestCommand::Increment,
                }],
                rejection: None,
            },
            KernelTestCommand::ArmThenCancel => Decision {
                events: Vec::new(),
                effects: vec![
                    Effect::Schedule {
                        after: Duration::from_millis(50),
                        timer: TimerId(1),
                        command: KernelTestCommand::Increment,
                    },
                    Effect::CancelSchedule(TimerId(1)),
                ],
                rejection: None,
            },
            KernelTestCommand::CancelTimer => Decision {
                events: Vec::new(),
                effects: vec![Effect::CancelSchedule(TimerId(1))],
                rejection: None,
            },
            KernelTestCommand::UpdateSettings(envelope) => {
                let parsed: KernelTestSettings =
                    serde_json::from_str(&envelope.document_json).expect("settings json");
                let changes = blueos_settings::diff_top_level_settings(
                    KernelTestSettings::restart_required_fields(),
                    &serde_json::to_value(&snapshot.settings).unwrap(),
                    &serde_json::to_value(&parsed).unwrap(),
                );
                snapshot.settings = parsed;
                let mut events = Vec::new();
                if !changes.restart_required.is_empty() {
                    events.push(KernelTestEvent::RestartRequired(RestartRequired {
                        fields: changes.restart_required,
                    }));
                }
                Decision {
                    events,
                    effects: vec![Effect::Persist],
                    rejection: None,
                }
            }
            KernelTestCommand::IoDone => {
                snapshot.counter += 10;
                Decision::new()
            }
            KernelTestCommand::ShutdownMarker => {
                snapshot.status_detail = "shutdown".into();
                Decision::new()
            }
        }
    }

    fn handle_query(
        snapshot: &Self::Snapshot,
        _jobs: &Jobs<Self::JobSpec>,
        _query: Self::Query,
    ) -> Self::View {
        KernelTestView {
            counter: snapshot.counter,
        }
    }

    fn io_from_job(_job_id: JobId, _job_spec: &Self::JobSpec) -> Self::IoRequest {
        KernelTestIo::Sleep(Duration::from_millis(1))
    }
}

fn test_app() -> App<KernelTestDomain> {
    App::new(KernelTestSnapshot {
        settings: KernelTestSettings {
            VERSION: 1,
            live_field: 1,
            restart_field: "old".into(),
        },
        ..KernelTestSnapshot::default()
    })
}

async fn spawn_test_service(
    temp_dir: Option<PathBuf>,
) -> (
    String,
    Session,
    tokio::task::JoinHandle<Result<(), blueos_service::ServiceError>>,
    Option<ShutdownHandle>,
) {
    let service_name = unique_service_name();
    let (service_backend, client_backend) = ChannelBackend::pair();
    let service_session = Session::with_channel(service_backend);
    let client_session = Session::with_channel(client_backend);

    let mut builder = ServiceBuilder::<KernelTestDomain>::new(&service_name)
        .app(test_app())
        .service_info(ServiceInfo {
            name: service_name.clone(),
            version: "0.1.0".into(),
            build: String::new(),
            capabilities: Vec::new(),
        })
        .status(|application| ServiceStatus {
            status: service_status_constants::STATUS_READY,
            detail: application.snapshot.status_detail.clone(),
        })
        .jobs(|_application| JobList { jobs: Vec::new() })
        .command("Increment", |_| Ok(KernelTestCommand::Increment))
        .command("DomainReject", |_| Ok(KernelTestCommand::DomainReject))
        .command("SlowIo", |_| Ok(KernelTestCommand::SlowIo))
        .command("ArmThenCancel", |_| Ok(KernelTestCommand::ArmThenCancel))
        .command("ArmTimer", |_| Ok(KernelTestCommand::ArmTimer))
        .command("CancelTimer", |_| Ok(KernelTestCommand::CancelTimer))
        .query("Counter", |_, application| {
            let view = application.query(KernelTestQuery::Counter);
            let payload = Payload::from_bytes(Bytes::from(view.counter.to_string()));
            Ok((payload, "text/plain".into()))
        })
        .io_query("Echo", |payload| async move {
            Ok((payload, "text/plain".into()))
        })
        .io_query("SlowEcho", |payload| async move {
            time::sleep(Duration::from_millis(300)).await;
            Ok((payload, "text/plain".into()))
        })
        .event(
            "RestartRequired",
            |event| matches!(event, KernelTestEvent::RestartRequired(_)),
            |event| {
                let KernelTestEvent::RestartRequired(message) = event;
                let bytes = message.encode().map_err(|error| error.to_string())?;
                Ok((
                    Payload::from_bytes(Bytes::from(bytes)),
                    cdr_encoding(RestartRequired::SCHEMA_NAME),
                ))
            },
        )
        .io(|_application, request| async move {
            let KernelTestIo::Sleep(duration) = request;
            time::sleep(duration).await;
            Ok(KernelTestCommand::IoDone)
        });

    if let Some(folder) = temp_dir {
        builder = builder
            .settings(
                Some(folder),
                |envelope| Ok(KernelTestCommand::UpdateSettings(envelope)),
                |application| application.snapshot.settings.clone(),
            )
            .expect("settings");
    }

    builder = builder.on_shutdown(KernelTestCommand::ShutdownMarker);
    let shutdown = builder.shutdown_handle();

    let handle = tokio::spawn(async move { builder.run_with_session(service_session).await });

    time::sleep(Duration::from_millis(50)).await;

    for _ in 0..200 {
        if client_session
            .query(
                &info_query_key(&service_name),
                Payload::empty(),
                "",
                Duration::from_millis(100),
            )
            .await
            .is_ok()
        {
            break;
        }
        time::sleep(Duration::from_millis(20)).await;
    }
    if handle.is_finished() {
        panic!("service exited during startup: {:?}", handle.await);
    }

    (service_name, client_session, handle, Some(shutdown))
}

async fn query_text(session: &Session, key: &str) -> Vec<u8> {
    for _ in 0..50 {
        match session
            .query(key, Payload::empty(), "", Duration::from_millis(200))
            .await
        {
            Ok(reply) => return reply.payload.as_slice(),
            Err(blueos_comms::CommsError::NoReplier) => {
                time::sleep(Duration::from_millis(10)).await;
            }
            Err(error) => panic!("query: {error}"),
        }
    }
    panic!("query: no replier after retries");
}

async fn command_ack(
    session: &Session,
    service: &str,
    command: &str,
    payload: &[u8],
) -> CommandAck {
    for _ in 0..100 {
        match session
            .query(
                &command_key(service, command),
                Payload::from_bytes(Bytes::copy_from_slice(payload)),
                "",
                Duration::from_millis(300),
            )
            .await
        {
            Ok(reply) => {
                return CommandAck::decode(reply.payload.as_slice().as_slice()).expect("ack cdr");
            }
            Err(blueos_comms::CommsError::NoReplier) => {
                time::sleep(Duration::from_millis(10)).await;
            }
            Err(error) => panic!("command query: {error}"),
        }
    }
    panic!("command query: no replier after retries");
}

#[tokio::test]
async fn rejected_command_returns_reason_without_publishing_state() {
    let (service_name, client, service_handle, _) = spawn_test_service(None).await;
    command_ack(&client, &service_name, "Increment", &[]).await;

    let ack = command_ack(&client, &service_name, "DomainReject", &[]).await;
    assert!(!ack.accepted);
    assert_eq!(ack.reason, "not allowed in tests");
    assert_eq!(ack.job_id, 0);

    assert_eq!(
        query_text(&client, &query_key(&service_name, "Counter")).await,
        b"1"
    );

    service_handle.abort();
}

#[tokio::test]
async fn io_query_round_trip_outside_inbox() {
    let (service_name, client, service_handle, _) = spawn_test_service(None).await;
    let payload = Payload::from_bytes(Bytes::from("hello"));
    let reply = client
        .query(
            &query_key(&service_name, "Echo"),
            payload,
            "",
            Duration::from_secs(1),
        )
        .await
        .expect("io_query");
    assert_eq!(reply.payload.as_slice(), b"hello");

    service_handle.abort();
}

#[tokio::test]
async fn slow_io_query_does_not_block_command_ack() {
    let (service_name, client, service_handle, _) = spawn_test_service(None).await;

    let slow = tokio::spawn({
        let client = client.clone();
        let service_name = service_name.clone();
        async move {
            client
                .query(
                    &query_key(&service_name, "SlowEcho"),
                    Payload::from_bytes(Bytes::from("slow")),
                    "",
                    Duration::from_secs(2),
                )
                .await
                .expect("slow io_query");
        }
    });
    time::sleep(Duration::from_millis(10)).await;
    let fast_ack = command_ack(&client, &service_name, "Increment", &[]).await;
    assert!(fast_ack.accepted);

    let _ = slow.await;
    service_handle.abort();
}

#[tokio::test]
async fn command_returns_ack_and_publishes_status_state() {
    let (service_name, client, service_handle, _) = spawn_test_service(None).await;
    let ack = command_ack(&client, &service_name, "Increment", &[]).await;
    assert!(ack.accepted);

    let reply = client
        .query(
            &status_state_key(&service_name),
            Payload::empty(),
            "",
            Duration::from_secs(1),
        )
        .await
        .expect("state query");
    let status = ServiceStatus::decode(reply.payload.as_slice().as_slice()).expect("status");
    assert_eq!(status.status, service_status_constants::STATUS_READY);

    service_handle.abort();
}

#[tokio::test]
async fn query_answered_from_inbox_snapshot() {
    let (service_name, client, service_handle, _) = spawn_test_service(None).await;
    command_ack(&client, &service_name, "Increment", &[]).await;
    command_ack(&client, &service_name, "Increment", &[]).await;

    let payload = query_text(&client, &query_key(&service_name, "Counter")).await;
    assert_eq!(payload, b"2");

    service_handle.abort();
}

#[tokio::test]
async fn io_effect_round_trip() {
    let (service_name, client, service_handle, _) = spawn_test_service(None).await;
    command_ack(&client, &service_name, "SlowIo", &[]).await;
    time::sleep(Duration::from_millis(100)).await;

    assert_eq!(
        query_text(&client, &query_key(&service_name, "Counter")).await,
        b"10"
    );

    service_handle.abort();
}

#[tokio::test]
async fn schedule_fires_when_not_cancelled() {
    let (service_name, client, service_handle, _) = spawn_test_service(None).await;

    command_ack(&client, &service_name, "ArmTimer", &[]).await;
    time::sleep(Duration::from_millis(200)).await;

    assert_eq!(
        query_text(&client, &query_key(&service_name, "Counter")).await,
        b"1"
    );

    service_handle.abort();
}

#[tokio::test]
async fn schedule_fires_and_cancel_prevents() {
    let (service_name, client, service_handle, _) = spawn_test_service(None).await;

    command_ack(&client, &service_name, "ArmThenCancel", &[]).await;
    time::sleep(Duration::from_millis(100)).await;

    assert_eq!(
        query_text(&client, &query_key(&service_name, "Counter")).await,
        b"0"
    );

    service_handle.abort();
}

#[tokio::test]
async fn schedule_cancel_via_separate_command() {
    let (service_name, client, service_handle, _) = spawn_test_service(None).await;

    command_ack(&client, &service_name, "ArmTimer", &[]).await;
    command_ack(&client, &service_name, "CancelTimer", &[]).await;
    time::sleep(Duration::from_millis(100)).await;

    assert_eq!(
        query_text(&client, &query_key(&service_name, "Counter")).await,
        b"0"
    );

    service_handle.abort();
}

#[tokio::test]
async fn persist_writes_settings_file() {
    let temp_dir = TempDir::new().expect("tempdir");
    let (service_name, client, service_handle, _) =
        spawn_test_service(Some(temp_dir.path().to_path_buf())).await;

    let new_settings = KernelTestSettings {
        VERSION: 1,
        live_field: 9,
        restart_field: "new".into(),
    };
    let envelope = SettingsEnvelope {
        document_json: serde_json::to_string(&new_settings).unwrap(),
        fields: Vec::new(),
    };
    let payload = envelope.encode().unwrap();
    let ack = command_ack(&client, &service_name, "UpdateSettings", &payload).await;
    assert!(ack.accepted);
    time::sleep(Duration::from_millis(50)).await;

    let settings_path = temp_dir
        .path()
        .join(format!("{service_name}/settings-1.json"));
    assert!(settings_path.is_file());
    let on_disk: KernelTestSettings =
        serde_json::from_str(&std::fs::read_to_string(&settings_path).unwrap()).unwrap();
    assert_eq!(on_disk.live_field, 9);

    service_handle.abort();
}

#[tokio::test]
async fn update_settings_emits_restart_required_event() {
    let temp_dir = TempDir::new().expect("tempdir");
    let (service_name, client, service_handle, _) =
        spawn_test_service(Some(temp_dir.path().to_path_buf())).await;

    let mut stream = client
        .subscribe(&event_key(&service_name, "RestartRequired"))
        .await
        .expect("subscribe");

    let new_settings = KernelTestSettings {
        VERSION: 1,
        live_field: 1,
        restart_field: "changed".into(),
    };
    let envelope = SettingsEnvelope {
        document_json: serde_json::to_string(&new_settings).unwrap(),
        fields: Vec::new(),
    };
    command_ack(
        &client,
        &service_name,
        "UpdateSettings",
        &envelope.encode().unwrap(),
    )
    .await;

    let sample = time::timeout(Duration::from_secs(1), stream.next())
        .await
        .expect("timeout")
        .expect("event");
    let decoded =
        RestartRequired::decode(sample.payload.as_slice().as_slice()).expect("restart event");
    assert_eq!(decoded.fields, vec!["restart_field".to_string()]);

    service_handle.abort();
}

#[tokio::test]
async fn late_joiner_reads_state_via_query() {
    let (service_name, client, service_handle, _) = spawn_test_service(None).await;
    command_ack(&client, &service_name, "Increment", &[]).await;

    let reply = client
        .query(
            &status_state_key(&service_name),
            Payload::empty(),
            "",
            Duration::from_secs(1),
        )
        .await
        .expect("late joiner");
    let status = ServiceStatus::decode(reply.payload.as_slice().as_slice()).expect("status");
    assert_eq!(status.status, service_status_constants::STATUS_READY);

    service_handle.abort();
}

#[tokio::test]
async fn graceful_shutdown_via_handle() {
    let (service_name, client, service_handle, shutdown) = spawn_test_service(None).await;
    let shutdown = shutdown.expect("shutdown handle");
    shutdown.trigger();

    let mut saw_shutdown_detail = false;
    for _ in 0..50 {
        if let Ok(reply) = client
            .query(
                &status_state_key(&service_name),
                Payload::empty(),
                "",
                Duration::from_millis(200),
            )
            .await
        {
            let status =
                ServiceStatus::decode(reply.payload.as_slice().as_slice()).expect("status");
            if status.detail == "shutdown" {
                saw_shutdown_detail = true;
                break;
            }
        }
        time::sleep(Duration::from_millis(10)).await;
    }
    assert!(saw_shutdown_detail);

    let join_result = time::timeout(Duration::from_secs(2), service_handle)
        .await
        .expect("shutdown should finish")
        .expect("join");
    assert!(join_result.is_ok());
}

#[tokio::test]
async fn slow_io_does_not_block_other_commands() {
    let (service_name, client, service_handle, _) = spawn_test_service(None).await;

    let slow = tokio::spawn({
        let client = client.clone();
        let service_name = service_name.clone();
        async move { command_ack(&client, &service_name, "SlowIo", &[]).await }
    });
    time::sleep(Duration::from_millis(10)).await;
    let fast_ack = command_ack(&client, &service_name, "Increment", &[]).await;
    assert!(fast_ack.accepted);

    let _ = slow.await;
    service_handle.abort();
}
