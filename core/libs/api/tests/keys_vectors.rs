//! Key helpers must match the JSON vectors shared with the TypeScript client.

use std::{fs, path::PathBuf};

use serde::Deserialize;

use blueos_api::{
    API_VERSION, ENCODING_APPLICATION_CDR, KEY_PREFIX, TYPE_HASH_ATTACHMENT_KEY, cdr_encoding,
    command_key, event_key, extension_log_key, http_gateway_prefix, info_query_key, jobs_key,
    log_key, query_key, service_liveliness_key, settings_key, state_key, status_state_key,
};

#[derive(Debug, Deserialize)]
struct KeysVectors {
    constants: ConstantsVectors,
    service_liveliness_key: Vec<ServiceCase>,
    command_key: Vec<NamedCase>,
    state_key: Vec<NamedCase>,
    event_key: Vec<NamedCase>,
    query_key: Vec<NamedCase>,
    jobs_key: Vec<ServiceCase>,
    settings_key: Vec<ServiceCase>,
    log_key: Vec<ServiceCase>,
    extension_log_key: Vec<ExtensionLogCase>,
    http_gateway_prefix: Vec<ServiceCase>,
    status_state_key: Vec<ServiceCase>,
    info_query_key: Vec<ServiceCase>,
    cdr_encoding: Vec<CdrEncodingCase>,
}

#[derive(Debug, Deserialize)]
struct ConstantsVectors {
    api_version: String,
    key_prefix: String,
    encoding_application_cdr: String,
    type_hash_attachment_key: String,
}

#[derive(Debug, Deserialize)]
struct ServiceCase {
    service: String,
    expected: String,
}

#[derive(Debug, Deserialize)]
struct NamedCase {
    service: String,
    name: String,
    expected: String,
}

#[derive(Debug, Deserialize)]
struct ExtensionLogCase {
    service: String,
    extension_identifier: String,
    expected: String,
}

#[derive(Debug, Deserialize)]
struct CdrEncodingCase {
    schema_name: String,
    expected: String,
}

#[test]
fn keys_vectors_match_rust_api() {
    let content = fs::read_to_string(vectors_path()).expect("read keys.json");
    let vectors: KeysVectors = serde_json::from_str(&content).expect("parse keys.json");

    assert_eq!(API_VERSION, vectors.constants.api_version);
    assert_eq!(KEY_PREFIX, vectors.constants.key_prefix);
    assert_eq!(
        ENCODING_APPLICATION_CDR,
        vectors.constants.encoding_application_cdr
    );
    assert_eq!(
        TYPE_HASH_ATTACHMENT_KEY,
        vectors.constants.type_hash_attachment_key
    );

    for case in vectors.service_liveliness_key {
        assert_eq!(service_liveliness_key(&case.service), case.expected);
    }
    for case in vectors.command_key {
        assert_eq!(command_key(&case.service, &case.name), case.expected);
    }
    for case in vectors.state_key {
        assert_eq!(state_key(&case.service, &case.name), case.expected);
    }
    for case in vectors.event_key {
        assert_eq!(event_key(&case.service, &case.name), case.expected);
    }
    for case in vectors.query_key {
        assert_eq!(query_key(&case.service, &case.name), case.expected);
    }
    for case in vectors.jobs_key {
        assert_eq!(jobs_key(&case.service), case.expected);
    }
    for case in vectors.settings_key {
        assert_eq!(settings_key(&case.service), case.expected);
    }
    for case in vectors.log_key {
        assert_eq!(log_key(&case.service), case.expected);
    }
    for case in vectors.extension_log_key {
        assert_eq!(
            extension_log_key(&case.service, &case.extension_identifier),
            case.expected
        );
    }
    for case in vectors.http_gateway_prefix {
        assert_eq!(http_gateway_prefix(&case.service), case.expected);
    }
    for case in vectors.status_state_key {
        assert_eq!(status_state_key(&case.service), case.expected);
    }
    for case in vectors.info_query_key {
        assert_eq!(info_query_key(&case.service), case.expected);
    }
    for case in vectors.cdr_encoding {
        assert_eq!(cdr_encoding(&case.schema_name), case.expected);
    }
}

fn vectors_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors/keys.json")
}
