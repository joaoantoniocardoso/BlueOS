//! Kernel settings: load, `UpdateSettings`, persist rollback, and pending restart fields.

mod common;

use core::{convert::Infallible, num::NonZeroU32, time::Duration};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tokio::time;

use blueos_api::{CommandAck, Message, job_result_key};
use blueos_comms::channel::ChannelBackend;
use blueos_domain::{Command, Decision, Domain, DomainDurable, IoError, Now, Outcome};
use blueos_idl::{
    msg::blueos_example_msgs::SetLevelGoal,
    msg::blueos_msgs::{
        CommandAckStatus, JobResult, JobStatus, JobStatusStatus, SettingsEnvelope,
        UpdateSettingsGoal, UpdateSettingsResult,
    },
};
use blueos_jobs::JobId;
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};
use blueos_settings::{SettingsError, SettingsSchema, settings_file_name, state_file_name};

const DURABLE_STATE_VERSION: NonZeroU32 = NonZeroU32::MIN;

#[derive(clap::Args, Default)]
struct SettingsTankArguments;

struct SettingsTankService;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct SettingsTankDocument {
    #[serde(rename = "VERSION")]
    version: NonZeroU32,
    live_field: u32,
    restart_field: String,
}

impl Default for SettingsTankDocument {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            live_field: 1,
            restart_field: "initial".into(),
        }
    }
}

impl SettingsSchema for SettingsTankDocument {
    const VERSION: NonZeroU32 = NonZeroU32::new(1).unwrap();

    fn migrate(_data: &mut serde_json::Value) -> Result<(), SettingsError> {
        Ok(())
    }

    fn restart_required_fields() -> &'static [&'static str] {
        &["restart_field"]
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct SettingsTankDurable {
    live_field: u32,
}

#[derive(Clone)]
struct SettingsTankSnapshot {
    level: u8,
    settings: SettingsTankDocument,
    durable: SettingsTankDurable,
}

enum SettingsTankRequest {
    SetLevel(u8),
    UpdateSettings(SettingsTankDocument),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SettingsTankTick {
    Restored,
}

struct SettingsTank;

impl Service for SettingsTankService {
    type Domain = SettingsTank;
    type Context = ();
    type Arguments = SettingsTankArguments;

    const NAME: &'static str = "settings_tank";
    const VERSION: &'static str = "1.0.0";

    fn context(_service: &ServiceContext<SettingsTankArguments>) -> Result<(), ServiceError> {
        Ok(())
    }

    fn build(
        service: &ServiceContext<SettingsTankArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<SettingsTank>, ServiceError> {
        let config_parent = service.settings_path().map(PathBuf::from);
        Ok(ServiceBuilder::new(SettingsTankSnapshot {
            level: 0,
            settings: SettingsTankDocument::default(),
            durable: SettingsTankDurable { live_field: 1 },
        })
        .command("SetLevel", |request: SetLevelGoal| {
            Ok(SettingsTankRequest::SetLevel(request.level))
        })
        .durable_state(Self::NAME, config_parent.clone(), DURABLE_STATE_VERSION)
        .settings(
            Self::NAME,
            config_parent,
            |snapshot: &mut SettingsTankSnapshot, settings| snapshot.settings = settings,
            |snapshot: &SettingsTankSnapshot| snapshot.settings.clone(),
            |envelope| {
                let document: SettingsTankDocument = serde_json::from_str(&envelope.document_json)?;
                Ok(SettingsTankRequest::UpdateSettings(document))
            },
        ))
    }
}

impl Domain for SettingsTank {
    type Snapshot = SettingsTankSnapshot;
    type Request = SettingsTankRequest;
    type IoResult = Infallible;
    type Tick = SettingsTankTick;
    type ObservedFact = Infallible;
    type Event = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut SettingsTankSnapshot,
        command: Command<SettingsTankRequest, Infallible, SettingsTankTick, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        match command {
            Command::Request(SettingsTankRequest::SetLevel(level)) => {
                snapshot.level = level;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::Request(SettingsTankRequest::UpdateSettings(settings)) => {
                snapshot.durable.live_field = settings.live_field;
                snapshot.settings = settings;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
            Command::IoResult(never) => match never {},
            Command::Tick(SettingsTankTick::Restored) => Outcome::Applied {
                events: Vec::new(),
                effects: Vec::new(),
            },
            Command::ObservedFact(never) => match never {},
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
    }
}

impl DomainDurable for SettingsTank {
    type DurableState = SettingsTankDurable;

    fn durable_state(snapshot: &Self::Snapshot) -> &Self::DurableState {
        &snapshot.durable
    }

    fn set_durable_state(snapshot: &mut Self::Snapshot, state: Self::DurableState) {
        snapshot.settings.live_field = state.live_field;
        snapshot.durable = state;
    }

    fn restored_tick() -> Self::Tick {
        SettingsTankTick::Restored
    }
}

fn temp_settings_parent(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "kernel-settings-{}-{}-{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

async fn start_with_settings_folder(parent: PathBuf) -> Harness<SettingsTankService> {
    let _ = std::fs::remove_dir_all(&parent);
    std::fs::create_dir_all(&parent).unwrap();
    let backend: Arc<dyn blueos_comms::CommsBackend> = Arc::new(ChannelBackend::default());
    Harness::start_on_with_context(
        Arc::clone(&backend),
        ServiceContext::with_settings_path(
            SettingsTankArguments,
            Some(parent),
            Arc::clone(&backend),
        ),
    )
    .await
    .expect("harness starts")
}

fn envelope_for(document: &SettingsTankDocument) -> SettingsEnvelope {
    SettingsEnvelope {
        document_json: serde_json::to_string(document).unwrap(),
        fields: Vec::new(),
    }
}

fn durable_state_path(parent: &Path) -> PathBuf {
    parent.join(format!(
        "{}/{}",
        SettingsTankService::NAME,
        state_file_name(DURABLE_STATE_VERSION)
    ))
}

fn durable_live_field_on_disk(parent: &Path) -> Option<u32> {
    common::durable_u32_from_json(&durable_state_path(parent), "/domain/live_field")
}

#[tokio::test(start_paused = true)]
async fn update_settings_persists_and_publishes_pending_restart() {
    let parent = temp_settings_parent("persist");
    let harness = start_with_settings_folder(parent.clone()).await;

    let updated = SettingsTankDocument {
        version: SettingsTankDocument::VERSION,
        live_field: 2,
        restart_field: "changed".into(),
    };
    let ack = harness
        .send("UpdateSettings", &envelope_for(&updated))
        .await;
    assert!(ack.accepted);

    let envelope = harness.settings::<SettingsEnvelope>().await;
    assert_eq!(envelope.fields.len(), 1);
    assert_eq!(envelope.fields[0].path, "restart_field");
    assert!(envelope.fields[0].restart_required);

    let on_disk_path = parent.join(format!(
        "{}/{}",
        SettingsTankService::NAME,
        settings_file_name(SettingsTankDocument::VERSION)
    ));
    let on_disk: SettingsTankDocument =
        serde_json::from_str(&std::fs::read_to_string(on_disk_path).unwrap()).unwrap();
    assert_eq!(on_disk.restart_field, "changed");

    let restored = SettingsTankDocument {
        version: SettingsTankDocument::VERSION,
        live_field: 2,
        restart_field: "initial".into(),
    };
    let restore_ack = harness
        .send("UpdateSettings", &envelope_for(&restored))
        .await;
    assert!(restore_ack.accepted);
    let cleared_state = harness.settings::<SettingsEnvelope>().await;
    assert!(cleared_state.fields.is_empty());

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn update_settings_is_an_instant_job_that_acks_its_final_status() {
    let parent = temp_settings_parent("instant-job");
    let harness = start_with_settings_folder(parent.clone()).await;
    let job_id = JobId::from_u128(7);
    let updated = SettingsTankDocument {
        version: SettingsTankDocument::VERSION,
        live_field: 2,
        restart_field: "changed".into(),
    };

    let ack = harness
        .submit("UpdateSettings", job_id, &envelope_for(&updated))
        .await;

    assert_eq!(
        ack,
        CommandAck {
            accepted: true,
            job_id: job_id.to_string(),
            status: CommandAckStatus::Succeeded,
            reason: String::new(),
        }
    );
    let running: SettingsTankDocument =
        serde_json::from_str(&harness.settings::<SettingsEnvelope>().await.document_json).unwrap();
    assert_eq!(running, updated);
    assert_eq!(
        harness.jobs().await.jobs,
        [JobStatus {
            job_id: job_id.to_string(),
            job_type: "UpdateSettings".to_owned(),
            status: JobStatusStatus::Succeeded,
            reason: String::new(),
        }]
    );

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn update_settings_takes_its_action_goal_and_publishes_its_action_result() {
    let parent = temp_settings_parent("action");
    let harness = start_with_settings_folder(parent.clone()).await;
    let mut results = harness
        .backend()
        .subscribe(&job_result_key(SettingsTankService::NAME, "UpdateSettings"))
        .await
        .unwrap();
    let job_id = JobId::from_u128(7);
    let goal = UpdateSettingsGoal {
        envelope: envelope_for(&SettingsTankDocument {
            live_field: 2,
            ..SettingsTankDocument::default()
        }),
    };

    let ack = harness.submit("UpdateSettings", job_id, &goal).await;
    let sample = time::timeout(Duration::from_secs(10), results.recv())
        .await
        .unwrap()
        .unwrap();
    let ended = JobResult::decode(&sample.payload().to_bytes()).unwrap();

    assert_eq!(ack.status, CommandAckStatus::Succeeded);
    assert_eq!(ended.job.status, JobStatusStatus::Succeeded);
    assert_eq!(
        UpdateSettingsResult::decode(&ended.result),
        Ok(UpdateSettingsResult::default())
    );

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn update_settings_appears_in_the_history_query_like_any_other_job_type() {
    let parent = temp_settings_parent("history");
    let harness = start_with_settings_folder(parent.clone()).await;
    let [first, second] = [7, 8].map(JobId::from_u128);
    for (job_id, live_field) in [(first, 2), (second, 3)] {
        let updated = SettingsTankDocument {
            version: SettingsTankDocument::VERSION,
            live_field,
            restart_field: "changed".into(),
        };
        harness
            .submit("UpdateSettings", job_id, &envelope_for(&updated))
            .await;
    }

    let history = harness.job_history("UpdateSettings").await;

    assert_eq!(
        history.jobs,
        [first, second].map(|job_id| JobStatus {
            job_id: job_id.to_string(),
            job_type: "UpdateSettings".to_owned(),
            status: JobStatusStatus::Succeeded,
            reason: String::new(),
        })
    );

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn a_retry_of_update_settings_does_not_write_again() {
    let parent = temp_settings_parent("retry");
    let harness = start_with_settings_folder(parent.clone()).await;
    let job_id = JobId::from_u128(7);
    let updated = envelope_for(&SettingsTankDocument {
        version: SettingsTankDocument::VERSION,
        live_field: 2,
        restart_field: "changed".into(),
    });
    let first = harness.submit("UpdateSettings", job_id, &updated).await;
    let settings_path = parent.join(format!(
        "{}/{}",
        SettingsTankService::NAME,
        settings_file_name(SettingsTankDocument::VERSION)
    ));
    std::fs::remove_file(&settings_path).unwrap();

    let retry = harness.submit("UpdateSettings", job_id, &updated).await;

    assert_eq!(retry, first);
    assert!(!settings_path.exists());
    assert_eq!(harness.jobs().await.jobs.len(), 1);

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn write_failure_restores_snapshot_and_rejects() {
    let parent = temp_settings_parent("write-fail");
    let harness = start_with_settings_folder(parent.clone()).await;
    let settings_path = parent.join(format!(
        "{}/{}",
        SettingsTankService::NAME,
        settings_file_name(SettingsTankDocument::VERSION)
    ));
    std::fs::remove_file(&settings_path).unwrap();
    std::fs::create_dir(&settings_path).unwrap();

    let updated = SettingsTankDocument {
        version: SettingsTankDocument::VERSION,
        live_field: 99,
        restart_field: "blocked".into(),
    };
    let ack = harness
        .send("UpdateSettings", &envelope_for(&updated))
        .await;
    assert!(!ack.accepted);

    let envelope = harness.settings::<SettingsEnvelope>().await;
    let running: SettingsTankDocument = serde_json::from_str(&envelope.document_json).unwrap();
    assert_eq!(running.live_field, 1);
    assert!(harness.jobs().await.jobs.is_empty());

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn settings_write_failure_does_not_queue_durable_snapshot() {
    let parent = temp_settings_parent("durable-rollback");
    let harness = start_with_settings_folder(parent.clone()).await;

    let accepted = SettingsTankDocument {
        version: SettingsTankDocument::VERSION,
        live_field: 2,
        restart_field: "ok".into(),
    };
    let ack = harness
        .send("UpdateSettings", &envelope_for(&accepted))
        .await;
    assert!(ack.accepted);
    time::advance(Duration::from_secs(1)).await;
    harness.flush_durable_writes().await;
    assert_eq!(durable_live_field_on_disk(&parent), Some(2));

    let settings_path = parent.join(format!(
        "{}/{}",
        SettingsTankService::NAME,
        settings_file_name(SettingsTankDocument::VERSION)
    ));
    std::fs::remove_file(&settings_path).unwrap();
    std::fs::create_dir(&settings_path).unwrap();

    let blocked = SettingsTankDocument {
        version: SettingsTankDocument::VERSION,
        live_field: 99,
        restart_field: "blocked".into(),
    };
    let blocked_ack = harness
        .send("UpdateSettings", &envelope_for(&blocked))
        .await;
    assert!(!blocked_ack.accepted);

    time::advance(Duration::from_secs(1)).await;
    harness.flush_durable_writes().await;
    assert_eq!(
        durable_live_field_on_disk(&parent),
        Some(2),
        "durable file must match the rolled-back snapshot, not the rejected update"
    );

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn rejects_foreign_version() {
    let parent = temp_settings_parent("foreign-version");
    let harness = start_with_settings_folder(parent.clone()).await;

    let document = serde_json::json!({
        "VERSION": 9,
        "live_field": 1,
        "restart_field": "x"
    });
    let envelope = SettingsEnvelope {
        document_json: document.to_string(),
        fields: Vec::new(),
    };
    let job_id = JobId::from_u128(7);
    let ack = harness.submit("UpdateSettings", job_id, &envelope).await;
    assert!(!ack.accepted);
    assert_eq!(ack.job_id, job_id.to_string());
    assert_eq!(ack.status, CommandAckStatus::StatusUnknown);

    let running: SettingsTankDocument =
        serde_json::from_str(&harness.settings::<SettingsEnvelope>().await.document_json).unwrap();
    assert_eq!(running, SettingsTankDocument::default());
    let on_disk: SettingsTankDocument = serde_json::from_str(
        &std::fs::read_to_string(parent.join(format!(
            "{}/{}",
            SettingsTankService::NAME,
            settings_file_name(SettingsTankDocument::VERSION)
        )))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(on_disk, SettingsTankDocument::default());
    assert!(harness.jobs().await.jobs.is_empty());

    let _ = std::fs::remove_dir_all(parent);
}

#[tokio::test(start_paused = true)]
async fn rejects_unknown_fields_in_document() {
    let parent = temp_settings_parent("unknown-field");
    let harness = start_with_settings_folder(parent.clone()).await;

    let document = serde_json::json!({
        "VERSION": 1,
        "live_field": 1,
        "restart_field": "x",
        "unexpected": true
    });
    let envelope = SettingsEnvelope {
        document_json: document.to_string(),
        fields: Vec::new(),
    };
    let ack = harness.send("UpdateSettings", &envelope).await;
    assert!(!ack.accepted);

    let _ = std::fs::remove_dir_all(parent);
}
