# 1.4-dev release readiness — catalog live QA

**Pin:** `bluerobotics/blueos-core:1.4-dev @ sha256:5b50dfafc3114651d459993c0c65639d7205634613fb62e0dea5ee04de8eebb1`  
**DUTs:** 177 Navigator Pi4 (play), 87 Pixhawk1, 2.2 Navigator USB vehicle, 124 Navigator Pi5  
**Date:** 2026-08-14  
**Campaign:** `catalog/extras/qa-1.4-full/` (closeout G0–G8)

## Verdict

**Coverage bar met:** journey matrix oracle shows UI empty = 0, backend planned = 0, both-empty gate = 0 among 79 present journeys. **Not product-green:** five product findings remain open (F-063, F-075, F-069, F-077, F-068). Shutdown was never run (hard exclude).

## Matrix oracle (2026-08-14)

```
Journeys: 100 (present on 1.4-dev: 79)
Backend measured pass: 39   UI measured pass: 37
UI empty (needs plan): 0   presence contradictions: 1
Both cells empty (gate): 0
```

Backend fails: `switch_local_blueos_version` @177 (F-063), `connect_to_wifi_network` @87 (F-075). Presence contradiction: `level_horizon` (F-069; git `not_on_1.4-dev`, live UI Pass).

## What passed (closeout)

| Area | Result |
|---|---|
| W0–W4, W6 | unchanged from pre-closeout; W4 177/124 10/10; 87 9/10 (F-075) |
| G1 typed UI skips | HttpPassthrough, no_sonar, would_reboot, fixture skips — UI empty → 0 |
| G2/G8 ui_plans | cable_guy internet cluster PASS; bag/rename/extensions/NMEA/bridges/version chooser |
| `install_custom_extension` | PASS via `ClickSelectorIfVisible` (1.4-dev beta.15 FAB vs later speed-dial) |
| Page-load + landmarks | 19/24 on 177 (F-070); pirate prefix + scoped selectors for extension/version |
| Calibration `--ui` | 177 SITL 7/7 (F-066); compass `pass+F-068` |
| No-hardware `--ui` | 177 (F-071), 124 (F-074) |
| Camera `--ui` | 87 5/5; UDP Stream 0 restored (F-073) |
| W5 autopilot/board/SITL | 177 Pass (F-072) |
| G3 backend planned | filled → planned = 0 (NMEA POST, camera POST, serial skip, ping skip) |
| G5 B6 service-down | probed on 177; F-077 filed |
| Cable-guy F-078–F-084 | harness plan fixes landed (not product regressions) |

## Product findings still open

| id | journey / probe | issue |
|---|---|---|
| F-063 | `switch_local_blueos_version` @177 | POST switch → 412; harness tags `:master`; pin unchanged (G4 re-run F-063.1) |
| F-075 | `connect_to_wifi_network` @87 | POST connect → 500 timeout; first failure sticks |
| F-069 | `level_horizon` presence | git absent on 1.4-dev; live bundle has Level Horizon UI Pass — do not hand-edit `journey_presence.rs` |
| F-077 | `view_configured_serial_bridges` / B6 | linux2rest down → GET serial_ports **200** `[]` (expect 502/error, not silent empty) |
| F-068 | `calibrate_compass` | MAG_CAL fitness 0; Dismiss never shown; UI `pass_with_finding` |

NP contract findings (F-030–F-034, F-045–F-048): wifi remove 200, scan_busy, kraken 400/200 shapes — unchanged from W2.

## Typed skips (coverage, not product pass)

- `switch_local_blueos_version` UI: `local BlueOS version required` (no fixture; backend F-063 kept)
- `disable_onboard_dhcp_server` UI: `onboard DHCP server active required` (eth0 DHCP inactive on 177)
- `connect_to_wifi_network` / hidden / saved-wifi UI: `known Wi-Fi network required` or `saved Wi-Fi network required` (177 fixture skips)
- `create_serial_to_udp_bridge` backend: `usb_serial_device`
- `enable_ping1d_rangefinder_mavlink` / sonar: `no_sonar` (ABSENT ×4)
- `enable_legacy_camera_support`: `would_reboot`
- `shutdown_onboard_computer`: `hard_exclude`
- 21 ids `not_on_1.4-dev` (F-022)

## Harness / environment (not 1.4-dev regressions)

- F-022 / W2 405s: theme, disk-usage, recorder not on 1.4-dev
- F-056–F-062: W3 NMEA fixture, RF stale scan, wlan0 name
- F-078–F-084: cable_guy `--ui` plan navigation (eth0 widget text, Network page, overlay) — harness, landed
- F-057/060: stale wifi scan after AP down

## Not run

- EEPROM update, firmware flash, settings reset — brick / restore pin
- RF on 2.2 — USB strand risk
- Network mutate on 2.2 — forbidden
- Track A new JourneyIds — Opus gated; none promoted
- Extra B6 beyond G5 allowlist

## Stop

All four DUTs `/status` 204. Pin unchanged at `sha256:5b50dfaf…ebb1`. Coverage oracle green; product findings above remain for a release sign-off.
