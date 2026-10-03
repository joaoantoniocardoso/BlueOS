//! Kernel-owned settings load, `UpdateSettings`, and persistence (D-11).

use core::{error::Error, num::NonZeroU32};
use std::{path::PathBuf, sync::Mutex};

use serde_json::Value;

use blueos_api::{Message, cdr_encoding};
use blueos_domain::Domain;
use blueos_idl::{
    Error as IdlError,
    msg::blueos_msgs::{SettingField, SettingsEnvelope},
};
use blueos_settings::{
    SettingsError, SettingsManager, SettingsSchema, diff_top_level_settings, read_version,
    serialize_settings_document,
};

use crate::service::ServiceError;

type SettingsDriverBox<D> = Box<dyn SettingsDriver<D>>;

/// Opens the settings of the Service named by the first argument in the settings folder of the second.
type SettingsStartup<D> =
    Box<dyn FnOnce(String, Option<PathBuf>) -> Result<SettingsDriverBox<D>, ServiceError>>;

/// Configuration collected in [`ServiceBuilder::settings`]; the [`Kernel`] opens the manager at startup.
pub(crate) struct SettingsRegistration<D: Domain> {
    pub start: SettingsStartup<D>,
}

/// Loads, persists, and builds the `settings` State for one service document type.
pub(crate) trait SettingsDriver<D: Domain>: Send {
    /// Applies the on-disk document to `snapshot` during Kernel startup.
    fn load_into(&mut self, snapshot: &mut D::Snapshot) -> Result<(), ServiceError>;

    /// Validates `envelope` and turns it into the Domain Request for `UpdateSettings`.
    fn request_from_envelope(
        &self,
        envelope: SettingsEnvelope,
    ) -> Result<D::Request, Box<dyn Error + Send + Sync>>;

    /// Writes the document derived from `snapshot` atomically.
    fn persist(&mut self, snapshot: &D::Snapshot) -> Result<(), SettingsError>;

    /// Encodes the `settings` State from `snapshot` and the startup baseline.
    fn encode_state(&self, snapshot: &D::Snapshot) -> Result<Vec<u8>, IdlError>;

    /// Reserved for a future baseline update; the startup baseline is fixed for pending restart hints.
    fn commit_persisted(&mut self, snapshot: &D::Snapshot);
}

#[expect(
    clippy::type_complexity,
    reason = "the settings port stores three independent service mappings"
)]
struct TypedSettingsDriver<D: Domain, S: SettingsSchema> {
    manager: Mutex<SettingsManager<S>>,
    /// Document loaded at process start; pending restart diffs compare the running snapshot to this baseline.
    baseline_at_start: Mutex<Value>,
    into_snapshot: Box<dyn Fn(&mut D::Snapshot, S) + Send + Sync>,
    from_snapshot: Box<dyn Fn(&D::Snapshot) -> S + Send + Sync>,
    into_request: Box<
        dyn Fn(SettingsEnvelope) -> Result<D::Request, Box<dyn Error + Send + Sync>> + Send + Sync,
    >,
}

impl<D, S> SettingsDriver<D> for TypedSettingsDriver<D, S>
where
    D: Domain,
    S: SettingsSchema + Send + Sync + 'static,
{
    fn load_into(&mut self, snapshot: &mut D::Snapshot) -> Result<(), ServiceError> {
        let manager = self.manager.lock().map_err(|_poisoned| {
            ServiceError::Settings(SettingsError::BadSettingsFile(
                "settings manager poisoned".into(),
            ))
        })?;
        (self.into_snapshot)(snapshot, manager.settings().clone());
        Ok(())
    }

    fn request_from_envelope(
        &self,
        envelope: SettingsEnvelope,
    ) -> Result<D::Request, Box<dyn Error + Send + Sync>> {
        reject_foreign_version(&envelope.document_json, S::VERSION)?;
        let value: Value = serde_json::from_str(&envelope.document_json)?;
        S::load_from_value(value)?;
        (self.into_request)(envelope)
    }

    fn persist(&mut self, snapshot: &D::Snapshot) -> Result<(), SettingsError> {
        let settings = (self.from_snapshot)(snapshot);
        let mut manager = self.manager.lock().map_err(|_poisoned| {
            SettingsError::BadSettingsFile("settings manager poisoned".into())
        })?;
        *manager.settings_mut() = settings;
        manager.save()
    }

    fn encode_state(&self, snapshot: &D::Snapshot) -> Result<Vec<u8>, IdlError> {
        let baseline = self
            .baseline_at_start
            .lock()
            .map_err(|_poisoned| IdlError::InvalidLength)?;
        settings_envelope(
            &(self.from_snapshot)(snapshot),
            &baseline,
            S::restart_required_fields(),
        )
        .encode()
    }

    fn commit_persisted(&mut self, _snapshot: &D::Snapshot) {}
}

/// Records the settings port; disk IO happens when the [`Kernel`] starts.
pub(crate) fn register_settings<D, S>(
    into_snapshot: impl Fn(&mut D::Snapshot, S) + Send + Sync + 'static,
    from_snapshot: impl Fn(&D::Snapshot) -> S + Send + Sync + 'static,
    into_request: impl Fn(SettingsEnvelope) -> Result<D::Request, Box<dyn Error + Send + Sync>>
    + Send
    + Sync
    + 'static,
) -> SettingsRegistration<D>
where
    D: Domain,
    S: SettingsSchema + Send + Sync + 'static,
{
    SettingsRegistration {
        start: Box::new(move |service_name, config_folder| {
            let manager = SettingsManager::<S>::new(service_name, config_folder)?;
            let loaded = manager.settings().clone();
            let driver = TypedSettingsDriver {
                manager: Mutex::new(manager),
                baseline_at_start: Mutex::new(
                    serde_json::to_value(&loaded).expect("settings serializes"),
                ),
                into_snapshot: Box::new(into_snapshot),
                from_snapshot: Box::new(from_snapshot),
                into_request: Box::new(into_request),
            };
            Ok(Box::new(driver))
        }),
    }
}

/// Rejects an update whose top-level `VERSION` is not this service's settings version.
pub(crate) fn reject_foreign_version(
    document_json: &str,
    expected: NonZeroU32,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let value: Value = serde_json::from_str(document_json)?;
    let version = read_version(&value)?;
    if version != expected {
        return Err(SettingsError::BadSettingsFile(format!(
            "settings VERSION {} does not match this service ({})",
            version, expected
        ))
        .into());
    }
    Ok(())
}

/// Builds the wire `SettingsEnvelope`: running document JSON plus pending restart-required diffs.
pub(crate) fn settings_envelope<S: SettingsSchema>(
    running: &S,
    baseline_at_start: &Value,
    restart_required_fields: &[&str],
) -> SettingsEnvelope {
    let document_json = String::from_utf8(
        serialize_settings_document(running).expect("settings document serializes"),
    )
    .expect("settings JSON is UTF-8");
    let running_value =
        serde_json::to_value(running).expect("settings document serializes to JSON");
    let changes =
        diff_top_level_settings(restart_required_fields, baseline_at_start, &running_value);
    let fields = changes
        .restart_required
        .into_iter()
        .map(|path| SettingField {
            path,
            restart_required: true,
        })
        .collect();
    SettingsEnvelope {
        document_json,
        fields,
    }
}

/// CDR encoding name for the `settings` endpoint.
pub(crate) fn settings_encoding() -> String {
    cdr_encoding(SettingsEnvelope::SCHEMA_NAME)
}
