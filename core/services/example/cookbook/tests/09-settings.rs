//! Settings load at start, persist on update, and publish restart-required fields.

use core::{convert::Infallible, num::NonZeroU32};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use blueos_comms::channel::ChannelBackend;
use blueos_domain::{Command, Decision, Domain, IoError, Now, Outcome};
use blueos_idl::msg::blueos_msgs::SettingsEnvelope;
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};
use blueos_settings::{SettingsError, SettingsSchema, settings_file_name};

struct SettingsCookbookService;

#[derive(Clone, Default, clap::Args)]
struct SettingsCookbookArguments;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct SettingsCookbookDocument {
    #[serde(rename = "VERSION")]
    version: NonZeroU32,
    live_field: u32,
    restart_field: String,
}

impl Default for SettingsCookbookDocument {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            live_field: 1,
            restart_field: "initial".into(),
        }
    }
}

impl SettingsSchema for SettingsCookbookDocument {
    const VERSION: NonZeroU32 = NonZeroU32::new(1).unwrap();

    fn migrate(_data: &mut serde_json::Value) -> Result<(), SettingsError> {
        Ok(())
    }

    fn restart_required_fields() -> &'static [&'static str] {
        &["restart_field"]
    }
}

#[derive(Clone)]
struct SettingsCookbookSnapshot {
    settings: SettingsCookbookDocument,
}

enum SettingsCookbookRequest {
    UpdateSettings(SettingsCookbookDocument),
}

struct SettingsCookbook;

impl Service for SettingsCookbookService {
    type Domain = SettingsCookbook;
    type Context = ();
    type Arguments = SettingsCookbookArguments;

    const NAME: &'static str = "cookbook_settings";
    const VERSION: &'static str = "1.0.0";

    fn build(
        context: &ServiceContext<SettingsCookbookArguments>,
    ) -> Result<ServiceBuilder<SettingsCookbook>, ServiceError> {
        let config_parent = context.settings_path().map(PathBuf::from);
        Ok(ServiceBuilder::new(SettingsCookbookSnapshot {
            settings: SettingsCookbookDocument::default(),
        })
        .settings(
            Self::NAME,
            config_parent,
            |snapshot: &mut SettingsCookbookSnapshot, settings| snapshot.settings = settings,
            |snapshot: &SettingsCookbookSnapshot| snapshot.settings.clone(),
            |envelope| {
                let document: SettingsCookbookDocument =
                    serde_json::from_str(&envelope.document_json)?;
                Ok(SettingsCookbookRequest::UpdateSettings(document))
            },
        ))
    }
}

impl Domain for SettingsCookbook {
    type Snapshot = SettingsCookbookSnapshot;
    type Request = SettingsCookbookRequest;
    type Event = Infallible;
    type IoResult = Infallible;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type IoRequest = Infallible;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut SettingsCookbookSnapshot,
        command: Command<SettingsCookbookRequest, Infallible, Infallible, Infallible>,
        _now: Now,
    ) -> Decision<Self> {
        let Command::Request(SettingsCookbookRequest::UpdateSettings(settings)) = command;
        snapshot.settings = settings;
        Outcome::Applied {
            events: Vec::new(),
            effects: Vec::new(),
        }
    }

    fn io_failed(
        request: Self::IoRequest,
        _error: IoError,
    ) -> Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        match request {}
    }
}

fn temp_settings_parent(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "cookbook-settings-{}-{}-{}",
        label,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}

fn envelope_for(document: &SettingsCookbookDocument) -> SettingsEnvelope {
    SettingsEnvelope {
        document_json: serde_json::to_string(document).unwrap(),
        fields: Vec::new(),
    }
}

async fn start_with_settings_folder(parent: PathBuf) -> Harness<SettingsCookbookService> {
    let _ = std::fs::remove_dir_all(&parent);
    std::fs::create_dir_all(&parent).unwrap();
    Harness::start_on_with_context(
        std::sync::Arc::new(ChannelBackend::default()),
        ServiceContext::with_settings_path(SettingsCookbookArguments, Some(parent)),
    )
    .await
    .expect("harness starts")
}

#[tokio::test(start_paused = true)]
async fn settings_load_at_start() {
    let parent = temp_settings_parent("load");
    let harness = start_with_settings_folder(parent).await;
    let envelope = harness.settings::<SettingsEnvelope>().await;
    let document: SettingsCookbookDocument = serde_json::from_str(&envelope.document_json).unwrap();
    assert_eq!(document.live_field, 1);
}

#[tokio::test(start_paused = true)]
async fn update_settings_persists_to_disk() {
    let parent = temp_settings_parent("persist");
    let harness = start_with_settings_folder(parent.clone()).await;
    let updated = SettingsCookbookDocument {
        version: SettingsCookbookDocument::VERSION,
        live_field: 9,
        restart_field: "changed".into(),
    };
    let ack = harness
        .send("UpdateSettings", &envelope_for(&updated))
        .await;
    assert!(ack.accepted);
    let on_disk_path = parent.join(format!(
        "{}/{}",
        SettingsCookbookService::NAME,
        settings_file_name(SettingsCookbookDocument::VERSION)
    ));
    let on_disk: SettingsCookbookDocument =
        serde_json::from_str(&std::fs::read_to_string(on_disk_path).unwrap()).unwrap();
    assert_eq!(on_disk.live_field, 9);
}

#[tokio::test(start_paused = true)]
async fn restart_required_field_is_published() {
    let parent = temp_settings_parent("restart");
    let harness = start_with_settings_folder(parent).await;
    let updated = SettingsCookbookDocument {
        version: SettingsCookbookDocument::VERSION,
        live_field: 1,
        restart_field: "needs-restart".into(),
    };
    harness
        .send("UpdateSettings", &envelope_for(&updated))
        .await;
    let envelope = harness.settings::<SettingsEnvelope>().await;
    assert_eq!(envelope.fields.len(), 1);
    assert_eq!(envelope.fields[0].path, "restart_field");
    assert!(envelope.fields[0].restart_required);
}
