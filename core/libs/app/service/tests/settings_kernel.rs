//! Kernel settings: load, `UpdateSettings`, persist rollback, and pending restart fields.

use core::{convert::Infallible, num::NonZeroU32};
use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};

use blueos_comms::channel::ChannelBackend;
use blueos_domain::{Command, Decision, Domain, Now, Outcome};
use blueos_idl::{msg::blueos_example_msgs::SetLevelRequest, msg::blueos_msgs::SettingsEnvelope};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};
use blueos_settings::{SettingsError, SettingsSchema, settings_file_name};

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

#[derive(Clone)]
struct SettingsTankSnapshot {
    level: u8,
    settings: SettingsTankDocument,
}

enum SettingsTankRequest {
    SetLevel(u8),
    UpdateSettings(SettingsTankDocument),
}

struct SettingsTank;

impl Service for SettingsTankService {
    type Domain = SettingsTank;
    type Arguments = SettingsTankArguments;

    const NAME: &'static str = "settings_tank";
    const VERSION: &'static str = "1.0.0";

    fn build(
        context: &ServiceContext<SettingsTankArguments>,
    ) -> Result<ServiceBuilder<SettingsTank>, ServiceError> {
        let config_parent = context.settings_path().map(PathBuf::from);
        Ok(ServiceBuilder::new(SettingsTankSnapshot {
            level: 0,
            settings: SettingsTankDocument::default(),
        })
        .command("SetLevel", |request: SetLevelRequest| {
            SettingsTankRequest::SetLevel(request.level)
        })
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
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type Event = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut SettingsTankSnapshot,
        command: Command<SettingsTankRequest, Infallible, Infallible, Infallible>,
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
                snapshot.settings = settings;
                Outcome::Applied {
                    events: Vec::new(),
                    effects: Vec::new(),
                }
            }
        }
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
    Harness::start_on_with_context(
        Arc::new(ChannelBackend::default()),
        ServiceContext::with_settings_path(SettingsTankArguments, Some(parent)),
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
    let ack = harness.send("UpdateSettings", &envelope).await;
    assert!(!ack.accepted);

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
