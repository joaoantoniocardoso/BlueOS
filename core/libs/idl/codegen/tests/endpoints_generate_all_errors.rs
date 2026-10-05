//! `generate_all` error path tests.

mod common;

use blueos_idl_codegen::endpoints::{EndpointsError, ManifestError, generate_all};
use common::endpoints::{Workspace, messages};

#[test]
fn generate_all_names_the_manifest_that_is_wrong() {
    let workspace = Workspace::new("wrong_manifest");
    workspace.write(
        "Cargo.toml",
        "[workspace]\nmembers = [\"other\", \"test/app\"]\n",
    );
    workspace.write(
        "test/app/endpoints.toml",
        "service = \"Info\"\n[state]\nInfo = { type = \"x\" }\n",
    );
    workspace.write(
        "test/logic/api/Cargo.toml",
        "[package]\nname = \"blueos-test-api\"\n",
    );

    let error = generate_all(&workspace.root, &messages()).unwrap_err();

    assert!(matches!(
        error,
        EndpointsError::Manifest { path, error: ManifestError::Reserved(_) }
            if path == workspace.root.join("test/app/endpoints.toml")
    ));
}

#[test]
fn generate_all_names_the_cargo_toml_it_cannot_use() {
    let cases = [
        ("no_members", "[workspace]\n", None),
        ("bad_toml", "[workspace\n", None),
        (
            "no_package_name",
            "[workspace]\nmembers = [\"test/app\"]\n",
            Some("[package]\n"),
        ),
    ];
    for (name, workspace_toml, api_toml) in cases {
        let workspace = Workspace::new(name);
        workspace.write("Cargo.toml", workspace_toml);
        workspace.write("test/app/endpoints.toml", "service = \"test\"\n");
        if let Some(api_toml) = api_toml {
            workspace.write("test/logic/api/Cargo.toml", api_toml);
        }

        let error = generate_all(&workspace.root, &messages()).unwrap_err();

        assert!(
            matches!(error, EndpointsError::Cargo { .. }),
            "{name}: {error}"
        );
    }
}

#[test]
fn generate_all_names_the_file_it_cannot_read() {
    let workspace = Workspace::new("no_api_crate");
    workspace.write("Cargo.toml", "[workspace]\nmembers = [\"test/app\"]\n");
    workspace.write("test/app/endpoints.toml", "service = \"test\"\n");

    let error = generate_all(&workspace.root, &messages()).unwrap_err();

    assert!(
        error.to_string().starts_with(
            &workspace
                .root
                .join("test/logic/api/Cargo.toml")
                .display()
                .to_string()
        ),
        "{error}"
    );
}
