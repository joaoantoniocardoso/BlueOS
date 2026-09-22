#!/usr/bin/env bash
# Full catalog QA on 1.4-dev — all waves, no safety skips (bench DUTs).
# Usage: run_full_campaign.sh [dut-ip ...]
set -uo pipefail

CAMPAIGN_ROOT="$(cd "$(dirname "$0")" && pwd)"
CATALOG_DIR="$(cd "$CAMPAIGN_ROOT/../.." && pwd)"
TAG="1.4-dev"
UTC="$(date -u +%Y-%m-%dT%H%M%SZ)"
FIXTURES="internet,pirate,advanced"
JH="$CAMPAIGN_ROOT/bin/journey_http"

if [ "$#" -gt 0 ]; then
  DUTS=("$@")
else
  DUTS=(192.168.0.177 192.168.0.124 192.168.2.2)
fi

run_jh() {
  "$JH" "$@"
}

report_dir() {
  local ip="$1"
  echo "$CAMPAIGN_ROOT/reports/$ip/$TAG"
}

mkdir -p "$CAMPAIGN_ROOT/locks"

w0_inventory() {
  local ip="$1"
  local out
  out="$(report_dir "$ip")"
  mkdir -p "$out"
  local md="$out/w0-inventory.md"
  {
    echo "# W0 inventory — $ip"
    echo ""
    echo "Collected: $(date -Iseconds)"
    echo ""
    echo "## Pin"
    echo ""
    curl -sS -m15 "http://$ip/version-chooser/v1.0/version/current" | python3 -c "
import sys, json
d = json.load(sys.stdin)
print(f\"- tag: {d.get('tag','?')}\")
print(f\"- sha: {d.get('sha','?')}\")
print(f\"- repository: {d.get('repository','?')}\")
" 2>/dev/null || echo "- version API unreachable"
    sshpass -p raspberry ssh -o StrictHostKeyChecking=no -o ConnectTimeout=10 "pi@$ip" '
echo "- hostname: $(hostname)"
echo "- model: $(cat /proc/device-tree/model 2>/dev/null | tr -d "\0")"
echo "- arch: $(uname -m)"
echo "- kernel: $(uname -r)"
echo "- throttled: $(vcgencmd get_throttled 2>/dev/null || echo n/a)"
echo "- core image: $(docker inspect blueos-core --format "{{.Config.Image}}" 2>/dev/null || echo n/a)"
echo ""
echo "## Network"
ip -br addr 2>/dev/null | sed "s/^/- /"
echo "- default route: $(ip route | grep "^default" | head -1)"
echo ""
echo "## Autopilot"
curl -sS -m10 "http://127.0.0.1/ardupilot-manager/v1.0/firmware_info" 2>/dev/null | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    print(f\"- platform: {d.get('platform','?')}\")
    print(f\"- version: {d.get('version','?')}\")
except Exception:
    print(\"- firmware_info: unreachable\")
" 2>/dev/null || echo "- firmware_info: ssh curl failed"
echo ""
echo "## Wifi"
curl -sS -m10 "http://127.0.0.1/wifi-manager/v1.0/status" 2>/dev/null | head -c 500
echo ""
echo "## Extensions"
curl -sS -m10 "http://127.0.0.1/kraken/v1.0/extension/details" 2>/dev/null | python3 -c "
import sys, json
try:
    exts = json.load(sys.stdin)
    if isinstance(exts, list):
        for e in exts[:10]:
            print(f\"- {e.get('identifier','?')} {e.get('tag','?')} enabled={e.get('enabled',e.get('is_enabled','?'))}\")
    else:
        print(exts)
except Exception as ex:
    print(f\"- parse error: {ex}\")
" 2>/dev/null
echo ""
echo "## Video"
ls -1 /dev/video* 2>/dev/null | head -10 | sed "s/^/- /" || echo "- no /dev/video*"
' 2>&1
  } >"$md"
  echo "W0_DONE $ip -> $md"
}

w1_smoke() {
  local ip="$1"
  local out
  out="$(report_dir "$ip")"
  mkdir -p "$out"
  local log="$out/w1-smoke.log"
  local json="$out/w1-smoke.json"
  echo "W1 smoke $ip $(date -Iseconds)" | tee "$log"
  run_jh --base "http://$ip" --smoke --fixtures "$FIXTURES" \
    --report "$json" >>"$log" 2>&1
  local rc=$?
  echo "W1_EXIT=$rc $(date -Iseconds)" | tee -a "$log"
  return "$rc"
}

w2_negative() {
  local ip="$1"
  local out
  out="$(report_dir "$ip")"
  mkdir -p "$out"
  local log="$out/w2-negative.log"
  local json="$out/w2-negative.json"
  echo "W2 negative $ip $(date -Iseconds)" | tee "$log"
  run_jh --base "http://$ip" --negative --fixtures "$FIXTURES" \
    --report "$json" >>"$log" 2>&1
  local rc=$?
  echo "W2_EXIT=$rc $(date -Iseconds)" | tee -a "$log"
  return "$rc"
}

w1w2_aux() {
  local ip="$1"
  local out
  out="$(report_dir "$ip")"
  mkdir -p "$out"
  local log="$out/w1-w2-cache.log"
  echo "W1W2CACHE $ip $(date -Iseconds)" | tee "$log"
  run_jh --base "http://$ip" --frontend-cache --report "$out/frontend-cache.json" \
    >>"$out/frontend-cache.log" 2>&1
  echo "frontend-cache EXIT=$?" | tee -a "$log"
  run_jh --base "http://$ip" --extension-lifecycle --allow-mutating --report "$out/extension-lifecycle.json" \
    >>"$out/extension-lifecycle.log" 2>&1
  echo "extension-lifecycle EXIT=$?" | tee -a "$log"
  run_jh --base "http://$ip" --wifi-endpoints --report "$out/wifi-endpoints.json" \
    >>"$out/wifi-endpoints.log" 2>&1
  echo "wifi-endpoints EXIT=$?" | tee -a "$log"
  echo "W1W2CACHE_DONE $(date -Iseconds)" | tee -a "$log"
}

w3_mutating() {
  local ip="$1"
  local out
  out="$(report_dir "$ip")"
  mkdir -p "$out"
  export HOST_WIFI_IFACE="${HOST_WIFI_IFACE:-__qa_no_rf__}"
  bash "$CAMPAIGN_ROOT/run_w3.sh" "http://$ip" "$out" "$JH"
}

w4_rf() {
  local ip="$1"
  local out
  out="$(report_dir "$ip")"
  mkdir -p "$out"
  unset HOST_WIFI_IFACE || true
  bash "$CAMPAIGN_ROOT/run_w4.sh" "http://$ip" "$out" "$JH"
}

w5_disruptive() {
  local ip="$1"
  local out
  out="$(report_dir "$ip")"
  mkdir -p "$out"
  export HOST_WIFI_IFACE="${HOST_WIFI_IFACE:-__qa_no_rf__}"
  local log="$out/w5-disruptive.log"
  : >"$log"
  echo "W5 $ip $(date -Iseconds)" | tee -a "$log"
  local journeys=(
    enable_legacy_camera_support
    vehicle_first_boot
    update_firmware_online
    restore_default_firmware
    upload_custom_firmware
    start_autopilot
    stop_autopilot
    restart_autopilot
    change_board
    run_sitl_simulation
  )
  local fail=0
  for j in "${journeys[@]}"; do
    echo "===== $j $(date -Iseconds) =====" | tee -a "$log"
    run_jh --base "http://$ip" --mutating-smoke --fixtures "$FIXTURES" \
      --journey "$j" --report "$out/w5-${j}.json" >>"$log" 2>&1
    local rc=$?
    echo "EXIT_$j=$rc" | tee -a "$log"
    if [ "$rc" -ne 0 ]; then fail=1; fi
  done
  echo "W5_DONE FAIL=$fail $ip $(date -Iseconds)" | tee -a "$log"
  return "$fail"
}

w6_destructive() {
  local ip="$1"
  local out
  out="$(report_dir "$ip")"
  mkdir -p "$out"
  export HOST_WIFI_IFACE="${HOST_WIFI_IFACE:-__qa_no_rf__}"
  bash "$CAMPAIGN_ROOT/run_w6.sh" "http://$ip" "$out" "$JH"
}

w7_ui() {
  local ip="$1"
  local out
  out="$(report_dir "$ip")"
  mkdir -p "$out"
  local log="$out/ui.log"
  : >"$log"
  echo "W7 UI $ip $(date -Iseconds)" | tee -a "$log"
  run_jh --base "http://$ip" --ui --fixtures "$FIXTURES" \
    --report "$out/ui.json" >>"$log" 2>&1
  local rc=$?
  echo "W7_UI_EXIT=$rc $(date -Iseconds)" | tee -a "$log"
  return "$rc"
}

run_dut_readonly() {
  local ip="$1"
  w0_inventory "$ip"
  w1_smoke "$ip" || true
  w2_negative "$ip" || true
  w1w2_aux "$ip" || true
  echo "READONLY_DONE $ip $(date -Iseconds)"
}

run_dut_mutating() {
  local ip="$1"
  w3_mutating "$ip" || true
  w5_disruptive "$ip" || true
  w6_destructive "$ip" || true
  w7_ui "$ip" || true
  echo "MUTATING_DONE $ip $(date -Iseconds)"
}

main_log="$CAMPAIGN_ROOT/reports/campaign-${UTC}.log"
mkdir -p "$CAMPAIGN_ROOT/reports"
echo "CAMPAIGN_START $UTC DUTS=${DUTS[*]}" | tee "$main_log"

# W0-W2 + aux: parallel per DUT
pids=()
for ip in "${DUTS[@]}"; do
  (run_dut_readonly "$ip" 2>&1 | tee -a "$main_log") &
  pids+=($!)
done
for pid in "${pids[@]}"; do wait "$pid" || true; done

# W3: parallel per DUT
pids=()
for ip in "${DUTS[@]}"; do
  (w3_mutating "$ip" 2>&1 | tee -a "$main_log"; echo "W3_DONE $ip") &
  pids+=($!)
done
for pid in "${pids[@]}"; do wait "$pid" || true; done

# W4: RF serial (one host radio)
for ip in "${DUTS[@]}"; do
  echo "W4_RF_START $ip $(date -Iseconds)" | tee -a "$main_log"
  w4_rf "$ip" 2>&1 | tee -a "$main_log" || true
  echo "W4_RF_DONE $ip $(date -Iseconds)" | tee -a "$main_log"
done

# W5-W7: one DUT at a time (disruptive/destructive/UI)
for ip in "${DUTS[@]}"; do
  echo "W5_W7_START $ip $(date -Iseconds)" | tee -a "$main_log"
  w5_disruptive "$ip" 2>&1 | tee -a "$main_log" || true
  w6_destructive "$ip" 2>&1 | tee -a "$main_log" || true
  w7_ui "$ip" 2>&1 | tee -a "$main_log" || true
  echo "W5_W7_DONE $ip $(date -Iseconds)" | tee -a "$main_log"
done

# Coverage matrix merge
echo "MATRIX_MERGE $(date -Iseconds)" | tee -a "$main_log"
(cd "$CATALOG_DIR" && cargo run -q --bin journey_matrix -- --merge-report "$CAMPAIGN_ROOT/reports") \
  2>&1 | tee -a "$main_log" || true

echo "CAMPAIGN_DONE $(date -Iseconds)" | tee -a "$main_log"
