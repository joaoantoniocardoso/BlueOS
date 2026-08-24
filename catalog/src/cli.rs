use std::path::Path;

pub fn flag_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find_map(|pair| {
        if pair[0] == name {
            Some(pair[1].clone())
        } else {
            None
        }
    })
}

pub fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|arg| arg == name)
}

pub fn percent(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64) * 100.0 / (total as f64)
    }
}

pub fn read_json_file<T: serde::de::DeserializeOwned>(
    path: &Path,
    label: &str,
) -> Result<T, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|err| format!("read {label} {}: {err}", path.display()))?;
    serde_json::from_str(&content).map_err(|err| format!("parse {label} {}: {err}", path.display()))
}

pub fn write_json_file<T: serde::Serialize>(
    path: &Path,
    value: &T,
    label: &str,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("create {label} dir {}: {err}", parent.display()))?;
    }
    let content =
        serde_json::to_string_pretty(value).map_err(|err| format!("serialize {label}: {err}"))?;
    std::fs::write(path, format!("{content}\n"))
        .map_err(|err| format!("write {label} {}: {err}", path.display()))
}
