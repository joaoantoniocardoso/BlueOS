//! Shared recording file name timestamp vectors (D-20 / D-24).

use serde::Deserialize;

use blueos_recorder_library::created_unix_seconds_from_filename;

#[derive(Deserialize)]
struct VectorEntry {
    file_name: String,
    file_time_unix_seconds: i64,
    created_unix_seconds: i64,
}

#[test]
fn recording_file_name_vectors_match_frontend() {
    let raw = include_str!("vectors/recording_file_names.json");
    let entries: Vec<VectorEntry> = serde_json::from_str(raw).expect("vector json");
    for entry in entries {
        let created =
            created_unix_seconds_from_filename(&entry.file_name, entry.file_time_unix_seconds);
        assert_eq!(
            created, entry.created_unix_seconds,
            "file_name {}",
            entry.file_name
        );
    }
}
