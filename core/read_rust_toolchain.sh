#!/usr/bin/env bash
# Reads core/rust-toolchain.toml for CI and hooks (D-33). Source this file; do not execute it.
# shellcheck disable=SC2034 # variables are consumed by the sourcing script
set -euo pipefail

RUST_TOOLCHAIN_FILE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/rust-toolchain.toml"

read_rust_toolchain_scalar() {
    local key=$1
    grep -E "^${key}[[:space:]]*=" "$RUST_TOOLCHAIN_FILE" | sed -E 's/^[^"]*"([^"]+)".*$/\1/'
}

read_rust_toolchain_list() {
    local key=$1
    grep -E "^${key}[[:space:]]*=" "$RUST_TOOLCHAIN_FILE" \
        | sed -E 's/^[^[]*\[([^]]*)\].*$/\1/' \
        | tr -d '"' \
        | tr -d '[:space:]'
}

# Variables are read when this file is sourced.
# shellcheck disable=SC2034
RUST_TOOLCHAIN_CHANNEL=$(read_rust_toolchain_scalar channel)
# shellcheck disable=SC2034
RUST_TOOLCHAIN_COMPONENTS=$(read_rust_toolchain_list components)
# shellcheck disable=SC2034
RUST_TOOLCHAIN_TARGETS=$(read_rust_toolchain_list targets)
