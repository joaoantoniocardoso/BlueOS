//! Multicall binary acceptance tests.

use std::process::Command;

#[test]
fn compiled_out_example_prints_one_clear_message() {
    let output = Command::new(env!("CARGO_BIN_EXE_blueos"))
        .arg("example")
        .output()
        .expect("run blueos");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not compiled into this binary"),
        "stderr was: {stderr}"
    );
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn multicall_help_and_version_exit_success() {
    let help = Command::new(env!("CARGO_BIN_EXE_blueos"))
        .arg("--help")
        .output()
        .expect("help");
    assert!(help.status.success());

    let version = Command::new(env!("CARGO_BIN_EXE_blueos"))
        .arg("--version")
        .output()
        .expect("version");
    assert!(version.status.success());
    let stdout = String::from_utf8_lossy(&version.stdout);
    assert!(stdout.starts_with("blueos "));
}
