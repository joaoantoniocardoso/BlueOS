use std::num::NonZeroU32;
use std::path::PathBuf;

use blueos_settings::{SettingsError, SettingsManager, SettingsSchema, read_version};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct GoldenAnimal {
    name: String,
    animal_type: String,
}

impl Default for GoldenAnimal {
    fn default() -> Self {
        Self {
            name: "bilica".into(),
            animal_type: "dog".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct GoldenSettingsV1 {
    #[serde(rename = "VERSION")]
    version: NonZeroU32,
    first_variable: i32,
    animal: GoldenAnimal,
}

impl Default for GoldenSettingsV1 {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            first_variable: 100,
            animal: GoldenAnimal {
                name: "pingu".into(),
                animal_type: "penguin".into(),
            },
        }
    }
}

impl SettingsSchema for GoldenSettingsV1 {
    const VERSION: NonZeroU32 = NonZeroU32::new(1).unwrap();

    fn migrate(data: &mut serde_json::Value) -> Result<(), SettingsError> {
        let version = read_version(data)?;
        if version == Self::VERSION {
            return Ok(());
        }
        data["VERSION"] = serde_json::to_value(Self::VERSION).expect("VERSION serializes");
        if data.get("first_variable").is_none() {
            data["first_variable"] = Self::default().first_variable.into();
        }
        if data.get("animal").is_none() {
            data["animal"] = serde_json::to_value(Self::default().animal).unwrap();
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct GoldenSettingsV2 {
    #[serde(rename = "VERSION")]
    version: NonZeroU32,
    first_variable: i32,
    new_animal: GoldenAnimal,
}

impl Default for GoldenSettingsV2 {
    fn default() -> Self {
        Self {
            version: Self::VERSION,
            first_variable: 66,
            new_animal: GoldenAnimal::default(),
        }
    }
}

impl SettingsSchema for GoldenSettingsV2 {
    const VERSION: NonZeroU32 = NonZeroU32::new(2).unwrap();

    fn migrate(data: &mut serde_json::Value) -> Result<(), SettingsError> {
        let version = read_version(data)?;
        if version == Self::VERSION {
            return Ok(());
        }
        if version < Self::VERSION {
            GoldenSettingsV1::migrate(data)?;
            data["VERSION"] = serde_json::to_value(Self::VERSION).expect("VERSION serializes");
            if data.get("first_variable").is_none() {
                data["first_variable"] = Self::default().first_variable.into();
            }
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

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

#[test]
fn golden_load_v1_fixture() {
    let path = fixtures_dir().join("golden_settings_v1.json");
    let loaded = GoldenSettingsV1::load(&path).unwrap();
    assert_eq!(loaded.first_variable, 100);
    assert_eq!(loaded.animal.name, "pingu");
}

#[test]
fn golden_migrate_v1_to_v2_matches_python_save() {
    let v1_path = fixtures_dir().join("golden_settings_v1.json");
    let expected =
        std::fs::read(fixtures_dir().join("golden_settings_v2_migrated_save.json")).unwrap();

    let migrated = GoldenSettingsV2::load(&v1_path).unwrap();
    assert_eq!(migrated.first_variable, 100);
    assert_eq!(migrated.new_animal.name, "pingu");

    let output = std::env::temp_dir().join(format!("golden-v2-migrated-{}", std::process::id()));
    migrated.save(&output).unwrap();
    let written = std::fs::read(&output).unwrap();
    assert_eq!(written, expected);
    let _ = std::fs::remove_file(output);
}

#[test]
fn golden_save_matches_python_bytes() {
    let expected =
        std::fs::read(fixtures_dir().join("golden_settings_v2_direct_save.json")).unwrap();
    let output = std::env::temp_dir().join(format!("golden-v2-direct-{}", std::process::id()));
    GoldenSettingsV2::default().save(&output).unwrap();
    let written = std::fs::read(&output).unwrap();
    assert_eq!(written, expected);
    let _ = std::fs::remove_file(output);
}

#[test]
fn golden_rejects_future_version_fixture() {
    let path = fixtures_dir().join("golden_settings_future_v9.json");
    let error = GoldenSettingsV2::load(&path).unwrap_err();
    assert!(matches!(error, SettingsError::SettingsFromTheFuture(_)));
}

#[test]
fn golden_manager_migrates_v1_fixture_on_disk() {
    let parent = std::env::temp_dir().join(format!("golden-manager-{}", std::process::id()));
    let service_dir = parent.join("golden-test");
    std::fs::create_dir_all(&service_dir).unwrap();
    std::fs::copy(
        fixtures_dir().join("golden_settings_v1.json"),
        service_dir.join("settings-1.json"),
    )
    .unwrap();

    let manager = SettingsManager::<GoldenSettingsV2>::new("golden-test", Some(parent)).unwrap();
    assert_eq!(manager.settings().first_variable, 100);
    assert_eq!(manager.settings().new_animal.animal_type, "penguin");
}
