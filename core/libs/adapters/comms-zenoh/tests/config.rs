//! Zenoh configuration from CLI-shaped options.

use blueos_comms_zenoh::config::{ZenohConnectOptions, client_config_from_options};

#[test]
fn zenoh_set_override_applies_after_defaults() {
    let options = ZenohConnectOptions {
        endpoint: "tcp/127.0.0.1:7447",
        config_file: None,
        zenoh_sets: &["transport/shared_memory/enabled=false".to_string()],
    };
    let configuration = client_config_from_options(&options).expect("config");
    let shared_memory = configuration
        .get_json("transport/shared_memory/enabled")
        .expect("json");
    assert_eq!(shared_memory, "false");
}
