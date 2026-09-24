use std::path::PathBuf;

use blueos_idl_codegen::{collect_messages_for_test, format_lock_line};

fn main() {
    let interfaces = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../interfaces");
    let mut lines = collect_messages_for_test(&interfaces)
        .into_iter()
        .map(|record| format_lock_line(&record.schema_name, 1, &record.field_signature))
        .collect::<Vec<_>>();
    lines.sort();
    for line in lines {
        println!("{line}");
    }
}
