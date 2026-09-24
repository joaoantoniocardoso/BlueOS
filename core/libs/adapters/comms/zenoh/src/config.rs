use blueos_comms_driver::{CommsError, Endpoint, Result};

pub fn load_config(service_name: &str, endpoint: Endpoint) -> Result<zenoh::Config> {
    if let Ok(path) = std::env::var("ZENOH_CONFIG") {
        return zenoh::Config::from_file(path)
            .map_err(|error| CommsError::Zenoh(error.to_string()));
    }
    validate_service_name(service_name)?;
    client_config(service_name, endpoint)
}

fn validate_service_name(service_name: &str) -> Result<()> {
    if service_name.is_empty() {
        return Err(CommsError::Message("service name cannot be empty".into()));
    }
    if service_name.contains('/') {
        return Err(CommsError::Message(
            "service name cannot contain forward slash character ('/')".into(),
        ));
    }
    if service_name.contains('.') {
        return Err(CommsError::Message(
            "service name cannot contain extension-separation character ('.')".into(),
        ));
    }
    Ok(())
}

fn client_config(service_name: &str, endpoint: Endpoint) -> Result<zenoh::Config> {
    let connect_endpoint = match endpoint {
        Endpoint::Local => "tcp/127.0.0.1:7447".to_string(),
        Endpoint::Remote { url } => url,
    };
    let config_json = format!(
        r#"{{
        mode: "client",
        connect: {{ endpoints: ["{connect_endpoint}"] }},
        adminspace: {{ enabled: true }},
        metadata: {{ name: "{service_name}" }},
        transport: {{ shared_memory: {{ enabled: true }} }}
    }}"#
    );
    zenoh::Config::from_json5(&config_json).map_err(|error| CommsError::Zenoh(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{client_config, validate_service_name};
    use blueos_comms_driver::Endpoint;

    #[test]
    fn validate_service_name_matches_python_rules() {
        assert!(validate_service_name("").is_err());
        assert!(validate_service_name("a/b").is_err());
        assert!(validate_service_name("a.b").is_err());
        assert!(validate_service_name("ardupilot_manager").is_ok());
    }

    #[test]
    fn client_config_loads() {
        assert!(client_config("ping", Endpoint::Local).is_ok());
    }
}
