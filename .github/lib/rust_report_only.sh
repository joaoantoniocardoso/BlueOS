#!/usr/bin/env bash
# Append report-only tool output to GITHUB_STEP_SUMMARY without failing the job.

set -uo pipefail

report_section() {
    local title=$1
    shift
    {
        echo "## ${title}"
        echo '```'
        "$@" || echo "(exit $?)"
        echo '```'
    } >>"${GITHUB_STEP_SUMMARY:?GITHUB_STEP_SUMMARY must be set}"
}
