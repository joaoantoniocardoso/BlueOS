//! Shared types and hex helpers for CDR vector tests.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CdrVector {
    pub schema_name: String,
    pub hex: String,
    pub decoded: serde_json::Value,
    pub category: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub skip_encode_round_trip: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CdrVectorsFile {
    pub vectors: Vec<CdrVector>,
}

fn is_false(value: &bool) -> bool {
    !*value
}

pub(crate) fn encode_hex(payload: &[u8]) -> String {
    payload.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub(crate) fn decode_hex(hex: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(hex.len() / 2);
    let mut index = 0;
    while index < hex.len() {
        let pair = hex.get(index..index + 2).expect("hex pair");
        bytes.push(u8::from_str_radix(pair, 16).expect("hex digit"));
        index += 2;
    }
    bytes
}

pub(crate) fn interfaces_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("interfaces")
}

pub(crate) fn vectors_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors/cdr.json")
}
