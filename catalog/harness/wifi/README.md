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
```

RF setup/teardown runs automatically when `wifi_rf` reports the host ready
(`nmcli` present and `HOST_WIFI_IFACE` exists). Otherwise RF-backed journeys skip
with an explicit reason.
