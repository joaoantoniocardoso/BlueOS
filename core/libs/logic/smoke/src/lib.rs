//! Minimal `no_std` logic crate used to exercise workspace gates (ticket #31).

#![no_std]

extern crate alloc;

/// Returns one so callers can assert the crate links.
pub const ANSWER: u8 = 1;
