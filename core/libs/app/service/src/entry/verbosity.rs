//! Read `-v` / `--verbose` from raw arguments before clap runs.

use std::ffi::OsString;

fn count_short_verbose_flags(text: &str) -> u8 {
    if !text.starts_with('-') || text.starts_with("--") {
        return 0;
    }
    text.chars()
        .skip(1)
        .filter(|character| *character == 'v')
        .count()
        .min(u8::MAX as usize) as u8
}

fn verbosity_from_argument(text: &str) -> Option<u8> {
    if text == "--verbose" || text == "-v" {
        return Some(1);
    }
    if text.starts_with("--verbose=") {
        return None;
    }
    let short = count_short_verbose_flags(text);
    if short > 0 { Some(short) } else { None }
}

/// Counts `-v`, `-vv`, and `--verbose` in `arguments` (including the program name).
pub fn verbosity_from_raw(arguments: &[OsString]) -> u8 {
    arguments
        .iter()
        .skip(1)
        .map(|argument| argument.to_string_lossy())
        .filter_map(|text| verbosity_from_argument(text.as_ref()))
        .fold(0u8, |count, added| count.saturating_add(added))
}
