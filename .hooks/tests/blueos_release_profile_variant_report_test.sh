#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
REPORT_SCRIPT="$ROOT_DIR/.github/lib/blueos_release_profile_variant_report.sh"

fail() {
    printf 'blueos_release_profile_variant_report_test: %s\n' "$1" >&2
    exit 1
}

test_usage_fails() {
    if "$REPORT_SCRIPT" 2>/dev/null; then
        fail "missing arguments should fail"
    fi
}

test_unknown_variant_fails() {
    if "$REPORT_SCRIPT" x86_64-unknown-linux-musl not-a-variant 2>/dev/null; then
        fail "unknown variant should fail"
    fi
}

test_usage_fails
test_unknown_variant_fails

printf 'blueos_release_profile_variant_report_test: ok\n'
