//! Workspace `Cargo.toml` IO for endpoint generation.

use std::{fs, path::Path};

use crate::endpoints::EndpointsError;

/// The `workspace.members` of the `Cargo.toml` at `core_dir`.
pub(crate) fn workspace_members(core_dir: &Path) -> Result<Vec<String>, EndpointsError> {
    let workspace = read_toml(&core_dir.join("Cargo.toml"))?;
    let members = workspace
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| EndpointsError::Cargo {
            path: core_dir.join("Cargo.toml"),
            reason: "no `workspace.members` list".to_owned(),
        })?;
    Ok(members
        .iter()
        .filter_map(toml::Value::as_str)
        .map(str::to_owned)
        .collect())
}

pub(crate) fn read_toml(path: &Path) -> Result<toml::Table, EndpointsError> {
    read(path)?
        .parse()
        .map_err(|error: toml::de::Error| EndpointsError::Cargo {
            path: path.to_path_buf(),
            reason: error.to_string(),
        })
}

pub(crate) fn read(path: &Path) -> Result<String, EndpointsError> {
    fs::read_to_string(path).map_err(|error| EndpointsError::Read {
        path: path.to_path_buf(),
        error,
    })
}
