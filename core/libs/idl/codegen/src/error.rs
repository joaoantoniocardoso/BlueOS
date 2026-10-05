//! Errors while reading interfaces or writing generated IDL artifacts.

use std::{io, path::PathBuf};

/// Why committed IDL output could not be regenerated.
#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    /// A path could not be read or written.
    #[error("{path}: {source}")]
    Io {
        /// The path involved.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: io::Error,
    },
    /// ROS interface sources did not parse.
    #[error("failed to parse interfaces: {0}")]
    ParseInterfaces(String),
    /// Generated Rust did not parse as valid syntax.
    #[error("generated Rust at {path} did not parse: {reason}")]
    InvalidGeneratedRust {
        /// The output file.
        path: PathBuf,
        /// Why `syn` rejected it.
        reason: String,
    },
    /// `rustfmt` failed on generated output.
    #[error("rustfmt failed for {path}")]
    Rustfmt {
        /// The file `rustfmt` rejected.
        path: PathBuf,
    },
    /// A `.msg` constant or field uses an unsupported ROS type name.
    #[error("unsupported ROS type {type_name}")]
    UnsupportedRosType {
        /// The type name from the `.msg` file.
        type_name: String,
    },
    /// A constant literal does not match its declared type.
    #[error("invalid {type_name} constant `{literal}`")]
    InvalidConstantLiteral {
        /// The ROS type of the constant.
        type_name: String,
        /// The literal text from the `.msg` file.
        literal: String,
    },
    /// Message dependency graph has a cycle.
    #[error("cycle in message dependencies at {schema_name}")]
    DependencyCycle {
        /// The schema name where the cycle was detected.
        schema_name: String,
    },
    /// An interface directory has no parent path (internal layout assumption).
    #[error("interfaces directory has no parent: {path}")]
    MissingParent {
        /// The interfaces root.
        path: PathBuf,
    },
    /// Endpoint manifest generation failed.
    #[error(transparent)]
    Endpoints(#[from] crate::endpoints::EndpointsError),
    /// Invalid CLI flags or committed endpoint output is stale.
    #[error("{0}")]
    Cli(String),
}

impl CodegenError {
    /// Wraps an I/O failure at `path`.
    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }
}
