#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
REPORT_SCRIPT="$ROOT_DIR/.github/lib/blueos_binary_size_report.sh"

fail() {
    printf 'blueos_binary_size_report_test: %s\n' "$1" >&2
    exit 1
}

test_reports_stripped_and_zstd_sizes() {
    local fixture_directory
    fixture_directory=$(mktemp -d)
    # Known payload: sizes are fixed for this byte sequence at zstd default level.
    printf '%s' 'blueos-binary-size-report-fixture-125' >"$fixture_directory/blueos"
    local stripped_bytes
    stripped_bytes=$(wc -c <"$fixture_directory/blueos" | tr -d ' ')
    local zstd_bytes
    zstd_bytes=$(zstd -q -c "$fixture_directory/blueos" | wc -c | tr -d ' ')
    local output
    output=$("$REPORT_SCRIPT" "$fixture_directory/blueos" "fixture-target")
    printf '%s\n' "$output" | grep -Fq "target: fixture-target" \
        || fail "expected target line in output: $output"
    printf '%s\n' "$output" | grep -Fq "stripped_bytes: $stripped_bytes" \
        || fail "expected stripped_bytes $stripped_bytes in output: $output"
    printf '%s\n' "$output" | grep -Fq "zstd_bytes: $zstd_bytes" \
        || fail "expected zstd_bytes $zstd_bytes in output: $output"
    rm -rf "$fixture_directory"
}

test_missing_binary_fails() {
    if "$REPORT_SCRIPT" /tmp/blueos-binary-size-report-missing 2>/dev/null; then
        fail "missing binary should fail"
    fi
}

test_reports_stripped_and_zstd_sizes
test_missing_binary_fails

printf 'blueos_binary_size_report_test: ok\n'
