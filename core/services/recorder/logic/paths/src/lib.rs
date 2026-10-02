//! Relative recording paths: validated once before IO or Domain Commands.

#![no_std]
#![expect(
    clippy::pub_use,
    reason = "calendar helpers are shared by the library Block and storage adapter"
)]

extern crate alloc;

mod calendar;

use alloc::string::String;

pub use calendar::{
    civil_date_from_days_since_epoch, civil_days_since_epoch, recorder_wall_clock_file_stamp,
};

const RECORDING_SUFFIX: &str = ".mcap";

/// Why a relative recording path string is not acceptable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordingPathError {
    /// Empty, absolute, contains `..`, or otherwise invalid.
    InvalidPath,
    /// The path does not end with `.mcap` (case-insensitive).
    UnsupportedSuffix,
}

/// A library-relative MCAP path that passed [`RecordingRelativePath::parse`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct RecordingRelativePath(String);

impl RecordingRelativePath {
    /// Parses and validates `relative` (syntax only; library membership is checked separately).
    pub fn parse(relative: &str) -> Result<Self, RecordingPathError> {
        validate_relative_recording_path(relative)?;
        Ok(Self(relative.into()))
    }

    /// The validated relative path string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Validates syntax for a user-supplied recording path.
pub fn validate_relative_recording_path(relative: &str) -> Result<(), RecordingPathError> {
    if relative.is_empty() || relative.starts_with('/') || relative.contains('\0') {
        return Err(RecordingPathError::InvalidPath);
    }
    if relative
        .split('/')
        .any(|segment| segment.is_empty() || segment == ".." || segment == ".")
    {
        return Err(RecordingPathError::InvalidPath);
    }
    if !relative.to_ascii_lowercase().ends_with(RECORDING_SUFFIX) {
        return Err(RecordingPathError::UnsupportedSuffix);
    }
    Ok(())
}

/// Client-facing refusal text for a path parse error.
pub fn recording_path_refusal(error: RecordingPathError) -> &'static str {
    match error {
        RecordingPathError::InvalidPath => "Invalid recording path.",
        RecordingPathError::UnsupportedSuffix => "Only .mcap recordings are supported.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_hostile_and_non_mcap_paths() {
        assert!(RecordingRelativePath::parse("../outside.mcap").is_err());
        assert!(RecordingRelativePath::parse("/etc/passwd.mcap").is_err());
        assert!(RecordingRelativePath::parse("notes.txt").is_err());
        assert!(RecordingRelativePath::parse("a//b.mcap").is_err());
        assert!(RecordingRelativePath::parse("./local.mcap").is_err());
        assert!(RecordingRelativePath::parse("dir/./file.mcap").is_err());
    }
}
