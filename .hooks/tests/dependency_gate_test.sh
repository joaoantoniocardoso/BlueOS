#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
# shellcheck disable=SC1091
source "$ROOT_DIR/.hooks/lib/rust_checks.sh"

TMPDIR=$(mktemp -d)
export TMPDIR
trap 'rm -rf "$TMPDIR"' EXIT

fail() {
    printf 'dependency_gate_test: %s\n' "$1" >&2
    exit 1
}

# Usage: write_workspace <workspace-dependencies> <member-dependency-sections> [exceptions]
# Creates a one-member workspace under $TMPDIR/workspace and prints its cargo-metadata-shaped JSON.
write_workspace() {
    local workspace_dependencies="$1"
    local member_sections="$2"
    local exceptions="${3:-}"
    local workspace_dir="$TMPDIR/workspace"
    rm -rf "$workspace_dir"
    mkdir -p "$workspace_dir/member"
    printf '[workspace]\nmembers = ["member"]\n\n[workspace.dependencies]\n%s\n' \
        "$workspace_dependencies" >"$workspace_dir/Cargo.toml"
    printf '[package]\nname = "member"\n\n%s\n' "$member_sections" >"$workspace_dir/member/Cargo.toml"
    printf '%s\n' "$exceptions" >"$workspace_dir/exceptions.toml"
    jq -n --arg manifest "$workspace_dir/member/Cargo.toml" \
        '{packages: [{name: "member", manifest_path: $manifest}]}'
}

assert_gate_fails() {
    local label="$1"
    local pattern="$2"
    local metadata="$3"
    local output
    if output=$(collect_dependency_violations "$metadata" "$TMPDIR/workspace/Cargo.toml" \
        "$TMPDIR/workspace/exceptions.toml"); then
        fail "$label: expected a violation"
    fi
    if ! grep -q "$pattern" <<<"$output"; then
        printf 'dependency_gate_test: %s: expected pattern %q in:\n%s\n' "$label" "$pattern" "$output" >&2
        exit 1
    fi
}

assert_gate_passes() {
    local label="$1"
    local metadata="$2"
    local output
    if ! output=$(collect_dependency_violations "$metadata" "$TMPDIR/workspace/Cargo.toml" \
        "$TMPDIR/workspace/exceptions.toml"); then
        printf 'dependency_gate_test: %s: expected no violation, got:\n%s\n' "$label" "$output" >&2
        exit 1
    fi
}

test_rejects_normal_dependency_outside_workspace() {
    local metadata
    metadata=$(write_workspace 'anyhow = { version = "1", default-features = false }' \
        '[dependencies]
anyhow = "1"')
    assert_gate_fails "normal dependency" 'member: \[dependencies\] anyhow' "$metadata"
}

test_rejects_dev_dependency_outside_workspace() {
    local metadata
    metadata=$(write_workspace 'anyhow = { version = "1", default-features = false }' \
        '[dev-dependencies]
anyhow = { version = "1", default-features = false }')
    assert_gate_fails "dev dependency" 'member: \[dev-dependencies\] anyhow' "$metadata"
}

test_rejects_build_dependency_outside_workspace() {
    local metadata
    metadata=$(write_workspace 'anyhow = { version = "1", default-features = false }' \
        '[build-dependencies]
anyhow = "1"')
    assert_gate_fails "build dependency" 'member: \[build-dependencies\] anyhow' "$metadata"
}

test_rejects_target_dependency_outside_workspace() {
    local metadata
    metadata=$(write_workspace 'anyhow = { version = "1", default-features = false }' \
        '[target.'"'"'cfg(unix)'"'"'.dev-dependencies]
anyhow = "1"')
    assert_gate_fails "target dependency" 'member: \[dev-dependencies\] anyhow' "$metadata"
}

test_rejects_workspace_entry_with_default_features() {
    local metadata
    metadata=$(write_workspace 'serde_json = "1"' '[dependencies]
serde_json.workspace = true')
    assert_gate_fails "bare version" 'workspace: serde_json keeps default features' "$metadata"

    metadata=$(write_workspace 'serde_json = { version = "1", default-features = true }' '')
    assert_gate_fails "explicit true" 'workspace: serde_json keeps default features' "$metadata"

    metadata=$(write_workspace 'blueos-thing = { path = "thing" }' '')
    assert_gate_fails "path entry" 'workspace: blueos-thing keeps default features' "$metadata"
}

test_accepts_workspace_inheritance_with_default_features_off() {
    local metadata
    metadata=$(write_workspace 'serde_json = { version = "1", default-features = false }' \
        '[dependencies]
serde_json.workspace = true

[dev-dependencies]
serde_json = { workspace = true, features = ["alloc"] }')
    assert_gate_passes "clean workspace" "$metadata"
}

test_exception_with_reason_is_accepted() {
    local metadata
    metadata=$(write_workspace 'mcap = "0.25"' '' '[exceptions]
mcap = "The crate has no feature that turns its compression backends off"')
    assert_gate_passes "exception with reason" "$metadata"
}

test_exception_without_reason_is_rejected() {
    local metadata
    metadata=$(write_workspace 'mcap = "0.25"' '' '[exceptions]
mcap = ""')
    assert_gate_fails "empty reason" 'exceptions: mcap has no reason' "$metadata"

    metadata=$(write_workspace 'mcap = "0.25"' '' '[exceptions]
mcap = true')
    assert_gate_fails "non-string reason" 'exceptions: mcap has no reason' "$metadata"
}

test_stale_exception_is_rejected() {
    local metadata
    metadata=$(write_workspace 'mcap = { version = "0.25", default-features = false }' '' '[exceptions]
mcap = "Left over from before the features were turned off"')
    assert_gate_fails "stale exception" 'exceptions: mcap is listed but' "$metadata"
}

test_repository_is_clean() {
    local output
    if ! output=$(check_dependency_gate "$ROOT_DIR/core" 2>&1); then
        printf 'dependency_gate_test: the repository violates the dependency gate:\n%s\n' "$output" >&2
        exit 1
    fi
}

main() {
    test_rejects_normal_dependency_outside_workspace
    test_rejects_dev_dependency_outside_workspace
    test_rejects_build_dependency_outside_workspace
    test_rejects_target_dependency_outside_workspace
    test_rejects_workspace_entry_with_default_features
    test_accepts_workspace_inheritance_with_default_features_off
    test_exception_with_reason_is_accepted
    test_exception_without_reason_is_rejected
    test_stale_exception_is_rejected
    test_repository_is_clean
    printf 'dependency_gate_test: ok\n'
}

main "$@"
