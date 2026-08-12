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
| Connect (right password) | `ConnectToWifiNetwork` | Undefer; host AP setup |
| Disconnect | `DisconnectFromWifiNetwork` | Host AP + associate first |
| Forget | `ForgetSavedWifiNetwork` | Host AP + save then remove |
| Wrong password | — | **Add** `RejectInvalidWifiCredentials` |
| Reconnect saved (empty PSK) | — | **Add** `ReconnectToSavedWifiNetwork` |
| Force new password | — | **Add** `ForceWifiNetworkPassword` (docs ADV:123) |
| Hidden connect | — | **Add** `ConnectToHiddenWifiNetwork` |
| AP drop detected | — | **Add** `DetectWifiApLoss` |
| AP restore autoconnect | — | **Add** `AutoconnectToSavedWifiNetwork` |
| Toggle hotspot + RF join | `ToggleHotspot` | Host station join after enable |
| Credentials / smart | `ConfigureHotspotCredentials` / `ToggleSmartHotspot` | Keep HTTP round-trip |

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
