//! Zenoh client configuration for BlueOS services.

use std::path::Path;

use blueos_comms::CommsError;

/// Inputs for building a Zenoh client configuration from the common CLI (D-25).
pub struct ZenohConnectOptions<'a> {
    /// Router endpoint when no config file replaces it.
    pub endpoint: &'a str,
    /// Optional JSON5 config file (`--zenoh-config` or `ZENOH_CONFIG`).
    pub config_file: Option<&'a Path>,
    /// Repeatable `--zenoh-set PATH=JSON5` overrides.
    pub zenoh_sets: &'a [String],
}

/// Why a `--zenoh-set` override does not parse.
#[derive(Debug, thiserror::Error)]
enum ZenohSetError {
    #[error("zenoh-set must be PATH=JSON5, got `{0}`")]
    MissingEquals(String),
    #[error("zenoh-set path must not be empty, got `{0}`")]
    EmptyPath(String),
}

/// Builds a client configuration from [`ZenohConnectOptions`].
pub fn client_config_from_options(
    options: &ZenohConnectOptions<'_>,
) -> Result<zenoh::Config, CommsError> {
    let mut configuration = if let Some(path) = options.config_file {
        zenoh::Config::from_file(path).map_err(CommsError::backend)?
    } else if let Ok(path) = std::env::var("ZENOH_CONFIG") {
        zenoh::Config::from_file(path).map_err(CommsError::backend)?
    } else {
        let text = format!(
            r#"{{
        mode: "client",
        connect: {{ endpoints: ["{}"] }},
        scouting: {{ multicast: {{ enabled: false }} }},
        transport: {{ shared_memory: {{ enabled: true }} }}
    }}"#,
            options.endpoint
        );
        zenoh::Config::from_json5(&text).map_err(CommsError::backend)?
    };
    for zenoh_set in options.zenoh_sets {
        let (path, value) = split_zenoh_set(zenoh_set)?;
        configuration
            .insert_json5(path, value)
            .map_err(CommsError::backend)?;
    }
    Ok(configuration)
}

fn split_zenoh_set(zenoh_set: &str) -> Result<(&str, &str), CommsError> {
    let (path, value) = zenoh_set
        .split_once('=')
        .ok_or_else(|| CommsError::backend(ZenohSetError::MissingEquals(zenoh_set.to_owned())))?;
    if path.is_empty() {
        return Err(CommsError::backend(ZenohSetError::EmptyPath(
            zenoh_set.to_owned(),
        )));
    }
    Ok((path, value))
}
