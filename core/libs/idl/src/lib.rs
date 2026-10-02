//! ROS 2 `.msg` types for BlueOS with embedded `ros2msg` schemas and a `#![no_std]` CDR codec (D-05).

#![no_std]
#![expect(
    clippy::pub_use,
    reason = "blueos-idl re-exports generated message modules and shared traits"
)]

extern crate alloc;

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
