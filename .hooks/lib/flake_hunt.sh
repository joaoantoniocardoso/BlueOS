#!/usr/bin/env bash
# Reruns Rust integration-test binaries changed since a base ref, one binary at a time.
# A race that vanishes when tests run in parallel still fails a serial rerun.
# Usage: flake_hunt.sh <base-ref>
# FLAKE_HUNT_ITERATIONS sets how many times each binary runs (default 200).

set -euo pipefail

package_name_from() {
    sed -n '/^\[package\]/,/^\[/ { /^name[[:space:]]*=/ { s/.*"\([^"]*\)".*/\1/; p; q; } }' "$1"
}

if [ "$#" -ne 1 ]; then
    printf 'usage: flake_hunt.sh <base-ref>\n' >&2
    exit 2
fi

base_ref=$1
iterations=${FLAKE_HUNT_ITERATIONS:-200}
if ! [[ $iterations =~ ^[1-9][0-9]*$ ]]; then
    printf 'flake_hunt: FLAKE_HUNT_ITERATIONS must be a positive integer\n' >&2
    exit 2
fi

repository=$(git rev-parse --show-toplevel)
cd "$repository"

if ! git rev-parse --verify --quiet "${base_ref}^{commit}" >/dev/null; then
    printf 'flake_hunt: unknown base ref %s\n' "$base_ref" >&2
    exit 2
fi

changed_paths=()
while IFS= read -r path; do
    [ -n "$path" ] || continue
    changed_paths+=("$path")
done < <(git diff --name-only --diff-filter=ACMR "${base_ref}...HEAD" -- \
    ':(glob)core/**/tests/*.rs' ':(glob)core/**/tests/common/**')

if [ "${#changed_paths[@]}" -eq 0 ]; then
    printf 'flake_hunt: no integration tests changed since %s\n' "$base_ref"
    exit 0
fi

test_files=()
for path in "${changed_paths[@]}"; do
    if [[ $path == */tests/common/* ]]; then
        tests_dir=${path%%/tests/common/*}/tests
        for test_file in "$tests_dir"/*.rs; do
            [ -f "$test_file" ] || continue
            test_files+=("$test_file")
        done
    elif [[ $path == *.rs ]] && [ -f "$path" ]; then
        test_files+=("$path")
    fi
done

if [ "${#test_files[@]}" -eq 0 ]; then
    printf 'flake_hunt: no integration tests changed since %s\n' "$base_ref"
    exit 0
fi

unique_test_files=()
while IFS= read -r path; do
    unique_test_files+=("$path")
done < <(printf '%s\n' "${test_files[@]}" | sort -u)

for test_file in "${unique_test_files[@]}"; do
    # trybuild checks compile errors. Those results do not change between runs, and one run is minutes.
    if grep -q 'trybuild' "$test_file"; then
        printf 'flake_hunt: skipping %s (trybuild)\n' "$test_file"
        continue
    fi
    crate_root=${test_file%/tests/*}
    manifest=$crate_root/Cargo.toml
    if [ ! -f "$manifest" ]; then
        printf 'flake_hunt: no Cargo.toml for %s\n' "$test_file" >&2
        exit 1
    fi
    package=$(package_name_from "$manifest")
    test_name=$(basename "$test_file" .rs)
    printf 'flake_hunt: running %s --test %s, %s serial iterations\n' \
        "$package" "$test_name" "$iterations"

    if ! artifact_json=$(cargo test --locked --manifest-path "$repository/core/Cargo.toml" \
        -p "$package" --test "$test_name" --no-run --message-format=json); then
        printf 'flake_hunt: failed to build %s --test %s\n' "$package" "$test_name" >&2
        exit 1
    fi
    executable=$(jq -sr --arg test_name "$test_name" '
        [ .[]
          | select(.reason == "compiler-artifact")
          | select(.executable != null)
          | select(.target.name == $test_name)
          | select(.target.kind | index("test"))
        ] | last | .executable // empty
    ' <<<"$artifact_json")
    if [ -z "$executable" ]; then
        printf 'flake_hunt: no test binary for %s --test %s\n' "$package" "$test_name" >&2
        exit 1
    fi

    for ((iteration = 1; iteration <= iterations; iteration++)); do
        if output=$("$executable" --test-threads=1 2>&1); then
            continue
        fi
        failed_test=$(grep -m 1 -E '^test .+ \.\.\. FAILED$' <<<"$output" || true)
        failed_test=${failed_test#test }
        failed_test=${failed_test% ... FAILED}
        if [ -z "$failed_test" ]; then
            failed_test=unknown
        fi
        printf 'flake_hunt: binary %s test %s iteration %s\n' \
            "$executable" "$failed_test" "$iteration" >&2
        printf '%s\n' "$output" >&2
        exit 1
    done
done
