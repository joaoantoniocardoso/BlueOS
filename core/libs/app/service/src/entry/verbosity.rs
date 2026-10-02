//! Read `-v` / `--verbose` from raw arguments before clap runs.

use std::ffi::OsString;

/// Counts `-v`, `-vv`, and `--verbose` in `arguments` (including the program name).
pub fn verbosity_from_raw(arguments: &[OsString]) -> u8 {
    let mut count = 0u8;
    for argument in arguments.iter().skip(1) {
        let text = argument.to_string_lossy();
        if text == "--verbose" {
            count = count.saturating_add(1);
            continue;
        }
        if text.starts_with("--verbose=") {
            continue;
        }
        if text == "-v" {
            count = count.saturating_add(1);
            continue;
        }
        if text.starts_with('-') && !text.starts_with("--") {
            for character in text.chars().skip(1) {
                if character == 'v' {
                    count = count.saturating_add(1);
                }
            }
        }
    }
    count
}
