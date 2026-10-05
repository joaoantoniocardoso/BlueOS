//! Parse the common CLI together with a Service's own arguments.

use std::{ffi::OsString, process::ExitCode};

use clap::{Args, CommandFactory, FromArgMatches, Parser};

use crate::Service;

use super::common::CommonArguments;

/// The common CLI and a Service's own arguments, parsed together.
#[derive(Parser, Clone, Debug)]
struct ServiceCli<Arguments: Args> {
    #[command(flatten)]
    common: CommonArguments,
    #[command(flatten)]
    arguments: Arguments,
}

/// Why parsing stopped before a Service could start.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// `--help` or `-h`.
    Help,
    /// `--version`.
    Version,
    /// Any other parse failure.
    Usage,
}

/// Parsed command-line arguments for one Service.
#[derive(Clone, Debug)]
pub struct ParsedServiceArguments<Arguments> {
    /// Arguments shared by every Service.
    pub common: CommonArguments,
    /// The Service's own arguments.
    pub service: Arguments,
}

impl ParseError {
    /// The process exit code that matches this outcome.
    pub fn exit_code(self) -> ExitCode {
        match self {
            ParseError::Help | ParseError::Version => ExitCode::SUCCESS,
            ParseError::Usage => ExitCode::from(2),
        }
    }
}

/// Parses `arguments` for `S`, flattening [`CommonArguments`] with `S::Arguments`.
///
/// # Errors
///
/// [`ParseError::Help`] or [`ParseError::Version`] when clap would print help or version and exit.
/// [`ParseError::Usage`] for any other failure.
pub fn parse_service_cli<S: Service>(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<ParsedServiceArguments<S::Arguments>, ParseError> {
    parse_service_cli_collected::<S>(arguments.into_iter().collect())
}

fn parse_service_cli_collected<S: Service>(
    arguments: Vec<OsString>,
) -> Result<ParsedServiceArguments<S::Arguments>, ParseError> {
    match reserved_service_flag(&arguments) {
        Some(stop) => {
            emit_reserved_flag_outcome::<S>(stop);
            Err(stop)
        }
        None => parse_service_cli_matches::<S>(arguments),
    }
}

fn emit_reserved_flag_outcome<S: Service>(stop: ParseError) {
    match stop {
        ParseError::Help => print_service_help::<S>(),
        ParseError::Version => print_service_version::<S>(),
        ParseError::Usage => {}
    }
}

fn print_service_help<S: Service>() {
    let mut command = ServiceCli::<S::Arguments>::command()
        .name(S::NAME)
        .version(S::VERSION);
    let _ = command.print_help();
    println!();
}

fn print_service_version<S: Service>() {
    println!("{} {}", S::NAME, S::VERSION);
}

fn parse_service_cli_matches<S: Service>(
    arguments: Vec<OsString>,
) -> Result<ParsedServiceArguments<S::Arguments>, ParseError> {
    let command = ServiceCli::<S::Arguments>::command()
        .name(S::NAME)
        .version(S::VERSION);
    let matches = match command.try_get_matches_from(&arguments) {
        Ok(matches) => matches,
        Err(error) => {
            let _ = error.print();
            return Err(ParseError::Usage);
        }
    };
    let parsed = ServiceCli::<S::Arguments>::from_arg_matches(&matches).map_err(|error| {
        let _ = error.print();
        ParseError::Usage
    })?;
    Ok(ParsedServiceArguments {
        common: parsed.common,
        service: parsed.arguments,
    })
}

fn reserved_service_flag(arguments: &[OsString]) -> Option<ParseError> {
    arguments
        .iter()
        .skip(1)
        .find_map(|argument| match argument.to_string_lossy().as_ref() {
            "--help" | "-h" => Some(ParseError::Help),
            "--version" => Some(ParseError::Version),
            _ => None,
        })
}
