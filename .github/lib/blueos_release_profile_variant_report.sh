#!/usr/bin/env bash
# Build blueos with one release-profile override and report size and build time (D-33).
# The committed [profile.release] in core/Cargo.toml is unchanged; overrides use cargo --config.

set -euo pipefail

if [ "$#" -ne 2 ]; then
    printf 'Usage: %s <target> <variant>\n' "$0" >&2
    printf 'Variants: opt-level-s, opt-level-z, codegen-units-1, lto-fat\n' >&2
    exit 2
fi

target=$1
variant=$2

case "$variant" in
    opt-level-s)
        cargo_config=(--config 'profile.release.opt-level="s"')
        ;;
    opt-level-z)
        cargo_config=(--config 'profile.release.opt-level="z"')
        ;;
    codegen-units-1)
        cargo_config=(--config 'profile.release.codegen-units=1')
        ;;
    lto-fat)
        cargo_config=(--config 'profile.release.lto="fat"')
        ;;
    *)
        printf 'unknown variant: %s\n' "$variant" >&2
        exit 2
        ;;
esac

repository_root=$(git rev-parse --show-toplevel)

printf '%s\n' 'comparison_guideline (D-33): About 1 MB of stripped binary is worth about 1 second of CI build time when comparing release-profile variants. A measurable slowdown in the load scenario rules a setting out.'
printf 'variant: %s\n' "$variant"
printf 'target: %s\n' "$target"

SECONDS=0
TARGETS="$target" "${repository_root}/core/build_cross.sh" "${cargo_config[@]}"
build_time_seconds=$SECONDS
printf 'build_time_seconds: %s\n' "$build_time_seconds"

binary="${repository_root}/core/target/build/${target}/${target}/release/blueos"
"${repository_root}/.github/lib/blueos_binary_size_report.sh" "$binary" "$target"
