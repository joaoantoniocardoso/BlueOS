//! CDR vector fixture types and builders for integration tests.

mod encode;
mod extra;
mod fixture;

pub(crate) use encode::encode_json_message;
pub(crate) use extra::extra_vectors;
pub(crate) use fixture::{
    CdrVector, CdrVectorsFile, decode_hex, encode_hex, interfaces_root, vectors_path,
};
