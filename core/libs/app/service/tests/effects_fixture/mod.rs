//! Fixtures for `effects` integration tests.

#![expect(
    unreachable_pub,
    reason = "fixture items are re-exported to integration test roots"
)]
#![expect(
    dead_code,
    reason = "fixture items are shared across sibling integration test binaries"
)]

pub mod domain;
pub mod helpers;
pub mod service;
