#![expect(
    clippy::pub_use,
    missing_docs,
    reason = "generated from ROS .msg sources"
)]
pub mod empty;
pub use empty::*;
pub mod header;
pub use header::*;
