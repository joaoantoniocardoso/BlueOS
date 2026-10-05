//! Zero-copy [`Payload`] bridging for Zenoh [`ZBytes`].

use std::borrow::Cow;

use bytes::Bytes;
use zenoh::bytes::ZBytes;

use blueos_comms::{Payload, PayloadBuffer};

/// Wraps received `ZBytes` so they stay refcounted through [`Payload`].
#[derive(Debug)]
struct ZenohBytes(ZBytes);

impl PayloadBuffer for ZenohBytes {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        self.0.to_bytes()
    }

    fn size_bytes(&self) -> usize {
        self.0.len()
    }
}

/// Builds a [`Payload`] from Zenoh bytes without copying contiguous storage.
pub(crate) fn payload_from_zbytes(bytes: ZBytes) -> Payload {
    Payload::new(ZenohBytes(bytes))
}

/// Sends a [`Payload`] on Zenoh, reusing an inner [`Bytes`] buffer when present.
pub(crate) fn zbytes_from_payload(payload: &Payload) -> ZBytes {
    if let Some(bytes) = payload.downcast_ref::<Bytes>() {
        ZBytes::from(bytes.clone())
    } else {
        ZBytes::from(payload.to_bytes().into_owned())
    }
}
