//! Pretty-print generated Rust and normalize import groups.

use std::{fs, path::Path};

use crate::error::CodegenError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ImportGroup {
    Std,
    ThirdParty,
    Owned,
}

pub(super) fn write_formatted_rust_file(path: &Path, source: &str) -> Result<(), CodegenError> {
    let _formatted = parse_and_format_source(path, source)?;
    run_rustfmt(path)?;
    let separated = separate_import_groups(
        &fs::read_to_string(path).map_err(|io_error| CodegenError::io(path, io_error))?,
    );
    fs::write(path, separated).map_err(|io_error| CodegenError::io(path, io_error))?;
    Ok(())
}

fn parse_and_format_source(path: &Path, source: &str) -> Result<String, CodegenError> {
    let syntax = syn::parse_file(source).map_err(|error| CodegenError::InvalidGeneratedRust {
        path: path.to_path_buf(),
        reason: error.to_string(),
    })?;
    let formatted = prettyplease::unparse(&syntax);
    fs::write(path, &formatted).map_err(|io_error| CodegenError::io(path, io_error))?;
    Ok(formatted)
}

fn run_rustfmt(path: &Path) -> Result<(), CodegenError> {
    let status = std::process::Command::new("rustfmt")
        .args(["--edition", "2024"])
        .arg(path)
        .status()
        .map_err(|io_error| CodegenError::io(path, io_error))?;
    if !status.success() {
        return Err(CodegenError::Rustfmt {
            path: path.to_path_buf(),
        });
    }
    Ok(())
}

fn separate_import_groups(source: &str) -> String {
    join_separated_lines(source.lines())
}

fn join_separated_lines(lines: impl IntoIterator<Item = impl AsRef<str>>) -> String {
    lines
        .into_iter()
        .fold(
            (None, String::new()),
            |(previous_group, mut output), line| {
                let line = line.as_ref();
                let next_group = append_separated_line(line, previous_group, &mut output);
                (next_group, output)
            },
        )
        .1
}

fn append_separated_line(
    line: &str,
    previous_group: Option<ImportGroup>,
    output: &mut String,
) -> Option<ImportGroup> {
    let trimmed = line.trim();
    let next_group = if let Some(group) = import_group_for_line(trimmed) {
        if let Some(previous_group) = previous_group
            && group != previous_group
            && !output.ends_with("\n\n")
        {
            output.push('\n');
        }
        Some(group)
    } else if !trimmed.is_empty() || previous_group.is_some() {
        None
    } else {
        previous_group
    };
    output.push_str(line);
    output.push('\n');
    next_group
}

fn import_group_for_line(line: &str) -> Option<ImportGroup> {
    let rest = line.strip_prefix("use ")?;
    let root = rest.split("::").next()?.split('{').next()?.trim();
    match root {
        "std" | "core" | "alloc" => Some(ImportGroup::Std),
        "crate" | "self" | "super" => Some(ImportGroup::Owned),
        _ => Some(ImportGroup::ThirdParty),
    }
}
