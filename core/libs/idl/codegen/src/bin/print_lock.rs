//! Prints `api.lock` lines for every message under `blueos-idl/interfaces`.

extern crate alloc;

use alloc::collections::BTreeSet;
use std::{env, path::PathBuf, process::ExitCode};

use blueos_idl_codegen::{
    collect_messages_for_test, core_dir, endpoints::collect_endpoint_lock_lines, format_lock_line,
    idl_root,
};

fn main() -> ExitCode {
    if let Err(error) = run() {
        eprintln!("{error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), blueos_idl_codegen::CodegenError> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let idl_root = idl_root(&manifest_dir, None)?;
    let interfaces_root = idl_root.join("interfaces");
    let core_directory = core_dir(&idl_root, None)?;
    let records = collect_messages_for_test(&interfaces_root)?;
    let messages = records
        .iter()
        .map(|record| record.schema_name.clone())
        .collect::<BTreeSet<_>>();
    let mut lines = records
        .iter()
        .map(|record| format_lock_line(&record.schema_name, 1, &record.field_signature))
        .collect::<Vec<_>>();
    lines.extend(collect_endpoint_lock_lines(&core_directory, &messages)?);
    lines.sort();
    for line in lines {
        println!("{line}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_prints_lock_lines() {
        run().expect("lock lines");
    }
}
