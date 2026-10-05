//! Default layout paths for the IDL crate and the Rust workspace.

use std::path::{Path, PathBuf};

use crate::error::CodegenError;

/// Resolves the `blueos-idl` root from the codegen crate manifest directory.
pub fn idl_root(
    manifest_dir: &Path,
    override_path: Option<PathBuf>,
) -> Result<PathBuf, CodegenError> {
    match override_path {
        Some(path) => Ok(path),
        None => manifest_dir
            .parent()
            .ok_or_else(|| CodegenError::MissingParent {
                path: manifest_dir.to_path_buf(),
            })
            .map(Path::to_path_buf),
    }
}

/// Resolves the `core/` directory that owns the workspace `Cargo.toml`.
pub fn core_dir(idl_root: &Path, override_path: Option<PathBuf>) -> Result<PathBuf, CodegenError> {
    match override_path {
        Some(path) => Ok(path),
        None => idl_root
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| CodegenError::MissingParent {
                path: idl_root.to_path_buf(),
            })
            .map(PathBuf::from),
    }
}
