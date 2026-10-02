//! Committed TypeScript IDL output must match `blueos-idl-codegen`.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn typescript_matches_codegen_output() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let committed = manifest_dir.join("typescript");
    assert!(
        committed.join("schemas.ts").is_file(),
        "missing typescript/schemas.ts; run: cargo run -p blueos-idl-codegen --bin blueos-idl-codegen -- --write"
    );

    let temporary = tempfile_directory();
    let generated_rust = temporary.join("generated");
    let generated_typescript = temporary.join("typescript");
    let core_workspace = manifest_dir.parent().expect("core workspace");
    let status = Command::new("cargo")
        .current_dir(core_workspace)
        .args([
            "run",
            "--quiet",
            "-p",
            "blueos-idl-codegen",
            "--bin",
            "blueos-idl-codegen",
            "--",
            "--output",
        ])
        .arg(&generated_rust)
        .arg("--typescript-output")
        .arg(&generated_typescript)
        .status()
        .expect("run blueos-idl-codegen");
    assert!(status.success(), "blueos-idl-codegen failed");

    compare_directories(&committed, &generated_typescript);
    fs::remove_dir_all(&temporary).expect("remove temp dir");
}

fn tempfile_directory() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "blueos-idl-typescript-stale-{}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

fn compare_directories(committed: &Path, generated: &Path) {
    for entry in fs::read_dir(committed).expect("read committed typescript") {
        let entry = entry.expect("typescript entry");
        let file_name = entry.file_name();
        let committed_path = entry.path();
        let generated_path = generated.join(&file_name);
        assert!(
            generated_path.is_file(),
            "missing generated {}",
            file_name.to_string_lossy()
        );
        let committed_bytes = fs::read(&committed_path).expect("read committed file");
        let generated_bytes = fs::read(&generated_path).expect("read generated file");
        assert_eq!(
            committed_bytes,
            generated_bytes,
            "{} differs from codegen output; run: cargo run -p blueos-idl-codegen --bin blueos-idl-codegen -- --write",
            committed_path.display()
        );
    }
}
