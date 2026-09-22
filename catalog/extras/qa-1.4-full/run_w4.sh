#!/usr/bin/env bash
# W4 RF wifi mutating — ONE DUT at a time. Uses host wlp8s0 from config.env.
# Usage: run_w4.sh <base-url> <report-dir>
set -u
BASE="${1:?base url}"
OUT="${2:?report dir}"
BIN="${3:-$(dirname "$0")/bin/journey_http}"
LOCK="$(dirname "$0")/locks/host-wifi"
mkdir -p "$OUT" "$(dirname "$LOCK")"
if [ -e "$LOCK" ]; then
  echo "RF lock held: $LOCK" >&2
  exit 2
fi
echo "$$ $BASE $(date -Iseconds)" >"$LOCK"
trap 'rm -f "$LOCK"' EXIT

# Do not override HOST_WIFI_IFACE — config.env wlp8s0 must be used.
unset HOST_WIFI_IFACE || true

JOURNEYS=(
  forget_saved_wifi_network
  connect_to_wifi_network
  connect_to_hidden_wifi_network
  reject_invalid_wifi_credentials
  reconnect_to_saved_wifi_network
  force_wifi_network_password
  disconnect_from_wifi_network
  detect_wifi_ap_loss
  autoconnect_to_saved_wifi_network
  toggle_hotspot
)

: >"$OUT/w4-rf.log"
echo "W4 base=$BASE" | tee -a "$OUT/w4-rf.log"
FAIL=0
for j in "${JOURNEYS[@]}"; do
  echo "===== $j =====" | tee -a "$OUT/w4-rf.log"
  "$BIN" --base "$BASE" --mutating-smoke \
    --journey "$j" --report "$OUT/w4-${j}.json" \
    >>"$OUT/w4-rf.log" 2>&1
  rc=$?
  echo "EXIT_$j=$rc" | tee -a "$OUT/w4-rf.log"
  if [ "$rc" -ne 0 ]; then FAIL=1; fi
done
echo "W4_DONE FAIL=$FAIL" | tee -a "$OUT/w4-rf.log"
exit "$FAIL"
