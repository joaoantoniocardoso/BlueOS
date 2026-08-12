#!/usr/bin/env bash
# Create / bring up / tear down host test APs used by BlueOS WiFi e2e.
#
# Usage:
#   ./host-ap.sh ensure              # create/update all mode profiles
#   ./host-ap.sh up <mode>           # open|wpa|wpa2|transition|wpa3
#   ./host-ap.sh up-wpa2 | up-wpa3   # shortcuts (compat)
#   ./host-ap.sh down
#   ./host-ap.sh status
#   ./host-ap.sh ssid <mode>         # print SSID for mode
#   ./host-ap.sh psk <mode>          # print PSK for mode (empty for open)
#   ./host-ap.sh restore-station [CONN]
#
set -euo pipefail

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck disable=SC1091
source "$DIR/lib.sh"

need_cmd nmcli
need_cmd ip

# Per-mode defaults (override via config.env)
: "${E2E_PSK:=changeme1234}"
: "${OPEN_CONN:=E2E-Open}"
: "${OPEN_SSID:=BlueOS-E2E-Open}"
: "${WPA_CONN:=E2E-WPA}"
: "${WPA_SSID:=BlueOS-E2E-WPA}"
: "${WPA2_CONN:=Hotspot-WPA2}"
: "${WPA2_SSID:=BlueOS-Hotspot}"
: "${WPA2_PSK:=$E2E_PSK}"
: "${TRANSITION_CONN:=E2E-Transition}"
: "${TRANSITION_SSID:=BlueOS-E2E-Transition}"
: "${WPA3_CONN:=Hotspot}"
: "${WPA3_SSID:=BlueOS-Hotspot-WPA3}"
: "${WPA3_PSK:=$E2E_PSK}"

mode_conn() {
  case "$1" in
    open) echo "$OPEN_CONN" ;;
    wpa) echo "$WPA_CONN" ;;
    wpa2) echo "$WPA2_CONN" ;;
    transition) echo "$TRANSITION_CONN" ;;
    wpa3) echo "$WPA3_CONN" ;;
    *)
      echo "unknown mode: $1" >&2
      return 2
      ;;
  esac
}

mode_ssid() {
  case "$1" in
    open) echo "$OPEN_SSID" ;;
    wpa) echo "$WPA_SSID" ;;
    wpa2) echo "$WPA2_SSID" ;;
    transition) echo "$TRANSITION_SSID" ;;
    wpa3) echo "$WPA3_SSID" ;;
    *) return 2 ;;
  esac
}

mode_psk() {
  case "$1" in
    open) echo "" ;;
    wpa | wpa2 | transition) echo "${WPA2_PSK:-$E2E_PSK}" ;;
    wpa3) echo "${WPA3_PSK:-$E2E_PSK}" ;;
    *) return 2 ;;
  esac
}

_shared_ipv4() {
  local conn=$1
  nmcli connection modify "$conn" \
    ipv4.method shared \
    ipv4.addresses "${AP_GATEWAY}/${AP_PREFIX}" \
    ipv6.method ignore
  nmcli connection modify "$conn" ipv4.shared-dhcp-range "$DHCP_RANGE" 2>/dev/null || true
}

_ensure_base() {
  local conn=$1 ssid=$2
  if ! nmcli connection show "$conn" >/dev/null 2>&1; then
    nmcli connection add type wifi ifname "$HOST_WIFI_IFACE" con-name "$conn" autoconnect no \
      ssid "$ssid" \
      802-11-wireless.mode ap \
      802-11-wireless.band bg \
      ipv4.method shared \
      ipv6.method ignore >/dev/null
  fi
  nmcli connection modify "$conn" \
    connection.interface-name "$HOST_WIFI_IFACE" \
    connection.autoconnect no \
    802-11-wireless.ssid "$ssid" \
    802-11-wireless.mode ap \
    802-11-wireless.band bg \
    802-11-wireless.channel "$AP_CHANNEL"
  _shared_ipv4 "$conn"
}

ensure_open_profile() {
  info "Ensure $OPEN_CONN (open SSID=$OPEN_SSID)"
  _ensure_base "$OPEN_CONN" "$OPEN_SSID"
  nmcli connection modify "$OPEN_CONN" wifi-sec.key-mgmt none || true
  # Drop any leftover PSK from a previous profile shape
  nmcli connection modify "$OPEN_CONN" wifi-sec.psk '' 2>/dev/null || true
}

ensure_wpa_profile() {
  # Legacy WPA (TKIP). Many modern chips refuse AP mode; journey will record AP-up failure.
  info "Ensure $WPA_CONN (WPA-PSK/TKIP SSID=$WPA_SSID)"
  _ensure_base "$WPA_CONN" "$WPA_SSID"
  nmcli connection modify "$WPA_CONN" \
    wifi-sec.key-mgmt wpa-psk \
    wifi-sec.proto wpa \
    wifi-sec.pairwise tkip \
    wifi-sec.group tkip \
    wifi-sec.pmf 1 \
    wifi-sec.psk "${WPA2_PSK:-$E2E_PSK}"
}

ensure_wpa2_profile() {
  info "Ensure $WPA2_CONN (WPA2-PSK-only SSID=$WPA2_SSID)"
  _ensure_base "$WPA2_CONN" "$WPA2_SSID"
  nmcli connection modify "$WPA2_CONN" \
    wifi-sec.key-mgmt wpa-psk \
    wifi-sec.proto rsn \
    wifi-sec.pairwise ccmp \
    wifi-sec.group ccmp \
    wifi-sec.pmf 1 \
    wifi-sec.psk "${WPA2_PSK:-$E2E_PSK}"
}

ensure_transition_profile() {
  # WPA2+WPA3 transition: wpa-psk + RSN, PMF optional (driver often adds SAE)
  info "Ensure $TRANSITION_CONN (transition SSID=$TRANSITION_SSID)"
  _ensure_base "$TRANSITION_CONN" "$TRANSITION_SSID"
  nmcli connection modify "$TRANSITION_CONN" \
    wifi-sec.key-mgmt wpa-psk \
    wifi-sec.proto rsn \
    wifi-sec.pairwise ccmp \
    wifi-sec.group ccmp \
    wifi-sec.pmf 2 \
    wifi-sec.psk "${WPA2_PSK:-$E2E_PSK}"
}

ensure_wpa3_profile() {
  info "Ensure $WPA3_CONN (WPA3-SAE SSID=$WPA3_SSID)"
  _ensure_base "$WPA3_CONN" "$WPA3_SSID"
  nmcli connection modify "$WPA3_CONN" \
    wifi-sec.key-mgmt sae \
    wifi-sec.proto rsn \
    wifi-sec.pairwise ccmp \
    wifi-sec.group ccmp \
    wifi-sec.pmf 3 \
    wifi-sec.psk "${WPA3_PSK:-$E2E_PSK}"
}

ALL_CONNS() {
  echo "$OPEN_CONN" "$WPA_CONN" "$WPA2_CONN" "$TRANSITION_CONN" "$WPA3_CONN" Hotspot-1 Hotspot-WPA3
}

down_ap_conns() {
  local c
  for c in $(ALL_CONNS); do
    nmcli connection down "$c" 2>/dev/null || true
  done
}

wait_ap() {
  local ssid=$1
  local i
  for i in $(seq 1 25); do
    if host_ap_addr_ok && nmcli -t -f DEVICE,STATE connection show --active 2>/dev/null | grep -q "^${HOST_WIFI_IFACE}:activated"; then
      info "AP up on $HOST_WIFI_IFACE ${AP_GATEWAY}/${AP_PREFIX} (want SSID=$ssid)"
      return 0
    fi
    # Also accept "connected" device state
    if host_ap_addr_ok; then
      local st
      st="$(nmcli -g GENERAL.STATE device show "$HOST_WIFI_IFACE" 2>/dev/null || true)"
      if echo "$st" | grep -qi connected; then
        info "AP up on $HOST_WIFI_IFACE ${AP_GATEWAY}/${AP_PREFIX} (want SSID=$ssid)"
        return 0
      fi
    fi
    sleep 0.4
  done
  echo "AP did not become ready for SSID=$ssid" >&2
  nmcli -f GENERAL.STATE,IP4.ADDRESS device show "$HOST_WIFI_IFACE" || true
  return 1
}

cmd_ensure() {
  ensure_open_profile
  ensure_wpa_profile
  ensure_wpa2_profile
  ensure_transition_profile
  ensure_wpa3_profile
  info "Profiles ready for modes: open wpa wpa2 transition wpa3"
}

cmd_up() {
  local mode=${1:?mode}
  local conn ssid
  conn="$(mode_conn "$mode")"
  ssid="$(mode_ssid "$mode")"
  host_disable_station_autoconnect
  case "$mode" in
    open) ensure_open_profile ;;
    wpa) ensure_wpa_profile ;;
    wpa2) ensure_wpa2_profile ;;
    transition) ensure_transition_profile ;;
    wpa3) ensure_wpa3_profile ;;
  esac
  down_ap_conns
  nmcli connection up "$conn"
  wait_ap "$ssid"
  nmcli -f 802-11-wireless.ssid,802-11-wireless-security.key-mgmt,802-11-wireless-security.proto,802-11-wireless-security.pmf,IP4.ADDRESS \
    connection show "$conn" 2>/dev/null || true
}

cmd_down() {
  down_ap_conns
  info "AP profiles down"
}

cmd_status() {
  echo "--- active ---"
  nmcli -t -f NAME,DEVICE,STATE connection show --active
  echo "--- $HOST_WIFI_IFACE ---"
  nmcli -f GENERAL.STATE,GENERAL.CONNECTION,IP4.ADDRESS device show "$HOST_WIFI_IFACE" 2>/dev/null || true
  ip -4 addr show "$HOST_WIFI_IFACE" 2>/dev/null | grep inet || true
}

cmd_restore_station() {
  local conn=${1:-}
  [[ -n "$conn" ]] || {
    echo "usage: $0 restore-station <connection-name>" >&2
    exit 2
  }
  down_ap_conns
  nmcli connection modify "$conn" connection.autoconnect yes 2>/dev/null || true
  nmcli connection up "$conn"
}

usage() {
  sed -n '2,14p' "$0" | sed 's/^# \?//'
}

main() {
  local cmd=${1:-}
  shift || true
  case "$cmd" in
    ensure) cmd_ensure ;;
    up) cmd_up "$@" ;;
    up-wpa2) cmd_up wpa2 ;;
    up-wpa3) cmd_up wpa3 ;;
    down) cmd_down ;;
    status) cmd_status ;;
    ssid) mode_ssid "${1:?mode}" ;;
    psk) mode_psk "${1:?mode}" ;;
    restore-station) cmd_restore_station "$@" ;;
    -h | --help | help | "") usage ;;
    *)
      echo "unknown command: $cmd" >&2
      usage
      exit 2
      ;;
  esac
}

main "$@"
