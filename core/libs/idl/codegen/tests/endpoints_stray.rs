//! Stray generated endpoint file detection.

mod common;

use std::fs;

use blueos_idl_codegen::endpoints::{generate, generate_all, stray_files};
use common::endpoints::{Workspace, messages};

#[test]
fn a_generated_file_without_its_manifest_is_stray() {
    let workspace = Workspace::new("stray_files");
    workspace.write(
        "Cargo.toml",
        "[workspace]\nmembers = [\"test/app\", \"test/logic/api\", \"gone/app\"]\n",
    );
    workspace.write("test/app/endpoints.toml", "service = \"test\"\n");
    workspace.write(
        "test/logic/api/Cargo.toml",
        "[package]\nname = \"blueos-test-api\"\n",
    );
    let files = generate_all(&workspace.root, &messages()).unwrap();
    for file in &files {
        fs::create_dir_all(file.path.parent().unwrap()).unwrap();
        fs::write(&file.path, &file.contents).unwrap();
    }
    let gone = generate("service = \"gone\"\n", "blueos_gone_api", &messages()).unwrap();
    workspace.write(
        "frontend/src/libs/blueos-api/services/gone.ts",
        &gone.typescript,
    );
    workspace.write("gone/app/src/endpoints.rs", &gone.app);
    workspace.write("gone/app/src/lib.rs", "//! Written by hand.\n");

    assert_eq!(
        stray_files(&workspace.root, &files).unwrap(),
        [
            workspace
                .root
                .join("frontend/src/libs/blueos-api/services/gone.ts"),
            workspace.root.join("gone/app/src/endpoints.rs"),
        ]
    );
}
