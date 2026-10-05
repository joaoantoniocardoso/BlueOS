//! Integration tests that decode committed CDR hex fixtures for every IDL message.

mod common;

mod cdr_codec_dispatch {
    include!("generated/cdr_codec_dispatch.rs");
}

use std::fs;

use blueos_idl_codegen::collect_messages_for_test;
use common::cdr_vectors::{
    CdrVector, CdrVectorsFile, decode_hex, encode_hex, encode_json_message, extra_vectors,
    interfaces_root, vectors_path,
};

fn build_vectors_file() -> CdrVectorsFile {
    let mut vectors = Vec::new();
    for record in collect_messages_for_test(&interfaces_root()).expect("parse interfaces") {
        let schema_name = record.schema_name;
        let payload = cdr_codec_dispatch::encode_default(&schema_name).expect("encode default");
        let decoded =
            cdr_codec_dispatch::decode_to_json(&schema_name, &payload).expect("decode default");
        vectors.push(CdrVector {
            schema_name,
            hex: encode_hex(&payload),
            decoded,
            category: "default".to_string(),
            skip_encode_round_trip: false,
            layout_note: None,
        });
    }
    vectors.extend(extra_vectors());
    CdrVectorsFile { vectors }
}

fn read_vectors_file() -> CdrVectorsFile {
    let content = fs::read_to_string(vectors_path()).expect("read cdr.json");
    serde_json::from_str(&content).expect("parse cdr.json")
}

#[test]
fn cdr_vectors_match_rust_codec() {
    let updating = std::env::var("BLUEOS_IDL_UPDATE_VECTORS").as_deref() == Ok("1");
    let expected = build_vectors_file();

    if updating {
        fs::create_dir_all(vectors_path().parent().expect("parent")).expect("create vectors dir");
        let content = serde_json::to_string_pretty(&expected).expect("serialize vectors");
        fs::write(vectors_path(), format!("{content}\n")).expect("write cdr.json");
        return;
    }

    let on_disk = read_vectors_file();
    assert_eq!(
        serde_json::to_value(&on_disk).expect("serialize on disk"),
        serde_json::to_value(&expected).expect("serialize expected"),
        "cdr.json drift; run BLUEOS_IDL_UPDATE_VECTORS=1 cargo test -p blueos-idl cdr_vectors_match_rust_codec"
    );

    for vector in &on_disk.vectors {
        let payload = decode_hex(&vector.hex);
        let decoded =
            cdr_codec_dispatch::decode_to_json(&vector.schema_name, &payload).expect("rust decode");
        assert_eq!(
            decoded, vector.decoded,
            "decode mismatch for {}",
            vector.schema_name
        );
        if !vector.skip_encode_round_trip {
            let reencoded = cdr_codec_dispatch::encode_default(&vector.schema_name);
            if vector.category == "default" {
                let reencoded = reencoded.expect("encode default");
                assert_eq!(
                    encode_hex(&reencoded),
                    vector.hex,
                    "default re-encode mismatch for {}",
                    vector.schema_name
                );
            } else {
                let message = decoded;
                let reencoded_hex =
                    encode_json_message(&vector.schema_name, &message).expect("re-encode example");
                assert_eq!(
                    reencoded_hex, vector.hex,
                    "example re-encode mismatch for {}",
                    vector.schema_name
                );
            }
        }
    }
}
