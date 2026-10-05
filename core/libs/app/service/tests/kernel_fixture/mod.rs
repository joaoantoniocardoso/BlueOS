//! Tank kernel fixtures shared by integration tests.

#![expect(
    clippy::pub_use,
    reason = "fixture module re-exports tank helpers to integration tests via glob import"
)]
#![expect(
    dead_code,
    reason = "each integration test binary uses a subset of the shared fixtures"
)]
#![expect(
    unreachable_pub,
    reason = "fixture items are re-exported to integration test binaries via glob import"
)]

mod alt_services;
mod backends;
mod tank;

pub use tank::*;
