use std::path::PathBuf;

pub fn catalog_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("catalog-paths lives under catalog/crates/")
        .to_path_buf()
}

pub fn repo_root() -> PathBuf {
    catalog_dir()
        .parent()
        .expect("catalog dir has a parent directory")
        .to_path_buf()
}

pub fn docs_root() -> PathBuf {
    repo_root().join("../BlueOS-docs")
}
