//! Multicall binary: `blueos <service>` or a symlink named after the service.

use std::{ffi::OsString, process::ExitCode};

use blueos_service::entry::{missing_feature, multicall_help, multicall_version, resolve, usage};

/// Service names that are always recognized, even when their feature is off.
const KNOWN: &[&str] = &["example", "recorder"];

fn main() -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().collect();
    if multicall_reserved(&arguments) {
        return handle_multicall_reserved(&arguments);
    }
    let Some((name, service_arguments)) = resolve(&arguments) else {
        return usage(KNOWN);
    };
    #[cfg(not(any(feature = "example", feature = "recorder")))]
    drop(service_arguments);
    match name.as_str() {
        #[cfg(feature = "example")]
        "example" => blueos_service::entry::run::<blueos_example_app::service::ExampleService>(
            service_arguments,
        ),
        #[cfg(feature = "recorder")]
        "recorder" => {
            blueos_service::entry::run::<blueos_recorder_app::RecorderService>(service_arguments)
        }
        other if KNOWN.contains(&other) => missing_feature(other),
        _ => usage(KNOWN),
    }
}

fn multicall_reserved(arguments: &[OsString]) -> bool {
    arguments.get(1).is_some_and(|argument| {
        matches!(
            argument.to_string_lossy().as_ref(),
            "-h" | "--help" | "--version"
        )
    })
}

fn handle_multicall_reserved(arguments: &[OsString]) -> ExitCode {
    match arguments.get(1).map(|argument| argument.to_string_lossy()) {
        Some(text) if text == "--version" => multicall_version(env!("CARGO_PKG_VERSION")),
        _ => multicall_help(KNOWN),
    }
}
