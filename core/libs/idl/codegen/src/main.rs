//! Regenerates committed `blueos-idl` Rust output from ROS 2 `.msg` sources, and every Service's endpoint
//! registration from its endpoint manifest.

use std::env;

fn main() {
    let arguments: Vec<String> = env::args().collect();
    if let Err(error) = blueos_idl_codegen::run_cli(&arguments) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
