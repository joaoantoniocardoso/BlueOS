use alloc::string::String;

/// Relative path validation failures before a path is used for IO.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathValidationError {
    InvalidPath,
    UnsupportedSuffix,
}

/// Validates a user-supplied recording path for library commands.
pub fn validate_relative_recording_path(path: &str) -> Result<(), String> {
    match validate_relative_recording_path_inner(path) {
        Ok(()) => Ok(()),
        Err(PathValidationError::InvalidPath) => Err("Invalid recording path.".into()),
        Err(PathValidationError::UnsupportedSuffix) => {
            Err("Only .mcap recordings are supported.".into())
        }
    }
}

fn validate_relative_recording_path_inner(path: &str) -> Result<(), PathValidationError> {
    if path.is_empty() || path.starts_with('/') {
        return Err(PathValidationError::InvalidPath);
    }
    if path.split('/').any(|segment| segment == "..") {
        return Err(PathValidationError::InvalidPath);
    }
    if !path.to_ascii_lowercase().ends_with(".mcap") {
        return Err(PathValidationError::UnsupportedSuffix);
    }
    Ok(())
}
