use std::path::PathBuf;

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let interfaces = manifest_dir.join("interfaces");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let typescript = manifest_dir.join("typescript");
    blueos_idl_codegen::generate(&interfaces, &out_dir, &typescript);
}
