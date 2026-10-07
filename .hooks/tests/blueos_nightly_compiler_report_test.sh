#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
REPORT_SCRIPT="$ROOT_DIR/.github/lib/blueos_nightly_compiler_report.sh"

fail() {
    printf 'blueos_nightly_compiler_report_test: %s\n' "$1" >&2
    exit 1
}

test_usage_fails() {
    if "$REPORT_SCRIPT" 2>/dev/null; then
        fail "missing arguments should fail"
    fi
}

test_unknown_mode_fails() {
    if "$REPORT_SCRIPT" not-a-mode /tmp 2>/dev/null; then
        fail "unknown mode should fail"
    fi
}

test_usage_fails
test_unknown_mode_fails

printf 'blueos_nightly_compiler_report_test: ok\n'
