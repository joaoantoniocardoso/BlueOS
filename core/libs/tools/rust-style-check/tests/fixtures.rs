use std::path::Path;

use blueos_rust_style_check::check_source;

fn fixture(rule: &str, outcome: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(rule)
        .join(format!("{outcome}.rs"));
    std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read fixture {}: {error}", path.display());
    })
}

fn assert_rule(rule: &str, outcome: &str, should_fail: bool) {
    let source = fixture(rule, outcome);
    let path = Path::new("fixture.rs");
    let diagnostics = check_source(&source, path).expect("fixture must parse");
    let matches = diagnostics.iter().any(|diagnostic| diagnostic.rule == rule);
    if should_fail {
        assert!(
            matches,
            "expected rule `{rule}` to fail on {outcome} fixture"
        );
    } else {
        assert!(
            !matches,
            "expected rule `{rule}` to pass on {outcome} fixture, got: {diagnostics:?}"
        );
    }
}

#[test]
fn import_groups_passes() {
    assert_rule("import_groups", "pass", false);
}

#[test]
fn import_groups_fails() {
    assert_rule("import_groups", "fail", true);
}

#[test]
fn import_chaining_passes() {
    assert_rule("import_chaining", "pass", false);
}

#[test]
fn import_chaining_fails() {
    assert_rule("import_chaining", "fail", true);
}

#[test]
fn declaration_order_passes() {
    assert_rule("declaration_order", "pass", false);
}

#[test]
fn declaration_order_fails() {
    assert_rule("declaration_order", "fail", true);
}

#[test]
fn structured_logging_passes() {
    assert_rule("structured_logging", "pass", false);
}

#[test]
fn structured_logging_fails() {
    assert_rule("structured_logging", "fail", true);
}

#[test]
fn clone_before_spawn_passes() {
    assert_rule("clone_before_spawn", "pass", false);
}

#[test]
fn clone_before_spawn_fails() {
    assert_rule("clone_before_spawn", "fail", true);
}

#[test]
fn allow_attributes_passes() {
    assert_rule("allow_attributes", "pass", false);
}

#[test]
fn allow_attributes_fails() {
    assert_rule("allow_attributes", "fail", true);
}
