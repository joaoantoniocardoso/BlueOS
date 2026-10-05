//! Committed endpoint file drift tests.

mod common;

use std::fs;

use blueos_idl_codegen::endpoints::{format, generate, generate_all};
use common::endpoints::{core_dir, example_message_schema_names};

#[test]
fn the_committed_endpoint_files_match_their_manifests() {
    let core_dir = core_dir();
    let messages = example_message_schema_names();

    let files = generate_all(&core_dir, &messages).unwrap();

    assert!(!files.is_empty());
    for file in files {
        assert_eq!(
            fs::read_to_string(&file.path).unwrap(),
            file.contents,
            "{} is stale: run `cargo run -p blueos-idl-codegen -- --write`",
            file.path.display()
        );
    }
}

#[test]
fn the_custom_handler_fixture_of_the_compile_fail_tests_matches_its_manifest() {
    let core_dir = core_dir();
    let fixture = core_dir.join("services/example/app/tests/compile_fail/custom");
    let manifest = fs::read_to_string(fixture.join("endpoints.toml")).unwrap();
    let messages = example_message_schema_names();

    let generated = generate(&manifest, "blueos_example_api", &messages).unwrap();

    assert_eq!(
        fs::read_to_string(fixture.join("endpoints.rs")).unwrap(),
        format(&generated.app).unwrap(),
        "{} is stale: write the generated app source there",
        fixture.join("endpoints.rs").display()
    );
}
