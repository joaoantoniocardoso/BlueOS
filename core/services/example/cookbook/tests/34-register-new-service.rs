//! Registering a new Service touches the multicall binary, manifest, and ops docs.

#[test]
fn example_readme_lists_registration_steps() {
    let readme = include_str!("../../README.md");
    for needle in [
        "core/app/blueos/src/main.rs",
        "core/app/blueos/Cargo.toml",
        "core/tools/nginx/nginx.conf",
        "core/start-blueos-core",
        "endpoints.toml",
    ] {
        assert!(
            readme.contains(needle),
            "example README should mention {needle}"
        );
    }
}

#[test]
fn multicall_knows_the_teaching_example_name() {
    let main_rs = include_str!("../../../../app/blueos/src/main.rs");
    assert!(main_rs.contains("\"example\""));
    assert!(main_rs.contains("blueos_example_app"));
}
