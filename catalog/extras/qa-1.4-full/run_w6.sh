#!/usr/bin/env bash
# W6 destructive subset — 177 only. No shutdown. No EEPROM. No firmware flash.
# Usage: run_w6.sh <base-url> <report-dir>
set -u
BASE="${1:?base url}"
OUT="${2:?report dir}"
BIN="${3:-$(dirname "$0")/bin/journey_http}"
mkdir -p "$OUT"
export HOST_WIFI_IFACE="${HOST_WIFI_IFACE:-__qa_no_rf__}"

# Alias switch first (DUT must be up). Reboot last.
JOURNEYS=(
  switch_local_blueos_version
  reboot_onboard_computer
)

: >"$OUT/w6-destructive.log"
echo "W6 base=$BASE" | tee -a "$OUT/w6-destructive.log"
FAIL=0
for j in "${JOURNEYS[@]}"; do
  echo "===== $j =====" | tee -a "$OUT/w6-destructive.log"
  "$BIN" --base "$BASE" --mutating-smoke \
    --journey "$j" --report "$OUT/w6-${j}.json" \
    >>"$OUT/w6-destructive.log" 2>&1
  rc=$?
  echo "EXIT_$j=$rc" | tee -a "$OUT/w6-destructive.log"
  if [ "$rc" -ne 0 ]; then FAIL=1; fi
done
echo "W6_DONE FAIL=$FAIL" | tee -a "$OUT/w6-destructive.log"
exit "$FAIL"
