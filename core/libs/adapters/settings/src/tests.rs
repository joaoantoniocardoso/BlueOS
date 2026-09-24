use std::{
    num::NonZeroU32,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use serde::{Deserialize, Serialize};

use super::{
    error::SettingsError,
    manager::SettingsManager,
    restart::diff_top_level_settings,
    schema::{SettingsSchema, read_version},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Animal {
    #[serde(default = "default_animal_name")]
    name: String,
    #[serde(default = "default_animal_type")]
    animal_type: String,
    #[serde(default)]
    parts: Vec<String>,
}

impl Default for Animal {
    fn default() -> Self {
        Self {
            name: default_animal_name(),
            animal_type: default_animal_type(),
            parts: Vec::new(),
        }
    }
}

fn default_animal_name() -> String {
    "bilica".into()
}

fn default_animal_type() -> String {
    "dog".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SettingsV1 {
    #[serde(rename = "VERSION")]
    version: NonZeroU32,
    #[serde(default = "default_first_variable")]
    first_variable: i32,
    #[serde(default)]
    animal: Animal,
}

impl Default for SettingsV1 {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            first_variable: default_first_variable(),
            animal: Animal::default(),
        }
    }
}

fn default_first_variable() -> i32 {
    42
}

impl SettingsSchema for SettingsV1 {
    const VERSION: NonZeroU32 = NonZeroU32::new(1).unwrap();

    fn migrate(data: &mut serde_json::Value) -> Result<(), SettingsError> {
        let version = read_version(data)?;
        if version == Self::VERSION {
            return Ok(());
        }
        data["VERSION"] = serde_json::to_value(Self::VERSION).expect("VERSION serializes");
        if data.get("first_variable").is_none() {
            data["first_variable"] = default_first_variable().into();
        }
        if data.get("animal").is_none() {
            data["animal"] = serde_json::to_value(Animal::default()).unwrap();
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct SettingsV2 {
    #[serde(rename = "VERSION")]
    version: NonZeroU32,
    #[serde(default = "default_v2_first_variable")]
    first_variable: i32,
    #[serde(default)]
    new_animal: Animal,
}

fn default_v2_first_variable() -> i32 {
    66
}

impl Default for SettingsV2 {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            first_variable: default_v2_first_variable(),
            new_animal: Animal::default(),
        }
    }
}

impl SettingsSchema for SettingsV2 {
    const VERSION: NonZeroU32 = NonZeroU32::new(2).unwrap();

    fn restart_required_fields() -> &'static [&'static str] {
        &["first_variable"]
    }

    fn migrate(data: &mut serde_json::Value) -> Result<(), SettingsError> {
        let version = read_version(data)?;
        if version == Self::VERSION {
            return Ok(());
        }
        if version < Self::VERSION {
            SettingsV1::migrate(data)?;
            let v1: SettingsV1 = serde_json::from_value(data.clone())?;
            data["VERSION"] = serde_json::to_value(Self::VERSION).expect("VERSION serializes");
            data["first_variable"] = v1.first_variable.into();
            if let Some(animal) = data.get("animal").cloned() {
                data["new_animal"] = animal;
                if let Some(map) = data.as_object_mut() {
                    map.remove("animal");
                }
            }
        }
        Ok(())
    }
}

static ON_SETTINGS_CREATED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct HookedSettings {
    #[serde(rename = "VERSION")]
    version: NonZeroU32,
    #[serde(default)]
    initialized: bool,
}

impl Default for HookedSettings {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            initialized: false,
        }
    }
}

impl SettingsSchema for HookedSettings {
    const VERSION: NonZeroU32 = NonZeroU32::new(1).unwrap();

    fn migrate(_data: &mut serde_json::Value) -> Result<(), SettingsError> {
        Ok(())
    }

    fn on_settings_created(&self, _path: &std::path::Path) -> Result<(), SettingsError> {
        ON_SETTINGS_CREATED.store(true, Ordering::SeqCst);
        Ok(())
    }
}

fn temp_config_dir(name: &str) -> PathBuf {
    let directory =
        std::env::temp_dir().join(format!("blueos-settings-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

#[test]
fn rejects_missing_version_field() {
    let value = serde_json::json!({"first_variable": 1});
    let error = SettingsV1::load_from_value(value).unwrap_err();
    assert!(matches!(error, SettingsError::BadSettingsFile(_)));
}

#[test]
fn rejects_zero_version() {
    let value = serde_json::json!({"VERSION": 0, "first_variable": 1});
    let error = SettingsV1::load_from_value(value).unwrap_err();
    assert!(matches!(error, SettingsError::BadAttributes));
}

#[test]
fn on_settings_created_runs_before_first_save() {
    let directory = temp_config_dir("hook");
    let path = directory.join("settings-1.json");
    ON_SETTINGS_CREATED.store(false, Ordering::SeqCst);

    HookedSettings::default().save(&path).unwrap();

    assert!(ON_SETTINGS_CREATED.load(Ordering::SeqCst));
}

#[test]
fn manager_creates_default_when_missing() {
    let directory = temp_config_dir("manager-default");
    let manager =
        SettingsManager::<SettingsV1>::new("test-service", Some(directory.clone())).unwrap();
    assert_eq!(manager.settings().first_variable, 42);
    assert!(manager.settings_file_path().is_file());
}

#[test]
fn migration_v1_to_v2() {
    let directory = temp_config_dir("migration");
    let v1_path = directory.join("settings-1.json");
    let v1 = SettingsV1 {
        first_variable: 66,
        animal: Animal {
            name: "pingu".into(),
            animal_type: "penguin".into(),
            parts: Vec::new(),
        },
        ..SettingsV1::default()
    };
    v1.save(&v1_path).unwrap();

    let v2 = SettingsV2::load(&v1_path).unwrap();
    assert_eq!(v2.first_variable, 66);
    assert_eq!(v2.new_animal.name, "pingu");
}

#[test]
fn diff_top_level_restart_fields() {
    let old_value = serde_json::json!({
        "VERSION": 2,
        "first_variable": 1,
        "new_animal": {"name": "a", "animal_type": "b", "parts": []}
    });
    let new_value = serde_json::json!({
        "VERSION": 2,
        "first_variable": 2,
        "new_animal": {"name": "c", "animal_type": "b", "parts": []}
    });
    let changes = diff_top_level_settings(
        SettingsV2::restart_required_fields(),
        &old_value,
        &new_value,
    );
    assert_eq!(changes.restart_required, vec!["first_variable".to_string()]);
    assert_eq!(changes.applied_live, vec!["new_animal".to_string()]);
}

#[test]
fn reset_restores_defaults() {
    let directory = temp_config_dir("reset");
    let path = directory.join("settings-1.json");
    let settings = SettingsV1 {
        first_variable: 66,
        ..SettingsV1::default()
    };
    settings.save(&path).unwrap();
    let mut settings = settings;

    settings.reset();
    assert_eq!(settings.first_variable, 42);
}
