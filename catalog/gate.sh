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
echo "== harness ratchet (committed baseline, no BLUEOS_BASE) =="
cargo run -q --bin harness_ratchet
echo "== api contracts (inventory vs committed baseline) =="
cargo run -q --bin api_contracts -- --check
echo "== extract (observed layer vs core/start-blueos-core and nginx.conf) =="
cargo run -q --bin extract
echo "== drift (asserted vs observed + runtime) =="
cargo run -q --bin drift
echo "== provenance (citations resolve) =="
cargo run -q --bin provenance_lint
echo "== export (schema/json build) =="
cargo run -q --bin export >/dev/null
echo "== sysml export (subset golden) =="
cargo run -q --bin sysml_export -- --check
# feature-traces local gate (NEXT15 P3): drift_check + sibling_matrix +
# --check-goldens, all offline/no-`gh` (mirrors N3's --strict-goldens opt-out,
# which stays a separate manual step). Skipped if the script isn't present
# (e.g. an older checkout) so gate.sh never hard-depends on the extras tree.
check_traces="extras/feature-traces-orch/next11/check-traces.sh"
if [ -f "$check_traces" ]; then
  echo "== check-traces (feature-traces local gate) =="
  bash "$check_traces"
fi
if [ -n "${BLUEOS_BASE:-}" ]; then
  echo "== journey_smoke (BLUEOS_BASE=$BLUEOS_BASE) =="
  cargo run -q --bin journey_http -- --base "$BLUEOS_BASE" --smoke --fixtures "${BLUEOS_SMOKE_FIXTURES:-internet,pirate,advanced}"
fi
echo "ALL GATES GREEN"
