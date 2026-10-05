#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
# shellcheck disable=SC1091
source "$ROOT_DIR/.hooks/lib/rust_checks.sh"

# A failing test exits before its own cleanup, so every temporary copy lives under one root removed on exit.
TMPDIR=$(mktemp -d)
export TMPDIR
trap 'rm -rf "$TMPDIR"' EXIT

# The build folder and node_modules can weigh gigabytes and no test reads them.
copy_core() {
    tar -C "$ROOT_DIR/core" --exclude=./target --exclude=./frontend/node_modules -cf - . | tar -C "$1" -xf -
}

fail() {
    printf 'rust_checks_test: %s\n' "$1" >&2
    exit 1
}

assert_violation() {
    local label="$1"
    local metadata="$2"
    local pattern="$3"
    local output
    if output=$(collect_folder_violations "$metadata"); then
        fail "$label: expected a folder violation"
    fi
    if ! grep -q "$pattern" <<<"$output"; then
        printf 'rust_checks_test: %s: expected pattern %q in:\n%s\n' "$label" "$pattern" "$output" >&2
        exit 1
    fi
}

assert_clean_metadata() {
    local label="$1"
    local metadata="$2"
    if ! collect_folder_violations "$metadata"; then
        fail "$label: expected no folder violations"
    fi
}

assert_style_drift() {
    local label="$1"
    local repository_dir="$2"
    local pattern="$3"
    local output
    if output=$(check_rust_style_copies "$repository_dir" 2>&1); then
        fail "$label: expected the Rust checklist drift check to fail"
    fi
    if ! grep -q "$pattern" <<<"$output"; then
        printf 'rust_checks_test: %s: expected pattern %q in:\n%s\n' "$label" "$pattern" "$output" >&2
        exit 1
    fi
}

test_crate_place() {
    local unit folder
    read -r unit folder <<<"$(crate_place "$ROOT_DIR/core/libs/logic/domain")"
    [ "$unit" = libs ] && [ "$folder" = logic ] || fail "crate_place for libs/logic"

    read -r unit folder <<<"$(crate_place "$ROOT_DIR/core/services/recorder/adapters/mcap")"
    [ "$unit" = recorder ] && [ "$folder" = adapters ] || fail "crate_place for service adapter"

    read -r unit folder <<<"$(crate_place "$ROOT_DIR/core/services/recorder/logic/api")"
    [ "$unit" = recorder ] && [ "$folder" = api ] || fail "crate_place for service logic/api"

    read -r unit folder <<<"$(crate_place "$ROOT_DIR/core/app/blueos")"
    [ "$unit" = multicall ] && [ "$folder" = app ] || fail "crate_place for multicall"
}

test_folder_rejects_logic_depending_on_adapter() {
    local metadata
    metadata=$(cat <<'EOF'
{
  "packages": [
    {
      "name": "blueos-bad-logic",
      "manifest_path": "/repo/core/libs/logic/bad/Cargo.toml",
      "dependencies": [
        { "name": "blueos-cli", "kind": null, "path": "/repo/core/libs/adapters/cli" }
      ]
    },
    {
      "name": "blueos-cli",
      "manifest_path": "/repo/core/libs/adapters/cli/Cargo.toml",
      "dependencies": []
    }
  ]
}
EOF
)
    assert_violation "logic to adapter" "$metadata" 'logic, so it may only depend on libs/logic'
}

test_folder_rejects_logic_api_depending_on_adapter() {
    local metadata
    metadata=$(cat <<'EOF'
{
  "packages": [
    {
      "name": "blueos-example-api",
      "manifest_path": "/repo/core/services/example/logic/api/Cargo.toml",
      "dependencies": [
        { "name": "blueos-idl", "kind": null, "path": "/repo/core/libs/idl" },
        { "name": "blueos-example-domain", "kind": null, "path": "/repo/core/services/example/logic/domain" },
        { "name": "blueos-cli", "kind": null, "path": "/repo/core/libs/adapters/cli" }
      ]
    }
  ]
}
EOF
)
    assert_violation "logic/api to adapter" "$metadata" 'logic/api, so it may only depend on libs/logic, blueos-idl'
    if [ "$(collect_folder_violations "$metadata" | wc -l)" -ne 1 ]; then
        fail "logic/api may depend on blueos-idl and its own Domain"
    fi
}

test_folder_rejects_cross_service_dependency() {
    local metadata
    metadata=$(cat <<'EOF'
{
  "packages": [
    {
      "name": "blueos-example-app",
      "manifest_path": "/repo/core/services/example/app/Cargo.toml",
      "dependencies": [
        { "name": "blueos-recorder", "kind": null, "path": "/repo/core/services/recorder/app" }
      ]
    },
    {
      "name": "blueos-recorder",
      "manifest_path": "/repo/core/services/recorder/app/Cargo.toml",
      "dependencies": []
    }
  ]
}
EOF
)
    assert_violation "cross service" "$metadata" 'belongs to another service'
}

test_workspace_metadata_is_clean() {
    local workspace_dir="$ROOT_DIR/core"
    local metadata
    metadata=$(cargo metadata --format-version 1 --no-deps --locked --manifest-path "$workspace_dir/Cargo.toml")
    assert_clean_metadata "workspace" "$metadata"
}

test_app_src_rejects_unknown_top_level_module() {
    local temporary output
    temporary=$(mktemp -d)
    copy_core "$temporary"
    printf '\n' >>"$temporary/services/recorder/app/src/library_io.rs"
    if output=$(collect_app_src_violations "$temporary"); then
        fail "app/src layout check should reject library_io.rs"
    fi
    if ! grep -q 'unknown top-level module library_io' <<<"$output"; then
        printf 'rust_checks_test: expected library_io violation in:\n%s\n' "$output" >&2
        exit 1
    fi
    rm -rf "$temporary"
}

test_app_src_allows_recorder_layout() {
    if ! collect_app_src_violations "$ROOT_DIR"; then
        fail "recorder app/src layout should pass the folder check"
    fi
}

test_fmt_check_fails_on_unformatted_source() {
    local temporary
    temporary=$(mktemp -d)
    copy_core "$temporary"
    printf '\n\npub const UNFORMATTED:()=();\n' >>"$temporary/libs/logic/domain/src/lib.rs"
    if (
        cd "$temporary"
        cargo fmt --all --check >/dev/null 2>&1
    ); then
        fail "cargo fmt --check should fail on unformatted source"
    fi
    rm -rf "$temporary"
}

test_syn_style_check_fails_on_mixed_import_groups() {
    local temporary
    temporary=$(mktemp -d)
    copy_core "$temporary"
    cat >>"$temporary/libs/logic/domain/Cargo.toml" <<'EOF'

[dependencies]
convert_case.workspace = true
EOF
    cat >>"$temporary/libs/logic/domain/src/lib.rs" <<'EOF'

use std::path::PathBuf;
use convert_case::Case;
EOF
    if (
        cd "$temporary"
        cargo run --locked -q -p blueos-rust-style-check -- . 2>/dev/null
    ); then
        fail "syn style check should reject mixed import groups"
    fi
    rm -rf "$temporary"
}

test_shipped_clippy_rejects_item_used_only_under_non_shipped_feature() {
    local temporary output
    temporary=$(mktemp -d)
    copy_core "$temporary"
    cat >>"$temporary/libs/app/service/src/lib.rs" <<'EOF'

use core::sync::atomic::AtomicUsize;

/// Used only when the non-shipped testing feature is enabled.
#[cfg(feature = "testing")]
#[must_use]
pub fn planted_non_shipped_item() -> usize {
    AtomicUsize::new(1).load(core::sync::atomic::Ordering::Relaxed)
}
EOF
    if ! output=$(
        cd "$temporary"
        cargo clippy --locked -p blueos-service --all-features -- -D warnings 2>&1
    ); then
        printf '%s\n' "$output" >&2
        fail "all-features clippy should pass an item used under a non-shipped feature"
    fi
    if output=$(
        cd "$temporary"
        run_shipped_clippy 2>&1
    ); then
        fail "shipped clippy should reject an item used only under a non-shipped feature"
    fi
    if ! grep -q 'unused import' <<<"$output"; then
        printf '%s\n' "$output" >&2
        fail "shipped clippy should report the unused import"
    fi
    rm -rf "$temporary"
}

test_clippy_fails_on_allow_attributes() {
    local temporary
    temporary=$(mktemp -d)
    copy_core "$temporary"
    cat >>"$temporary/libs/logic/domain/src/lib.rs" <<'EOF'

#[allow(dead_code)]
fn planted() {}
EOF
    if (
        cd "$temporary"
        cargo clippy --workspace --all-targets --all-features --locked -- -D warnings 2>/dev/null
    ); then
        fail "clippy should reject #[allow] (allow_attributes)"
    fi
    rm -rf "$temporary"
}

test_no_std_build_fails_on_io_dependency() {
    local temporary
    temporary=$(mktemp -d)
    copy_core "$temporary"
    sed -i '/blueos-domain = /a socket2 = "0.5"' "$temporary/Cargo.toml"
    cat >>"$temporary/libs/logic/domain/Cargo.toml" <<'EOF'

[dependencies]
socket2.workspace = true
EOF
    cargo generate-lockfile --manifest-path "$temporary/Cargo.toml" >/dev/null
    if (
        cd "$temporary"
        cargo check --locked --target "$RUST_NO_STD_TARGET" -p blueos-domain 2>/dev/null
    ); then
        fail "thumbv7em build should fail when logic depends on socket2"
    fi
    rm -rf "$temporary"
}

test_machete_fails_on_unused_dependency() {
    local temporary
    temporary=$(mktemp -d)
    copy_core "$temporary"
    sed -i '/blueos-domain = /a libc = "0.2"' "$temporary/Cargo.toml"
    cat >>"$temporary/libs/logic/domain/Cargo.toml" <<'EOF'

[dependencies]
libc.workspace = true
EOF
    cargo generate-lockfile --manifest-path "$temporary/Cargo.toml" >/dev/null
    if (
        cd "$temporary"
        cargo machete >/dev/null 2>&1
    ); then
        fail "cargo machete should reject an unused dependency"
    fi
    rm -rf "$temporary"
}

test_typos_fails_on_misspelling() {
    local temporary
    temporary=$(mktemp -d)
    cp -a "$ROOT_DIR/core/libs/logic/domain/." "$temporary/"
    printf '\nconst PLANTED_TYPO: &str = "teh";\n' >>"$temporary/src/lib.rs"
    if typos --config "$ROOT_DIR/typos.toml" "$temporary" >/dev/null 2>&1; then
        fail "typos should reject a misspelling"
    fi
    rm -rf "$temporary"
}

test_typos_checks_every_service_and_honours_its_excludes() {
    local temporary output
    temporary=$(mktemp -d)
    cp "$ROOT_DIR/typos.toml" "$temporary/"
    mkdir -p "$temporary/core/services/recorder/app/src" "$temporary/core/services/wifi" \
        "$temporary/core/libs/commonwealth"
    printf 'const PLANTED_TYPO: &str = "teh";\n' >"$temporary/core/services/recorder/app/src/lib.rs"
    printf 'PLANTED_TYPO = "teh"\n' >"$temporary/core/services/wifi/main.py"
    printf 'PLANTED_TYPO = "teh"\n' >"$temporary/core/libs/commonwealth/settings.py"
    printf '[rustqual]\nteh_warnings = 0\n' >"$temporary/core/quality-ratchet.toml"
    output=$(cd / && check_typos "$temporary" 2>&1 || true)
    if ! grep -q 'core/services/recorder/app/src/lib.rs' <<<"$output"; then
        fail "typos should check the Rust services"
    fi
    if ! grep -q 'core/services/wifi/main.py' <<<"$output"; then
        fail "typos should check the Python services"
    fi
    if ! grep -q 'core/quality-ratchet.toml' <<<"$output"; then
        fail "typos should check the quality ratchet"
    fi
    if grep -q 'core/libs/commonwealth' <<<"$output"; then
        fail "typos should skip what typos.toml excludes"
    fi
    rm -rf "$temporary"
}

test_nextest_fails_on_hanging_test() {
    local temporary
    temporary=$(mktemp -d)
    copy_core "$temporary"
    mkdir -p "$temporary/.config"
    cat >"$temporary/.config/nextest.toml" <<'EOF'
[profile.default]
slow-timeout = { period = "2s", terminate-after = 1 }
EOF
    cat >"$temporary/libs/logic/domain/tests/hanging.rs" <<'EOF'
//! Planted test that never finishes.

use core::time::Duration;

#[test]
fn sleeps_forever() {
    std::thread::sleep(Duration::from_secs(30));
}
EOF
    if (
        cd "$temporary"
        cargo nextest run --workspace --locked 2>/dev/null
    ); then
        fail "nextest should fail a test that exceeds the slow timeout"
    fi
    rm -rf "$temporary"
}

test_coverage_ratchet_fails_when_floor_is_too_high() {
    local temporary
    temporary=$(mktemp -d)
    copy_core "$temporary"
    sed -i 's/^workspace = [0-9]\+/workspace = 101/' "$temporary/coverage-ratchet.toml"
    export RUSTC_WRAPPER=
    if check_rust_coverage_ratchet "$temporary" 2>/dev/null; then
        fail "coverage ratchet should fail when the floor is above measured coverage"
    fi
    rm -rf "$temporary"
}

test_deny_licenses_rejects_unlisted_license() {
    local temporary
    temporary=$(mktemp -d)
    copy_core "$temporary"
    sed -i '/^allow = \[/,/\]/d' "$temporary/deny.toml"
    if (
        cd "$temporary"
        cargo deny check licenses >/dev/null 2>&1
    ); then
        fail "cargo deny licenses should fail when MIT is not allowed"
    fi
    rm -rf "$temporary"
}

test_style_copies_accept_repository() {
    check_rust_style_copies "$ROOT_DIR" || fail "repository checklist copies should match rust-style.md"
}

test_style_copies_reject_drifting_agents_md() {
    local temporary
    temporary=$(mktemp -d)
    mkdir -p "$temporary/docs/architecture" "$temporary/.cursor/rules"
    cp "$ROOT_DIR/docs/architecture/rust-style.md" "$temporary/docs/architecture/"
    cp "$ROOT_DIR/AGENTS.md" "$temporary/"
    cp "$ROOT_DIR/.cursor/rules/rust-blueos.mdc" "$temporary/.cursor/rules/"
    sed -i '/<!-- rust-style:begin -->/,/<!-- rust-style:end -->/s/Write the test first/Write the test second/' \
        "$temporary/AGENTS.md"
    assert_style_drift "agents.md drift" "$temporary" 'AGENTS.md'
    rm -rf "$temporary"
}

test_style_copies_reject_drifting_cursor_rule() {
    local temporary
    temporary=$(mktemp -d)
    mkdir -p "$temporary/docs/architecture" "$temporary/.cursor/rules"
    cp "$ROOT_DIR/docs/architecture/rust-style.md" "$temporary/docs/architecture/"
    cp "$ROOT_DIR/AGENTS.md" "$temporary/"
    cp "$ROOT_DIR/.cursor/rules/rust-blueos.mdc" "$temporary/.cursor/rules/"
    sed -i '/<!-- rust-style:begin -->/,/<!-- rust-style:end -->/s/Write the test first/Write the test second/' \
        "$temporary/.cursor/rules/rust-blueos.mdc"
    assert_style_drift "cursor rule drift" "$temporary" '.cursor/rules/rust-blueos.mdc'
    rm -rf "$temporary"
}

test_deny_bans_direct_zenoh() {
    local temporary
    temporary=$(mktemp -d)
    copy_core "$temporary"
    cat >>"$temporary/libs/logic/domain/Cargo.toml" <<'EOF'

[dependencies]
zenoh.workspace = true
EOF
    cargo generate-lockfile --manifest-path "$temporary/Cargo.toml" >/dev/null
    if (
        cd "$temporary"
        cargo deny check bans licenses sources >/dev/null 2>&1
    ); then
        fail "cargo deny should reject a direct zenoh dependency"
    fi
    rm -rf "$temporary"
}

test_test_only_features_stay_out_of_normal_builds() {
    if ! collect_test_only_feature_violations "$ROOT_DIR/core" >/dev/null; then
        fail "the workspace enables a test-only feature outside [dev-dependencies]"
    fi
    local temporary output
    temporary=$(mktemp -d)
    copy_core "$temporary"
    cat >>"$temporary/libs/logic/domain/Cargo.toml" <<'EOF'

[dependencies]
blueos-comms = { workspace = true, features = ["channel"] }
EOF
    if output=$(collect_test_only_feature_violations "$temporary"); then
        fail "a test-only feature in [dependencies] should be a violation"
    fi
    grep -q 'blueos-comms/channel' <<<"$output" || fail "the violation should name blueos-comms/channel"
    rm -rf "$temporary"
}

main() {
    test_crate_place
    test_style_copies_accept_repository
    test_style_copies_reject_drifting_agents_md
    test_style_copies_reject_drifting_cursor_rule
    test_folder_rejects_logic_depending_on_adapter
    test_folder_rejects_logic_api_depending_on_adapter
    test_folder_rejects_cross_service_dependency
    test_workspace_metadata_is_clean
    test_app_src_rejects_unknown_top_level_module
    test_app_src_allows_recorder_layout
    test_fmt_check_fails_on_unformatted_source
    test_syn_style_check_fails_on_mixed_import_groups
    test_shipped_clippy_rejects_item_used_only_under_non_shipped_feature
    test_clippy_fails_on_allow_attributes
    test_no_std_build_fails_on_io_dependency
    test_machete_fails_on_unused_dependency
    test_typos_fails_on_misspelling
    test_typos_checks_every_service_and_honours_its_excludes
    test_nextest_fails_on_hanging_test
    test_coverage_ratchet_fails_when_floor_is_too_high
    test_deny_licenses_rejects_unlisted_license
    test_deny_bans_direct_zenoh
    test_test_only_features_stay_out_of_normal_builds
    printf 'rust_checks_test: ok\n'
}

main "$@"
