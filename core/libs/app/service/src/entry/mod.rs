//! Multicall entry: resolve a service name, parse the common CLI, and run one [`Service`].

mod common;
mod multicall;
mod parse;
mod resolve;
mod run;
mod verbosity;

pub use common::CommonArguments;
pub use multicall::{missing_feature, multicall_help, multicall_version, usage};
pub use parse::{ParseError, ParsedServiceArguments, parse_service_cli};
pub use resolve::resolve;
pub use run::run;
pub use verbosity::verbosity_from_raw;
