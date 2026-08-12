#!/usr/bin/env bash
# Host joins BlueOS soft-AP (inverse of host-ap.sh).
#
# Usage:
#   ./host-station.sh up [ssid] [psk]   # defaults: E2E_HOTSPOT_SSID / E2E_HOTSPOT_PSK
#   ./host-station.sh down
#   ./host-station.sh wait-lease [timeout_sec]
#   ./host-station.sh scan-has <ssid>
#
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$DIR/lib.sh"

need_cmd nmcli
need_cmd ip
need_cmd python3

cmd_up() {
  local ssid=${1:-$E2E_HOTSPOT_SSID}
  local psk=${2:-$E2E_HOTSPOT_PSK}
  "$DIR/host-ap.sh" down || true
  host_station_up "$ssid" "$psk"
  info "host station up ssid=$ssid"
}

cmd_down() {
  host_station_down
  info "host station down"
}

cmd_wait_lease() {
  local timeout=${1:-45}
  host_wait_hotspot_lease "$timeout"
}

cmd_scan_has() {
  local ssid=${1:?ssid}
  host_wifi_scan_has_ssid "$ssid"
}

usage() {
  sed -n '2,10p' "$0" | sed 's/^# \?//'
}

main() {
  local cmd=${1:-}
  shift || true
  case "$cmd" in
    up) cmd_up "$@" ;;
    down) cmd_down ;;
    wait-lease) cmd_wait_lease "$@" ;;
    scan-has) cmd_scan_has "$@" ;;
    -h | --help | help | "") usage ;;
    *)
      echo "unknown command: $cmd" >&2
      usage
      exit 2
      ;;
  esac
}

main "$@"
