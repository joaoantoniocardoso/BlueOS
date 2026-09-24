use std::sync::Arc;

use blueos_comms_driver::{Payload, PayloadStorage};
use zenoh::bytes::ZBytes;

struct ZBytesStorage(ZBytes);

impl PayloadStorage for ZBytesStorage {
    fn contiguous(&self) -> Option<&[u8]> {
        match self.0.to_bytes() {
            std::borrow::Cow::Borrowed(slice) => Some(slice),
            std::borrow::Cow::Owned(_) => None,
        }
    }

    fn to_vec(&self) -> Vec<u8> {
        self.0.to_bytes().into_owned()
    }
}

pub fn payload_from_zbytes(bytes: ZBytes) -> Payload {
    Payload::from_storage(Arc::new(ZBytesStorage(bytes)))
}

pub fn zbytes_from_payload(payload: Payload) -> ZBytes {
    ZBytes::from(payload.into_bytes())
}
