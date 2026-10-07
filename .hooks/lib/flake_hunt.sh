#!/usr/bin/env bash
# Reruns Rust integration-test binaries changed since a base ref, each running its tests one at a time.
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

executables=()
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
        -p "$package" --test "$test_name" --all-features --no-run --message-format=json); then
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
    executables+=("$executable")
done

if [ "${#executables[@]}" -eq 0 ]; then
    exit 0
fi

hunt_binary() {
    local executable=$1 iteration output failed_test
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
        printf 'flake_hunt: binary %s test %s iteration %s\n%s\n' \
            "$executable" "$failed_test" "$iteration" "$output" >&2
        return 1
    done
}
export -f hunt_binary
export iterations

# Binaries run side by side, as nextest runs them; only the tests inside one binary stay serial.
# shellcheck disable=SC2016
printf '%s\0' "${executables[@]}" | xargs -0 -n 1 -P "$(nproc)" bash -c 'hunt_binary "$1"' hunt_binary || exit 1
