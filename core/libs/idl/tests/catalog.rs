//! Vendored catalog schema lookup (`catalog` feature).

use blueos_idl::catalog;

#[test]
fn foxglove_sdk_names_resolve_with_their_dependencies() {
    let text = catalog::schema("foxglove.CompressedVideo").expect("CompressedVideo schema");
    assert!(text.starts_with("# foxglove_msgs/msg/CompressedVideo"));
    assert!(text.contains("\nMSG: builtin_interfaces/Time\n"));
    assert_eq!(
        catalog::schema("foxglove_msgs/msg/CompressedVideo"),
        Some(text)
    );
}

#[test]
fn foxglove_dot_alias_resolves_to_foxglove_msgs_path() {
    let sdk = catalog::schema("foxglove.Log").expect("foxglove.Log");
    let ros = catalog::schema("foxglove_msgs/msg/Log").expect("foxglove_msgs/msg/Log");
    assert_eq!(sdk, ros);
}

#[test]
fn ros2_messages_carry_nested_dependencies() {
    let text = catalog::schema("visualization_msgs/msg/MarkerArray").expect("MarkerArray schema");
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
    assert_eq!(catalog::schema("visualization_msgs/msg/Missing"), None);
}
