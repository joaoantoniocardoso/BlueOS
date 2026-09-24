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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_version_is_v1() {
        assert_eq!(API_VERSION, "v1");
    }

    #[test]
    fn key_prefix() {
        assert_eq!(KEY_PREFIX, "blueos/v1");
    }

    #[test]
    fn encoding_application_cdr() {
        assert_eq!(ENCODING_APPLICATION_CDR, "application/cdr");
    }

    #[test]
    fn type_hash_attachment_key() {
        assert_eq!(TYPE_HASH_ATTACHMENT_KEY, "blueos.type_hash");
    }

    #[test]
    fn service_liveliness_key_string() {
        assert_eq!(
            super::service_liveliness_key("recorder"),
            "blueos/v1/services/recorder"
        );
    }

    #[test]
    fn service_info_key_string() {
        assert_eq!(
            super::service_info_key("recorder"),
            "blueos/v1/services/recorder/info"
        );
    }

    #[test]
    fn command_key_format() {
        assert_eq!(
            command_key("recorder", "Start"),
            "blueos/v1/recorder/command/Start"
        );
    }

    #[test]
    fn state_key_format() {
        assert_eq!(
            state_key("recorder", "status"),
            "blueos/v1/recorder/state/status"
        );
    }

    #[test]
    fn event_key_format() {
        assert_eq!(
            event_key("recorder", "SessionOpened"),
            "blueos/v1/recorder/event/SessionOpened"
        );
    }

    #[test]
    fn query_key_format() {
        assert_eq!(
            query_key("recorder", "info"),
            "blueos/v1/recorder/query/info"
        );
    }

    #[test]
    fn jobs_key_format() {
        assert_eq!(jobs_key("recorder"), "blueos/v1/recorder/jobs");
    }

    #[test]
    fn settings_key_format() {
        assert_eq!(settings_key("recorder"), "blueos/v1/recorder/settings");
    }

    #[test]
    fn log_key_format() {
        assert_eq!(log_key("recorder"), "blueos/v1/recorder/log");
    }

    #[test]
    fn status_state_key_format() {
        assert_eq!(
            status_state_key("recorder"),
            "blueos/v1/recorder/state/status"
        );
    }

    #[test]
    fn info_query_key_format() {
        assert_eq!(info_query_key("recorder"), "blueos/v1/recorder/query/info");
    }

    #[test]
    fn cdr_encoding_format() {
        assert_eq!(
            cdr_encoding("blueos_msgs/ServiceStatus"),
            "application/cdr;blueos_msgs/ServiceStatus"
        );
    }
}
