//! Relative recording path validation.

use std::path::{Component, Path};

use super::StorageError;

const RECORDING_SUFFIX: &str = ".mcap";

pub(crate) fn validate_relative_recording_path(relative: &str) -> Result<(), StorageError> {
    if relative.is_empty() || relative.starts_with('/') || Path::new(relative).is_absolute() {
        return Err(StorageError::InvalidPath);
    }
    if relative.contains('\0') {
        return Err(StorageError::InvalidPath);
    }
    if !path_components_are_safe(relative)? {
        return Err(StorageError::InvalidPath);
    }
    if !relative.to_ascii_lowercase().ends_with(RECORDING_SUFFIX) {
        return Err(StorageError::InvalidPath);
    }
    Ok(())
}

fn path_components_are_safe(relative: &str) -> Result<bool, StorageError> {
    for component in Path::new(relative).components() {
        match component {
            Component::Normal(segment) => {
                if segment.is_empty() || segment == "." || segment == ".." {
                    return Ok(false);
                }
            }
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return Ok(false),
            Component::CurDir => return Ok(false),
        }
    }
    Ok(true)
}
