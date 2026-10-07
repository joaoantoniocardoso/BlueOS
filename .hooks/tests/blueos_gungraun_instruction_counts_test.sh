#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
REPORT_SCRIPT="$ROOT_DIR/.github/lib/blueos_gungraun_instruction_counts.sh"

fail() {
    printf 'blueos_gungraun_instruction_counts_test: %s\n' "$1" >&2
    exit 1
}

test_summary_table_parses_instruction_rows() {
    [ -x "$REPORT_SCRIPT" ] || fail "blueos_gungraun_instruction_counts.sh must be executable"
    local fixture_output table
    fixture_output=$(cat <<'EOF'
cdr_codec_gungraun::cdr_codec::cdr_encode_command_ack
  Instructions:                1734|1800             (-3.67%)
write_sample_gungraun::write_sample::mcap_write_sample_request
  Instructions:              123456|120000             (+2.88%)
EOF
)
    export GUNGRAUN_RUNNER_LABEL=fixture-runner
    # shellcheck disable=SC1090
    source "$REPORT_SCRIPT"
    table=$(format_gungraun_instruction_summary_table "$fixture_output")
    printf '%s\n' "$table" | grep -Fq '| cdr_codec_gungraun::cdr_codec::cdr_encode_command_ack | 1800 | 1734 | (-3.67%) | fixture-runner |' \
        || fail "expected first benchmark row in table: $table"
    printf '%s\n' "$table" | grep -Fq '| write_sample_gungraun::write_sample::mcap_write_sample_request | 120000 | 123456 | (+2.88%) | fixture-runner |' \
        || fail "expected second benchmark row in table: $table"
}

test_workflow_targets_both_runners() {
    local workflow="$ROOT_DIR/.github/workflows/test-and-deploy.yml"
    grep -q 'ubuntu-latest' "$workflow" || fail "workflow must run on ubuntu-latest"
    grep -q 'ubuntu-24.04-arm' "$workflow" || fail "workflow must run on ubuntu-24.04-arm"
    grep -q 'gungraun/setup-gungraun@v1' "$workflow" || fail "workflow must install Valgrind through setup-gungraun"
}

test_summary_table_parses_instruction_rows
test_workflow_targets_both_runners

printf 'blueos_gungraun_instruction_counts_test: ok\n'
