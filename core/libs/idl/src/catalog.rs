//! Schema text of the ROS 2 and Foxglove messages vendored in `catalog/`, which are not part of the BlueOS API.

use alloc::format;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/schema_catalog.rs"));
}

/// ROS 2 `.msg` text of a vendored message named `package/msg/Name`, or `foxglove.Name` as the Foxglove SDK (and
/// mavlink-camera-manager) names `foxglove_msgs/msg/Name`.
pub fn schema(schema_name: &str) -> Option<&'static str> {
    match schema_name.strip_prefix("foxglove.") {
        Some(name) => generated::schema(&format!("foxglove_msgs/msg/{name}")),
        None => generated::schema(schema_name),
    }
}

#[cfg(test)]
mod tests {
    use alloc::format;

    use super::schema;

    #[test]
    fn foxglove_sdk_names_resolve_with_their_dependencies() {
        let text = schema("foxglove.CompressedVideo").expect("CompressedVideo schema");
        assert!(text.starts_with("# foxglove_msgs/msg/CompressedVideo"));
        assert!(text.contains("\nMSG: builtin_interfaces/Time\n"));
        assert_eq!(schema("foxglove_msgs/msg/CompressedVideo"), Some(text));
    }

    #[test]
    fn ros2_messages_carry_nested_dependencies() {
        let text = schema("visualization_msgs/msg/MarkerArray").expect("MarkerArray schema");
        for dependency in [
            "visualization_msgs/Marker",
            "std_msgs/Header",
            "builtin_interfaces/Time",
            "geometry_msgs/Pose",
        ] {
            assert!(
                text.contains(&format!("\nMSG: {dependency}\n")),
                "missing {dependency}"
            );
        }
        assert_eq!(schema("visualization_msgs/msg/Missing"), None);
    }
}
