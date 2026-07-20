#!/usr/bin/env bash
# Release/QA workflow: regenerate HTML report and optionally diff two snapshots.
# Invoke from anywhere: `bash catalog/extras/feature-traces-orch/abc/release_traces_workflow.sh`
set -euo pipefail

abc_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
orch_dir="$(cd "$abc_dir/.." && pwd)"
catalog_dir="$(cd "$orch_dir/../.." && pwd)"
cd "$catalog_dir"

out_dir="${orch_dir}/reports"
journeys_mode="golden"
old_path=""

usage() {
  cat <<'EOF'
usage: release_traces_workflow.sh [options]

  --out-dir PATH   HTML report output directory (default: extras/feature-traces-orch/reports)
  --old PATH       Compare PATH against catalog/feature_traces.json via feature_trace_diff
  --journeys MODE  golden (default) | all | list J1,J2,...

Regenerates feature_trace_report.html, optionally prints a provenance diff, then
prints the remaining release checklist steps.
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --out-dir)
      shift
      out_dir="${1:?--out-dir requires a path}"
      ;;
    --old)
      shift
      old_path="${1:?--old requires a path}"
      ;;
    --journeys)
      shift
      journeys_mode="${1:?--journeys requires golden, all, or list}"
      shift
      if [ "$journeys_mode" = list ]; then
        journey_list=("$@")
        set --
      fi
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "release_traces_workflow.sh: unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

traces_json="${catalog_dir}/feature_traces.json"
if [ ! -f "$traces_json" ]; then
  echo "release_traces_workflow.sh: missing $traces_json" >&2
  exit 1
fi

run_report() {
  local -a report_args=(--format html --output "$out_dir")
  if [ -n "${1:-}" ]; then
    report_args=(--journey "$1" "${report_args[@]}")
  fi
  cargo run -q -p blueos-catalog --bin feature_trace_report -- "${report_args[@]}"
}

mkdir -p "$out_dir"

echo "== feature_trace_report (journeys=$journeys_mode) =="
case "$journeys_mode" in
  golden)
    run_report
  ;;
  all)
    while IFS= read -r journey_id; do
      run_report "$journey_id"
      mv -f "${out_dir}/feature_trace_report.html" "${out_dir}/${journey_id}.html"
    done < <(jq -r '.journeys[].journey' "$traces_json")
    echo "Wrote per-journey HTML under ${out_dir}/"
  ;;
  list)
    if [ "${#journey_list[@]}" -eq 0 ]; then
      echo "release_traces_workflow.sh: --journeys list requires J1,J2,..." >&2
      exit 2
    fi
    IFS=',' read -r -a ids <<<"${journey_list[0]}"
    if [ "${#ids[@]}" -eq 1 ] && [ "${#journey_list[@]}" -gt 1 ]; then
      ids=("${journey_list[@]}")
    fi
    for journey_id in "${ids[@]}"; do
      journey_id="${journey_id#"${journey_id%%[![:space:]]*}"}"
      journey_id="${journey_id%"${journey_id##*[![:space:]]}"}"
      [ -n "$journey_id" ] || continue
      run_report "$journey_id"
      if [ "${#ids[@]}" -gt 1 ]; then
        mv -f "${out_dir}/feature_trace_report.html" "${out_dir}/${journey_id}.html"
      fi
    done
    if [ "${#ids[@]}" -gt 1 ]; then
      echo "Wrote per-journey HTML under ${out_dir}/"
    fi
  ;;
  *)
    echo "release_traces_workflow.sh: unknown --journeys mode: $journeys_mode" >&2
    exit 2
  ;;
esac

html_path="${out_dir}/feature_trace_report.html"
if [ -f "$html_path" ]; then
  echo "HTML report: $html_path"
fi

if [ -n "$old_path" ]; then
  if [ ! -f "$old_path" ]; then
    echo "release_traces_workflow.sh: --old file not found: $old_path" >&2
    exit 1
  fi
  echo "== feature_trace_diff =="
  cargo run -q -p blueos-catalog --bin feature_trace_diff -- "$old_path" "$traces_json"
fi

cat <<EOF

== Next release checklist steps ==
1. Review HTML report: ${html_path:-$out_dir}
2. bash extras/feature-traces-orch/next11/check-traces.sh
3. bash gate.sh
See extras/feature-traces-orch/abc/RELEASE_CHECKLIST.md for the full operator flow.
EOF
