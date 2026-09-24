#!/usr/bin/env bash
set -euo pipefail

DEFAULT_TARGETS=(
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
    "armv7-unknown-linux-musleabihf"
)
if [[ -n "${TARGETS:-}" ]]; then
    read -r -a TARGETS_ARRAY <<< "${TARGETS}"
else
    TARGETS_ARRAY=("${DEFAULT_TARGETS[@]}")
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "${ROOT}"

for TARGET in "${TARGETS_ARRAY[@]}"; do
    export CARGO_TARGET_DIR="target/build/${TARGET}"

    echo "Building blueos for target: ${TARGET}"
    cross build --release --locked --features recorder -p blueos --target "${TARGET}" "$@"
done

echo "All builds completed successfully."
