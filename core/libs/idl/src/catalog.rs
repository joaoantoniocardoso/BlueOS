//! Schema text of vendored ROS 2 Jazzy and Foxglove messages (not part of the BlueOS API).

mod generated {
    include!("generated/schema_catalog.rs");
}

/// ROS 2 `.msg` text of a vendored message named `package/msg/Name`, or `foxglove.Name` as the Foxglove SDK (and
/// mavlink-camera-manager) names `foxglove_msgs/msg/Name`.
pub fn schema(schema_name: &str) -> Option<&'static str> {
    match schema_name.strip_prefix("foxglove.") {
        Some(name) => generated::schema(&alloc::format!("foxglove_msgs/msg/{name}")),
        None => generated::schema(schema_name),
    }
}
