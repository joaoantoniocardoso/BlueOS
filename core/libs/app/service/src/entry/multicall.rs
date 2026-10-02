//! Multicall `blueos` messages when no Service is selected.

use std::process::ExitCode;

/// Prints multicall usage and lists `known` service names.
pub fn usage(known: &[&str]) -> ExitCode {
    eprint!("usage: blueos <service> [args...]\n\nservices:");
    for name in known {
        eprint!(" {name}");
    }
    eprintln!();
    ExitCode::from(2)
}

/// Prints multicall `--help` and exits successfully.
pub fn multicall_help(known: &[&str]) -> ExitCode {
    eprintln!("BlueOS service multicall binary.");
    eprint!("usage: blueos <service> [args...]\n\nservices:");
    for name in known {
        eprint!(" {name}");
    }
    eprintln!();
    ExitCode::SUCCESS
}

/// Prints the multicall binary version and exits successfully.
pub fn multicall_version(version: &str) -> ExitCode {
    println!("blueos {version}");
    ExitCode::SUCCESS
}

/// A known service name that was not compiled into this binary.
pub fn missing_feature(service: &str) -> ExitCode {
    eprintln!(
        "blueos: service `{service}` is not compiled into this binary; rebuild with the right --features"
    );
    ExitCode::from(2)
}
