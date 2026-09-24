use std::ffi::OsString;
use std::path::Path;
use std::process;

fn main() {
    let arguments: Vec<OsString> = std::env::args_os().collect();
    let invocation = arguments
        .first()
        .and_then(|path| Path::new(path).file_name());
    let subcommand = arguments.get(1).map(OsString::as_os_str);
    if !run(invocation, arguments.clone())
        && !run(subcommand, arguments.get(1..).unwrap_or_default().to_vec())
    {
        eprint!("usage: blueos <service> [args...]\n\nservices:");
        #[cfg(feature = "autopilot")]
        eprint!(" autopilot");
        #[cfg(feature = "calibration")]
        eprint!(" calibration");
        eprintln!();
        process::exit(2);
    }
}

fn run(name: Option<&std::ffi::OsStr>, arguments: Vec<OsString>) -> bool {
    match name.and_then(|name| name.to_str()) {
        #[cfg(feature = "autopilot")]
        Some("autopilot") => autopilot::run(arguments),
        #[cfg(feature = "calibration")]
        Some("calibration") => calibration::run(arguments),
        _ => return false,
    }
    true
}
