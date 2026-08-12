# Catalog WiFi RF harness

Host-side NetworkManager helpers for catalog Tier-2 wifi/hotspot mutating smoke.
Adapted from `BlueOS-docker/support/wifi-e2e` host AP/station logic — owned here so
`journey_http` does not call that project’s runners.

## Roles

| Journey class | Host radio | DUT |
|---|---|---|
| Client (`ConnectToWifiNetwork`, …) | AP (`host-ap.sh up wpa2`) | station via wifi-manager |
| Hotspot (`ToggleHotspot`, …) | station (`host-station.sh up`) | soft-AP via wifi-manager |

Management HTTP always uses the DUT ethernet IP (`BLUEOS_BASE` / `--base`).

## Setup

```bash
cd catalog/harness/wifi
cp config.example.env config.env   # edit HOST_WIFI_IFACE, HOST_STATION_CONNS
./host-ap.sh ensure
```

## Smoke

```bash
# Host must have nmcli + a WiFi iface; DUT on ethernet:
BLUEOS_BASE=http://192.168.0.177 cargo run -q -p blueos-catalog --bin journey_http -- \
  --base http://192.168.0.177 --mutating-smoke --journey connect_to_wifi_network
```

RF setup/teardown runs automatically when `wifi_rf` reports the host ready
(`nmcli` present and `HOST_WIFI_IFACE` exists). Otherwise RF-backed journeys skip
with an explicit reason (not the old Pi3 deferral).
