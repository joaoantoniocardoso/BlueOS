//! Shared proptest strategies for CDR round-trip properties.

use core::fmt::Debug;

use proptest::{collection::vec, prelude::*};

const MAX_STRING_LEN: usize = 128;
const MAX_SEQUENCE_LEN: usize = 16;

/// Bounded ROS string payloads for property tests.
pub(crate) fn bounded_string() -> impl Strategy<Value = String> {
    vec(any::<char>(), 0..MAX_STRING_LEN).prop_map(|characters| characters.into_iter().collect())
}

/// Bounded `uint8[]` sequences.
pub(crate) fn bounded_bytes() -> impl Strategy<Value = Vec<u8>> {
    vec(any::<u8>(), 0..MAX_SEQUENCE_LEN)
}

/// Bounded homogeneous vectors.
pub(crate) fn bounded_vec<T: Debug>(
    element: impl Strategy<Value = T>,
) -> impl Strategy<Value = Vec<T>> {
    vec(element, 0..MAX_SEQUENCE_LEN)
}
