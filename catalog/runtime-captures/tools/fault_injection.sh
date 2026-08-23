#!/usr/bin/env bash
# Kill a tmux session or a single process inside the BlueOS core container, then watch dependent
# routes through a timed window. Answers what a status code alone cannot: does the dependent API
# degrade visibly, or keep returning a silent 200 over stale data?
#
# Prefer --kill-process when the modelled failure mode is one subprocess dying: a BlueOS tmux
# session holds the supervising service too, so killing the session also kills the manager whose
# auto-restart is the behaviour under test.
#
# Freshness comes from a monotonic counter in a JSON body (mavlink2rest exposes
# .status.time.counter per message), so a stalled stream is detectable while HTTP stays 200.
# Sampling starts before the kill so the baseline proves the counter was advancing, and runs long
# enough afterwards to catch an auto-restart masking the fault.
#
# Usage:
#   fault_injection.sh --host <ip> (--session <tmux-session> | --kill-process <exact-name>)
#                      --freshness-url <url>
#                      [--freshness-filter <jq-path>] [--watch "<url> <url>"]
#                      [--user pi] [--password raspberry] [--container blueos-core]
#                      [--baseline S] [--duration S] [--interval S] [--stale-after S]
#                      [--restore-post <url>] [--label L] [--out FILE] [--dry-run]
#
# --restore-post is POSTed after the observation window to put the system back, and the result is
# sampled and reported. Pass it whenever the fault is not known to self-heal.
#
# --stale-after guards against a false stall: sampling faster than the stream produces repeated
# counters in healthy operation, so only a gap this long counts as stale (default 3s, i.e. 3
# missed beats of a 1Hz HEARTBEAT).
set -euo pipefail

HOST=""; SESSION=""; KILL_PROCESS=""; FRESH_URL=""; FRESH_FILTER=".status.time.counter"; WATCH=""
USER_NAME="pi"; PASSWORD="raspberry"; CONTAINER="blueos-core"
BASELINE=5; DURATION=60; INTERVAL=0.5; STALE_AFTER=3; RESTORE_POST=""
LABEL=""; OUT=""; DRY_RUN=0
while [ $# -gt 0 ]; do
  case "$1" in
    --host) HOST="$2"; shift 2;; --session) SESSION="$2"; shift 2;;
    --kill-process) KILL_PROCESS="$2"; shift 2;;
    --freshness-url) FRESH_URL="$2"; shift 2;; --freshness-filter) FRESH_FILTER="$2"; shift 2;;
    --watch) WATCH="$2"; shift 2;; --user) USER_NAME="$2"; shift 2;;
    --password) PASSWORD="$2"; shift 2;; --container) CONTAINER="$2"; shift 2;;
    --baseline) BASELINE="$2"; shift 2;; --duration) DURATION="$2"; shift 2;;
    --interval) INTERVAL="$2"; shift 2;; --stale-after) STALE_AFTER="$2"; shift 2;;
    --restore-post) RESTORE_POST="$2"; shift 2;; --label) LABEL="$2"; shift 2;;
    --out) OUT="$2"; shift 2;; --dry-run) DRY_RUN=1; shift;;
    -h|--help) grep '^#' "$0" | sed 's/^# \{0,1\}//'; exit 0;;
    *) echo "unknown arg: $1" >&2; exit 1;;
  esac
done
[ -n "$HOST" ] && [ -n "$FRESH_URL" ] || {
  echo "error: --host and --freshness-url are required" >&2; exit 1; }
if { [ -n "$SESSION" ] && [ -n "$KILL_PROCESS" ]; } || { [ -z "$SESSION" ] && [ -z "$KILL_PROCESS" ]; }; then
  echo "error: pass exactly one of --session or --kill-process" >&2; exit 1
fi
if [ -n "$SESSION" ]; then
  KILL_CMD="tmux kill-session -t $SESSION"; TARGET="session:$SESSION"
else
  # -x matches the exact process name, so a pattern cannot sweep up unintended children.
  KILL_CMD="pkill -x $KILL_PROCESS"; TARGET="process:$KILL_PROCESS"
fi

SAMPLES=$(mktemp); trap 'rm -f "$SAMPLES"' EXIT

# epoch, phase, freshness counter ("null" when unreadable), freshness HTTP status, then one status
# per --watch URL. Kept as TSV so the sampling loop stays cheap relative to --interval.
sample() {
  local phase="$1" now counter status line
  now=$(date +%s.%N)
  status=$(curl -s -m 5 -o /tmp/_fi_body -w '%{http_code}' "$FRESH_URL" || echo 000)
  counter=$(jq -r "$FRESH_FILTER // \"null\"" < /tmp/_fi_body 2>/dev/null || echo null)
  line=$(printf '%s\t%s\t%s\t%s' "$now" "$phase" "$counter" "$status")
  for url in $WATCH; do
    line+=$(printf '\t%s' "$(curl -s -m 5 -o /dev/null -w '%{http_code}' "$url" || echo 000)")
  done
  printf '%s\n' "$line" >> "$SAMPLES"
}

sample_for() {
  local phase="$1" deadline
  deadline=$(( $(date +%s) + $2 ))
  while [ "$(date +%s)" -lt "$deadline" ]; do
    sample "$phase"
    sleep "$INTERVAL"
  done
}

echo "baseline: sampling ${BASELINE}s before killing $TARGET" >&2
sample_for baseline "$BASELINE"

kill_cmd="docker exec $CONTAINER $KILL_CMD"
if [ "$DRY_RUN" -eq 1 ]; then
  echo "dry-run: would run '$kill_cmd' on $HOST" >&2
  KILLED_AT="null"; KILL_OUTPUT="dry-run"
else
  echo "injecting: $kill_cmd" >&2
  KILLED_AT=$(date +%s.%N)
  KILL_OUTPUT=$(SSHPASS="$PASSWORD" sshpass -e ssh -o StrictHostKeyChecking=no \
    -o UserKnownHostsFile=/dev/null -o LogLevel=ERROR -o ConnectTimeout=10 \
    "$USER_NAME@$HOST" "$kill_cmd" 2>&1 || true)
fi

echo "observing: sampling ${DURATION}s after the kill" >&2
sample_for after "$DURATION"

# A fault injector that cannot put the system back is not safe to run against a shared device, so
# restoration is part of the measurement and its effect is sampled like any other phase.
RESTORE_STATUS="null"
if [ -n "$RESTORE_POST" ] && [ "$DRY_RUN" -eq 0 ]; then
  echo "restoring: POST $RESTORE_POST" >&2
  RESTORE_STATUS=$(curl -s -m 120 -o /dev/null -w '%{http_code}' -X POST "$RESTORE_POST" || echo 000)
  echo "restore returned $RESTORE_STATUS; sampling ${BASELINE}s to confirm" >&2
  sample_for restored "$BASELINE"
fi

json=$(jq -Rn \
  --arg label "$LABEL" --arg host "$HOST" --arg target "$TARGET" --arg kill_cmd "$kill_cmd" \
  --arg fresh_url "$FRESH_URL" --arg fresh_filter "$FRESH_FILTER" --arg watch "$WATCH" \
  --arg kill_output "$KILL_OUTPUT" --argjson killed_at "${KILLED_AT:-null}" \
  --argjson baseline "$BASELINE" --argjson duration "$DURATION" --arg interval "$INTERVAL" \
  --argjson stale_after "$STALE_AFTER" --arg restore_post "$RESTORE_POST" \
  --argjson restore_status "${RESTORE_STATUS:-null}" '
  [inputs | select(length > 0) | split("\t")
   | {t: (.[0]|tonumber), phase: .[1], counter: (.[2]|tonumber? // null),
      freshness_status: (.[3]|tonumber? // .[3]), watch: .[4:] | map(tonumber? // .)}]
  | sort_by(.t)
  # Sampling faster than the stream means consecutive equal counters are normal, so staleness is
  # measured as elapsed time since the counter last advanced -- never as a single repeated reading.
  | [foreach .[] as $x ({advanced_at: null, previous: null};
       if $x.counter != null and (.previous == null or $x.counter > .previous)
       then {advanced_at: $x.t, previous: $x.counter} else . end;
       $x + {since_advance: (if .advanced_at == null then null
                             else (($x.t - .advanced_at) * 1000 | round) / 1000 end)})] as $s
  | ($watch | split(" ") | map(select(length > 0))) as $watch_urls
  | ($s | map(select(.phase == "baseline"))) as $before
  | ($s | map(select(.phase == "after"))) as $after
  | ($before | map(.counter) | map(select(. != null))) as $bc
  | ($bc | last) as $last_healthy
  | ($after | map(select(.counter != null and .counter > $last_healthy)) | first) as $first_fresh
  | ($after | map(select(.since_advance != null and .since_advance >= $stale_after))) as $stale
  | {
      label: $label, host: $host, target: $target,
      method: ("`\($kill_cmd)`, sampled through nginx before and after"),
      parameters: {freshness_url: $fresh_url, freshness_filter: $fresh_filter,
                   watch_urls: $watch_urls, baseline_s: $baseline, duration_s: $duration,
                   interval_s: ($interval|tonumber)},
      kill_output: $kill_output, killed_at: $killed_at,
      baseline: {
        samples: ($before | length),
        counter_advanced: (($bc | length) > 1 and ($bc | last) > ($bc | first)),
        counter_first: ($bc | first), counter_last: ($bc | last)
      },
      recovery: {
        samples: ($after | length),
        # Longest observed gap with no new data: the actual outage the fault caused.
        max_stall_seconds: ($after | map(.since_advance) | map(select(. != null)) | max),
        stale_samples: ($stale | length),
        # Buffered messages can deliver one more beat after the fault, so a single new counter is
        # not recovery. Recovery means data was still flowing when the window closed.
        first_new_counter_after_s: (if $first_fresh == null or $killed_at == null then null
                                    else (($first_fresh.t - $killed_at) * 1000 | round) / 1000 end),
        self_recovered: (($after | last | .since_advance) < $stale_after),
        counter_last: ($after | map(.counter) | map(select(. != null)) | last)
      },
      restore: (if $restore_post == "" then null else {
        post: $restore_post, status: $restore_status,
        samples: ($s | map(select(.phase == "restored")) | length),
        # Same test as self_recovered, applied after the restore call.
        restored: (($s | map(select(.phase == "restored")) | last | .since_advance) < $stale_after)
      } end),
      # The point of the probe: a dependent route that never leaves 200 while the counter is stale
      # is reporting health it cannot know.
      freshness_status_histogram: ($s | map(.freshness_status | tostring)
                                    | group_by(.) | map({key: .[0], value: length}) | from_entries),
      watch_status_histogram: (reduce range(0; $watch_urls | length) as $i ({};
        .[$watch_urls[$i]] = ($s | map(.watch[$i] | tostring) | group_by(.)
                              | map({key: .[0], value: length}) | from_entries))),
      # Per route, not aggregated: one dependent that correctly fails must not hide another that
      # keeps answering 200 over data it knows nothing about.
      silent_200_while_stale: (if ($stale | length) == 0 then {} else
        ({($fresh_url): ($stale | all(.freshness_status == 200))}
         + (reduce range(0; $watch_urls | length) as $i ({};
             .[$watch_urls[$i]] = ($stale | all(.watch[$i] == 200))))) end),
      samples: $s
    }' < "$SAMPLES")

if [ -n "$OUT" ]; then printf '%s\n' "$json" > "$OUT"; echo "wrote $OUT" >&2; else printf '%s\n' "$json"; fi
