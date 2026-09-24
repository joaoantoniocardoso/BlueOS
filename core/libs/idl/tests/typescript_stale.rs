use std::fs;
use std::path::PathBuf;

#[test]
fn typescript_outputs_are_fresh() {
    if std::env::var("BLUEOS_IDL_REGEN_TYPESCRIPT").as_deref() == Ok("1") {
        return;
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schemas = manifest.join("typescript/schemas.ts");
    let messages = manifest.join("typescript/messages.d.ts");
    assert!(
        schemas.is_file(),
        "missing typescript/schemas.ts; run cargo build -p blueos-idl"
    );
    assert!(
        messages.is_file(),
        "missing typescript/messages.d.ts; run cargo build -p blueos-idl"
    );

    let build_rs = manifest.join("build.rs");
    let build_mtime = fs::metadata(build_rs)
        .and_then(|meta| meta.modified())
        .expect("build.rs mtime");
    for path in [schemas, messages] {
        let generated_mtime = fs::metadata(&path)
            .and_then(|meta| meta.modified())
            .expect("typescript mtime");
        assert!(
            generated_mtime >= build_mtime,
            "{} is stale; run cargo build -p blueos-idl",
            path.display()
        );
    }
}
