# 1.4-dev release readiness — catalog live QA

**Pin (W0):** `bluerobotics/blueos-core:1.4-dev @ sha256:f615d7caef4d3e99f1c068e082350c1af43d5fcc0f45379ed97ea27c6dc89805`  
**Pin (177 live 2026-08-14):** `sha256:5b50dfafc3114651d459993c0c65639d7205634613fb62e0dea5ee04de8eebb1` (same tag, newer digest; F-069)  
**DUTs:** 177 Navigator Pi4 (play), 87 Pixhawk1, 2.2 Navigator USB vehicle, 124 Navigator Pi5  
**Date:** 2026-08-13  
**Campaign:** `catalog/extras/qa-1.4-full/`

## Verdict

**1.4-dev is exercisable on all four boards for HTTP smoke, Track B negatives that exist on this channel, reversible mutating, and wifi RF (two Navigators).** It is **not** a full catalog-coverage release: 21 catalog journeys are 1.4.4+/1.5.0+ only (F-022). Core-image switch smoke is **not** valid on this pin (F-063). Shutdown was never run (hard exclude).

## What passed

| Wave | Result |
|---|---|
| W0 inventory | all 4 reachable; same digest |
| W1 `--smoke` | **19 pass / 0 fail / 72 skip** ×4 (skips = presence + fixtures + GET-only) |
| W2 `--negative` | **22 pass / 13 fail / 28 unasserted** ×4 (identical). 9 fails are absent routes (405/404). 4 are 1.4-dev contracts |
| W3 reversible mutating | Pass: lan_speed, rename, bag, manifest, smart_hotspot, hotspot_creds, mdns (not 2.2). NMEA Pass after F-056 fixture fix |
| W4 RF | **10/10 Pass** on 177 and 124 (connect, hidden, **reject invalid**, reconnect, force PSK, disconnect, AP loss, autoconnect, hotspot + L3) |
| W6 reboot | **Pass** on 177; recovered same digest |
| Page-load | **19 pass / 5 skip** on 177 (F-070) |
| `--ui` no-hardware | **4 pass** on 177 (F-071) |
| W5 autopilot/board/SITL | **5 pass** on 177; Navigator restored (F-072) |

## Product findings on this pin (keep)

| id | issue |
|---|---|
| NP-31 / F-030 | `POST /wifi-manager/v1.0/remove` unknown SSID → **200** (catalog 400) |
| NP-38 / F-046 | concurrent `GET /scan` → **200,200** not 425 `scan_busy` |
| NP-53 | missing extension restart → **400** not 404 (still rejects) |
| NP-54 | missing container log → **200** (false success) |

Commander `i_know_what_i_am_doing=false` → 400 (NP-01..08) **Pass**. NP-62 delete-running-tag → 500 **Pass** (guard). Reject-invalid-wifi RF **Pass**.

## Harness / environment (not 1.4-dev regressions)

- F-022 / W2 405s: theme, disk-usage, recorder not on 1.4-dev
- F-056: W3 fixture override skipped NMEA; fixed; retry Pass
- F-057/060: stale wifi scan after AP down
- F-058/061: cable-guy static IP on `wlan0` → 500; DHCP still worked
- F-063: version-switch smoke assumes local `master`; 412 on 1.4-dev (did **not** change the pin)

## Not run (typed skip)

- `ShutdownOnboardComputer` — hard exclude
- EEPROM update, firmware flash, settings reset — 177-only, not executed (brick / restore pin)
- RF on 87 / 2.2 — 87 optional; 2.2 USB strand risk
- Network mutate on 2.2 — forbidden
- Camera UI (`configure_video_stream`) — DUT 87, not this wave

## Harness shipped this campaign

- `catalog/src/negative_probes.rs` + `journey_http --negative` (63 probes)
- `run_w3.sh` fixture fix, `run_w4.sh`, `run_w6.sh`

## Stop

All four DUTs `/status` 204. 2.2 WAN via 192.168.2.1. Image pin unchanged. Further CONTINUE must **halt**.
