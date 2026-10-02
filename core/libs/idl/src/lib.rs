//! The BlueOS Messages as Rust types, with their ROS 2 schemas and the CDR codec that reads and writes them.

#![no_std]
#![expect(
    clippy::pub_use,
    reason = "blueos-idl re-exports generated message modules and shared traits"
)]

extern crate alloc;

#[cfg(feature = "catalog")]
pub mod catalog;
/// CDR reader and writer used by generated message codecs.
pub mod cdr;
/// Zenoh `application/cdr;<schema_name>` encoding helpers.
pub mod encoding;
/// Errors returned by the CDR codec.
pub mod error;
/// Traits shared by every generated message type.
pub mod message;

mod generated;

pub use error::Error;
pub use generated::{msg, schema};
pub use message::Message;
