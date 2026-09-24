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
        #[cfg(feature = "example")]
        eprint!(" example");
        #[cfg(feature = "recorder")]
        eprint!(" recorder");
        eprintln!();
        process::exit(2);
    }
}

fn run(name: Option<&std::ffi::OsStr>, arguments: Vec<OsString>) -> bool {
    match name.and_then(|name| name.to_str()) {
        Some("example") => dispatch_example(arguments),
        Some("recorder") => dispatch_recorder(arguments),
        _ => false,
    }
}

#[cfg(feature = "example")]
fn dispatch_example(arguments: Vec<OsString>) -> bool {
    example::run(arguments);
    true
}

#[cfg(not(feature = "example"))]
fn dispatch_example(arguments: Vec<OsString>) -> bool {
    let _ = arguments;
    eprintln!("blueos: rebuild with --features example");
    false
}

#[cfg(feature = "recorder")]
fn dispatch_recorder(arguments: Vec<OsString>) -> bool {
    if recorder::run(arguments) != std::process::ExitCode::SUCCESS {
        process::exit(1);
    }
    true
}

#[cfg(not(feature = "recorder"))]
fn dispatch_recorder(arguments: Vec<OsString>) -> bool {
    let _ = arguments;
    eprintln!("blueos: rebuild with --features recorder");
    false
}
