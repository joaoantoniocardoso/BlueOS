#!/usr/bin/env bash
# One-shot catalog gate. Run after every service slice and on every resume.
# Fails fast on the first failing check. Invoke from anywhere: `bash catalog/gate.sh`.
set -euo pipefail
cd "$(dirname "$0")"

echo "== fmt =="
cargo fmt --check
echo "== clippy =="
cargo clippy --all-targets -- -D warnings
echo "== test (validate() exercised here) =="
cargo test
echo "== extract (observed layer vs core/start-blueos-core) =="
cargo run -q --bin extract
echo "== drift (asserted vs observed + runtime) =="
cargo run -q --bin drift
echo "== export (schema/json build) =="
cargo run -q --bin export >/dev/null
echo "ALL GATES GREEN"
