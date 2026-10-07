#!/usr/bin/env bash
# Report stripped and zstd-compressed size of a release blueos binary (D-33).

set -euo pipefail

if [ "$#" -ne 2 ]; then
    printf 'Usage: %s <binary-path> <target>\n' "$0" >&2
    exit 2
fi

binary=$1
target=$2

if [ ! -f "$binary" ]; then
    printf 'binary not found: %s\n' "$binary" >&2
    exit 1
fi

stripped_bytes=$(wc -c <"$binary" | tr -d ' ')
zstd_bytes=$(zstd -q -c "$binary" | wc -c | tr -d ' ')

printf 'target: %s\n' "$target"
printf 'stripped_bytes: %s\n' "$stripped_bytes"
printf 'zstd_bytes: %s\n' "$zstd_bytes"
if command -v numfmt >/dev/null 2>&1; then
    printf 'stripped: %s\n' "$(numfmt --to=iec-i --suffix=B "$stripped_bytes")"
    printf 'zstd: %s\n' "$(numfmt --to=iec-i --suffix=B "$zstd_bytes")"
fi
