//! Pure import-line plan for generated app `endpoints.rs`.

use super::{import_lines, imports::ImportBlockInput};

pub(super) fn planned_import_lines(input: &ImportBlockInput<'_>) -> Vec<String> {
    import_lines::planned_import_lines(input)
}

pub(super) fn planned_import_block(input: &ImportBlockInput<'_>) -> String {
    let lines = planned_import_lines(input);
    let mut block = String::new();
    for line in &lines {
        block.push_str(line);
        block.push('\n');
    }
    block
}
