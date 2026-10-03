//! Parses [`README.md`](../README.md) and checks every P4.3 row against disk and naming rules.

use std::{collections::BTreeMap, path::PathBuf};

enum EntryKind {
    Cookbook { file: String },
    ExampleMinimal { relative: String },
}

struct Row {
    number: u8,
    entry: EntryKind,
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn example_minimal_root() -> PathBuf {
    manifest_dir()
        .parent()
        .expect("example service root")
        .to_path_buf()
}

fn tests_dir() -> PathBuf {
    manifest_dir().join("tests")
}

fn parse_readme_table(readme: &str) -> Vec<Row> {
    let mut rows = Vec::new();
    for line in readme.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        if line.contains("---") || line.contains("| # |") {
            continue;
        }
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        if cells.len() < 5 {
            continue;
        }
        let number: u8 = cells[1].parse().expect("question number");
        let entry_cell = cells[3];
        let status = cells[4];
        assert!(
            !status.contains("not written yet"),
            "question {number} must point at a written entry"
        );
        let entry = if entry_cell.contains("../") {
            EntryKind::ExampleMinimal {
                relative: extract_example_minimal_path(entry_cell),
            }
        } else {
            EntryKind::Cookbook {
                file: extract_cookbook_filename(entry_cell),
            }
        };
        rows.push(Row { number, entry });
    }
    rows
}

fn extract_cookbook_filename(cell: &str) -> String {
    if let Some(start) = cell.find('`') {
        let rest = &cell[start + 1..];
        if let Some(end) = rest.find('`') {
            let token = &rest[..end];
            return token.trim_start_matches("tests/").to_owned();
        }
    }
    panic!("could not parse cookbook entry cell: {cell}");
}

fn extract_example_minimal_path(cell: &str) -> String {
    if let Some(start) = cell.find("](../") {
        let rest = &cell[start + "](../".len()..];
        if let Some(end) = rest.find(')') {
            return rest[..end].to_owned();
        }
    }
    if let Some(start) = cell.find('`') {
        let rest = &cell[start + 1..];
        if let Some(end) = rest.find('`') {
            return rest[..end].to_owned();
        }
    }
    panic!("could not parse example-minimal entry cell: {cell}");
}

fn leading_entry_number(filename: &str) -> u8 {
    filename
        .split('-')
        .next()
        .expect("entry name")
        .parse()
        .expect("entry number prefix")
}

#[test]
fn readme_table_maps_questions_one_through_thirty_five() {
    let readme = include_str!("../README.md");
    let rows = parse_readme_table(readme);
    assert_eq!(rows.len(), 35, "expected 35 question rows in README");
    for question in 1..=35 {
        assert_eq!(
            rows.iter().filter(|row| row.number == question).count(),
            1,
            "question {question} must appear exactly once"
        );
    }
}

#[test]
fn readme_entries_exist_and_follow_numbering_rule() {
    let readme = include_str!("../README.md");
    let rows = parse_readme_table(readme);
    let mut first_question_by_cookbook_file: BTreeMap<String, u8> = BTreeMap::new();

    for row in &rows {
        match &row.entry {
            EntryKind::Cookbook { file } => {
                first_question_by_cookbook_file
                    .entry(file.clone())
                    .and_modify(|first| *first = (*first).min(row.number))
                    .or_insert(row.number);
                let path = tests_dir().join(file);
                assert!(path.is_file(), "missing cookbook entry tests/{file}");
            }
            EntryKind::ExampleMinimal { relative } => {
                let path = example_minimal_root().join(relative);
                assert!(path.is_file(), "missing example-minimal path {relative}");
            }
        }
    }

    for (file, first_question) in &first_question_by_cookbook_file {
        assert_eq!(
            leading_entry_number(file),
            *first_question,
            "entry {file} must be numbered for question {first_question}"
        );
    }
}
