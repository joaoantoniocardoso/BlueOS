#!/usr/bin/env bash
# Local feature-traces gate (`NEXT11_DESIGN.md` N3). Runs the 3 checks already
# in ad hoc use, in order; exits non-zero on the first failure. Invoke from
# anywhere: `bash catalog/extras/feature-traces-orch/next11/check-traces.sh`.
#
# NOT run here: `cargo run -p blueos-catalog --bin enrich_feature_traces --
# --strict-goldens`. That's the heavy, opt-in gate — it hits `gh`/GitHub and
# needs a cache/token, so it's a separate manual step, not part of this local
# gate:
#
#   cd catalog
#   cargo run -p blueos-catalog --bin enrich_feature_traces -- --strict-goldens
set -euo pipefail

catalog_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$catalog_dir"

echo "== drift_check (feature_presence_map.json vs feature_traces.json) =="
./extras/feature-traces-orch/improve/drift_check.sh

echo "== sibling_matrix (sibling-ratio regression gates) =="
cargo run -q -p blueos-catalog --bin sibling_matrix

echo "== feature_trace_report --check-goldens =="
cargo run -q -p blueos-catalog --bin feature_trace_report -- --check-goldens

echo "ALL FEATURE-TRACES GATES GREEN"
