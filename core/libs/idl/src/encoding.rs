//! Zenoh encoding strings for IDL payloads.

use alloc::{format, string::String};

/// Base CDR media type without a schema suffix.
pub const ENCODING_APPLICATION_CDR: &str = "application/cdr";

/// Builds `application/cdr;<schema_name>` for Zenoh and MCAP channel descriptors.
pub fn cdr_encoding(schema_name: &str) -> String {
    format!("{ENCODING_APPLICATION_CDR};{schema_name}")
}
