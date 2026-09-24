use serde_json::Value;

/// Top-level settings keys that changed, split by whether they can apply live (D-11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsFieldChanges {
    pub applied_live: Vec<String>,
    pub restart_required: Vec<String>,
}

/// Compares two settings JSON objects at the top level only.
pub fn diff_top_level_settings(
    restart_required_fields: &[&str],
    old_settings: &Value,
    new_settings: &Value,
) -> SettingsFieldChanges {
    let empty_map = serde_json::Map::new();
    let old_object = old_settings.as_object().unwrap_or(&empty_map);
    let new_object = new_settings.as_object().unwrap_or(&empty_map);

    let mut keys: Vec<&str> = old_object
        .keys()
        .chain(new_object.keys())
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    keys.dedup();

    let mut applied_live = Vec::new();
    let mut restart_required = Vec::new();

    for key in keys {
        let old_value = old_object.get(key);
        let new_value = new_object.get(key);
        if old_value == new_value {
            continue;
        }
        if restart_required_fields.contains(&key) {
            restart_required.push(key.to_string());
        } else {
            applied_live.push(key.to_string());
        }
    }

    SettingsFieldChanges {
        applied_live,
        restart_required,
    }
}
