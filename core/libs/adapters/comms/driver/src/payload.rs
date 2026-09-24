use std::fmt;
use std::sync::Arc;

use bytes::Bytes;

/// Cheap-clone byte payload. Channel drivers store [`Bytes`]; the Zenoh driver may hold
/// non-contiguous shared-memory slices behind the same type.
///
/// Reading a contiguous slice via [`as_slice`](Self::as_slice) may copy when the backing
/// storage is not a single buffer (typical for Zenoh SHM payloads).
pub struct Payload {
    inner: Arc<dyn PayloadStorage + Send + Sync>,
}

pub trait PayloadStorage {
    fn contiguous(&self) -> Option<&[u8]>;
    fn to_vec(&self) -> Vec<u8>;
}

struct BytesStorage(Bytes);

impl PayloadStorage for BytesStorage {
    fn contiguous(&self) -> Option<&[u8]> {
        Some(self.0.as_ref())
    }

    fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Payload {
    pub fn empty() -> Self {
        Self::from_bytes(Bytes::new())
    }

    pub fn from_bytes(bytes: Bytes) -> Self {
        Self {
            inner: Arc::new(BytesStorage(bytes)),
        }
    }

    pub fn from_static(data: &'static [u8]) -> Self {
        Self::from_bytes(Bytes::from_static(data))
    }

    pub fn from_storage(storage: Arc<dyn PayloadStorage + Send + Sync>) -> Self {
        Self { inner: storage }
    }

    /// Returns a contiguous copy of the payload bytes (always allocates).
    pub fn as_slice(&self) -> Vec<u8> {
        match self.inner.contiguous() {
            Some(slice) => slice.to_vec(),
            None => self.inner.to_vec(),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.inner.to_vec()
    }

    pub fn into_bytes(self) -> Bytes {
        match self.inner.contiguous() {
            Some(slice) => Bytes::copy_from_slice(slice),
            None => Bytes::from(self.inner.to_vec()),
        }
    }
}

impl Clone for Payload {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl fmt::Debug for Payload {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Payload")
            .field("len", &self.to_vec().len())
            .finish()
    }
}

impl PartialEq for Payload {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for Payload {}
