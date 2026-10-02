//! Zenoh client configuration for BlueOS services.

use blueos_comms::CommsError;

/// Builds a client-only session configuration that connects to `endpoint` (for example `tcp/127.0.0.1:7447`).
pub(crate) fn client_config(endpoint: &str) -> Result<zenoh::Config, CommsError> {
    if let Ok(path) = std::env::var("ZENOH_CONFIG") {
        return zenoh::Config::from_file(path).map_err(|error| CommsError::Backend {
            message: error.to_string(),
        });
    }
    let configuration = format!(
        r#"{{
        mode: "client",
        connect: {{ endpoints: ["{endpoint}"] }},
        scouting: {{ multicast: {{ enabled: false }} }},
        transport: {{ shared_memory: {{ enabled: true }} }}
    }}"#
    );
    zenoh::Config::from_json5(&configuration).map_err(|error| CommsError::Backend {
        message: error.to_string(),
    })
}
