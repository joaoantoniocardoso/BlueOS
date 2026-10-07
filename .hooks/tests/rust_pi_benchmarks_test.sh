#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
WORKFLOW="$ROOT_DIR/.github/workflows/rust-pi-benchmarks.yml"
RUNNER_SCRIPT="$ROOT_DIR/.github/lib/rust_pi_benchmarks.sh"

fail() {
    printf 'rust_pi_benchmarks_test: %s\n' "$1" >&2
    exit 1
}

test_workflow_and_script_exist() {
    [ -f "$WORKFLOW" ] || fail "missing $WORKFLOW"
    [ -x "$RUNNER_SCRIPT" ] || fail "$RUNNER_SCRIPT must be executable"
}

test_master_push_only_on_upstream() {
    grep -qE '^[[:space:]]+branches:[[:space:]]*$' "$WORKFLOW" \
        || fail "workflow must list master under push.branches"
    grep -qE '^[[:space:]]+- master$' "$WORKFLOW" \
        || fail "workflow must trigger on pushes to master"
    if grep -q '^on:' "$WORKFLOW" && grep -q 'pull_request' "$WORKFLOW"; then
        fail "Pi benchmarks must not run on pull requests"
    fi
    grep -q "github.repository_owner == 'bluerobotics'" "$WORKFLOW" \
        || fail "Pi benchmarks must run only on the upstream repository"
}

test_runner_is_the_reference_pi() {
    grep -q 'runs-on: pi4-builder2' "$WORKFLOW" \
        || fail "job must run on the reference Pi runner"
}

test_pinned_toolchain_and_artifacts() {
    grep -q 'pinned-rust-toolchain' "$WORKFLOW" \
        || fail "Pi benchmarks must use the pinned Rust toolchain (D-33)"
    grep -q 'upload-artifact' "$WORKFLOW" \
        || fail "Pi benchmark results must be uploaded as artifacts"
    grep -q 'rust_pi_benchmarks.sh' "$WORKFLOW" \
        || fail "workflow must call rust_pi_benchmarks.sh"
}

test_workflow_and_script_exist
test_master_push_only_on_upstream
test_runner_is_the_reference_pi
test_pinned_toolchain_and_artifacts
