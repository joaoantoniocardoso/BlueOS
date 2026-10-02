//! Resolve which Service name to run from multicall or symlink invocation.

use std::{ffi::OsString, path::Path};

/// Tries symlink invocation (`argv[0]` basename), then subcommand invocation (`argv[1]`).
///
/// On success, returns the service name and arguments for [`parse_service_cli`]: the first element is always the
/// service name, followed by the service's flags.
pub fn resolve(arguments: &[OsString]) -> Option<(String, Vec<OsString>)> {
    if arguments.is_empty() {
        return None;
    }
    if let Some(name) = service_name_from_path(arguments.first()?) {
        if arguments.len() < 2 {
            return None;
        }
        let service_arguments = service_argv(&name, &arguments[1..]);
        return Some((name, service_arguments));
    }
    if arguments.len() < 2 {
        return None;
    }
    let subcommand = arguments.get(1)?;
    if is_reserved_multicall_flag(subcommand) {
        return None;
    }
    let name = subcommand.to_string_lossy().into_owned();
    let service_arguments = service_argv(&name, &arguments[2..]);
    Some((name, service_arguments))
}

fn service_argv(name: &str, tail: &[OsString]) -> Vec<OsString> {
    let mut service_arguments = Vec::with_capacity(1 + tail.len());
    service_arguments.push(OsString::from(name));
    service_arguments.extend(tail.iter().cloned());
    service_arguments
}

fn service_name_from_path(program: &OsString) -> Option<String> {
    let file_name = Path::new(program).file_name()?;
    let name = file_name.to_string_lossy();
    if name.is_empty() || name == "blueos" {
        return None;
    }
    Some(name.into_owned())
}

fn is_reserved_multicall_flag(argument: &OsString) -> bool {
    matches!(
        argument.to_string_lossy().as_ref(),
        "-h" | "--help" | "--version"
    )
}
