//! Regenerates vendored catalog schema lookup (`schema_catalog.rs`, `catalog.ts`).

use std::{env, path::PathBuf};

use blueos_idl_codegen::{generate_catalog_outputs, write_typescript_index};

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let idl_root = argument_value(&arguments, "--idl-root")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.parent().expect("idl crate root").to_path_buf());
    let generated_output = argument_value(&arguments, "--generated-output")
        .map(PathBuf::from)
        .unwrap_or_else(|| idl_root.join("src/generated"));
    let typescript_output = argument_value(&arguments, "--typescript-output")
        .map(PathBuf::from)
        .unwrap_or_else(|| idl_root.join("typescript"));
    generate_catalog_outputs(&idl_root, &generated_output, &typescript_output);
    write_typescript_index(&typescript_output);
}

fn argument_value(arguments: &[String], flag: &str) -> Option<String> {
    arguments
        .windows(2)
        .find_map(|window| (window[0] == flag).then(|| window[1].clone()))
}
