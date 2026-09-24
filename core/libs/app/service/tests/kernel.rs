use std::num::NonZeroU32;
use std::path::PathBuf;
use std::time::Duration;

use blueos_api::{cdr_encoding, command_key, info_query_key, query_key, status_state_key};
use blueos_comms::{ChannelBackend, Payload, Session};
use blueos_cqrs::{App, Decision, Domain, Effect, TimerId};
use blueos_idl::Message;
use blueos_idl::msg::blueos_msgs::{
    CommandAck, JobList, RestartRequired, ServiceInfo, ServiceStatus, SettingsEnvelope,
};
use blueos_jobs::{JobId, Jobs};
use blueos_service::ServiceBuilder;
use blueos_settings::{SettingsError, SettingsSchema};
use bytes::Bytes;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tempfile::TempDir;
use tokio::sync::Mutex;
use tokio::time;

static KERNEL_TEST_LOCK: Mutex<()> = Mutex::const_new(());

async fn kernel_test_guard() -> tokio::sync::MutexGuard<'static, ()> {
    KERNEL_TEST_LOCK.lock().await
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
    SlowIo,
    ArmTimer,
    ArmThenCancel,
    CancelTimer,
    UpdateSettings(SettingsEnvelope),
    IoDone,
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
            KernelTestCommand::SlowIo => Decision {
                events: Vec::new(),
                effects: vec![Effect::Io(KernelTestIo::Sleep(Duration::from_millis(50)))],
            },
            KernelTestCommand::ArmTimer => Decision {
                events: Vec::new(),
                effects: vec![Effect::Schedule {
                    after: Duration::from_millis(50),
                    timer: TimerId(1),
                    command: KernelTestCommand::Increment,
                }],
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
            },
            KernelTestCommand::CancelTimer => Decision {
                events: Vec::new(),
                effects: vec![Effect::CancelSchedule(TimerId(1))],
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
                }
            }
            KernelTestCommand::IoDone => {
                snapshot.counter += 10;
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

fn test_service_info() -> ServiceInfo {
    ServiceInfo {
        name: "kernel_test".into(),
        version: "0.1.0".into(),
        build: String::new(),
        capabilities: Vec::new(),
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
    Session,
    tokio::task::JoinHandle<Result<(), blueos_service::ServiceError>>,
) {
    let (service_backend, client_backend) = ChannelBackend::pair();
    let service_session = Session::with_channel(service_backend);
    let client_session = Session::with_channel(client_backend);

    let mut builder = ServiceBuilder::<KernelTestDomain>::new("kernel_test")
        .app(test_app())
        .service_info(test_service_info())
        .status(|application| ServiceStatus {
            status: 2,
            detail: application.snapshot.status_detail.clone(),
        })
        .jobs(|_application| JobList { jobs: Vec::new() })
        .command("Increment", |_| Ok(KernelTestCommand::Increment))
        .command("SlowIo", |_| Ok(KernelTestCommand::SlowIo))
        .command("ArmThenCancel", |_| Ok(KernelTestCommand::ArmThenCancel))
        .command("ArmTimer", |_| Ok(KernelTestCommand::ArmTimer))
        .command("CancelTimer", |_| Ok(KernelTestCommand::CancelTimer))
        .query("Counter", |_, application| {
            let view = application.query(KernelTestQuery::Counter);
            let payload = Payload::from_bytes(Bytes::from(view.counter.to_string()));
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

    let handle = tokio::spawn(async move { builder.run_with_session(service_session).await });

    time::sleep(Duration::from_millis(20)).await;

    for _ in 0..100 {
        if client_session
            .query(
                &info_query_key("kernel_test"),
                Payload::empty(),
                "",
                Duration::from_millis(50),
            )
            .await
            .is_ok()
        {
            break;
        }
        time::sleep(Duration::from_millis(10)).await;
    }

    (client_session, handle)
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
    for _ in 0..50 {
        match session
            .query(
                &command_key(service, command),
                Payload::from_bytes(Bytes::copy_from_slice(payload)),
                "",
                Duration::from_millis(200),
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
async fn command_returns_ack_and_publishes_status_state() {
    let _guard = kernel_test_guard().await;
    let (client, service_handle) = spawn_test_service(None).await;
    let ack = command_ack(&client, "kernel_test", "Increment", &[]).await;
    assert!(ack.accepted);

    let reply = client
        .query(
            &status_state_key("kernel_test"),
            Payload::empty(),
            "",
            Duration::from_secs(1),
        )
        .await
        .expect("state query");
    let status = ServiceStatus::decode(reply.payload.as_slice().as_slice()).expect("status");
    assert_eq!(status.status, 2);

    service_handle.abort();
}

#[tokio::test]
async fn query_answered_from_inbox_snapshot() {
    let _guard = kernel_test_guard().await;
    let (client, service_handle) = spawn_test_service(None).await;
    command_ack(&client, "kernel_test", "Increment", &[]).await;
    command_ack(&client, "kernel_test", "Increment", &[]).await;

    let payload = query_text(&client, &query_key("kernel_test", "Counter")).await;
    assert_eq!(payload, b"2");

    service_handle.abort();
}

#[tokio::test]
async fn io_effect_round_trip() {
    let _guard = kernel_test_guard().await;
    let (client, service_handle) = spawn_test_service(None).await;
    command_ack(&client, "kernel_test", "SlowIo", &[]).await;
    time::sleep(Duration::from_millis(100)).await;

    assert_eq!(
        query_text(&client, &query_key("kernel_test", "Counter")).await,
        b"10"
    );

    service_handle.abort();
}

#[tokio::test]
async fn schedule_fires_when_not_cancelled() {
    let _guard = kernel_test_guard().await;
    let (client, service_handle) = spawn_test_service(None).await;

    command_ack(&client, "kernel_test", "ArmTimer", &[]).await;
    time::sleep(Duration::from_millis(200)).await;

    assert_eq!(
        query_text(&client, &query_key("kernel_test", "Counter")).await,
        b"1"
    );

    service_handle.abort();
}

#[tokio::test]
async fn schedule_fires_and_cancel_prevents() {
    let _guard = kernel_test_guard().await;
    let (client, service_handle) = spawn_test_service(None).await;

    command_ack(&client, "kernel_test", "ArmThenCancel", &[]).await;
    time::sleep(Duration::from_millis(100)).await;

    assert_eq!(
        query_text(&client, &query_key("kernel_test", "Counter")).await,
        b"0"
    );

    service_handle.abort();
}

#[tokio::test]
async fn schedule_cancel_via_separate_command() {
    let _guard = kernel_test_guard().await;
    let (client, service_handle) = spawn_test_service(None).await;

    command_ack(&client, "kernel_test", "ArmTimer", &[]).await;
    command_ack(&client, "kernel_test", "CancelTimer", &[]).await;
    time::sleep(Duration::from_millis(100)).await;

    assert_eq!(
        query_text(&client, &query_key("kernel_test", "Counter")).await,
        b"0"
    );

    service_handle.abort();
}

#[tokio::test]
async fn persist_writes_settings_file() {
    let _guard = kernel_test_guard().await;
    let temp_dir = TempDir::new().expect("tempdir");
    let (client, service_handle) = spawn_test_service(Some(temp_dir.path().to_path_buf())).await;

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
    let ack = command_ack(&client, "kernel_test", "UpdateSettings", &payload).await;
    assert!(ack.accepted);
    time::sleep(Duration::from_millis(50)).await;

    let settings_path = temp_dir.path().join("kernel_test/settings-1.json");
    assert!(settings_path.is_file());
    let on_disk: KernelTestSettings =
        serde_json::from_str(&std::fs::read_to_string(&settings_path).unwrap()).unwrap();
    assert_eq!(on_disk.live_field, 9);

    service_handle.abort();
}

#[tokio::test]
async fn update_settings_emits_restart_required_event() {
    let _guard = kernel_test_guard().await;
    let temp_dir = TempDir::new().expect("tempdir");
    let (client, service_handle) = spawn_test_service(Some(temp_dir.path().to_path_buf())).await;

    let mut stream = client
        .subscribe("blueos/v1/kernel_test/event/RestartRequired")
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
        "kernel_test",
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
    let _guard = kernel_test_guard().await;
    let (client, service_handle) = spawn_test_service(None).await;
    command_ack(&client, "kernel_test", "Increment", &[]).await;

    let reply = client
        .query(
            &status_state_key("kernel_test"),
            Payload::empty(),
            "",
            Duration::from_secs(1),
        )
        .await
        .expect("late joiner");
    let status = ServiceStatus::decode(reply.payload.as_slice().as_slice()).expect("status");
    assert_eq!(status.status, 2);

    service_handle.abort();
}

#[tokio::test]
async fn slow_io_does_not_block_other_commands() {
    let _guard = kernel_test_guard().await;
    let (client, service_handle) = spawn_test_service(None).await;

    let slow = tokio::spawn({
        let client = client.clone();
        async move { command_ack(&client, "kernel_test", "SlowIo", &[]).await }
    });
    time::sleep(Duration::from_millis(10)).await;
    let fast_ack = command_ack(&client, "kernel_test", "Increment", &[]).await;
    assert!(fast_ack.accepted);

    let _ = slow.await;
    service_handle.abort();
}
