//! Reserved multicall flags (`--help`, `--version`).

use std::{ffi::OsString, process::ExitCode};

use blueos_service::entry::{multicall_help, multicall_version};

/// Returns true when `arguments` ask for help or version instead of a service name.
pub(crate) fn multicall_reserved(arguments: &[OsString]) -> bool {
    arguments.get(1).is_some_and(|argument| {
        matches!(
            argument.to_string_lossy().as_ref(),
            "-h" | "--help" | "--version"
        )
    })
}

/// Handles `--help` and `--version` for the multicall binary.
pub(crate) fn handle_multicall_reserved(
    arguments: &[OsString],
    known_services: &[&str],
    package_version: &str,
) -> ExitCode {
    match arguments.get(1).map(|argument| argument.to_string_lossy()) {
        Some(text) if text == "--version" => multicall_version(package_version),
        _ => multicall_help(known_services),
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;

    use super::{handle_multicall_reserved, multicall_reserved};

    #[test]
    fn reserved_flags_are_detected() {
        assert!(multicall_reserved(&[
            OsString::from("blueos"),
            OsString::from("--help"),
        ]));
        assert!(multicall_reserved(&[
            OsString::from("blueos"),
            OsString::from("-h"),
        ]));
        assert!(multicall_reserved(&[
            OsString::from("blueos"),
            OsString::from("--version"),
        ]));
        assert!(!multicall_reserved(&[
            OsString::from("blueos"),
            OsString::from("example"),
        ]));
    }

    #[test]
    fn version_and_help_exit_success() {
        let known = &["example"];
        let version = handle_multicall_reserved(
            &[OsString::from("blueos"), OsString::from("--version")],
            known,
            "9.9.9-test",
        );
        assert_eq!(version, std::process::ExitCode::SUCCESS);

        let help = handle_multicall_reserved(
            &[OsString::from("blueos"), OsString::from("--help")],
            known,
            "9.9.9-test",
        );
        assert_eq!(help, std::process::ExitCode::SUCCESS);
    }
}
