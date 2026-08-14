#!/usr/bin/env bash
# W3 reversible mutating subset. RF skipped via HOST_WIFI_IFACE.
# Usage: run_w3.sh <base-url> <report-dir>
set -u
BASE="${1:?base url}"
OUT="${2:?report dir}"
BIN="${3:-$(dirname "$0")/bin/journey_http-w2}"
mkdir -p "$OUT"
export HOST_WIFI_IFACE="${HOST_WIFI_IFACE:-__qa_no_rf__}"

JOURNEYS=(
  run_lan_speed_test
  rename_vehicle
  modify_bag_database
  add_custom_manifest
  toggle_smart_hotspot
  configure_hotspot_credentials
  remove_configured_nmea_socket
)

case "$BASE" in
  *192.168.2.2*)
    # USB vehicle: no mDNS rename (discovery), no IP/DHCP/priority
    ;;
  *)
    JOURNEYS+=(change_mdns_hostname)
    ;;
esac

: >"$OUT/w3-mutating.log"
echo "W3 base=$BASE iface=$HOST_WIFI_IFACE" | tee -a "$OUT/w3-mutating.log"
FAIL=0
for j in "${JOURNEYS[@]}"; do
  echo "===== $j =====" | tee -a "$OUT/w3-mutating.log"
  "$BIN" --base "$BASE" --mutating-smoke \
    --journey "$j" --report "$OUT/w3-${j}.json" \
    >>"$OUT/w3-mutating.log" 2>&1
  rc=$?
  echo "EXIT_$j=$rc" | tee -a "$OUT/w3-mutating.log"
  if [ "$rc" -ne 0 ]; then FAIL=1; fi
done
echo "W3_DONE FAIL=$FAIL" | tee -a "$OUT/w3-mutating.log"
exit "$FAIL"
