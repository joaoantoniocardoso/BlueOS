#!/usr/bin/env bash
# Probe a set of GET routes: capture HTTP status + body + latency percentiles
# (p50/p95/p99 over N repeats). Emits JSON keyed by route. Reusable across states
# (running / stopped / SITL). POST/mutating routes are handled outside this tool.
#
# Usage:
#   probe_http.sh --base <url> --gets "/a /b /c" [--repeats N] [--label L] [--out FILE]
set -euo pipefail

BASE=""; GETS=""; REPEATS=60; LABEL=""; OUT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --base) BASE="$2"; shift 2;; --gets) GETS="$2"; shift 2;;
    --repeats) REPEATS="$2"; shift 2;; --label) LABEL="$2"; shift 2;; --out) OUT="$2"; shift 2;;
    -h|--help) grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) echo "unknown arg: $1" >&2; exit 1;;
  esac
done
[ -n "$BASE" ] && [ -n "$GETS" ] || { echo "error: --base and --gets are required" >&2; exit 1; }

entries=""
for path in $GETS; do
  resp=$(curl -s -m 12 -w '\n%{http_code}' "$BASE$path" || echo $'\n000')
  status=$(printf '%s' "$resp" | tail -n1)
  body=$(printf '%s' "$resp" | sed '$d')
  : > /tmp/_lat
  for _ in $(seq 1 "$REPEATS"); do curl -s -m 12 -o /dev/null -w '%{time_total}\n' "$BASE$path" >> /tmp/_lat || echo 12 >> /tmp/_lat; done
  read -r p50 p95 p99 < <(sort -n /tmp/_lat | awk '{a[NR]=$1} END{n=NR; printf "%.1f %.1f %.1f\n", a[int(n*0.5)]*1000, a[int(n*0.95)]*1000, a[int(n*0.99)]*1000}')
  entry=$(jq -cn --arg s "$status" --arg b "$body" --argjson p50 "$p50" --argjson p95 "$p95" --argjson p99 "$p99" --argjson n "$REPEATS" \
    '{status: ($s|tonumber? // $s), body: $b, latency_ms: {p50:$p50, p95:$p95, p99:$p99, n:$n}}')
  entries=$(printf '%s\n%s\t%s' "$entries" "$path" "$entry")
done

json=$(printf '%s' "$entries" | jq -Rn --arg label "$LABEL" --arg base "$BASE" '
  {label: $label, base: $base, routes: (reduce (inputs | select(length>0) | split("\t")) as $e ({}; .[$e[0]] = ($e[1]|fromjson)))}')

if [ -n "$OUT" ]; then printf '%s\n' "$json" > "$OUT"; echo "wrote $OUT" >&2; else printf '%s\n' "$json"; fi
