#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck disable=SC1091
source "$ROOT/core/read_rust_toolchain.sh"

if [ -z "${GITHUB_OUTPUT:-}" ]; then
    printf 'write_stable_rust_toolchain_outputs.sh: GITHUB_OUTPUT is not set\n' >&2
    exit 1
fi

{
    echo "channel=$RUST_TOOLCHAIN_CHANNEL"
    echo "components=$RUST_TOOLCHAIN_COMPONENTS"
    echo "targets=$RUST_TOOLCHAIN_TARGETS"
} >>"$GITHUB_OUTPUT"
