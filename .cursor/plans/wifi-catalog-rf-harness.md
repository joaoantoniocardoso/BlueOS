# Wifi / hotspot RF harness in the catalog

## Goal

Integrate `../BlueOS-docker/support/wifi-e2e` **coverage** into the catalog harness — not by invoking that project's runners. This machine (NetworkManager WiFi) is the RF peer:

- **Client journeys** (BlueOS joins WiFi): host runs AP; DUT is station.
- **Hotspot journeys** (BlueOS is AP): host is station; DUT soft-AP.

## Topology (replaces deferred Pi3 plan)

```
┌────────────────────────────┐  eth (TARGET)   ┌──────────────────┐
│ Runner (this PC)           │◄───────────────►│ BlueOS DUT       │
│ catalog/harness/wifi +     │  HTTP via nginx │ eth = management │
│ journey_http --mutating-   │                 │ wlan0 = STA/AP   │
│ smoke                      │◄──── RF ───────►│                  │
└────────────────────────────┘                 └──────────────────┘
```

## Gap map (wifi-e2e → catalog)

| e2e scenario | Catalog journey | Action |
|---|---|---|
| Connect (right password) | `ConnectToWifiNetwork` | Done — host AP RF + L3 ping |
| Disconnect | `DisconnectFromWifiNetwork` | Done — host AP RF |
| Forget | `ForgetSavedWifiNetwork` | Done — host AP RF |
| Wrong password | `RejectInvalidWifiCredentials` | Done |
| Reconnect saved (empty PSK) | `ReconnectToSavedWifiNetwork` | Done — + L3 ping |
| Force new password | `ForceWifiNetworkPassword` | Done — + L3 ping |
| Hidden connect | `ConnectToHiddenWifiNetwork` | Done — + L3 ping |
| AP drop detected | `DetectWifiApLoss` | Done — RF mid-journey |
| AP restore autoconnect | `AutoconnectToSavedWifiNetwork` | Done — RF mid-journey + L3 |
| Toggle hotspot + RF join | `ToggleHotspot` | Done — host station join + L3 gateway ping |
| Credentials / smart | `ConfigureHotspotCredentials` / `ToggleSmartHotspot` | Done — HTTP round-trip + credentials snapshot restore |

L3 checks (ping over wlan IP / hotspot gateway) are post-connect assertions on RF-backed journeys, not separate JourneyIds.

## Ownership

- Config lives under `catalog/harness/wifi/` (`config.env` / `config.example.env`).
- Rust `catalog/src/wifi_rf.rs` drives NetworkManager via `nmcli` / `ip` (ported from the old host-ap/station shell helpers).
- `SmokeRepair::HostWifiRf` documents the repair class.
- Skip RF journeys when `nmcli` / `HOST_WIFI_IFACE` unavailable (not a hard Pi3 deferral).

## Phases

1. Host RF harness + und defer existing six wifi journeys
2. Add six missing JourneyIds + capabilities + presence + smoke allowlist
3. RF assertions (lease, host scan/associate, AP drop/restore) + L3 ping
4. Update feature_traces / sibling gates / coverage_mappings — Done
