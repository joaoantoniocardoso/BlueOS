# W3 reversible mutating (RF off)

`HOST_WIFI_IFACE=__qa_no_rf__`. Script initially passed `--fixtures internet,pirate,advanced`, which **overrode** mutating-smoke defaults and omitted `nmea-socket` → NMEA skip (F-055). Script fixed; NMEA re-run recorded as `w3-remove_configured_nmea_socket-retry.json`.

| DUT | lan_speed | rename | bag | manifest | smart_hotspot | hotspot_creds | nmea (first) | mdns |
|---|---|---|---|---|---|---|---|---|
| 177 | pass | pass | pass | pass | pass | pass | skip fixture | pass |
| 87 | pass | pass | pass | pass | pass | pass | skip fixture | pass |
| 2.2 | pass | pass | pass | pass | pass | pass | skip fixture | excluded (USB) |
| 124 | pass | pass | pass | pass | pass | pass | skip fixture | pass |

NMEA retry (default mutating fixtures, no `--fixtures` override): **Pass on all 4**.

All `/status` 204 after. 2.2 WAN still via 192.168.2.1.
