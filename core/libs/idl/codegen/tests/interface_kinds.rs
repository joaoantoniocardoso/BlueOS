//! The generator on `.srv` and `.action` fixtures: the output it must regenerate byte for byte, the types and
//! schema text of every part, and the `api.lock` lines of every part.

use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process,
};

use blueos_idl_codegen::{
    check_message_lock, collect_messages_for_test, format_lock_line, generate, parse_lock_line,
};

/// Every part of the fixtures: its schema name, its type name, and the module of its Rust type.
const PARTS: [(&str, &str, &str); 8] = [
    (
        "fixture_msgs/action/Drain_Feedback",
        "DrainFeedback",
        "drain_feedback",
    ),
    ("fixture_msgs/action/Drain_Goal", "DrainGoal", "drain_goal"),
    (
        "fixture_msgs/action/Drain_Result",
        "DrainResult",
        "drain_result",
    ),
    (
        "fixture_msgs/action/Fill_Feedback",
        "FillFeedback",
        "fill_feedback",
    ),
    ("fixture_msgs/action/Fill_Goal", "FillGoal", "fill_goal"),
    (
        "fixture_msgs/action/Fill_Result",
        "FillResult",
        "fill_result",
    ),
    (
        "fixture_msgs/srv/Measure_Request",
        "MeasureRequest",
        "measure_request",
    ),
    (
        "fixture_msgs/srv/Measure_Response",
        "MeasureResponse",
        "measure_response",
    ),
];

#[test]
fn fixtures_regenerate_to_the_committed_output() {
    let expected = fixture_root().join("expected");
    let updating = env::var("BLUEOS_IDL_UPDATE_FIXTURES").as_deref() == Ok("1");
    let output = if updating {
        expected.clone()
    } else {
        env::temp_dir().join(format!("blueos-idl-interface-kinds-{}", process::id()))
    };
    if output.exists() {
        fs::remove_dir_all(&output).expect("remove stale output");
    }

    generate(
        &fixture_root().join("interfaces"),
        &output.join("generated"),
        Some(&output.join("typescript")),
        Some(&output.join("tests/generated")),
    )
    .expect("generate fixture output");
    if updating {
        return;
    }

    let committed = files(&expected);
    let generated = files(&output);
    fs::remove_dir_all(&output).expect("remove output");
    assert_eq!(
        committed.keys().collect::<Vec<_>>(),
        generated.keys().collect::<Vec<_>>(),
        "fixture output files differ; run: BLUEOS_IDL_UPDATE_FIXTURES=1 cargo test -p blueos-idl-codegen --test \
         interface_kinds"
    );
    for (path, contents) in &committed {
        assert!(
            generated[path] == *contents,
            "{} differs from codegen output; run: BLUEOS_IDL_UPDATE_FIXTURES=1 cargo test -p blueos-idl-codegen \
             --test interface_kinds",
            path.display()
        );
    }
}

#[test]
fn every_part_has_a_rust_and_a_typescript_type() {
    let expected = fixture_root().join("expected");
    let messages =
        fs::read_to_string(expected.join("typescript/messages.d.ts")).expect("read messages.d.ts");

    for (schema_name, type_name, module) in PARTS {
        let rust =
            fs::read_to_string(expected.join(format!("generated/msg/fixture_msgs/{module}.rs")))
                .expect("read the Rust type of a part");
        assert!(
            rust.contains(&format!("pub struct {type_name} ")),
            "no Rust type {type_name}"
        );
        assert!(rust.contains(&format!(
            "const SCHEMA_NAME: &'static str = \"{schema_name}\";"
        )));
        assert!(
            messages.contains(&format!("  \"{schema_name}\": {type_name};\n")),
            "MessageBySchema has no {schema_name}"
        );
        assert!(messages.contains(&format!("export interface {type_name} ")));
    }
    let empty_goal = fs::read_to_string(expected.join("generated/msg/fixture_msgs/drain_goal.rs"))
        .expect("read");
    assert!(empty_goal.contains("pub struct DrainGoal {}"));
    assert!(messages.contains("export interface DrainGoal {}\n"));
}

#[test]
fn a_uint8_sequence_is_a_uint8_array_in_typescript() {
    let messages = fs::read_to_string(fixture_root().join("expected/typescript/messages.d.ts"))
        .expect("read messages.d.ts");

    assert!(messages.contains("  samples: Uint8Array;\n"));
    assert!(messages.contains("  tag: Uint8Array;\n"));
    assert!(messages.contains("  counts: number[];\n"));
}

#[test]
fn a_uint8_sequence_is_written_and_read_whole_in_rust() {
    let samples =
        fs::read_to_string(fixture_root().join("expected/generated/msg/fixture_msgs/samples.rs"))
            .expect("read samples.rs");

    assert!(samples.contains("writer.write_bytes(&self.samples)?;"));
    assert!(samples.contains("reader.read_bytes(length as usize)?.to_vec()"));
    assert!(samples.contains("writer.write_u16(*element)?;"));
}

#[test]
fn the_schema_text_of_an_interface_lists_every_part() {
    let lookup =
        fs::read_to_string(fixture_root().join("expected/generated/mod.rs")).expect("read mod.rs");
    let progress =
        "MSG: fixture_msgs/Progress\n# fixture_msgs/msg/Progress\nuint64 done\nuint64 total";
    let separator =
        "================================================================================";
    let measure = format!(
        "# fixture_msgs/srv/Measure\nstring probe\n---\nfloat32 level\nProgress progress\n{separator}\n{progress}"
    );
    let drain = format!(
        "# fixture_msgs/action/Drain\n# An empty Goal: draining needs no input.\n---\nfloat32 drained\n---\nProgress \
         progress\n{separator}\n{progress}"
    );

    for (schema_name, text) in [
        ("fixture_msgs/srv/Measure", measure),
        ("fixture_msgs/action/Drain", drain),
    ] {
        let arm = lookup
            .split_once(&format!("\"{schema_name}\" => Some("))
            .map(|(_, arm)| arm.trim_start())
            .expect("schema has an arm for the interface type");
        assert!(
            arm.starts_with(&format!("{text:?}")),
            "{schema_name}: {arm}"
        );
    }
}

#[test]
fn the_lock_has_one_line_per_part() {
    assert_eq!(
        lock_lines(),
        [
            "fixture_msgs/action/Drain_Feedback 1 progress:fixture_msgs/Progress",
            "fixture_msgs/action/Drain_Goal 1",
            "fixture_msgs/action/Drain_Result 1 drained:float32",
            "fixture_msgs/action/Fill_Feedback 1 progress:fixture_msgs/Progress",
            "fixture_msgs/action/Fill_Goal 1 level:float32;rate:float32",
            "fixture_msgs/action/Fill_Result 1 reached:bool",
            "fixture_msgs/msg/Progress 1 done:uint64;total:uint64",
            "fixture_msgs/msg/Samples 1 samples:uint8[];tag:uint8[4];counts:uint16[]",
            "fixture_msgs/srv/Measure_Request 1 probe:string",
            "fixture_msgs/srv/Measure_Response 1 level:float32;progress:fixture_msgs/Progress",
        ]
    );
}

#[test]
fn the_lock_comparison_rejects_a_non_append_change_to_a_goal() {
    let current = collect_messages_for_test(&fixture_root().join("interfaces")).expect("parse");
    let mut locked = locked();
    assert_eq!(check_message_lock(&locked, &current), Ok(()));

    locked.insert(
        "fixture_msgs/action/Fill_Goal".to_owned(),
        (1, "level:float64;rate:float32".to_owned()),
    );
    let reason =
        check_message_lock(&locked, &current).expect_err("a retyped Goal field is rejected");

    assert!(reason.contains("fixture_msgs/action/Fill_Goal"), "{reason}");
    assert!(reason.contains("breaking"), "{reason}");
}

#[test]
fn the_lock_comparison_treats_a_goal_as_top_level() {
    let current = collect_messages_for_test(&fixture_root().join("interfaces")).expect("parse");
    let mut locked = locked();
    locked.insert(
        "fixture_msgs/action/Fill_Goal".to_owned(),
        (1, "level:float32".to_owned()),
    );

    let reason = check_message_lock(&locked, &current).expect_err("the lock must be refreshed");

    assert!(reason.contains("append-only"), "{reason}");
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/interface_kinds")
}

fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut directories = vec![root.to_path_buf()];
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(&directory).expect("read directory") {
            let path = entry.expect("directory entry").path();
            if path.is_dir() {
                directories.push(path);
            } else {
                let relative = path.strip_prefix(root).expect("under root").to_path_buf();
                files.insert(relative, fs::read(&path).expect("read file"));
            }
        }
    }
    files
}

fn lock_lines() -> Vec<String> {
    collect_messages_for_test(&fixture_root().join("interfaces"))
        .expect("parse")
        .iter()
        .map(|record| format_lock_line(&record.schema_name, 1, &record.field_signature))
        .collect()
}

fn locked() -> BTreeMap<String, (u32, String)> {
    lock_lines()
        .iter()
        .map(|line| {
            let (schema_name, major, field_signature) = parse_lock_line(line).expect("lock line");
            (schema_name, (major, field_signature))
        })
        .collect()
}
