#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
# shellcheck disable=SC1091
source "$ROOT_DIR/.github/lib/rust_report_only.sh"

TMPDIR=$(mktemp -d)
export TMPDIR
trap 'rm -rf "$TMPDIR"' EXIT

fail() {
    printf 'report_section_test: %s\n' "$1" >&2
    exit 1
}

assert_summary_contains() {
    local label="$1"
    local expected="$2"
    if ! grep -Fqx -- "$expected" "$GITHUB_STEP_SUMMARY"; then
        printf 'report_section_test: %s: expected line %q in:\n%s\n' "$label" "$expected" \
            "$(cat "$GITHUB_STEP_SUMMARY")" >&2
        exit 1
    fi
}

assert_summary_lacks() {
    local label="$1"
    local unexpected="$2"
    if grep -Fq -- "$unexpected" "$GITHUB_STEP_SUMMARY"; then
        printf 'report_section_test: %s: did not expect %q in:\n%s\n' "$label" "$unexpected" \
            "$(cat "$GITHUB_STEP_SUMMARY")" >&2
        exit 1
    fi
}

# Usage: step_fails <script>
# Runs the script as a workflow step runs it, under bash -e, which a caller's `if` would turn off.
step_fails() {
    ! bash -ec "source \"\$1\"; $1" _ "$ROOT_DIR/.github/lib/rust_report_only.sh" >/dev/null 2>&1
}

test_accepted_exit_code_passes_and_writes_the_summary() {
    export GITHUB_STEP_SUMMARY="$TMPDIR/accepted.md"
    if ! report_section "finder" --ok "0 1" --version echo "finder 1.2.3" -- \
        bash -c 'echo "3 findings"; exit 1' >/dev/null; then
        fail "accepted exit code: expected success"
    fi
    assert_summary_contains "accepted exit code" "## finder"
    assert_summary_contains "accepted exit code" "Version: finder 1.2.3"
    assert_summary_contains "accepted exit code" "3 findings"
    assert_summary_contains "accepted exit code" "Exit code: 1 (accepted: 0 1)"
}

test_unexpected_exit_code_fails_after_writing_the_summary() {
    export GITHUB_STEP_SUMMARY="$TMPDIR/unexpected.md"
    if ! step_fails 'report_section "broken tool" --ok 0 --version echo "broken 0.1" -- \
        bash -c "echo \"cannot parse manifest\"; exit 101"'; then
        fail "unexpected exit code: expected failure"
    fi
    assert_summary_contains "unexpected exit code" "## broken tool"
    assert_summary_contains "unexpected exit code" "Version: broken 0.1"
    assert_summary_contains "unexpected exit code" "cannot parse manifest"
    assert_summary_contains "unexpected exit code" "Exit code: 101 (accepted: 0)"
}

test_every_section_runs_before_the_step_fails() {
    export GITHUB_STEP_SUMMARY="$TMPDIR/every.md"
    if ! step_fails 'report_section "first" --ok 0 --version echo "first 1" -- false
        report_section "second" --ok 0 --version echo "second 1" -- echo "second ran"'; then
        fail "every section: expected the step to fail"
    fi
    assert_summary_contains "every section" "Exit code: 1 (accepted: 0)"
    assert_summary_contains "every section" "second ran"
    assert_summary_contains "every section" "Exit code: 0 (accepted: 0)"
}

test_failing_command_inside_a_function_fails_the_section() {
    export GITHUB_STEP_SUMMARY="$TMPDIR/function.md"
    if ! step_fails 'measure() { false; echo "measured nothing"; }
        report_section "function" --ok 0 --version echo "function 1" -- measure'; then
        fail "failing command in a function: expected failure"
    fi
    assert_summary_lacks "failing command in a function" "measured nothing"
    assert_summary_contains "failing command in a function" "Exit code: 1 (accepted: 0)"
}

test_failing_version_command_fails_the_step() {
    export GITHUB_STEP_SUMMARY="$TMPDIR/version.md"
    if ! step_fails 'report_section "missing tool" --ok 0 --version missing-tool --version -- missing-tool'; then
        fail "failing version command: expected failure"
    fi
    assert_summary_contains "failing version command" "The version command failed:"
}

test_section_without_a_tool_prints_no_version() {
    export GITHUB_STEP_SUMMARY="$TMPDIR/no-version.md"
    report_section "script" --ok 0 --no-version -- echo "measured" >/dev/null
    assert_summary_lacks "no version" "Version:"
    assert_summary_contains "no version" "measured"
    assert_summary_contains "no version" "Exit code: 0 (accepted: 0)"
}

test_tool_output_reaches_the_summary_as_plain_ascii() {
    export GITHUB_STEP_SUMMARY="$TMPDIR/plain.md"
    report_section "colourful" --ok 0 --version echo "colourful 1" -- \
        printf '\033[1;92m   Compiling\033[0m blueos\n\360\237\223\201 Report written \342\234\224\n' >/dev/null
    assert_summary_contains "plain ascii" "   Compiling blueos"
    if LC_ALL=C grep -q $'[\x1b\x80-\xff]' "$GITHUB_STEP_SUMMARY"; then
        fail "plain ascii: the summary keeps an escape code or a non-ASCII character: $(cat -v "$GITHUB_STEP_SUMMARY")"
    fi
}

test_usage_errors_fail() {
    export GITHUB_STEP_SUMMARY="$TMPDIR/usage.md"
    local label arguments
    while IFS='|' read -r label arguments; do
        # shellcheck disable=SC2086
        if report_section "usage" $arguments >/dev/null 2>&1; then
            fail "usage: $label: expected a usage error"
        fi
    done <<'EOF'
no version|--ok 0 -- true
no separator|--ok 0 --version true
no command|--ok 0 --version true --
bare ok|--ok
ok without codes|--ok -- true
no ok|--version true -- true
EOF
}

main() {
    test_accepted_exit_code_passes_and_writes_the_summary
    test_unexpected_exit_code_fails_after_writing_the_summary
    test_every_section_runs_before_the_step_fails
    test_failing_command_inside_a_function_fails_the_section
    test_failing_version_command_fails_the_step
    test_section_without_a_tool_prints_no_version
    test_tool_output_reaches_the_summary_as_plain_ascii
    test_usage_errors_fail
    printf 'report_section_test: ok\n'
}

main "$@"
