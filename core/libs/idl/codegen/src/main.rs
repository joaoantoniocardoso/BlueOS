//! Regenerates committed `blueos-idl` Rust output from ROS 2 `.msg` sources.

use std::env;
use std::path::PathBuf;

use blueos_idl_codegen::generate;

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let write_committed = arguments.iter().any(|argument| argument == "--write");
    let output = argument_value(&arguments, "--output").map(PathBuf::from);

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let idl_root = manifest_dir.parent().expect("idl crate root");
    let interfaces_root = idl_root.join("interfaces");

    let out_dir = match (write_committed, output) {
        (true, None) => idl_root.join("src/generated"),
        (_, Some(path)) => path,
        (false, None) => {
            eprintln!("usage: blueos-idl-codegen --write");
            eprintln!("       blueos-idl-codegen --output <dir>");
            std::process::exit(1);
        }
    };

    generate(&interfaces_root, &out_dir);
}

fn argument_value(arguments: &[String], flag: &str) -> Option<String> {
    arguments
        .windows(2)
        .find_map(|window| (window[0] == flag).then(|| window[1].clone()))
}
