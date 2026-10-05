//! Write generated endpoint files for every Service in the workspace.

use std::{fs, path::Path};

use alloc::collections::BTreeSet;

use super::{
    super::{
        EndpointsError, GENERATED_NOTICE, GeneratedFile, MANIFEST_FILE, TYPESCRIPT_DIR,
        TYPESCRIPT_GENERATED_NOTICE,
        discover::{format, workspace},
    },
    manifest::generate,
};

/// Generates the endpoint files of every Service in the workspace at `core_dir`.
pub fn generate_all(
    core_dir: &Path,
    messages: &BTreeSet<String>,
) -> Result<Vec<GeneratedFile>, EndpointsError> {
    let typescript_dir = core_dir.join(TYPESCRIPT_DIR);
    fs::create_dir_all(&typescript_dir).map_err(|error| EndpointsError::Read {
        path: typescript_dir.clone(),
        error,
    })?;
    let mut files = Vec::new();
    for member in workspace::workspace_members(core_dir)? {
        files.extend(generate_service_files(
            core_dir,
            &member,
            messages,
            &typescript_dir,
        )?);
    }
    Ok(files)
}

fn generate_service_files(
    core_dir: &Path,
    member: &str,
    messages: &BTreeSet<String>,
    typescript_dir: &Path,
) -> Result<Vec<GeneratedFile>, EndpointsError> {
    let app_dir = core_dir.join(member);
    let manifest_path = app_dir.join(MANIFEST_FILE);
    if !manifest_path.exists() {
        return Ok(Vec::new());
    }
    let api_dir = app_dir.with_file_name("logic").join("api");
    let api_cargo = api_dir.join("Cargo.toml");
    let api_crate = workspace::read_toml(&api_cargo)?
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| EndpointsError::Cargo {
            path: api_cargo,
            reason: "no `package.name`".to_owned(),
        })?
        .replace('-', "_");
    let generated =
        generate(&workspace::read(&manifest_path)?, &api_crate, messages).map_err(|error| {
            EndpointsError::Manifest {
                path: manifest_path,
                error,
            }
        })?;
    let typescript_path = typescript_dir.join(format!("{}.ts", generated.service));
    let mut files = Vec::new();
    for (path, source, format_rust) in [
        (api_dir.join("src/endpoints.rs"), generated.api, true),
        (app_dir.join("src/endpoints.rs"), generated.app, true),
        (typescript_path, generated.typescript, false),
    ] {
        files.push(GeneratedFile {
            path,
            contents: if format_rust {
                format::format(&source)?
            } else {
                source
            },
        });
    }
    Ok(files)
}

/// Paths of generated endpoint files that no longer have a manifest.
pub fn stray_files(
    core_dir: &Path,
    generated: &[GeneratedFile],
) -> Result<Vec<std::path::PathBuf>, EndpointsError> {
    let clients = fs::read_dir(core_dir.join(TYPESCRIPT_DIR))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path());
    let wiring = workspace::workspace_members(core_dir)?
        .into_iter()
        .map(|member| core_dir.join(member).join("src/endpoints.rs"));
    let mut stray = Vec::new();
    for path in clients.chain(wiring) {
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(EndpointsError::Read { path, error }),
        };
        let is_generated =
            contents.contains(GENERATED_NOTICE) || contents.contains(TYPESCRIPT_GENERATED_NOTICE);
        if is_generated && !generated.iter().any(|file| file.path == path) {
            stray.push(path);
        }
    }
    stray.sort();
    Ok(stray)
}
