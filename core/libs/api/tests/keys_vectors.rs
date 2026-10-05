//! Key helpers must match the JSON vectors shared with the TypeScript client.

use std::{fs, path::PathBuf};

use serde::Deserialize;

use blueos_api::{
    API_VERSION, ENCODING_APPLICATION_CDR, KEY_PREFIX, TYPE_HASH_ATTACHMENT_KEY, cdr_encoding,
    command_key, event_key, extension_log_key, http_gateway_prefix, info_query_key,
    job_feedback_key, job_history_key, job_result_key, jobs_key, log_key, query_key,
    service_liveliness_key, settings_key, state_key, status_state_key,
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
    job_feedback_key: Vec<NamedCase>,
    job_result_key: Vec<NamedCase>,
    job_history_key: Vec<NamedCase>,
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

    assert_constants(&vectors.constants);
    assert_service_cases(&vectors.service_liveliness_key, service_liveliness_key);
    assert_named_cases(&vectors.command_key, command_key);
    assert_named_cases(&vectors.state_key, state_key);
    assert_named_cases(&vectors.event_key, event_key);
    assert_named_cases(&vectors.query_key, query_key);
    assert_service_cases(&vectors.jobs_key, jobs_key);
    assert_named_cases(&vectors.job_feedback_key, job_feedback_key);
    assert_named_cases(&vectors.job_result_key, job_result_key);
    assert_named_cases(&vectors.job_history_key, job_history_key);
    assert_service_cases(&vectors.settings_key, settings_key);
    assert_service_cases(&vectors.log_key, log_key);
    assert_extension_log_cases(&vectors.extension_log_key);
    assert_service_cases(&vectors.http_gateway_prefix, http_gateway_prefix);
    assert_service_cases(&vectors.status_state_key, status_state_key);
    assert_service_cases(&vectors.info_query_key, info_query_key);
    assert_cdr_encoding_cases(&vectors.cdr_encoding);
}

fn assert_constants(constants: &ConstantsVectors) {
    assert_eq!(API_VERSION, constants.api_version);
    assert_eq!(KEY_PREFIX, constants.key_prefix);
    assert_eq!(ENCODING_APPLICATION_CDR, constants.encoding_application_cdr);
    assert_eq!(TYPE_HASH_ATTACHMENT_KEY, constants.type_hash_attachment_key);
}

fn assert_service_cases(cases: &[ServiceCase], build: fn(&str) -> String) {
    for case in cases {
        assert_eq!(build(&case.service), case.expected);
    }
}

fn assert_named_cases(cases: &[NamedCase], build: fn(&str, &str) -> String) {
    for case in cases {
        assert_eq!(build(&case.service, &case.name), case.expected);
    }
}

fn assert_extension_log_cases(cases: &[ExtensionLogCase]) {
    for case in cases {
        assert_eq!(
            extension_log_key(&case.service, &case.extension_identifier),
            case.expected
        );
    }
}

fn assert_cdr_encoding_cases(cases: &[CdrEncodingCase]) {
    for case in cases {
        assert_eq!(cdr_encoding(&case.schema_name), case.expected);
    }
}

fn vectors_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/vectors/keys.json")
}
