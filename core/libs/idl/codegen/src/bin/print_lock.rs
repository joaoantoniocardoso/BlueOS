//! Prints `api.lock` lines for every message under `blueos-idl/interfaces`.

use std::{collections::BTreeSet, env, path::PathBuf};

use blueos_idl_codegen::{
    collect_messages_for_test, endpoints::collect_endpoint_lock_lines, format_lock_line,
};

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let idl_root = manifest_dir.parent().expect("idl crate root");
    let interfaces_root = idl_root.join("interfaces");
    let core_dir = idl_root
        .parent()
        .expect("libs")
        .parent()
        .expect("core root");
    let messages = collect_messages_for_test(&interfaces_root)
        .into_iter()
        .map(|record| record.schema_name)
        .collect::<BTreeSet<_>>();
    let mut lines = collect_messages_for_test(&interfaces_root)
        .into_iter()
        .map(|record| format_lock_line(&record.schema_name, 1, &record.field_signature))
        .collect::<Vec<_>>();
    lines.extend(
        collect_endpoint_lock_lines(core_dir, &messages)
            .expect("collect endpoint keys for api.lock"),
    );
    lines.sort();
    for line in lines {
        println!("{line}");
    }
}
