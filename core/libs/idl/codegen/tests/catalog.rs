//! Vendored catalog generation smoke test.

use std::{env, fs, path::PathBuf};

use blueos_idl_codegen::generate_schema_catalog;

#[test]
fn generate_schema_catalog_writes_lookup() {
    let idl_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("idl root")
        .to_path_buf();
    let output = env::temp_dir().join(format!(
        "blueos-catalog-{}-{}",
        env!("CARGO_PKG_NAME"),
        std::process::id()
    ));
    _ = fs::remove_dir_all(&output);
    fs::create_dir_all(&output).expect("temp dir");
    generate_schema_catalog(&idl_root.join("catalog/interfaces"), &output).expect("catalog");
    let schema_catalog = output.join("schema_catalog.rs");
    assert!(schema_catalog.is_file());
    let contents = fs::read_to_string(schema_catalog).expect("read");
    assert!(contents.contains("pub(crate) fn schema"));
    _ = fs::remove_dir_all(output);
}
