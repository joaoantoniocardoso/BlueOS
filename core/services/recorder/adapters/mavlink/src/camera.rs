//! Builds `video/...` topic names from MAVLink stream names.

/// Sanitizes a MAVLink stream name into a `video/.../stream` backbone key.
pub fn video_topic_from_name(name: &str) -> String {
    let sanitized_stream_name = name
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect::<String>();
    format!("video/{sanitized_stream_name}/stream")
}
