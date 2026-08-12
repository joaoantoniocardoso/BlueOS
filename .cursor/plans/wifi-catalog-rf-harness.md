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
| Connect (right password) | `ConnectToWifiNetwork` | Done — host AP RF |
| Disconnect | `DisconnectFromWifiNetwork` | Exists |
| Forget | `ForgetSavedWifiNetwork` | Done — host AP RF |
| Wrong password | `RejectInvalidWifiCredentials` | Done |
| Reconnect saved (empty PSK) | `ReconnectToSavedWifiNetwork` | Done |
| Force new password | `ForceWifiNetworkPassword` | Done |
| Hidden connect | `ConnectToHiddenWifiNetwork` | Done |
| AP drop detected | `DetectWifiApLoss` | Done (GET /status; RF mid-journey TBD) |
| AP restore autoconnect | `AutoconnectToSavedWifiNetwork` | Done (GET /status; RF mid-journey TBD) |
| Toggle hotspot + RF join | `ToggleHotspot` | Done — host station join |
| Credentials / smart | `ConfigureHotspotCredentials` / `ToggleSmartHotspot` | HTTP round-trip |

L3 checks (ping/route/file xfer over wlan IP) become post-connect assertions on RF-backed journeys, not separate JourneyIds.

## Ownership

- Scripts live under `catalog/harness/wifi/` (catalog-owned, adapted from wifi-e2e helpers).
- Rust `catalog/src/wifi_rf.rs` drives them from mutating-smoke setup/teardown.
- `SmokeRepair::HostWifiRf` documents the repair class.
- Skip RF journeys when `nmcli` / `HOST_WIFI_IFACE` unavailable (not a hard Pi3 deferral).

## Phases

1. Host RF harness + und defer existing six wifi journeys
2. Add six missing JourneyIds + capabilities + presence + smoke allowlist
3. RF assertions (lease, host scan/associate, AP drop/restore)
4. Update feature_traces / sibling gates / coverage_mappings
