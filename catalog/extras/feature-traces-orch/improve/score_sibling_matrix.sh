#!/usr/bin/env bash
# Phase 2 (T6/T1) sibling-ratio matrix scorer.
#
# Prints |intersection|/|union| of follow_up_prs for each pair in the T1
# matrix (IMPROVE_DESIGN.md "T1 sibling-pair matrix"). Ratio close to 1.0
# means the pair's follow-up discovery is effectively indistinguishable
# (shared-legit, see T3); ratio close to 0 means well separated.
#
# Exit non-zero if the primary gate (pair #1, ConfigureCameraStream vs
# ViewCameraStreams) is >= 0.40 — the regression this script exists to catch.
#
# Usage:
#   ./score_sibling_matrix.sh [<feature_traces.json>]

set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
json="${1:-${script_dir}/../../../feature_traces.json}"

if [[ ! -f "$json" ]]; then
  echo "usage: $0 [<feature_traces.json>]" >&2
  exit 1
fi

# Pair list from IMPROVE_DESIGN.md "T1 sibling-pair matrix" (10 pairs, #1 and
# #10 carry an explicit numeric gate; the rest are regression-watch).
pairs=(
  "ConfigureCameraStream|ViewCameraStreams"
  "ConnectToWifiNetwork|ForgetSavedWifiNetwork"
  "ConnectToWifiNetwork|DisconnectFromWifiNetwork"
  "ConnectToWifiNetwork|ConnectToHiddenWifiNetwork"
  "DetectWifiApLoss|AutoconnectToSavedWifiNetwork"
  "ForceWifiNetworkPassword|ReconnectToSavedWifiNetwork"
  "ConfigureHotspotCredentials|ToggleSmartHotspot"
  "InspectRaspberryEepromBootloader|UpdateRaspberryEepromBootloader"
  "AcquireDynamicIpAddress|DisableOnboardDhcpServer"
  "RenameVehicle|ChangeMdnsHostname"
  "BrowseAvailableWebServices|MonitorInternetConnectivity"
  "VerifyInternetConnectivity|ProbeInterfaceInternetConnectivity"
  "UpdateBootstrapImage|DeleteLocalBlueosVersion"
  "StartAutopilot|UpdateFirmwareOnline"
  "ConfigureVideoStream|ViewCameraStreams"
  "ConfigureVideoStream|ConfigureCameraStream"
)

sibling_ratio() {
  jq -r --arg a "$1" --arg b "$2" '
    ([.intro_clusters[] | select(.by_journey[$a] != null) | .by_journey[$a].follow_up_prs] | first // []) as $fa
    | ([.intro_clusters[] | select(.by_journey[$b] != null) | .by_journey[$b].follow_up_prs] | first // []) as $fb
    | ($fa + $fb | unique) as $u
    | ($fa - ($fa - $fb)) as $inter
    | "\($inter|length)\t\($u|length)\t\(if ($u|length) == 0 then 0 else ($inter|length) / ($u|length) end)"
  ' "$json"
}

printf '%-40s %-40s %6s %6s %8s\n' "journey A" "journey B" "inter" "union" "ratio"

camera_ratio=""
fail=0
for pair in "${pairs[@]}"; do
  a="${pair%%|*}"
  b="${pair##*|}"
  IFS=$'\t' read -r inter union ratio <<<"$(sibling_ratio "$a" "$b")"
  printf '%-40s %-40s %6s %6s %8.3f\n' "$a" "$b" "$inter" "$union" "$ratio"
  if [[ "$a" == "ConfigureCameraStream" && "$b" == "ViewCameraStreams" ]]; then
    camera_ratio="$ratio"
  fi
done

if [[ -z "$camera_ratio" ]]; then
  echo "score_sibling_matrix: camera pair missing from pair list (should be unreachable)" >&2
  exit 1
fi

if awk -v r="$camera_ratio" 'BEGIN { exit !(r >= 0.40) }'; then
  echo "GATE FAILED: ConfigureCameraStream/ViewCameraStreams ratio ${camera_ratio} >= 0.40" >&2
  fail=1
fi

exit "$fail"
