use std::fs;
use std::path::Path;
use std::str::FromStr;

use serde::de::DeserializeOwned;
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("{0}")]
    Message(String),
}

#[derive(Clone, Debug)]
pub struct Configs {
    value: Value,
}

impl Default for Configs {
    fn default() -> Self {
        Self {
            value: Value::Object(serde_json::Map::new()),
        }
    }
}

impl FromStr for Configs {
    type Err = ConfigError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let value =
            json5::from_str(text).map_err(|error| ConfigError::Message(error.to_string()))?;
        Ok(Self { value })
    }
}

impl Configs {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let text =
            fs::read_to_string(path).map_err(|error| ConfigError::Message(error.to_string()))?;
        text.parse()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.value.get(key)
    }

    pub fn deserialize<T: DeserializeOwned>(&self) -> Result<T, ConfigError> {
        serde_json::from_value(self.value.clone())
            .map_err(|error| ConfigError::Message(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_json5_with_comments() {
        let path =
            std::env::temp_dir().join(format!("blueos-configs-{}.json5", std::process::id()));
        let mut file = std::fs::File::create(&path).expect("create temp config");
        writeln!(
            file,
            "{{
  // comment
  moving: true,
  name: \"stub\",
  count: 3,
}}"
        )
        .unwrap();
        drop(file);
        let configs = Configs::load(&path).unwrap();
        assert_eq!(configs.get("moving").and_then(Value::as_bool), Some(true));
        assert_eq!(configs.get("name").and_then(Value::as_str), Some("stub"));
        assert_eq!(configs.get("count").and_then(Value::as_i64), Some(3));
        let _ = std::fs::remove_file(path); // best-effort temp cleanup
    }
}
