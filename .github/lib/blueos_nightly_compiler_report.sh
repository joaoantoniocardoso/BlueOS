#!/usr/bin/env bash
# Nightly-only compiler measurements for the pinned nightly job (D-33).

set -euo pipefail

if [ "$#" -ne 2 ]; then
    printf 'Usage: %s <mode> <output-directory>\n' "$0" >&2
    printf 'Modes: self-profile, macro-stats, build-std-size\n' >&2
    exit 2
fi

mode=$1
output_directory=$2
mkdir -p "$output_directory"

repository_root=$(git rev-parse --show-toplevel)
core_directory="${repository_root}/core"
# shellcheck disable=SC1091
source "${core_directory}/shipped_features.sh"

nightly_toolchain=$(tr -d '[:space:]' <"${core_directory}/nightly-toolchain.txt")
host_target=x86_64-unknown-linux-gnu

cd "${core_directory}"
shopt -s nullglob
stale_profiles=(*.mm_profdata)
if [ "${#stale_profiles[@]}" -gt 0 ]; then
    rm -f "${stale_profiles[@]}"
fi

build_blueos() {
    cargo "+${nightly_toolchain}" build --release --locked \
        -p "${BLUEOS_SHIPPED_PACKAGE}" \
        --features "${BLUEOS_SHIPPED_FEATURES[*]}" \
        --target "${host_target}" \
        "$@"
}

case "$mode" in
    self-profile)
        measurement_target_directory="${output_directory}/self-profile/target"
        export CARGO_TARGET_DIR="${measurement_target_directory}"
        export RUSTFLAGS='-Zself-profile'
        build_blueos
        mkdir -p "${output_directory}/self-profile/raw"
        profiles=(*.mm_profdata)
        if [ "${#profiles[@]}" -eq 0 ]; then
            printf 'no self-profile data files after build\n' >&2
            exit 1
        fi
        mv "${profiles[@]}" "${output_directory}/self-profile/raw/"
        blueos_profile=$(
            find "${output_directory}/self-profile/raw" -maxdepth 1 -name 'blueos-*.mm_profdata' \
                ! -name 'blueos_*' -printf '%s %p\n' | sort -n | tail -1 | cut -d' ' -f2-
        )
        if [ -z "${blueos_profile:-}" ]; then
            printf 'no blueos self-profile data file after build\n' >&2
            exit 1
        fi
        profile_prefix="${blueos_profile%.mm_profdata}"
        summarize summarize "${profile_prefix}" | tee "${output_directory}/self-profile/summary.txt"
        ;;
    macro-stats)
        measurement_target_directory="${output_directory}/macro-stats/target"
        export CARGO_TARGET_DIR="${measurement_target_directory}"
        export RUSTFLAGS='-Zmacro-stats'
        macro_stats_log="${output_directory}/macro-stats/build.log"
        mkdir -p "${output_directory}/macro-stats"
        build_blueos 2>"${macro_stats_log}"
        grep '^macro-stats' "${macro_stats_log}" >"${output_directory}/macro-stats/macro-stats.txt"
        cat "${output_directory}/macro-stats/macro-stats.txt"
        ;;
    build-std-size)
        measurement_target_directory="${output_directory}/build-std-size/target"
        export CARGO_TARGET_DIR="${measurement_target_directory}"
        export RUSTFLAGS='-Zlocation-detail=none'
        printf 'experiment: build-std=std with -Zlocation-detail=none on %s (report only; not a shipped profile)\n' \
            "$host_target"
        build_blueos -Z build-std=std
        binary="${measurement_target_directory}/${host_target}/release/blueos"
        "${repository_root}/.github/lib/blueos_binary_size_report.sh" "$binary" "$host_target"
        ;;
    *)
        printf 'unknown mode: %s\n' "$mode" >&2
        exit 2
        ;;
esac
