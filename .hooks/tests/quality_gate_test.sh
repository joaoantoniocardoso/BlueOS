#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
# shellcheck disable=SC1091
source "$ROOT_DIR/.hooks/lib/rust_checks.sh"

TMPDIR=$(mktemp -d)
export TMPDIR
trap 'rm -rf "$TMPDIR"' EXIT

fail() {
    printf 'quality_gate_test: %s\n' "$1" >&2
    exit 1
}

# Usage: install_fake_tools <rustqual-exit> <thailint-linter-that-finds-something> [rustqual-output]
# Puts a rustqual and a thailint on PATH that log their arguments to calls.log. rustqual prints the given output
# and exits with the given status; thailint exits 1 for the named linter and 0 for the others.
install_fake_tools() {
    mkdir -p "$TMPDIR/bin"
    : >"$TMPDIR/calls.log"
    cat >"$TMPDIR/bin/rustqual" <<EOF
#!/usr/bin/env bash
if [ "\$1" = --version ]; then echo "rustqual $RUST_RUSTQUAL_VERSION"; exit 0; fi
echo "rustqual \$*" >>"$TMPDIR/calls.log"
echo "${3:-No findings.}" >&2
exit $1
EOF
    cat >"$TMPDIR/bin/thailint" <<EOF
#!/usr/bin/env bash
if [ "\$1" = --version ]; then echo "thailint, version $RUST_THAILINT_VERSION"; exit 0; fi
echo "thailint \$*" >>"$TMPDIR/calls.log"
[ "\$1" != "$2" ]
EOF
    chmod +x "$TMPDIR/bin/rustqual" "$TMPDIR/bin/thailint"
}

run_gate() {
    PATH="$TMPDIR/bin:$PATH" check_rust_quality "$TMPDIR"
}

test_clean_tools_pass_and_rustqual_fails_on_warnings() {
    install_fake_tools 0 none
    run_gate >/dev/null 2>&1 || fail "clean tools: expected the gate to pass"
    grep -qF -- 'rustqual --fail-on-warnings' "$TMPDIR/calls.log" \
        || fail "rustqual must run with --fail-on-warnings, so an exceeded suppression ratio fails"
}

test_a_rustqual_finding_fails_and_every_linter_still_runs() {
    install_fake_tools 1 none
    if run_gate >/dev/null 2>&1; then
        fail "rustqual finding: expected the gate to fail"
    fi
    local linter
    for linter in "${RUST_THAILINT_LINTERS[@]}"; do
        grep -qF "thailint $linter libs app services" "$TMPDIR/calls.log" \
            || fail "rustqual finding: thailint $linter did not run"
    done
}

test_a_file_rustqual_cannot_parse_fails() {
    install_fake_tools 0 none 'Warning: Could not parse src/broken.rs: unexpected end of input'
    if run_gate >/dev/null 2>&1; then
        fail "unparsed file: expected the gate to fail"
    fi
}

test_a_thailint_finding_fails() {
    install_fake_tools 0 clone-abuse
    if run_gate >/dev/null 2>&1; then
        fail "thailint finding: expected the gate to fail"
    fi
}

test_missing_rustqual_names_the_install_command() {
    local output
    if output=$(PATH="$TMPDIR/empty" check_rust_quality "$TMPDIR" 2>&1); then
        fail "missing rustqual: expected the gate to fail"
    fi
    if ! grep -qF "cargo install --locked rustqual@$RUST_RUSTQUAL_VERSION" <<<"$output"; then
        printf 'quality_gate_test: missing rustqual: expected the install command in:\n%s\n' "$output" >&2
        exit 1
    fi
}

main() {
    test_clean_tools_pass_and_rustqual_fails_on_warnings
    test_a_rustqual_finding_fails_and_every_linter_still_runs
    test_a_file_rustqual_cannot_parse_fails
    test_a_thailint_finding_fails
    test_missing_rustqual_names_the_install_command
    printf 'quality_gate_test: ok\n'
}

main "$@"
