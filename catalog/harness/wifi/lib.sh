#!/usr/bin/env bash
# Catalog-owned host RF helpers (adapted from BlueOS-docker support/wifi-e2e).
# Host AP for BlueOS-as-station journeys; host station for BlueOS-hotspot journeys.
# shellcheck disable=SC2034

set -euo pipefail

E2E_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

_e2e_load_config() {
  local cfg="${E2E_CONFIG:-$E2E_DIR/config.env}"
  [[ -f "$cfg" ]] || cfg="$E2E_DIR/config.example.env"
  [[ -f "$cfg" ]] || return 0

  local _saved_vars=()
  local _line _key _val
  while IFS= read -r _line || [[ -n "$_line" ]]; do
    [[ "$_line" =~ ^[[:space:]]*# ]] && continue
    [[ "$_line" =~ ^[[:space:]]*$ ]] && continue
    _key="${_line%%=*}"
    _key="${_key#"${_key%%[![:space:]]*}"}"
    _key="${_key%"${_key##*[![:space:]]}"}"
    [[ -z "$_key" ]] && continue
    if [[ -n "${!_key+x}" ]]; then
      _saved_vars+=("$_key=${!_key}")
    fi
  done <"$cfg"

  set -a
  # shellcheck disable=SC1090
  source "$cfg"
  set +a

  local _pair
  for _pair in "${_saved_vars[@]+"${_saved_vars[@]}"}"; do
    _key="${_pair%%=*}"
    _val="${_pair#*=}"
    printf -v "$_key" '%s' "$_val"
    export "$_key"
  done
}

_e2e_load_config

: "${HOST_WIFI_IFACE:=wlp8s0}"
: "${HOST_STATION_CONNS:=}"
: "${WPA2_CONN:=Hotspot-WPA2}"
: "${WPA2_SSID:=BlueOS-Hotspot}"
: "${WPA2_PSK:=changeme1234}"
: "${WPA3_CONN:=Hotspot}"
: "${WPA3_SSID:=BlueOS-Hotspot-WPA3}"
: "${WPA3_PSK:=changeme1234}"
: "${E2E_PSK:=changeme1234}"
: "${OPEN_CONN:=E2E-Open}"
: "${OPEN_SSID:=BlueOS-E2E-Open}"
: "${WPA_CONN:=E2E-WPA}"
: "${WPA_SSID:=BlueOS-E2E-WPA}"
: "${TRANSITION_CONN:=E2E-Transition}"
: "${TRANSITION_SSID:=BlueOS-E2E-Transition}"
: "${AP_CHANNEL:=6}"
: "${AP_GATEWAY:=10.42.0.1}"
: "${AP_PREFIX:=24}"
: "${DHCP_RANGE:=10.42.0.10,10.42.0.200}"
: "${PING_COUNT:=3}"
: "${HOTSPOT_GATEWAY:=192.168.42.1}"
: "${HOTSPOT_PREFIX:=24}"
: "${HOST_STATION_E2E_CONN:=BlueOS-E2E-Station}"
: "${E2E_HOTSPOT_SSID:=BlueOS-E2E-Hotspot}"
: "${E2E_HOTSPOT_PSK:=changeme1234}"

info() { printf '==> %s\n' "$*"; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 2
  }
}

host_disable_station_autoconnect() {
  local c
  for c in $HOST_STATION_CONNS; do
    nmcli connection show "$c" >/dev/null 2>&1 || continue
    nmcli connection modify "$c" connection.autoconnect no 2>/dev/null || true
    nmcli connection down "$c" 2>/dev/null || true
  done
}

host_ap_addr_ok() {
  ip -4 addr show "$HOST_WIFI_IFACE" 2>/dev/null | grep -q "${AP_GATEWAY}/${AP_PREFIX}"
}

host_ping() {
  local ip=$1
  ping -c "$PING_COUNT" -W 2 "$ip" >/dev/null
}

ip_in_ap_subnet() {
  local ip=$1
  python3 -c "
import ipaddress,sys
ip=sys.argv[1]; gw=sys.argv[2]; pref=int(sys.argv[3])
net=ipaddress.ip_network(f'{gw}/{pref}', strict=False)
raise SystemExit(0 if ipaddress.ip_address(ip) in net else 1)
" "$ip" "$AP_GATEWAY" "$AP_PREFIX"
}

ip_in_hotspot_subnet() {
  local ip=$1
  python3 -c "
import ipaddress,sys
ip=sys.argv[1]; gw=sys.argv[2]; pref=int(sys.argv[3])
net=ipaddress.ip_network(f'{gw}/{pref}', strict=False)
raise SystemExit(0 if ipaddress.ip_address(ip) in net else 1)
" "$ip" "$HOTSPOT_GATEWAY" "$HOTSPOT_PREFIX"
}

host_station_ensure() {
  local ssid=$1 psk=$2
  local conn=$HOST_STATION_E2E_CONN
  if ! nmcli -g NAME connection show | grep -Fxq "$conn"; then
    nmcli connection add type wifi ifname "$HOST_WIFI_IFACE" con-name "$conn" ssid "$ssid" \
      wifi-sec.key-mgmt wpa-psk wifi-sec.psk "$psk" \
      ipv4.method auto ipv6.method ignore >/dev/null
  else
    nmcli connection modify "$conn" \
      802-11-wireless.ssid "$ssid" \
      wifi-sec.key-mgmt wpa-psk \
      wifi-sec.psk "$psk" \
      ipv4.method auto \
      connection.autoconnect no >/dev/null
  fi
}

host_station_up() {
  local ssid=$1 psk=$2
  host_disable_station_autoconnect
  host_station_ensure "$ssid" "$psk"
  nmcli connection up "$HOST_STATION_E2E_CONN" ifname "$HOST_WIFI_IFACE"
}

host_station_down() {
  nmcli connection down "$HOST_STATION_E2E_CONN" 2>/dev/null || true
}

host_wait_hotspot_lease() {
  local timeout=${1:-45}
  local i ip
  for i in $(seq 1 "$timeout"); do
    ip="$(ip -4 -o addr show dev "$HOST_WIFI_IFACE" 2>/dev/null | awk '{print $4}' | cut -d/ -f1 | head -1)"
    if [[ -n "$ip" ]] && ip_in_hotspot_subnet "$ip"; then
      printf '%s\n' "$ip"
      return 0
    fi
    sleep 1
  done
  return 1
}

host_wifi_scan_has_ssid() {
  local ssid=$1
  nmcli -t -f SSID device wifi list ifname "$HOST_WIFI_IFACE" --rescan yes 2>/dev/null | grep -Fxq "$ssid"
}
