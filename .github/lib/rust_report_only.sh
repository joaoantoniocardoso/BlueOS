#!/usr/bin/env bash
# Append report tool output to GITHUB_STEP_SUMMARY and the log. A finding never fails the job; a tool that
# cannot run does (D-30).

set -uo pipefail

# Titles of the sections whose tool could not run. Every section of a step still runs, and the step fails when its
# shell exits.
REPORT_SECTION_FAILURES=()
trap report_section_exit EXIT

# Usage: report_section <title> --ok "<exit codes>" (--version <command>... | --no-version) -- <command>...
# Writes the tool's version, its output and its exit code. The command runs under errexit, so a failing command
# inside a shell function fails the section. A version command that fails, or an exit code that is not one of the
# exit codes that mean the tool ran, fails the step once every section has run. --no-version is for a section that
# runs an in-repository script, which has no version of its own.
report_section() {
    local title=$1
    shift
    local accepted="" version_command=() no_version=false
    while [ "$#" -gt 0 ] && [ "$1" != -- ]; do
        case $1 in
            --ok)
                if [ "$#" -lt 2 ] || [ -z "$2" ] || [ "$2" = -- ]; then
                    report_section_usage "$title" "--ok needs the accepted exit codes"
                    return 2
                fi
                accepted=$2
                shift 2
                ;;
            --version)
                shift
                while [ "$#" -gt 0 ] && [ "$1" != -- ]; do
                    version_command+=("$1")
                    shift
                done
                ;;
            --no-version)
                no_version=true
                shift
                ;;
            *)
                report_section_usage "$title" "unknown argument $1"
                return 2
                ;;
        esac
    done
    if [ -z "$accepted" ]; then
        report_section_usage "$title" "--ok is required"
        return 2
    fi
    if [ "${#version_command[@]}" -eq 0 ] && [ "$no_version" = false ]; then
        report_section_usage "$title" "--version <command> or --no-version is required"
        return 2
    fi
    if [ "$#" -lt 2 ]; then
        report_section_usage "$title" "-- <command> is required"
        return 2
    fi
    shift
    local summary=${GITHUB_STEP_SUMMARY:?GITHUB_STEP_SUMMARY must be set}
    local version="" status errexit=false
    if [ "$no_version" = false ] && ! version=$("${version_command[@]}" 2>&1); then
        {
            printf '## %s\n\nThe version command failed:\n\n```\n%s\n' "$title" "$version"
            printf '```\n\n'
        } | tee -a "$summary"
        printf '::error title=%s::%s is not installed or cannot run\n' "$title" "$title"
        REPORT_SECTION_FAILURES+=("$title")
        return 0
    fi
    printf '## %s\n\n' "$title" | tee -a "$summary"
    if [ "$no_version" = false ]; then
        printf 'Version: %s\n\n' "$version" | tee -a "$summary"
    fi
    printf '```\n' | tee -a "$summary"
    # errexit is ignored in any command that `||` or `if` tests, so the command runs outside one.
    [[ $- == *e* ]] && errexit=true
    set +e
    (
        set -e
        "$@"
    ) 2>&1 | tee -a "$summary"
    status=${PIPESTATUS[0]}
    [ "$errexit" = false ] || set -e
    printf '```\n\nExit code: %s (accepted: %s)\n\n' "$status" "$accepted" | tee -a "$summary"
    if [[ " $accepted " != *" $status "* ]]; then
        printf '::error title=%s::%s exited with %s, which is not an accepted exit code (%s)\n' \
            "$title" "$title" "$status" "$accepted"
        REPORT_SECTION_FAILURES+=("$title")
    fi
}

# Usage: report_section_usage <title> <message>
report_section_usage() {
    printf 'report_section %s: %s\n' "$1" "$2" >&2
}

report_section_exit() {
    local status=$?
    if [ "${#REPORT_SECTION_FAILURES[@]}" -gt 0 ]; then
        printf '::error::Report sections that could not run: %s\n' "$(printf '%s; ' "${REPORT_SECTION_FAILURES[@]}")"
        exit 1
    fi
    exit "$status"
}
