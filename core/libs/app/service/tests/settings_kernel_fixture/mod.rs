//! Kernel settings: load, `UpdateSettings`, persist rollback, and pending restart fields.

#![expect(
    unreachable_pub,
    reason = "fixture items are re-exported to integration test roots"
)]
#![expect(
    dead_code,
    reason = "fixture items are shared across sibling integration test binaries"
)]

#[path = "../common/mod.rs"]
mod common;

use core::{convert::Infallible, num::NonZeroU32};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use blueos_comms::channel::ChannelBackend;
use blueos_domain::{Command, Decision, Domain, DomainDurable, IoError, Now, Outcome};
use blueos_idl::{msg::blueos_example_msgs::SetLevelGoal, msg::blueos_msgs::SettingsEnvelope};
use blueos_service::{Service, ServiceBuilder, ServiceContext, ServiceError, testing::Harness};
use blueos_settings::{SettingsError, SettingsSchema, settings_file_name, state_file_name};

const DURABLE_STATE_VERSION: NonZeroU32 = NonZeroU32::MIN;

#[derive(clap::Args, Default)]
pub struct SettingsTankArguments;

pub struct SettingsTankService;

#[derive(Clone)]
pub struct SettingsTankSnapshot {
    level: u8,
    settings: SettingsTankDocument,
    durable: SettingsTankDurable,
}

pub enum SettingsTankRequest {
    SetLevel(u8),
    UpdateSettings(SettingsTankDocument),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SettingsTankDocument {
    #[serde(rename = "VERSION")]
    pub version: NonZeroU32,
    pub live_field: u32,
    pub restart_field: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsTankDurable {
    live_field: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SettingsTankTick {
    Restored,
}

pub struct SettingsTank;

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
        _service: &ServiceContext<SettingsTankArguments>,
        _context: &(),
    ) -> Result<ServiceBuilder<SettingsTank>, ServiceError> {
        Ok(ServiceBuilder::new(SettingsTankSnapshot {
            level: 0,
            settings: SettingsTankDocument::default(),
            durable: SettingsTankDurable { live_field: 1 },
        })
        .command("SetLevel", |request: SetLevelGoal| {
            Ok(SettingsTankRequest::SetLevel(request.level))
        })
        .durable_state(DURABLE_STATE_VERSION)
        .settings(
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

pub fn settings_tank_document(
    live_field: u32,
    restart_field: impl Into<String>,
) -> SettingsTankDocument {
    SettingsTankDocument {
        live_field,
        restart_field: restart_field.into(),
        ..SettingsTankDocument::default()
    }
}

pub fn settings_document_path(parent: &Path) -> PathBuf {
    parent.join(format!(
        "{}/{}",
        SettingsTankService::NAME,
        settings_file_name(SettingsTankDocument::VERSION)
    ))
}

pub fn temp_settings_parent(label: &str) -> PathBuf {
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

pub async fn start_with_settings_folder(parent: PathBuf) -> Harness<SettingsTankService> {
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

pub fn envelope_for(document: &SettingsTankDocument) -> SettingsEnvelope {
    SettingsEnvelope {
        document_json: serde_json::to_string(document).unwrap(),
        fields: Vec::new(),
    }
}

pub fn durable_live_field_on_disk(parent: &Path) -> Option<u32> {
    common::durable_u32_from_json(&durable_state_path(parent), "/domain/live_field")
}

pub fn durable_state_path(parent: &Path) -> PathBuf {
    parent.join(format!(
        "{}/{}",
        SettingsTankService::NAME,
        state_file_name(DURABLE_STATE_VERSION)
    ))
}
