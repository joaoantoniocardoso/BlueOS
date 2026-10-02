//! Regenerates committed `blueos-idl` Rust output from ROS 2 `.msg` sources, and every Service's endpoint
//! registration from its endpoint manifest.

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use blueos_idl_codegen::{
    endpoints::{GeneratedFile, generate_all},
    generate, generate_catalog, generate_catalog_outputs, message_schema_names,
};

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let write_committed = arguments.iter().any(|argument| argument == "--write");
    let check_endpoints = arguments
        .iter()
        .any(|argument| argument == "--check-endpoints");
    let output = argument_value(&arguments, "--output").map(PathBuf::from);
    let typescript_output = argument_value(&arguments, "--typescript-output").map(PathBuf::from);

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let idl_root = manifest_dir.parent().expect("idl crate root");
    let interfaces_root = idl_root.join("interfaces");
    let core_dir = idl_root
        .parent()
        .and_then(Path::parent)
        .expect("core directory");

    if check_endpoints {
        let stale: Vec<GeneratedFile> = endpoint_files(core_dir, &interfaces_root)
            .into_iter()
            .filter(|file| fs::read_to_string(&file.path).ok().as_ref() != Some(&file.contents))
            .collect();
        for file in &stale {
            eprintln!("{} differs from its endpoint manifest", file.path.display());
        }
        if !stale.is_empty() {
            std::process::exit(1);
        }
        return;
    }

    if write_committed {
        if output.is_some() || typescript_output.is_some() {
            eprintln!("--write cannot be combined with --output or --typescript-output");
            std::process::exit(1);
        }
        generate(
            &interfaces_root,
            &idl_root.join("src/generated"),
            Some(&idl_root.join("typescript")),
        );
        generate_catalog(idl_root);
        for file in endpoint_files(core_dir, &interfaces_root) {
            fs::write(&file.path, file.contents).expect("write generated endpoints");
        }
        return;
    }

    let out_dir = match output {
        Some(path) => path,
        None => {
            eprintln!("usage: blueos-idl-codegen --write");
            eprintln!("       blueos-idl-codegen --check-endpoints");
            eprintln!("       blueos-idl-codegen --output <dir> [--typescript-output <dir>]");
            std::process::exit(1);
        }
    };

    generate(&interfaces_root, &out_dir, typescript_output.as_deref());
    if let Some(typescript_dir) = typescript_output.as_deref() {
        generate_catalog_outputs(idl_root, &out_dir, typescript_dir);
    }
}

fn endpoint_files(core_dir: &Path, interfaces_root: &Path) -> Vec<GeneratedFile> {
    generate_all(core_dir, &message_schema_names(interfaces_root)).unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(1);
    })
}

fn argument_value(arguments: &[String], flag: &str) -> Option<String> {
    arguments
        .windows(2)
        .find_map(|window| (window[0] == flag).then(|| window[1].clone()))
}
