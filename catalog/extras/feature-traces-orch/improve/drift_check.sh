#!/usr/bin/env bash
# Phase 2 (T6) drift-detection script.
#
# Detects drift between the two generated JSON artifacts that must stay
# intro-commit-consistent for every OVERRIDES-touched journey (T7):
#   - catalog/feature_presence_map.json  (cargo run --bin generate_feature_presence)
#   - catalog/feature_traces.json        (cargo run --bin enrich_feature_traces)
#
# Full regen (NOT run automatically here — enrich hits `gh`/GitHub and needs a
# cache/token; local-only per orchestrator constraints, see IMPROVE_DESIGN.md T6).
# Run generate_feature_presence FIRST — enrich reads its OVERRIDES-derived intro
# commits (T7 ordering):
#
#   cd catalog
#   cargo run -p blueos-catalog --bin generate_feature_presence
#   cargo run -p blueos-catalog --bin enrich_feature_traces
#   ./extras/feature-traces-orch/improve/drift_check.sh
#
# Practical local check (no regen, no network): for every journey referenced
# in feature_presence.rs::OVERRIDES, assert intro_commit agrees between the
# two committed JSON files. A mismatch means one artifact is stale relative to
# the other — usually because OVERRIDES changed and only one regen bin was
# re-run.

set -euo pipefail

catalog_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
presence_json="${catalog_dir}/feature_presence_map.json"
traces_json="${catalog_dir}/feature_traces.json"
overrides_rs="${catalog_dir}/crates/catalog-git/src/feature_presence.rs"
id_rs="${catalog_dir}/crates/catalog-kernel/src/id/journey.rs"

for f in "$presence_json" "$traces_json" "$overrides_rs" "$id_rs"; do
  if [[ ! -f "$f" ]]; then
    echo "drift_check: missing ${f}" >&2
    exit 1
  fi
done

# Journey ids touched by an OVERRIDES entry, extracted from the const block
# itself (not hand-maintained, so this can't silently go stale). A journey
# name line is the tuple's first element, always immediately followed by an
# `Override::` line; this distinguishes it from Pickaxe search-term/path
# string literals, which are not.
mapfile -t journeys < <(
  awk '
    /pub\(crate\) const OVERRIDES/ { f = 1 }
    f && /^\];[ \t]*$/ { exit }
    f {
      if (prev ~ /^[ \t]*"[A-Za-z0-9_]+",[ \t]*$/ && $0 ~ /Override::/) {
        line = prev
        gsub(/^[ \t]*"/, "", line)
        gsub(/",[ \t]*$/, "", line)
        print line
      }
      prev = $0
    }
  ' "$overrides_rs" | sort -u
)

if [[ ${#journeys[@]} -eq 0 ]]; then
  echo "drift_check: could not extract any OVERRIDES journeys from ${overrides_rs}" >&2
  exit 1
fi

fail=0
wire_journey() {
  python3 - "$1" "$id_rs" <<'PY'
import re, sys
from pathlib import Path
variant, id_rs = sys.argv[1], Path(sys.argv[2]).read_text()
block = id_rs[id_rs.index("pub enum JourneyId") : id_rs.index("impl JourneyId")]
lines = block.splitlines()
for i, line in enumerate(lines):
    m = re.search(r'rename\s*=\s*"([^"]+)"', line)
    if m and i + 1 < len(lines):
        name = lines[i + 1].strip().rstrip(",")
        if name == variant:
            print(m.group(1))
            break
PY
}

for journey in "${journeys[@]}"; do
  wire=$(wire_journey "$journey" "$id_rs")
  if [[ -z "$wire" ]]; then
    echo "drift_check: no JourneyId wire id for OVERRIDES key ${journey}" >&2
    fail=1
    continue
  fi
  presence_intro=$(jq -r --arg j "$wire" '.journeys[] | select(.journey == $j) | .intro_commit' "$presence_json")
  traces_intro=$(jq -r --arg j "$wire" '.journeys[] | select(.journey == $j) | .intro_commit' "$traces_json")

  if [[ -z "$presence_intro" || -z "$traces_intro" ]]; then
    echo "DRIFT: ${journey} (${wire}) missing from one of the two JSON files (presence='${presence_intro}' traces='${traces_intro}')" >&2
    fail=1
    continue
  fi

  if [[ "$presence_intro" != "$traces_intro" ]]; then
    echo "DRIFT: ${journey} intro_commit mismatch: feature_presence_map.json=${presence_intro} feature_traces.json=${traces_intro}" >&2
    fail=1
  fi
done

if [[ "$fail" -ne 0 ]]; then
  cat >&2 <<'EOF'

Fix by re-running the regen pair in order (T7), then re-run this script:
  cd catalog
  cargo run -p blueos-catalog --bin generate_feature_presence
  cargo run -p blueos-catalog --bin enrich_feature_traces
EOF
  exit 1
fi

echo "drift_check: ${#journeys[@]} OVERRIDES journeys agree between feature_presence_map.json and feature_traces.json"
