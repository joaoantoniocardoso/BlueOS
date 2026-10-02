//! Zenoh key layout, encoding strings and the [`CommandAck`] reply type for BlueOS services (D-07, D-10).
//!
//! Keys are always built through the helpers here, never as ad-hoc strings. Payloads use `blueos-idl`
//! [`Message`] codecs; Zenoh encoding metadata uses [`cdr_encoding`].

#![no_std]
#![expect(
    clippy::pub_use,
    reason = "blueos-api is the published facade for CommandAck and IDL Message"
)]

extern crate alloc;

mod command_ack;

use alloc::{format, string::String};

pub use command_ack::{CommandAck, JOB_ID_NONE, Message};

/// Base CDR media type and `application/cdr;<schema_name>` builder from `blueos-idl`.
pub use blueos_idl::encoding::{ENCODING_APPLICATION_CDR, cdr_encoding};

/// API version segment in every public key (`blueos/v1/...`).
pub const API_VERSION: &str = "v1";

/// Prefix shared by every versioned BlueOS zenoh key.
pub const KEY_PREFIX: &str = "blueos/v1";

/// Zenoh attachment key for a message type hash (schema evolution, D-06).
pub const TYPE_HASH_ATTACHMENT_KEY: &str = "blueos.type_hash";

/// Liveliness token for a running service (`blueos/v1/services/<name>`).
pub fn service_liveliness_key(service: &str) -> String {
    format!("{KEY_PREFIX}/services/{service}")
}

/// Command queryable (`blueos/v1/<service>/command/<name>`).
pub fn command_key(service: &str, name: &str) -> String {
    format!("{KEY_PREFIX}/{service}/command/{name}")
}

/// Standard service status state (`blueos/v1/<service>/state/status`).
pub fn status_state_key(service: &str) -> String {
    state_key(service, "status")
}

/// State sample key (`blueos/v1/<service>/state/<name>`).
pub fn state_key(service: &str, name: &str) -> String {
    format!("{KEY_PREFIX}/{service}/state/{name}")
}

/// Event sample key (`blueos/v1/<service>/event/<name>`).
pub fn event_key(service: &str, name: &str) -> String {
    format!("{KEY_PREFIX}/{service}/event/{name}")
}

/// Standard service info query (`blueos/v1/<service>/query/info`).
pub fn info_query_key(service: &str) -> String {
    query_key(service, "info")
}

/// Ad-hoc query queryable (`blueos/v1/<service>/query/<name>`).
pub fn query_key(service: &str, name: &str) -> String {
    format!("{KEY_PREFIX}/{service}/query/{name}")
}

/// Jobs projection stream (`blueos/v1/<service>/jobs`).
pub fn jobs_key(service: &str) -> String {
    format!("{KEY_PREFIX}/{service}/jobs")
}

/// Settings queryable and update stream (`blueos/v1/<service>/settings`).
pub fn settings_key(service: &str) -> String {
    format!("{KEY_PREFIX}/{service}/settings")
}

/// Service log stream (`blueos/v1/<service>/log`).
pub fn log_key(service: &str) -> String {
    format!("{KEY_PREFIX}/{service}/log")
}

/// Extension log stream under a service (`blueos/v1/<service>/log/extension/<safe_id>`).
pub fn extension_log_key(service: &str, extension_identifier: &str) -> String {
    let safe_identifier = extension_identifier.replace(['/', ' '], "_");
    format!("{}/extension/{safe_identifier}", log_key(service))
}

/// HTTP gateway mount prefix (`blueos/v1/<service>/http`).
pub fn http_gateway_prefix(service: &str) -> String {
    format!("{KEY_PREFIX}/{service}/http")
}
