use std::path::PathBuf;

use clap::Args;
use thiserror::Error;

pub use clap::{Parser, Subcommand};

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{0}")]
    Message(String),
}

#[derive(Args, Clone, Debug, Default)]
pub struct Common {
    /// JSON5 config file
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Clone, Debug, Default)]
pub struct Argv {
    pub args: Vec<String>,
}

impl Argv {
    pub fn from_args(args: impl Into<Vec<String>>) -> Self {
        Self { args: args.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::path::PathBuf;

    #[derive(Parser, Debug)]
    struct Probe {
        #[command(flatten)]
        common: Common,
        #[arg(long)]
        moving: bool,
        #[arg(trailing_var_arg = true)]
        rest: Vec<String>,
    }

    #[test]
    fn common_flatten_config_verbose_and_flag() {
        let cli =
            Probe::try_parse_from(["service", "-vv", "--config", "a.json5", "--moving"]).unwrap();
        assert_eq!(cli.common.verbose, 2);
        assert_eq!(
            cli.common.config.as_deref(),
            Some(PathBuf::from("a.json5").as_path())
        );
        assert!(cli.moving);
        assert!(cli.rest.is_empty());
    }
}
