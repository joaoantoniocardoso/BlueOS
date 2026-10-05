//! Command-line entry for `blueos-idl-codegen`.

use std::{env, fs, path::PathBuf};

use crate::{
    endpoints::{GeneratedFile, generate_all, stray_files},
    error::CodegenError,
    generate, generate_catalog, generate_catalog_outputs, message_schema_names,
    paths::{core_dir, idl_root},
};

/// Runs the `blueos-idl-codegen` CLI from `arguments` (including argv[0]).
pub fn run(arguments: &[String]) -> Result<(), CodegenError> {
    let write_committed = arguments.iter().any(|argument| argument == "--write");
    let check_endpoints = arguments
        .iter()
        .any(|argument| argument == "--check-endpoints");
    let output = flag_value(arguments, "--output").map(PathBuf::from);
    let typescript_output = flag_value(arguments, "--typescript-output").map(PathBuf::from);
    let test_generated_output = flag_value(arguments, "--test-generated-output").map(PathBuf::from);

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let idl_root = idl_root(
        &manifest_dir,
        flag_value(arguments, "--idl-root").map(PathBuf::from),
    )?;
    let interfaces_root = idl_root.join("interfaces");
    let core_directory = core_dir(
        &idl_root,
        flag_value(arguments, "--core-dir").map(PathBuf::from),
    )?;

    if check_endpoints {
        return check_committed_endpoints(&core_directory, &interfaces_root);
    }

    if write_committed {
        if output.is_some() || typescript_output.is_some() {
            return Err(CodegenError::Cli(
                "--write cannot be combined with --output or --typescript-output".to_owned(),
            ));
        }
        write_committed_outputs(&idl_root, &core_directory, &interfaces_root)?;
        return Ok(());
    }

    let out_dir = output.ok_or_else(|| {
        CodegenError::Cli(
            "usage: blueos-idl-codegen --write [--idl-root <dir>] [--core-dir <dir>]\n       \
             blueos-idl-codegen --check-endpoints [--idl-root <dir>] [--core-dir <dir>]\n       \
             blueos-idl-codegen --output <dir> [--typescript-output <dir>] [--idl-root <dir>]"
                .to_owned(),
        )
    })?;

    generate(
        &interfaces_root,
        &out_dir,
        typescript_output.as_deref(),
        test_generated_output.as_deref(),
    )?;
    if let Some(typescript_dir) = typescript_output.as_deref() {
        generate_catalog_outputs(&idl_root, &out_dir, typescript_dir)?;
    }
    Ok(())
}

/// Reads the value after `flag` in `arguments`, if present.
pub fn flag_value(arguments: &[String], flag: &str) -> Option<String> {
    arguments
        .windows(2)
        .find_map(|window| (window[0] == flag).then(|| window[1].clone()))
}

fn endpoint_files(
    core_directory: &std::path::Path,
    interfaces_root: &std::path::Path,
) -> Result<Vec<GeneratedFile>, CodegenError> {
    let messages = message_schema_names(interfaces_root)?;
    Ok(generate_all(core_directory, &messages)?)
}

fn check_committed_endpoints(
    core_directory: &std::path::Path,
    interfaces_root: &std::path::Path,
) -> Result<(), CodegenError> {
    let files = endpoint_files(core_directory, interfaces_root)?;
    let stray = stray_files(core_directory, &files)?;
    let stale: Vec<&GeneratedFile> = files
        .iter()
        .filter(|file| fs::read_to_string(&file.path).ok().as_ref() != Some(&file.contents))
        .collect();
    for file in &stale {
        eprintln!("{} differs from its endpoint manifest", file.path.display());
    }
    for path in &stray {
        eprintln!("{} has no endpoint manifest: delete it", path.display());
    }
    if !stale.is_empty() || !stray.is_empty() {
        return Err(CodegenError::Cli(
            "endpoint files are stale or stray".to_owned(),
        ));
    }
    Ok(())
}

fn write_committed_outputs(
    idl_root: &std::path::Path,
    core_directory: &std::path::Path,
    interfaces_root: &std::path::Path,
) -> Result<(), CodegenError> {
    generate(
        interfaces_root,
        &idl_root.join("src/generated"),
        Some(&idl_root.join("typescript")),
        Some(&idl_root.join("tests/generated")),
    )?;
    generate_catalog(idl_root)?;
    for file in endpoint_files(core_directory, interfaces_root)? {
        let path = file.path;
        fs::write(&path, file.contents).map_err(|source| CodegenError::io(path, source))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_without_output_reports_usage() {
        let arguments = vec!["blueos-idl-codegen".to_owned()];
        assert!(matches!(run(&arguments), Err(CodegenError::Cli(_))));
    }

    #[test]
    fn check_committed_endpoints_matches_committed_files() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let idl_root = idl_root(&manifest_dir, None).expect("idl root");
        let core_directory = core_dir(&idl_root, None).expect("core dir");
        check_committed_endpoints(&core_directory, &idl_root.join("interfaces"))
            .expect("committed endpoints");
    }

    #[test]
    fn write_committed_outputs_rejects_conflicting_flags() {
        let arguments = vec![
            "blueos-idl-codegen".to_owned(),
            "--write".to_owned(),
            "--output".to_owned(),
            "/tmp/out".to_owned(),
        ];
        assert!(matches!(run(&arguments), Err(CodegenError::Cli(_))));
    }
}
