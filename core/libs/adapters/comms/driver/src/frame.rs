use crate::{CommsError, Frame, Result};

/// Wire format: little-endian `u32` correlation id, then raw payload bytes. No protobuf.
impl Frame {
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(4 + self.payload.len());
        out.extend_from_slice(&self.correlation.to_le_bytes());
        out.extend_from_slice(&self.payload);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() < 4 {
            return Err(CommsError::Message(
                "frame shorter than 4-byte correlation header".into(),
            ));
        }
        let correlation = u32::from_le_bytes(bytes[..4].try_into().expect("len checked"));
        Ok(Self {
            correlation,
            payload: bytes[4..].to_vec(),
        })
    }
}
