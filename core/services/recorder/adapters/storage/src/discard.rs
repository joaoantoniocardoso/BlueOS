//! Removes leftover repair and snapshot temporary files from the recordings tree.

use std::{fs, path::Path};

use super::{RECOVER_SUFFIX, RecordingsFolder, SNAPSHOT_PARTIAL_SUFFIX, relative_path_string};

pub(crate) fn discard_recover_files(folder: &RecordingsFolder) {
    log_discarded_repair_files(collect_removed_files(folder, RECOVER_SUFFIX));
}

pub(crate) fn discard_snapshot_partial_files(folder: &RecordingsFolder) {
    log_discarded_snapshot_files(collect_removed_files(folder, SNAPSHOT_PARTIAL_SUFFIX));
}

fn collect_removed_files(folder: &RecordingsFolder, suffix: &str) -> Vec<(String, u64)> {
    let mut removed = Vec::new();
    discard_files_with_suffix_under(folder.base(), folder.base(), suffix, &mut removed);
    removed
}

fn log_discarded_repair_files(removed: Vec<(String, u64)>) {
    removed.into_iter().for_each(|(relative, size_bytes)| {
        tracing::info!(
            path = %relative,
            size_bytes,
            "Discarded leftover repair temporary file"
        );
    });
}

fn log_discarded_snapshot_files(removed: Vec<(String, u64)>) {
    for (relative, size_bytes) in removed {
        tracing::info!(
            path = %relative,
            size_bytes,
            "Discarded leftover snapshot temporary file"
        );
    }
}

fn discard_files_with_suffix_under(
    base: &Path,
    directory: &Path,
    suffix: &str,
    removed: &mut Vec<(String, u64)>,
) {
    let mut directories = vec![directory.to_path_buf()];
    while let Some(current) = directories.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(value) => value,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                directories.push(path);
                continue;
            }
            let name = entry.file_name();
            if !name.to_string_lossy().ends_with(suffix) {
                continue;
            }
            let relative = path
                .strip_prefix(base)
                .map(relative_path_string)
                .unwrap_or_else(|_| name.to_string_lossy().into_owned());
            let size_bytes = entry.metadata().map(|metadata| metadata.len()).unwrap_or(0);
            if fs::remove_file(&path).is_ok() {
                removed.push((relative, size_bytes));
            }
        }
    }
}
