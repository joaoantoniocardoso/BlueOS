# G0 Inventory — post harness-improve H0–H7 STOP

Oracle: `cargo run -q --offline --bin journey_matrix -- --merge-report extras/qa-1.4-full/reports`  
Run: 2026-08-14. Harness-improve `DONE.md` + `STOP` confirmed.

## Matrix totals (79 present on 1.4-dev)

| Axis | pass | skip | planned | fail | empty |
|------|------|------|---------|------|-------|
| **Backend** | 36 | 36 | 5 | 2 | 0 |
| **UI** | 18 | 15 | 12 | 0 | 34 |

- Catalog journeys: 100 total; 21 absent on 1.4-dev.
- UI measured pass includes `pass_with_finding` (e.g. `calibrate_compass` F-068).
- Both cells empty (gate): **0**.
- Presence contradiction: **1** (`level_horizon` — live pass vs git-absent, F-069).

## Backend planned (5)

| id | G3 disposition |
|----|----------------|
| `add_external_nmea_gps_socket` | mutating-smoke + sock restore |
| `configure_camera_stream` | 87 POST `/streams` + `McmStreamRestore` |
| `configure_uvc_device_controls` | 87 POST `/v4l` snapshot/restore |
| `create_serial_to_udp_bridge` | serial fixture or `usb_serial_device` skip |
| `enable_ping1d_rangefinder_mavlink` | typed skip `no_sonar` (ABSENT ×4) |

## Backend fail (2)

| id | finding | dut |
|----|---------|-----|
| `switch_local_blueos_version` | F-063 | 177 |
| `connect_to_wifi_network` | F-075 | 87 |

## UI empty (34) — leftover coverage holes

```
access_blueos_web_interface
acquire_dynamic_ip_address
add_external_nmea_gps_socket
assign_static_ip_address
autoconnect_to_saved_wifi_network
change_mdns_hostname
configure_host_dns
configure_hotspot_credentials
connect_ping_viewer_to_sonar
connect_to_hidden_wifi_network
connect_to_wifi_network
create_serial_to_udp_bridge
detect_wifi_ap_loss
disable_onboard_dhcp_server
disconnect_from_wifi_network
discover_blueos_on_network
enable_onboard_dhcp_server
enable_ping1d_rangefinder_mavlink
force_wifi_network_password
forget_saved_wifi_network
modify_bag_database
monitor_internet_connectivity
probe_interface_internet_connectivity
reconnect_to_saved_wifi_network
reject_invalid_wifi_credentials
remove_configured_nmea_socket
remove_serial_bridge
run_lan_speed_test
set_network_interface_priority
toggle_hotspot
toggle_smart_hotspot
verify_internet_connectivity
view_configured_nmea_sockets
view_detected_sonar_devices
```

### Diff vs H4 `ui_plan` landings

H4 shipped `ui_plan` for 30 ids across five const clusters (`UI_CALIBRATION_*`, `UI_NO_HARDWARE_*`, `UI_CAMERA_*`, `UI_EXTENSION_*`, `UI_VERSION_SETTINGS_*`) plus `UI_TYPED_SKIP` (17 orchestrated). None of the 34 UI-empty ids have `ui_plan()`; 12 other present ids have `ui_plan` but UI cell is `planned` (never `--ui` run on 1.4-full reports).

### Diff vs H2 skip mapping (`ui_typed_skip_reason` / runner)

- **Already typed in `UI_TYPED_SKIP`:** 17 orchestrated ids (e.g. `enable_legacy_camera_support` → `no_frontend_actor_step`; not in UI-empty).
- **G1 targets (not yet in matrix):** sonar trio → `no_sonar`; `enable_legacy_camera_support` → `would_reboot` (never toggle); HttpPassthrough helper/internet ids → typed skip (`http_is_operator_contract`), **not** Playwright clones.
- **Wifi / cable_guy / NMEA / bag:** backend mostly pass or planned; UI empty because no `ui_plan` and no typed skip — need G2 plans or G1 strand/reboot skips where applicable.

## G2 suggested clusters — green vs need_plan

| Cluster | H4 ui_plan? | Status |
|---------|-------------|--------|
| 1 Bag + beacon | `rename_vehicle` yes; `modify_bag_database` **no** | **need_plan** (`modify_bag_database`) |
| 2 Kraken extensions | all 7 + manifest/dev/docker | **green** (plans exist; UI `planned`, fixture skips on backend) |
| 3 NMEA | **no** | **need_plan** (3 ids) |
| 4 Bridget | page-load only for view; create/remove **no** | **need_plan** (2 ids) |
| 5 Version chooser | all 5 in `UI_VERSION_SETTINGS_*` | **green** (plans exist; `switch_*` blocked F-063 until G4) |
| 6 Helper/ping | **no** | G1 `no_sonar` (3 sonar ids); `verify_internet_connectivity` likely HttpPassthrough skip |
| — Calibration / no-hw / camera | H4 shipped | **green** (do not rewrite) |
| — Wifi RF + network UI (18 ids) | **no** | **need_plan** or G1 typed skips (not in original G2 list; largest hole) |

## Dual-required status

| Journey / group | Backend | UI | Both green? |
|-----------------|---------|-----|-------------|
| `rename_vehicle` | pass@2.2 | planned (has `ui_plan`) | **no** — needs G8 `--ui` |
| `modify_bag_database` | pass@2.2 | empty | **no** — needs G2 plan + G8 |
| Extension trio (`install_extension`, `configure_installed_extension`, `uninstall_extension`) | skip (fixture/GET) | planned (H4 plans) | **no** — needs extension fixture + G8 |
| Video (`UI_CAMERA_JOURNEYS` ×5) | 2 planned / 3 pass or skip | 5 pass@87 | **mostly green** — G3 for `configure_camera_stream`, `configure_uvc_device_controls` backend |
| Parameters (`apply_parameter_file`) | skip:no_route | pass@124/177 | **yes** |

## UI planned (12) — H4 plans awaiting G8 `--ui`

`add_custom_manifest`, `browse_extension_store`, `configure_installed_extension`, `docker_registry_login`, `edit_extension_dev_version`, `install_custom_extension`, `install_extension`, `pull_blueos_version_without_switch`, `rename_vehicle`, `switch_local_blueos_version`, `uninstall_extension`, `update_blueos_version`

## Policy notes (closeout invariants)

- **HttpPassthrough** (23 per H4_ORACLE_COUNTS): typed UI skip in matrix/runner, never REST-clone Playwright scripts.
- **Sonar:** `connect_ping_viewer_to_sonar`, `enable_ping1d_rangefinder_mavlink`, `view_detected_sonar_devices` → `no_sonar`.
- **`enable_legacy_camera_support`:** `would_reboot` / never toggle (already `no_frontend_actor_step` in H4 typed skip).
- Do not rewrite H4-accepted `ui_plan`s. Playwright interprets `UiAction` only.

## Refused this phase

- No edits to `catalog/src/**`, `catalog/e2e/**`, reports, FINDINGS, RELEASE_READINESS, sister campaign files.
- No live DUT runs, no `--ui`, no H0–H7 harness work, no commit/PR.
- G0 taxonomy table (`G0_TAXONOMY.md`) deferred to Opus-5 per orchestrator plan.
