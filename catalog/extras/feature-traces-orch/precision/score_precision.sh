#!/usr/bin/env bash
# Phase 4b precision scorer scaffold.
#
# Prints the per-journey golden fields (follow_up_prs, backport_prs, issues,
# landing_prs, discovery_paths) so they can be diffed against the goldens
# frozen in PRECISION_DESIGN.md §5 / ORCHESTRATOR_PROMPT.xml.
#
# Journey data lives at .intro_clusters[<sha>].by_journey[<name>]; landing_prs
# is a sibling of by_journey on the same cluster object (not per-journey), so
# it is pulled from whichever cluster contains that journey key.
#
# Usage:
#   score_precision.sh <feature_traces.json> <JourneyName> [<JourneyName>...]
#
# Example (the 5 goldens from PRECISION_DESIGN.md §5):
#   ./score_precision.sh ../../../feature_traces.json \
#     InspectZenohNetwork ChangeUiThemeColor InspectDiskUsage \
#     RunInternetSpeedTest LevelHorizon

set -euo pipefail

if [[ $# -lt 2 ]]; then
  echo "usage: $0 <feature_traces.json> <JourneyName> [<JourneyName>...]" >&2
  exit 1
fi

json="$1"
shift

for journey in "$@"; do
  echo "=== ${journey} ==="
  jq --arg j "$journey" '
    [.intro_clusters[] | select(.by_journey[$j] != null)][0] as $cluster
    | if $cluster == null then
        {error: "journey not found"}
      else
        $cluster.by_journey[$j] + {landing_prs: ($cluster.landing_prs // [])}
      end
  ' "$json"
  echo
done

# --- Hub precision (PRECISION_DESIGN.md §5) --------------------------------
# NOT a stub: .pull_requests[<number>].files_changed is present in the JSON,
# so precision can be computed directly here rather than needing gh/cache.
#
# hub precision % = |{pr in follow_up_prs : files_changed(pr) intersects
#                     discovery_paths(journey)}| / |follow_up_prs| * 100
#
# "Intersects" should match discovery_path_covers semantics in
# feature_trace_enrich.rs: a file counts if it equals a discovery path, or is
# nested under one that names a directory (prefix match on "dir/"), not a
# bare substring match. This scorer only reads the frozen JSON; it does not
# re-derive discovery_paths, so it cannot judge whether the hint set itself
# is correct — only whether follow-ups agree with the hints already stored.
#
# TODO(Phase 4b real run): add a --precision flag that, per journey, computes
# the ratio above via:
#   jq --arg j "$journey" '
#     [.intro_clusters[] | select(.by_journey[$j] != null)][0] as $c
#     | $c.by_journey[$j] as $bj
#     | ($bj.follow_up_prs | length) as $n
#     | [ $bj.follow_up_prs[] as $pr
#         | ($pr|tostring) as $prs
#         | (.pull_requests[$prs].files_changed // []) as $files
#         | any($files[]; . as $f | any($bj.discovery_paths[]; $f == . or ($f|startswith(. + "/"))))
#       ] as $hits
#     | if $n == 0 then null else (($hits|map(select(.))|length) / $n * 100) end
#   ' "$json"
# Left commented (not wired to a flag) per task: scaffold only, no full enrich/QA run.
