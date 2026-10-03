#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel)
# shellcheck disable=SC1091
source "$ROOT_DIR/.hooks/lib/rust_checks.sh"

TMPDIR=$(mktemp -d)
export TMPDIR
trap 'rm -rf "$TMPDIR"' EXIT

fail() {
    printf 'quality_ratchet_test: %s\n' "$1" >&2
    exit 1
}

# Usage: write_measurements <suppression_ratio_exceeded>
# Writes what rustqual 1.8.3 and thailint 0.25.0 print for a tree with 3 IOSP violations, 5 findings in all,
# 2 unwrap calls and 1 clone in a loop.
write_measurements() {
    cat >"$TMPDIR/baseline.json" <<'EOF'
{
  "version": 2,
  "quality_score": 0.5,
  "iosp_score": 0.9,
  "violations": 3,
  "total": 40,
  "complexity_warnings": 2,
  "total_findings": 5,
  "violation_details": [{"name": "handle", "file": "src/lib.rs", "line": 1}]
}
EOF
    printf '{"summary": {"violations": 3, "suppression_ratio_exceeded": %s}}\n' "$1" >"$TMPDIR/report.json"
    cat >"$TMPDIR/thailint.json" <<'EOF'
{
  "total": 2,
  "violations": [
    {"rule_id": "unwrap-abuse.unwrap-call", "file_path": "a.rs", "line": 1, "column": 1, "message": "", "severity": "ERROR"},
    {"rule_id": "unwrap-abuse.unwrap-call", "file_path": "b.rs", "line": 2, "column": 1, "message": "", "severity": "ERROR"}
  ]
}
{
  "total": 1,
  "violations": [
    {"rule_id": "clone-abuse.clone-in-loop", "file_path": "c.rs", "line": 3, "column": 1, "message": "", "severity": "ERROR"}
  ]
}
{"total": 0, "violations": []}
EOF
}

# Usage: write_ratchet <rustqual-table> <thailint-table>
write_ratchet() {
    printf '# Ceilings\n[rustqual]\n%s\n\n[thailint]\n%s\n' "$1" "$2" >"$TMPDIR/quality-ratchet.toml"
}

collect() {
    collect_quality_ratchet_violations "$TMPDIR/quality-ratchet.toml" "$TMPDIR/baseline.json" "$TMPDIR/report.json" \
        "$TMPDIR/thailint.json"
}

assert_ratchet_passes() {
    local label="$1"
    local output
    if ! output=$(collect); then
        printf 'quality_ratchet_test: %s: expected no violation, got:\n%s\n' "$label" "$output" >&2
        exit 1
    fi
}

assert_ratchet_fails() {
    local label="$1"
    local pattern="$2"
    local output
    if output=$(collect); then
        fail "$label: expected a violation"
    fi
    if ! grep -qF -- "$pattern" <<<"$output"; then
        printf 'quality_ratchet_test: %s: expected %q in:\n%s\n' "$label" "$pattern" "$output" >&2
        exit 1
    fi
}

readonly matching_rustqual='violations = 3
complexity_warnings = 2
total_findings = 5'
readonly matching_thailint='"unwrap-abuse.unwrap-call" = 2
"clone-abuse.clone-in-loop" = 1
"blocking-async.sleep-in-async" = 0'

test_counts_equal_to_their_ceilings_pass() {
    write_measurements false
    write_ratchet "$matching_rustqual" "$matching_thailint"
    assert_ratchet_passes "equal counts"
}

test_count_above_its_ceiling_fails_and_names_the_category() {
    write_measurements false
    write_ratchet 'violations = 2
complexity_warnings = 2
total_findings = 5' "$matching_thailint"
    assert_ratchet_fails "rustqual above" "rustqual: violations is 3, above its ceiling of 2"
    assert_ratchet_fails "rustqual listing" "(cd core && rustqual .)"

    write_ratchet "$matching_rustqual" '"unwrap-abuse.unwrap-call" = 1
"clone-abuse.clone-in-loop" = 1
"blocking-async.sleep-in-async" = 0'
    assert_ratchet_fails "thailint above" "thailint: unwrap-abuse.unwrap-call is 2, above its ceiling of 1"
    assert_ratchet_fails "thailint listing" "(cd core && thailint unwrap-abuse libs app services)"
}

test_count_below_its_ceiling_fails_and_asks_for_fix() {
    write_measurements false
    write_ratchet 'violations = 3
complexity_warnings = 2
total_findings = 9' "$matching_thailint"
    assert_ratchet_fails "rustqual below" \
        "rustqual: total_findings is 5, below its ceiling of 9 in quality-ratchet.toml; lower the ceiling with ./.hooks/pre-push --fix"

    write_ratchet "$matching_rustqual" '"unwrap-abuse.unwrap-call" = 2
"clone-abuse.clone-in-loop" = 1
"blocking-async.sleep-in-async" = 4'
    assert_ratchet_fails "thailint below" "thailint: blocking-async.sleep-in-async is 0, below its ceiling of 4"
}

test_unknown_or_missing_category_fails() {
    write_measurements false
    write_ratchet 'violations = 3
complexity_warnings = 2' "$matching_thailint"
    assert_ratchet_fails "rustqual missing" "rustqual: total_findings has no ceiling in quality-ratchet.toml"

    write_ratchet "$matching_rustqual
renamed_warnings = 0" "$matching_thailint"
    assert_ratchet_fails "rustqual unknown" "rustqual: renamed_warnings in quality-ratchet.toml is not a category"

    write_ratchet "$matching_rustqual" '"clone-abuse.clone-in-loop" = 1
"blocking-async.sleep-in-async" = 0'
    assert_ratchet_fails "thailint missing" "thailint: unwrap-abuse.unwrap-call has no ceiling"

    write_ratchet "$matching_rustqual" "$matching_thailint
\"magic-numbers.literal\" = 0"
    assert_ratchet_fails "thailint unknown" "thailint: magic-numbers.literal in quality-ratchet.toml is not a category"
}

test_exceeded_suppression_ratio_fails() {
    write_measurements true
    write_ratchet "$matching_rustqual" "$matching_thailint"
    assert_ratchet_fails "suppression ratio" "rustqual: suppressions exceed max_suppression_ratio in rustqual.toml"
}

test_fix_lowers_ceilings_and_never_raises_them() {
    write_measurements false
    write_ratchet 'violations = 2
complexity_warnings = 7
total_findings = 5' '"unwrap-abuse.unwrap-call" = 1
"clone-abuse.clone-in-loop" = 4
"blocking-async.sleep-in-async" = 3'
    lower_quality_ratchet_ceilings "$TMPDIR/quality-ratchet.toml" "$TMPDIR/baseline.json" "$TMPDIR/thailint.json"
    local expected
    expected=$(printf '%s\n' '# Ceilings' '[rustqual]' 'violations = 2' 'complexity_warnings = 2' 'total_findings = 5' '' \
        '[thailint]' '"unwrap-abuse.unwrap-call" = 1' '"clone-abuse.clone-in-loop" = 1' \
        '"blocking-async.sleep-in-async" = 0')
    if [ "$(cat "$TMPDIR/quality-ratchet.toml")" != "$expected" ]; then
        printf 'quality_ratchet_test: fix: expected:\n%s\ngot:\n%s\n' "$expected" \
            "$(cat "$TMPDIR/quality-ratchet.toml")" >&2
        exit 1
    fi
}

# Usage: install_fake_tools <thailint-behaviour>
# Puts a rustqual and a thailint on PATH that print the fixture measurements. With "crash", thailint exits 1, as
# it does when it finds something, but writes no JSON.
install_fake_tools() {
    mkdir -p "$TMPDIR/bin"
    cat >"$TMPDIR/bin/rustqual" <<EOF
#!/usr/bin/env bash
if [ "\$1" = --version ]; then echo "rustqual $RUST_RUSTQUAL_VERSION"; exit 0; fi
while [ "\$#" -gt 0 ]; do
    if [ "\$1" = --save-baseline ]; then cp "$TMPDIR/baseline.json" "\$2"; fi
    shift
done
cat "$TMPDIR/report.json"
EOF
    cat >"$TMPDIR/bin/thailint" <<EOF
#!/usr/bin/env bash
if [ "\$1" = --version ]; then echo "thailint, version $RUST_THAILINT_VERSION"; exit 0; fi
if [ "$1" = crash ]; then echo "Traceback (most recent call last):" >&2; exit 1; fi
case \$1 in
    unwrap-abuse) jq -c 'select(.violations[0].rule_id // "" | startswith("unwrap-abuse"))' "$TMPDIR/thailint.json" ;;
    clone-abuse) jq -c 'select(.violations[0].rule_id // "" | startswith("clone-abuse"))' "$TMPDIR/thailint.json" ;;
    *) echo '{"total": 0, "violations": []}' ;;
esac
exit 1
EOF
    chmod +x "$TMPDIR/bin/rustqual" "$TMPDIR/bin/thailint"
}

# Usage: run_quality_ratchet <fixing>
run_quality_ratchet() {
    (
        export PATH="$TMPDIR/bin:$PATH"
        # shellcheck disable=SC2034
        fixing="$1"
        check_rust_quality_ratchet "$TMPDIR" "$TMPDIR/quality-ratchet.toml"
    )
}

test_quality_ratchet_passes_on_a_complete_measurement() {
    write_measurements false
    write_ratchet "$matching_rustqual" "$matching_thailint"
    install_fake_tools complete
    if ! run_quality_ratchet false >/dev/null 2>&1; then
        fail "complete measurement: expected the ratchet to pass"
    fi
}

test_crashed_tool_fails_and_fix_mode_writes_nothing() {
    write_measurements false
    write_ratchet 'violations = 3
complexity_warnings = 2
total_findings = 9' "$matching_thailint"
    install_fake_tools crash
    local before output
    before=$(cat "$TMPDIR/quality-ratchet.toml")
    if output=$(run_quality_ratchet false 2>&1); then
        fail "crashed thailint: expected the ratchet to fail"
    fi
    if ! grep -qF "thailint unwrap-abuse wrote no findings" <<<"$output"; then
        printf 'quality_ratchet_test: crashed thailint: expected the linter named in:\n%s\n' "$output" >&2
        exit 1
    fi
    if run_quality_ratchet true >/dev/null 2>&1; then
        fail "crashed thailint in fix mode: expected the ratchet to fail"
    fi
    if [ "$(cat "$TMPDIR/quality-ratchet.toml")" != "$before" ]; then
        fail "crashed thailint in fix mode: the ceilings changed"
    fi
    if [ -n "$(find "$TMPDIR" -maxdepth 1 -name 'quality-ratchet.toml?*')" ]; then
        fail "crashed thailint in fix mode: left a temporary file next to the ratchet file"
    fi
}

test_missing_rustqual_baseline_fails() {
    write_measurements false
    write_ratchet "$matching_rustqual" "$matching_thailint"
    install_fake_tools complete
    sed -i '/save-baseline/d' "$TMPDIR/bin/rustqual"
    if run_quality_ratchet true >/dev/null 2>&1; then
        fail "missing rustqual baseline: expected the ratchet to fail"
    fi
    if run_quality_ratchet false >/dev/null 2>&1; then
        fail "missing rustqual baseline: expected the ratchet to fail"
    fi
}

test_missing_rustqual_names_the_install_command() {
    local output
    if output=$(PATH="$TMPDIR/empty" check_rust_quality_ratchet "$TMPDIR" 2>&1); then
        fail "missing rustqual: expected the ratchet to fail"
    fi
    if ! grep -qF "cargo install --locked rustqual@$RUST_RUSTQUAL_VERSION" <<<"$output"; then
        printf 'quality_ratchet_test: missing rustqual: expected the install command in:\n%s\n' "$output" >&2
        exit 1
    fi
}

main() {
    test_counts_equal_to_their_ceilings_pass
    test_quality_ratchet_passes_on_a_complete_measurement
    test_crashed_tool_fails_and_fix_mode_writes_nothing
    test_missing_rustqual_baseline_fails
    test_missing_rustqual_names_the_install_command
    test_fix_lowers_ceilings_and_never_raises_them
    test_count_above_its_ceiling_fails_and_names_the_category
    test_count_below_its_ceiling_fails_and_asks_for_fix
    test_unknown_or_missing_category_fails
    test_exceeded_suppression_ratio_fails
    printf 'quality_ratchet_test: ok\n'
}

main "$@"
