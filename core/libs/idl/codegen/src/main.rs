//! Regenerates committed `blueos-idl` Rust output from ROS 2 `.msg` sources.

use std::{env, path::PathBuf};

use blueos_idl_codegen::{generate, generate_catalog, generate_catalog_outputs};

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let write_committed = arguments.iter().any(|argument| argument == "--write");
    let output = argument_value(&arguments, "--output").map(PathBuf::from);
    let typescript_output = argument_value(&arguments, "--typescript-output").map(PathBuf::from);

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let idl_root = manifest_dir.parent().expect("idl crate root");
    let interfaces_root = idl_root.join("interfaces");

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
        return;
    }

    let out_dir = match output {
        Some(path) => path,
        None => {
            eprintln!("usage: blueos-idl-codegen --write");
            eprintln!("       blueos-idl-codegen --output <dir> [--typescript-output <dir>]");
            std::process::exit(1);
        }
    };

    generate(&interfaces_root, &out_dir, typescript_output.as_deref());
    if let Some(typescript_dir) = typescript_output.as_deref() {
        generate_catalog_outputs(idl_root, &out_dir, typescript_dir);
    }
}

fn argument_value(arguments: &[String], flag: &str) -> Option<String> {
    arguments
        .windows(2)
        .find_map(|window| (window[0] == flag).then(|| window[1].clone()))
}
