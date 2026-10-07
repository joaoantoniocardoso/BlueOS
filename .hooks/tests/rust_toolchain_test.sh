#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
CORE_DIR="$ROOT_DIR/core"
WORKFLOW="$ROOT_DIR/.github/workflows/test-and-deploy.yml"
CARGO_CONFIG="$CORE_DIR/.cargo/config.toml"
TOOLCHAIN_FILE="$CORE_DIR/rust-toolchain.toml"
READ_SCRIPT="$CORE_DIR/read_rust_toolchain.sh"
WRITE_OUTPUTS="$ROOT_DIR/.github/lib/write_stable_rust_toolchain_outputs.sh"

fail() {
    printf 'rust_toolchain_test: %s\n' "$1" >&2
    exit 1
}

test_rust_toolchain_file_exists() {
    [ -f "$TOOLCHAIN_FILE" ] || fail "missing core/rust-toolchain.toml"
}

test_read_script_matches_toolchain_file() {
    [ -x "$READ_SCRIPT" ] || fail "core/read_rust_toolchain.sh must be executable"
    # shellcheck disable=SC1090
    source "$READ_SCRIPT"
    local expected_channel
    expected_channel=$(grep -E '^channel\s*=' "$TOOLCHAIN_FILE" | sed -E 's/^[^"]*"([^"]+)".*$/\1/')
    [ "$RUST_TOOLCHAIN_CHANNEL" = "$expected_channel" ] \
        || fail "channel mismatch: got $RUST_TOOLCHAIN_CHANNEL, want $expected_channel"
    [ -n "$RUST_TOOLCHAIN_COMPONENTS" ] || fail "RUST_TOOLCHAIN_COMPONENTS is empty"
    [ -n "$RUST_TOOLCHAIN_TARGETS" ] || fail "RUST_TOOLCHAIN_TARGETS is empty"
}

test_write_outputs_matches_read_script() {
    [ -x "$WRITE_OUTPUTS" ] || fail "write_stable_rust_toolchain_outputs.sh must be executable"
    local output
    output=$(GITHUB_OUTPUT=/dev/stdout "$WRITE_OUTPUTS")
    # shellcheck disable=SC1090
    source "$READ_SCRIPT"
    grep -qF "channel=$RUST_TOOLCHAIN_CHANNEL" <<<"$output" \
        || fail "GITHUB_OUTPUT channel does not match read_rust_toolchain.sh"
    grep -qF "components=$RUST_TOOLCHAIN_COMPONENTS" <<<"$output" \
        || fail "GITHUB_OUTPUT components do not match read_rust_toolchain.sh"
    grep -qF "targets=$RUST_TOOLCHAIN_TARGETS" <<<"$output" \
        || fail "GITHUB_OUTPUT targets do not match read_rust_toolchain.sh"
}

test_ci_does_not_install_floating_stable() {
    if grep -q 'dtolnay/rust-toolchain@stable' "$WORKFLOW"; then
        fail "test-and-deploy.yml still uses dtolnay/rust-toolchain@stable; CI must use core/rust-toolchain.toml (D-33)"
    fi
    if ! grep -q 'pinned-rust-toolchain' "$WORKFLOW"; then
        fail "test-and-deploy.yml must install Rust through the pinned-rust-toolchain action"
    fi
}

test_blueos_release_alias_matches_shipped_features() {
    [ -f "$CARGO_CONFIG" ] || fail "missing core/.cargo/config.toml"
    # shellcheck disable=SC1091
    source "$CORE_DIR/shipped_features.sh"
    local alias_line
    alias_line=$(grep -E '^blueos-release\s*=' "$CARGO_CONFIG" || true)
    [ -n "$alias_line" ] || fail "cargo alias blueos-release is missing"
    grep -qF -- '-p blueos' "$CARGO_CONFIG" || fail "blueos-release alias must pass -p blueos"
    grep -qF -- '--release' "$CARGO_CONFIG" || fail "blueos-release alias must pass --release"
    grep -qF -- '--locked' "$CARGO_CONFIG" || fail "blueos-release alias must pass --locked"
    grep -qF -- "recorder" "$CARGO_CONFIG" || fail "blueos-release alias must enable the shipped recorder feature"
}

test_rust_toolchain_file_exists
test_read_script_matches_toolchain_file
test_write_outputs_matches_read_script
test_ci_does_not_install_floating_stable
test_blueos_release_alias_matches_shipped_features
