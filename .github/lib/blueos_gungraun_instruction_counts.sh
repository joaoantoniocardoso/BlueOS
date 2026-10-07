#!/usr/bin/env bash
# Compare Gungraun instruction-count benchmarks between base and head revisions (D-33).

set -euo pipefail

repository_root=$(git rev-parse --show-toplevel)
core_directory="$repository_root/core"
baseline_name=${GUNGRAUN_BASELINE_NAME:-base}
runner_label=${GUNGRAUN_RUNNER_LABEL:-}
base_ref=""
head_ref=""
raw_output_path=""
summary_path=""

BENCHMARK_PACKAGES=(
    "blueos-idl:cdr_codec_gungraun"
    "blueos-recorder-mcap:write_sample_gungraun"
    "blueos-recorder-library:library_command_gungraun"
)

gungraun_harnesses_present() {
    [ -f "$core_directory/libs/idl/benches/cdr_codec_gungraun.rs" ]
}

# Gungraun exits with 3 when a benchmark regresses past --callgrind-limits; main fails the job on it after reporting.
run_gungraun_benches() {
    local -a bench_arguments=("$@")
    local combined_output=""
    local overall_status=0
    local entry package bench bench_output bench_status

    for entry in "${BENCHMARK_PACKAGES[@]}"; do
        package=${entry%%:*}
        bench=${entry##*:}
        set +e
        bench_output=$(cd "$core_directory" && cargo bench -p "$package" --bench "$bench" -- "${bench_arguments[@]}" 2>&1)
        bench_status=$?
        set -e
        combined_output+=$(printf '\n\n=== %s / %s (exit %s) ===\n%s' "$package" "$bench" "$bench_status" "$bench_output")
        if [ "$bench_status" -ne 0 ] && [ "$bench_status" -ne 3 ]; then
            overall_status=$bench_status
        fi
    done
    printf '%s' "$combined_output"
    return "$overall_status"
}

format_gungraun_instruction_summary_table() {
    local compare_output=$1
    awk -v runner="$runner_label" '
        function trim(string) {
            sub(/^[ \t]+/, "", string)
            sub(/[ \t]+$/, "", string)
            return string
        }
        /^[^ \t].*::/ {
            current_benchmark = trim($0)
            next
        }
        /^[ \t]+Instructions:/ {
            if (current_benchmark == "") {
                next
            }
            if (match($0, /Instructions:[ \t]+([0-9]+)\|([^ \t|]+)[ \t]*(.*)$/, parts)) {
                head = parts[1]
                base = parts[2]
                difference = trim(parts[3])
                if (difference == "") {
                    difference = "N/A"
                }
                printf "| %s | %s | %s | %s | %s |\n", current_benchmark, base, head, difference, runner
            }
            current_benchmark = ""
        }
        END {
            if (NR == 0) {
                exit 0
            }
        }
    ' <<<"$compare_output"
}

write_job_summary() {
    local base_sha=$1
    local head_sha=$2
    local compare_output=$3
    local table

    {
        printf '## Gungraun instruction counts\n\n'
        printf 'Runner: %s\n\n' "$runner_label"
        printf "Baseline: \`%s\` (%s)\n\n" "$base_ref" "$base_sha"
        printf "Head: \`%s\` (%s)\n\n" "$head_ref" "$head_sha"
        printf '| Benchmark | Base instructions | Head instructions | Difference | Runner |\n'
        printf '| --- | ---: | ---: | --- | --- |\n'
    } >>"$summary_path"

    table=$(format_gungraun_instruction_summary_table "$compare_output")
    if [ -z "$table" ]; then
        printf '| (no instruction rows parsed) | | | | %s |\n' "$runner_label" >>"$summary_path"
    else
        printf '%s\n' "$table" >>"$summary_path"
    fi
    printf '\n' >>"$summary_path"
}

main() {
    local head_sha base_sha base_output compare_output run_status
    local -a compare_arguments

    base_ref=${GUNGRAUN_BASE_REF:?GUNGRAUN_BASE_REF must be set}
    head_ref=${GUNGRAUN_HEAD_REF:?GUNGRAUN_HEAD_REF must be set}
    runner_label=${GUNGRAUN_RUNNER_LABEL:?GUNGRAUN_RUNNER_LABEL must be set}
    raw_output_path=${GUNGRAUN_RAW_OUTPUT:?GUNGRAUN_RAW_OUTPUT must be set}
    summary_path=${GITHUB_STEP_SUMMARY:?GITHUB_STEP_SUMMARY must be set}

    if ! gungraun_harnesses_present; then
        printf '::error::Gungraun harnesses are missing from %s\n' "$core_directory" >&2
        exit 1
    fi

    head_sha=$(git -C "$repository_root" rev-parse "$head_ref")
    # A push to a new branch, or after a force-push, has no base commit to compare against; it only records counts.
    base_sha=$(git -C "$repository_root" rev-parse --verify --quiet "$base_ref^{commit}" || true)

    base_output=""
    compare_arguments=(--save-baseline="$baseline_name")
    if [ -n "$base_sha" ]; then
        compare_arguments=(--baseline="$baseline_name" --callgrind-limits="ir=${GUNGRAUN_INSTRUCTION_LIMIT:-1%}")
        git -C "$repository_root" checkout --force "$base_sha"
        if gungraun_harnesses_present; then
            set +e
            base_output=$(run_gungraun_benches --save-baseline="$baseline_name")
            run_status=$?
            set -e
            if [ "$run_status" -ne 0 ] && [ "$run_status" -ne 3 ]; then
                printf '::error::Gungraun baseline run failed with exit %s\n' "$run_status" >&2
                git -C "$repository_root" checkout --force "$head_sha"
                exit "$run_status"
            fi
        else
            base_output="(baseline revision has no Gungraun harnesses)"
        fi
    fi

    git -C "$repository_root" checkout --force "$head_sha"
    set +e
    compare_output=$(run_gungraun_benches "${compare_arguments[@]}")
    run_status=$?
    set -e
    if [ "$run_status" -ne 0 ] && [ "$run_status" -ne 3 ]; then
        printf '::error::Gungraun compare run failed with exit %s\n' "$run_status" >&2
        exit "$run_status"
    fi

    {
        printf '=== BASE %s (%s) ===\n%s\n\n' "$base_ref" "$base_sha" "$base_output"
        printf '=== HEAD %s (%s) compare (exit %s) ===\n%s\n' "$head_ref" "$head_sha" "$run_status" "$compare_output"
    } >"$raw_output_path"

    write_job_summary "$base_sha" "$head_sha" "$compare_output"

    if ! grep -q 'Instructions:' <<<"$compare_output"; then
        printf '::error::Gungraun reported no instruction counts; is gungraun-runner installed?\n' >&2
        exit 1
    fi
    if [ "$run_status" -eq 3 ]; then
        printf '::error::Instruction count regressed past %s\n' "${GUNGRAUN_INSTRUCTION_LIMIT:-1%}" >&2
        exit 1
    fi
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
