//! Run `rustfmt` on generated endpoint Rust.

use std::{
    io::Write as _,
    process::{Command, Output, Stdio},
};

use crate::endpoints::EndpointsError;

/// Runs `rustfmt` on generated endpoint Rust source text.
pub fn format(source: &str) -> Result<String, EndpointsError> {
    let output = run_rustfmt_subprocess(source)?;
    decode_rustfmt_stdout(source, output)
}

fn run_rustfmt_subprocess(source: &str) -> Result<Output, EndpointsError> {
    let mut rustfmt = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|error| EndpointsError::Rustfmt {
            reason: error.to_string(),
        })?;
    write_rustfmt_stdin(&mut rustfmt, source)?;
    rustfmt
        .wait_with_output()
        .map_err(|error| EndpointsError::Rustfmt {
            reason: error.to_string(),
        })
}

fn write_rustfmt_stdin(
    rustfmt: &mut std::process::Child,
    source: &str,
) -> Result<(), EndpointsError> {
    let mut stdin = rustfmt
        .stdin
        .take()
        .ok_or_else(|| EndpointsError::Rustfmt {
            reason: "rustfmt stdin unavailable".to_owned(),
        })?;
    stdin
        .write_all(source.as_bytes())
        .map_err(|error| EndpointsError::Rustfmt {
            reason: error.to_string(),
        })
}

fn decode_rustfmt_stdout(source: &str, output: Output) -> Result<String, EndpointsError> {
    ensure_rustfmt_success(source, &output)?;
    decode_stdout_utf8(&output.stdout)
}

fn ensure_rustfmt_success(source: &str, output: &Output) -> Result<(), EndpointsError> {
    if output.status.success() {
        Ok(())
    } else {
        Err(EndpointsError::Rustfmt {
            reason: rustfmt_failure_message(source, &output.stderr),
        })
    }
}

fn decode_stdout_utf8(stdout: &[u8]) -> Result<String, EndpointsError> {
    String::from_utf8(stdout.to_vec()).map_err(|error| EndpointsError::Rustfmt {
        reason: error.to_string(),
    })
}

fn rustfmt_failure_message(source: &str, stderr: &[u8]) -> String {
    format!(
        "rustfmt rejected generated endpoints:\n{source}\n{}",
        String::from_utf8_lossy(stderr)
    )
}
