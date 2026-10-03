#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
# shellcheck disable=SC1091
source "$ROOT_DIR/.hooks/lib/rust_checks.sh"

TMPDIR=$(mktemp -d)
export TMPDIR
trap 'rm -rf "$TMPDIR"' EXIT

fail() {
    printf 'ratchet_loosen_test: %s\n' "$1" >&2
    exit 1
}

readonly base_ceilings='[rustqual]
violations = 78
total_findings = 699

[thailint]
"unwrap-abuse.unwrap-call" = 6'
readonly base_floors='[lines]
workspace = 75
services_app = 77'

# Usage: assert_loosening <label> <kind> <base> <head> <expected-pattern>
assert_loosening() {
    local label="$1"
    local output
    printf '%s\n' "$3" >"$TMPDIR/base.toml"
    printf '%s\n' "$4" >"$TMPDIR/head.toml"
    if output=$(collect_ratchet_loosening "$2" "$TMPDIR/base.toml" "$TMPDIR/head.toml"); then
        fail "$label: expected the change to count as loosening"
    fi
    if ! grep -qF -- "$5" <<<"$output"; then
        printf 'ratchet_loosen_test: %s: expected %q in:\n%s\n' "$label" "$5" "$output" >&2
        exit 1
    fi
}

# Usage: assert_not_loosened <label> <kind> <base> <head>
assert_not_loosened() {
    local label="$1"
    local output
    printf '%s\n' "$3" >"$TMPDIR/base.toml"
    printf '%s\n' "$4" >"$TMPDIR/head.toml"
    if ! output=$(collect_ratchet_loosening "$2" "$TMPDIR/base.toml" "$TMPDIR/head.toml"); then
        printf 'ratchet_loosen_test: %s: expected no loosening, got:\n%s\n' "$label" "$output" >&2
        exit 1
    fi
}

test_raised_ceiling_fails() {
    assert_loosening "raised ceiling" ceilings "$base_ceilings" "${base_ceilings/violations = 78/violations = 79}" \
        "rustqual.violations ceiling rose from 78 to 79"
}

test_lowered_floor_fails() {
    assert_loosening "lowered floor" floors "$base_floors" "${base_floors/workspace = 75/workspace = 74.5}" \
        "lines.workspace floor fell from 75 to 74.5"
}

test_removed_key_fails() {
    assert_loosening "removed ceiling" ceilings "$base_ceilings" "${base_ceilings/total_findings = 699/}" \
        "rustqual.total_findings was removed"
    assert_loosening "removed floor" floors "$base_floors" "${base_floors/services_app = 77/}" \
        "lines.services_app was removed"
}

test_tightened_file_passes() {
    assert_not_loosened "tightened ceilings" ceilings "$base_ceilings" "${base_ceilings/violations = 78/violations = 70}
new_warnings = 0"
    assert_not_loosened "tightened floors" floors "$base_floors" "${base_floors/workspace = 75/workspace = 80}"
}

# Usage: commit_ratchets <repository> <quality-ratchet> <coverage-ratchet>
# Commits both ratchet files, skipping one given as empty, and prints the commit.
commit_ratchets() {
    mkdir -p "$1/core"
    rm -f "$1/core/quality-ratchet.toml" "$1/core/coverage-ratchet.toml"
    [ -z "$2" ] || printf '%s\n' "$2" >"$1/core/quality-ratchet.toml"
    [ -z "$3" ] || printf '%s\n' "$3" >"$1/core/coverage-ratchet.toml"
    git -C "$1" add -A core
    git -C "$1" -c user.name=test -c user.email=test@example.com commit -q --allow-empty -m ratchets
    git -C "$1" rev-parse HEAD
}

new_repository() {
    local repository
    repository=$(mktemp -d)
    git -C "$repository" init -q
    printf '%s\n' "$repository"
}

# Usage: assert_check <label> <expected: passes|fails> <repository> <base> [expected-pattern]
assert_check() {
    local output
    if output=$(check_ratchets_not_loosened "$3" "$4" 2>&1); then
        [ "$2" = passes ] || fail "$1: expected the check to fail"
    else
        [ "$2" = fails ] || {
            printf 'ratchet_loosen_test: %s: expected the check to pass, got:\n%s\n' "$1" "$output" >&2
            exit 1
        }
    fi
    if [ -n "${5:-}" ] && ! grep -qF -- "$5" <<<"$output"; then
        printf 'ratchet_loosen_test: %s: expected %q in:\n%s\n' "$1" "$5" "$output" >&2
        exit 1
    fi
}

test_check_reads_the_base_revision() {
    local repository base
    repository=$(new_repository)
    base=$(commit_ratchets "$repository" "$base_ceilings" "$base_floors")
    commit_ratchets "$repository" "${base_ceilings/violations = 78/violations = 79}" "$base_floors" >/dev/null
    assert_check "raised ceiling since the base" fails "$repository" "$base" \
        "rustqual.violations ceiling rose from 78 to 79"
    assert_check "raised ceiling since the base" fails "$repository" "$base" "core/quality-ratchet.toml loosened"
    assert_check "no change since the head" passes "$repository" HEAD
}

test_check_reads_coverage_as_floors() {
    local repository base
    repository=$(new_repository)
    base=$(commit_ratchets "$repository" "$base_ceilings" "$base_floors")
    commit_ratchets "$repository" "$base_ceilings" "${base_floors/workspace = 75/workspace = 80}" >/dev/null
    assert_check "raised floor" passes "$repository" "$base"
    commit_ratchets "$repository" "$base_ceilings" "${base_floors/workspace = 75/workspace = 70}" >/dev/null
    assert_check "lowered floor" fails "$repository" "$base" "lines.workspace floor fell from 75 to 70"
}

test_check_skips_a_file_the_base_lacks() {
    local repository base
    repository=$(new_repository)
    base=$(commit_ratchets "$repository" "" "$base_floors")
    commit_ratchets "$repository" "$base_ceilings" "$base_floors" >/dev/null
    assert_check "new ratchet file" passes "$repository" "$base"
}

test_check_fails_on_an_unresolvable_base() {
    local repository
    repository=$(new_repository)
    commit_ratchets "$repository" "$base_ceilings" "$base_floors" >/dev/null
    assert_check "unresolvable base" fails "$repository" 0123456789abcdef0123456789abcdef01234567 \
        "cannot resolve the base revision"
}

main() {
    test_raised_ceiling_fails
    test_lowered_floor_fails
    test_removed_key_fails
    test_tightened_file_passes
    test_check_reads_the_base_revision
    test_check_reads_coverage_as_floors
    test_check_skips_a_file_the_base_lacks
    test_check_fails_on_an_unresolvable_base
    printf 'ratchet_loosen_test: ok\n'
}

main "$@"
