#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::String;

pub const API_VERSION: &str = "v1";
pub const KEY_PREFIX: &str = "blueos/v1";

pub const ENCODING_APPLICATION_CDR: &str = "application/cdr";
pub const TYPE_HASH_ATTACHMENT_KEY: &str = "blueos.type_hash";

pub fn service_liveliness_key(service: &str) -> String {
    format!("{KEY_PREFIX}/services/{service}")
}

pub fn service_info_key(service: &str) -> String {
    format!("{KEY_PREFIX}/services/{service}/info")
}

pub fn command_key(service: &str, name: &str) -> String {
    format!("{KEY_PREFIX}/{service}/command/{name}")
}

pub fn state_key(service: &str, name: &str) -> String {
    format!("{KEY_PREFIX}/{service}/state/{name}")
}

pub fn event_key(service: &str, name: &str) -> String {
    format!("{KEY_PREFIX}/{service}/event/{name}")
}

pub fn query_key(service: &str, name: &str) -> String {
    format!("{KEY_PREFIX}/{service}/query/{name}")
}

pub fn jobs_key(service: &str) -> String {
    format!("{KEY_PREFIX}/{service}/jobs")
}

pub fn settings_key(service: &str) -> String {
    format!("{KEY_PREFIX}/{service}/settings")
}

pub fn log_key(service: &str) -> String {
    format!("{KEY_PREFIX}/{service}/log")
}

pub fn status_state_key(service: &str) -> String {
    state_key(service, "status")
}

pub fn info_query_key(service: &str) -> String {
    query_key(service, "info")
}

pub fn cdr_encoding(schema_name: &str) -> String {
    format!("{ENCODING_APPLICATION_CDR};{schema_name}")
}
