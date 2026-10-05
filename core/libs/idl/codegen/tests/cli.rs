//! CLI flag parsing and catalog path helpers.

use std::path::PathBuf;

use blueos_idl_codegen::{flag_value, idl_root};

#[test]
fn flag_value_reads_following_argument() {
    let arguments = vec![
        "blueos-idl-codegen".to_owned(),
        "--output".to_owned(),
        "/tmp/out".to_owned(),
    ];
    assert_eq!(
        flag_value(&arguments, "--output").as_deref(),
        Some("/tmp/out")
    );
    assert_eq!(flag_value(&arguments, "--missing"), None);
}

#[test]
fn idl_root_defaults_to_sibling_of_codegen_crate() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = idl_root(&manifest_dir, None).expect("idl root");
    assert!(root.join("interfaces").is_dir());
}
