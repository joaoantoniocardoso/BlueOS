#!/usr/bin/env bash
# Wall-clock Criterion benchmarks and hyperfine start-up timing on reference Pi hardware (D-33).
# Report-only: numbers never fail the job; a tool that cannot run does.

# The workflow only runs on bluerobotics/BlueOS master with a pi4-builder2 runner. To run the Criterion
# benchmarks by hand on any Pi (verified on a Pi 5, aarch64), copy the repository over and run, on the Pi:
#   curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal && . ~/.cargo/env
#   cd core && for entry in blueos-idl:cdr_codec blueos-recorder-mcap:write_sample \
#       blueos-recorder-library:library_command; do
#       cargo bench --locked -p "${entry%%:*}" --bench "${entry#*:}" -- --noplot; done
# This script itself additionally needs GITHUB_STEP_SUMMARY and GITHUB_SHA exported, plus cross, cargo-auditable
# and hyperfine installed for the cross-build and start-up timing steps.

set -euo pipefail

if [ "$#" -ne 1 ]; then
    printf 'usage: %s <artifact-staging-dir>\n' "$0" >&2
    exit 2
fi

artifact_staging_directory=$1
repository_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
core_directory="${repository_root}/core"

# shellcheck disable=SC1091
source "${repository_root}/.github/lib/rust_report_only.sh"

reference_target_from_uname() {
    case "$(uname -m)" in
        aarch64) printf '%s\n' aarch64-unknown-linux-musl ;;
        armv7l | armv6l) printf '%s\n' armv7-unknown-linux-musleabihf ;;
        *)
            printf 'unsupported Pi benchmark host architecture: %s\n' "$(uname -m)" >&2
            return 1
            ;;
    esac
}

write_job_summary_header() {
    local summary=${GITHUB_STEP_SUMMARY:?GITHUB_STEP_SUMMARY must be set}
    local reference_target=$1
    local blueos_binary=$2
    {
        printf '## Pi benchmarks (D-33)\n\n'
        printf '| Field | Value |\n'
        printf '|-------|-------|\n'
        printf '| Runner | %s |\n' "$(hostname)"
        printf '| Architecture | %s |\n' "$(uname -m)"
        printf '| Reference target | %s |\n' "$reference_target"
        printf '| Commit | %s |\n' "${GITHUB_SHA:?GITHUB_SHA must be set}"
        printf '| blueos binary | %s |\n' "$blueos_binary"
        printf '| Baseline | none (wall-clock trend; each run is standalone) |\n\n'
    } >>"$summary"
}

set_cpu_governor_performance() {
    if command -v cpufreq-set >/dev/null 2>&1; then
        sudo cpufreq-set -g performance 2>/dev/null || true
    fi
    if [ -r /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor ]; then
        printf 'CPU governor: %s\n' "$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor)"
    fi
}

run_criterion_benchmarks() {
    local -a packages=(
        blueos-idl:cdr_codec
        blueos-recorder-mcap:write_sample
        blueos-recorder-library:library_command
    )
    local entry package bench_name
    for entry in "${packages[@]}"; do
        package=${entry%%:*}
        bench_name=${entry#*:}
        report_section "Criterion ${package} (${bench_name})" \
            --ok 0 --version cargo -- \
            cargo bench --locked -p "${package}" --bench "${bench_name}" -- --noplot
    done
}

stage_criterion_reports() {
    local criterion_directory="${core_directory}/target/criterion"
    if [ -d "$criterion_directory" ]; then
        mkdir -p "${artifact_staging_directory}/criterion"
        cp -a "${criterion_directory}/." "${artifact_staging_directory}/criterion/"
    fi
}

run_hyperfine_startup() {
    local blueos_binary=$1
    local json_path="${artifact_staging_directory}/hyperfine-blueos-startup.json"
    mkdir -p "$artifact_staging_directory"
    chmod +x "$blueos_binary"
    report_section "hyperfine blueos start-up (--help)" \
        --ok 0 --version hyperfine -- \
        hyperfine \
        --warmup 3 \
        --export-json "$json_path" \
        "${blueos_binary} --help"
}

main() {
    local reference_target blueos_binary
    reference_target=$(reference_target_from_uname)
    blueos_binary="${core_directory}/target/build/${reference_target}/${reference_target}/release/blueos"

    write_job_summary_header "$reference_target" "$blueos_binary"
    set_cpu_governor_performance | tee -a "${GITHUB_STEP_SUMMARY}"

    cd "$core_directory"
    rm -rf target/criterion

    report_section "cross-build blueos (${reference_target})" \
        --ok 0 --no-version -- \
        env TARGETS="${reference_target}" ./build_cross.sh

    run_criterion_benchmarks
    stage_criterion_reports
    run_hyperfine_startup "$blueos_binary"
}

main
