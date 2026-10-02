//! Prints `api.lock` lines for every message under `blueos-idl/interfaces`.

use std::{env, path::PathBuf};

use blueos_idl_codegen::{collect_messages_for_test, format_lock_line};

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let interfaces_root = manifest_dir
        .parent()
        .expect("idl crate root")
        .join("interfaces");
    let mut lines = collect_messages_for_test(&interfaces_root)
        .into_iter()
        .map(|record| format_lock_line(&record.schema_name, 1, &record.field_signature))
        .collect::<Vec<_>>();
    lines.sort();
    for line in lines {
        println!("{line}");
    }
}
