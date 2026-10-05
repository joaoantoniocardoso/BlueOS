//! Multicall binary: `blueos <service>` or a symlink named after the service.

mod multicall;

use std::{ffi::OsString, process::ExitCode};

use blueos_service::entry::{missing_feature, resolve, usage};

use crate::multicall::{handle_multicall_reserved, multicall_reserved};

/// Service names that are always recognized, even when their feature is off.
const KNOWN: &[&str] = &["example", "recorder"];

fn main() -> ExitCode {
    let arguments: Vec<OsString> = std::env::args_os().collect();
    if multicall_reserved(&arguments) {
        return handle_multicall_reserved(&arguments, KNOWN, env!("CARGO_PKG_VERSION"));
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
