# Catalog WiFi RF harness

Host-side NetworkManager config for catalog Tier-2 wifi/hotspot mutating smoke.
RF control lives in Rust (`catalog/src/wifi_rf.rs`); this directory only holds
`config.env` / `config.example.env` (SSID, iface, DHCP, competing station profiles).

## Roles

| Journey class | Host radio | DUT |
|---|---|---|
| Client (`ConnectToWifiNetwork`, …) | AP (`wifi_rf::host_ap_up("wpa2")`) | station via wifi-manager |
| Hotspot (`ToggleHotspot`, …) | station (`wifi_rf::host_station_up`) | soft-AP via wifi-manager |

Management HTTP always uses the DUT ethernet IP (`--base`).

## Setup

```bash
cd catalog/harness/wifi
cp config.example.env config.env   # edit HOST_WIFI_IFACE, HOST_STATION_CONNS
```

Optional override for the config directory: `BLUEOS_WIFI_HARNESS_DIR=/path/to/dir`.

## Smoke

```bash
# Host must have nmcli + a WiFi iface; DUT on ethernet:
cargo run -q -p blueos-catalog --bin journey_http -- \
  --base http://192.168.0.177 --mutating-smoke --journey connect_to_wifi_network

# RF mode matrix; omit --wifi-modes to retain the WPA2-only default:
cargo run -q -p blueos-catalog --bin journey_http -- \
  --base http://192.168.0.177 --mutating-smoke --journey connect_to_wifi_network \
  --wifi-modes open,wpa,wpa2,transition,wpa3

# Exercise the wifi-manager endpoint surface and restore hotspot credentials:
cargo run -q -p blueos-catalog --bin journey_http -- \
  --base http://192.168.0.177 --wifi-endpoints
```

RF setup/teardown runs automatically when `wifi_rf` reports the host ready
(`nmcli` present and `HOST_WIFI_IFACE` exists). Otherwise RF-backed journeys skip
with an explicit reason.

Before client RF, the harness POSTs `smart_hotspot?enable=false` and
`hotspot?enable=false` on the DUT (soft-AP left on blocks station join and
pollutes neighboring scans). `--wifi-endpoints` restores the prior smart-hotspot
flag, then forces hotspot off again for multi-DUT hygiene.

Scan-absent gating: if the DUT still lists our SSID after the host AP is
confirmed down, the harness continues with a warning (stale `/scan` cache).

The host keeps `HOST_STATION_CONNS` down during RF tests. Set `RESTORE_STATION=1`
to re-enable them after teardown (can break ethernet routes to the DUT).
`EXPECT_WPA3=yes|no` overrides WPA3 capability; `auto` uses the WPA3 scan
entry's `supported` field.
