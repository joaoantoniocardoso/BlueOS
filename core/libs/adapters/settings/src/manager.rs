use std::path::{Path, PathBuf};

pub(crate) use std::{marker::PhantomData, num::NonZeroU32};

use super::{error::SettingsError, schema::SettingsSchema};

pub const SETTINGS_NAME_PREFIX: &str = "settings-";

pub fn settings_file_name(version: NonZeroU32) -> String {
    format!("{SETTINGS_NAME_PREFIX}{version}.json")
}

/// Resolves the config directory the same way as Python `PydanticManager`:
/// `appdirs.user_config_dir(project_name.lower())`, or `config_folder.join(project_name.lower())`.
pub fn resolve_config_folder(
    project_name: impl AsRef<str>,
    config_folder: Option<PathBuf>,
) -> PathBuf {
    let project_name = project_name.as_ref().to_lowercase();
    match config_folder {
        Some(parent) => parent.join(project_name),
        None => user_config_dir().join(project_name),
    }
}

pub struct SettingsManager<S: SettingsSchema> {
    config_folder: PathBuf,
    settings: S,
    _marker: PhantomData<S>,
}

impl<S: SettingsSchema> SettingsManager<S> {
    pub fn new(
        project_name: impl Into<String>,
        config_folder: Option<PathBuf>,
    ) -> Result<Self, SettingsError> {
        Self::with_load(project_name, config_folder, true)
    }

    pub fn with_load(
        project_name: impl Into<String>,
        config_folder: Option<PathBuf>,
        load: bool,
    ) -> Result<Self, SettingsError> {
        let project_name = project_name.into();
        if project_name.is_empty() {
            return Err(SettingsError::BadSettingsFile(
                "project_name should be not empty".into(),
            ));
        }

        let config_folder = resolve_config_folder(&project_name, config_folder);
        std::fs::create_dir_all(&config_folder)?;

        let mut manager = Self {
            config_folder,
            settings: S::default(),
            _marker: PhantomData,
        };
        if load {
            manager.load()?;
        }
        Ok(manager)
    }

    pub fn from_loaded(config_folder: PathBuf, settings: S) -> Self {
        Self {
            config_folder,
            settings,
            _marker: PhantomData,
        }
    }

    pub fn settings(&self) -> &S {
        &self.settings
    }

    pub fn settings_mut(&mut self) -> &mut S {
        &mut self.settings
    }

    pub fn set(&mut self, settings: S) -> Result<(), SettingsError> {
        self.settings = settings;
        self.save()
    }

    pub fn config_folder(&self) -> &Path {
        &self.config_folder
    }

    pub fn settings_file_path(&self) -> PathBuf {
        self.config_folder.join(settings_file_name(S::VERSION))
    }

    pub fn load_from_file(path: &Path) -> Result<S, SettingsError> {
        if path.is_file() {
            S::load(path)
        } else {
            let settings = S::default();
            settings.save(path)?;
            Ok(settings)
        }
    }

    pub fn save(&mut self) -> Result<(), SettingsError> {
        let path = self.settings_file_path();
        self.settings.save(&path)
    }

    pub fn load(&mut self) -> Result<(), SettingsError> {
        self.clear_temp_files();

        let mut candidates: Vec<PathBuf> = std::fs::read_dir(&self.config_folder)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name.starts_with(SETTINGS_NAME_PREFIX) && name.ends_with(".json")
                    })
            })
            .collect();
        candidates.sort_by(|left, right| {
            settings_version_from_path(right).cmp(&settings_version_from_path(left))
        });

        for candidate in candidates {
            if let Ok(settings) = Self::load_from_file(&candidate) {
                self.settings = settings;
                return Ok(());
            }
        }

        for legacy_path in S::legacy_load_paths(&self.config_folder) {
            if !legacy_path.is_file() {
                continue;
            }
            if let Ok(settings) = Self::load_legacy_file(&legacy_path) {
                self.settings = settings;
                self.save()?;
                return Ok(());
            }
        }

        self.settings = S::default();
        self.save()
    }

    pub fn load_legacy_file(path: &Path) -> Result<S, SettingsError> {
        let data = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&data)?;
        S::load_from_value(value)
    }

    fn clear_temp_files(&self) {
        let Ok(entries) = std::fs::read_dir(&self.config_folder) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|extension| extension == "tmp") {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

fn settings_version_from_path(path: &Path) -> Option<NonZeroU32> {
    let version = path
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| {
            name.strip_prefix(SETTINGS_NAME_PREFIX)
                .and_then(|rest| rest.strip_suffix(".json"))
        })?;
    version.parse().ok().and_then(NonZeroU32::new)
}

fn user_config_dir() -> PathBuf {
    if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME")
        && !xdg_config_home.is_empty()
    {
        return PathBuf::from(xdg_config_home);
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config");
    }
    PathBuf::from("/root/.config")
}
