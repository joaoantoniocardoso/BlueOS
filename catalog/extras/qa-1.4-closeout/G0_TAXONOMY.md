# G0 Taxonomy — per-present-journey disposition (1.4-dev)

Oracle: `cargo run -q --offline --bin journey_matrix -- --merge-report extras/qa-1.4-full/reports`
Run: 2026-08-14, offline, no live DUT. Sources read-only (`catalog/src/ui.rs`, `catalog/src/journey_matrix.rs`, `G0_INVENTORY.md`).

Scope: the **79 journeys present on 1.4-dev**. `level_horizon` is git-absent and therefore not a row here; see §6.

## Bucket definitions

| Bucket | Meaning |
|--------|---------|
| `already_green` | UI needs no G2 authoring: measured UI pass, **or** an H4-accepted `ui_plan` already shipped (awaiting a G8 `--ui` run). |
| `skip_reason` | UI cell is a typed skip — measured/typed today, or a G1 target that must land as a typed skip rather than a Playwright script. |
| `need_plan` | UI cell empty, no `ui_plan`, no typed skip — G2 must author a `UiAction` plan. |
| backend `ok` | Backend cell is `pass` or a typed `skip` (fixture/precondition/no-route). No G3 action. |
| backend `planned` / `fail` | G3 action required. |

DUT rule (auditable): rows with outstanding UI work take the DUT of the fixture that work needs — **87** for camera/video mutation and for the wifi RF fixture (where those backends were measured), **124** for the no-hardware replicate set, **177** default otherwise. Completed rows carry their DUT of record. `n/a` = no run is owed. Six backend passes were measured on **2.2**; 2.2 is never a `--ui` target and never strands a journey, so those rows resolve to 177 for any remaining work.

## 1. Counts

**UI (79 present):** `already_green` **29** · `skip_reason` **19** · `need_plan` **31**

- `already_green` 29 = 17 measured UI pass (incl. `calibrate_compass` `pass_with_finding` F-068) + 12 H4-shipped `ui_plan` in `planned`.
- `skip_reason` 19 = 16 measured/typed skips today + 3 sonar ids retyped by G1 as `no_sonar`.
- `need_plan` 31 = the 34 UI-empty ids minus those 3 sonar ids.

**Backend (79 present):** `ok` **72** (36 pass + 36 typed skip) · `planned` **5** · `fail` **2**

> **Count reconciliation vs `G0_INVENTORY.md`.** The inventory records UI `18 pass / 15 skip`; recounted over present rows only the matrix yields `17 pass / 16 skip` (planned 12 and empty 34 agree). The extra pass is `level_horizon`, which is **absent** on 1.4-dev yet has a live UI pass — the F-069 contradiction leaking into the tally. Flagged, not reconciled by hand (§6).

## 2. Taxonomy table

| journey_id | ui | backend | dual | dut | notes |
|------------|----|---------|------|-----|-------|
| `access_blueos_web_interface` | need_plan | ok | no | 87 | G1? |
| `access_web_terminal` | already_green | ok | no | 124 | |
| `acquire_dynamic_ip_address` | need_plan | ok | no | 177 | |
| `add_custom_manifest` | already_green | ok | no | 177 | H4 plan, awaits G8; backend @2.2 |
| `add_external_nmea_gps_socket` | need_plan | planned | no | 177 | |
| `apply_parameter_file` | already_green | ok | yes | 124 | dual complete |
| `assign_static_ip_address` | need_plan | ok | no | 177 | |
| `autoconnect_to_saved_wifi_network` | need_plan | ok | no | 87 | |
| `browse_available_web_services` | already_green | ok | no | 177 | page-load UI |
| `browse_extension_store` | already_green | ok | no | 177 | H4 plan, awaits G8 |
| `calibrate_accelerometer` | already_green | ok | no | 177 | |
| `calibrate_barometer` | already_green | ok | no | 177 | |
| `calibrate_compass` | already_green | ok | no | 177 | pass_with_finding F-068 |
| `calibrate_gyroscope` | already_green | ok | no | 177 | |
| `change_board` | skip_reason | ok | no | 177 | no_frontend_actor_step |
| `change_mdns_hostname` | need_plan | ok | no | 124 | |
| `configure_camera_stream` | already_green | planned | yes | 87 | G3 backend |
| `configure_host_dns` | need_plan | ok | no | 177 | |
| `configure_hotspot_credentials` | need_plan | ok | no | 124 | |
| `configure_installed_extension` | already_green | ok | yes | 177 | H4 plan, awaits G8 |
| `configure_uvc_device_controls` | already_green | planned | yes | 87 | G3 backend |
| `configure_video_stream` | already_green | ok | yes | 87 | |
| `connect_ping_viewer_to_sonar` | skip_reason | ok | no | n/a | no_sonar |
| `connect_to_hidden_wifi_network` | need_plan | ok | no | 87 | |
| `connect_to_wifi_network` | need_plan | fail | no | 87 | F-075 |
| `create_serial_to_udp_bridge` | need_plan | planned | no | 177 | |
| `detect_motor_directions` | already_green | ok | no | 177 | |
| `detect_wifi_ap_loss` | need_plan | ok | no | 87 | |
| `disable_onboard_dhcp_server` | need_plan | ok | no | 177 | |
| `disconnect_from_wifi_network` | need_plan | ok | no | 87 | |
| `discover_blueos_on_network` | need_plan | ok | no | 177 | G1? |
| `docker_registry_login` | already_green | ok | no | 177 | H4 plan, awaits G8 |
| `edit_extension_dev_version` | already_green | ok | no | 177 | H4 plan, awaits G8 |
| `enable_legacy_camera_support` | skip_reason | ok | no | n/a | would_reboot; never click |
| `enable_onboard_dhcp_server` | need_plan | ok | no | 177 | |
| `enable_ping1d_rangefinder_mavlink` | skip_reason | planned | no | n/a | no_sonar |
| `force_wifi_network_password` | need_plan | ok | no | 87 | |
| `forget_saved_wifi_network` | need_plan | ok | no | 87 | |
| `inspect_mavlink_messages_in_browser` | already_green | ok | no | 124 | |
| `inspect_raspberry_eeprom_bootloader` | skip_reason | ok | no | n/a | no_frontend_actor_step |
| `install_custom_extension` | already_green | ok | no | 177 | H4 plan, awaits G8 |
| `install_extension` | already_green | ok | yes | 177 | H4 plan, awaits G8 |
| `manage_blueos_files` | already_green | ok | no | 124 | |
| `modify_bag_database` | need_plan | ok | yes | 177 | backend @2.2 |
| `monitor_internet_connectivity` | need_plan | ok | no | 87 | G1? |
| `probe_interface_internet_connectivity` | need_plan | ok | no | 87 | G1? |
| `pull_blueos_version_without_switch` | already_green | ok | no | 177 | H4 plan, awaits G8 |
| `reboot_onboard_computer` | skip_reason | ok | no | 177 | http_only_host_reboot |
| `reconnect_to_saved_wifi_network` | need_plan | ok | no | 87 | |
| `reject_invalid_wifi_credentials` | need_plan | ok | no | 87 | |
| `remove_camera_stream` | already_green | ok | yes | 87 | |
| `remove_configured_nmea_socket` | need_plan | ok | no | 177 | backend @2.2 |
| `remove_serial_bridge` | need_plan | ok | no | 177 | |
| `rename_vehicle` | already_green | ok | yes | 177 | H4 plan, awaits G8; backend @2.2 |
| `restart_autopilot` | skip_reason | ok | no | 177 | no_frontend_actor_step |
| `restore_default_firmware` | skip_reason | ok | no | n/a | no_frontend_actor_step |
| `run_host_command` | skip_reason | ok | no | n/a | http_only_dev_shell |
| `run_lan_speed_test` | need_plan | ok | no | 177 | G1?; backend @2.2 |
| `run_sitl_simulation` | skip_reason | ok | no | 177 | no_frontend_actor_step |
| `set_network_interface_priority` | need_plan | ok | no | 177 | |
| `shutdown_onboard_computer` | skip_reason | ok | no | n/a | hard_exclude |
| `start_autopilot` | skip_reason | ok | no | 177 | no_frontend_actor_step |
| `stop_autopilot` | skip_reason | ok | no | 177 | no_frontend_actor_step |
| `switch_local_blueos_version` | already_green | fail | no | 177 | H4 plan, awaits G8; F-063 |
| `sync_system_time` | skip_reason | ok | no | n/a | no_frontend_actor_step |
| `toggle_hotspot` | need_plan | ok | no | 87 | |
| `toggle_smart_hotspot` | need_plan | ok | no | 177 | backend @2.2 |
| `uninstall_extension` | already_green | ok | yes | 177 | H4 plan, awaits G8 |
| `update_blueos_version` | already_green | ok | no | 177 | H4 plan, awaits G8 |
| `update_firmware_online` | skip_reason | ok | no | n/a | no_frontend_actor_step |
| `update_raspberry_eeprom_bootloader` | skip_reason | ok | no | n/a | no_frontend_actor_step |
| `upload_custom_firmware` | skip_reason | ok | no | n/a | no_frontend_actor_step |
| `vehicle_first_boot` | skip_reason | ok | no | n/a | no_frontend_actor_step |
| `verify_internet_connectivity` | need_plan | ok | no | 87 | G1? |
| `view_camera_streams` | already_green | ok | yes | 87 | dual complete |
| `view_configured_nmea_sockets` | need_plan | ok | no | 177 | |
| `view_configured_serial_bridges` | already_green | ok | no | 177 | page-load UI |
| `view_detected_sonar_devices` | skip_reason | ok | no | n/a | no_sonar |
| `view_system_information` | already_green | ok | no | 177 | page-load UI |

## 3. G1 — typed skips to land

Playwright never runs these; they land as typed skips in `UI_TYPED_SKIP` / the runner.

**Confirmed (3 new):**

| id | reason |
|----|--------|
| `connect_ping_viewer_to_sonar` | `no_sonar` |
| `enable_ping1d_rangefinder_mavlink` | `no_sonar` (also G3: backend `planned` → typed skip `no_sonar`) |
| `view_detected_sonar_devices` | `no_sonar` |

**Already typed, reason to restate (no new skip):**

| id | reason |
|----|--------|
| `enable_legacy_camera_support` | `would_reboot` — never toggle; currently `no_frontend_actor_step` |

**Candidates pending classifier confirmation (6, marked `G1?` above):** `access_blueos_web_interface`, `discover_blueos_on_network`, `monitor_internet_connectivity`, `probe_interface_internet_connectivity`, `run_lan_speed_test`, `verify_internet_connectivity` — all single-service, `Actor::Service`-only journeys (`Nginx`, `Beacon`, `Helper` ×3, `Pardal`) whose consuming pages carry empty `frontend_features`, so they are `ClientComposed`/`HttpPassthrough`, never `ClientOrchestrated`. If `HttpPassthrough`, the reason is `http_is_operator_contract` and they must **not** become Playwright clones of REST calls.

**Why unresolved here:** `H4_ORACLE_COUNTS.md` gives the aggregate (23 `HttpPassthrough`) but no per-id list, and no existing bin emits `derive_oracle_class` per journey. Producing that list requires a source edit, which is out of scope for G0. G1 must dump the per-id class first, then type these six.

Projection if all six confirm: UI `already_green` 29 · `skip_reason` 25 · `need_plan` 25.

**Guard:** no dual-required id appears in any G1 list above.

## 4. G2 — `need_plan` clusters (31)

H4-shipped and already-green ids are excluded by construction — every id below has an empty UI cell and no `ui_plan`.

| # | Cluster | n | ids | dut |
|---|---------|---|-----|-----|
| A | Wifi RF | 12 | `autoconnect_to_saved_wifi_network`, `configure_hotspot_credentials`, `connect_to_hidden_wifi_network`, `connect_to_wifi_network`, `detect_wifi_ap_loss`, `disconnect_from_wifi_network`, `force_wifi_network_password`, `forget_saved_wifi_network`, `reconnect_to_saved_wifi_network`, `reject_invalid_wifi_credentials`, `toggle_hotspot`, `toggle_smart_hotspot` | 87 (124 for `configure_hotspot_credentials`; 177 for `toggle_smart_hotspot`) |
| B | cable_guy / network config | 7 | `acquire_dynamic_ip_address`, `assign_static_ip_address`, `change_mdns_hostname`, `configure_host_dns`, `disable_onboard_dhcp_server`, `enable_onboard_dhcp_server`, `set_network_interface_priority` | 177 (124 for `change_mdns_hostname`) |
| C | NMEA | 3 | `add_external_nmea_gps_socket`, `remove_configured_nmea_socket`, `view_configured_nmea_sockets` | 177 |
| D | Bridget serial | 2 | `create_serial_to_udp_bridge`, `remove_serial_bridge` | 177 |
| E | Bag | 1 | `modify_bag_database` — **dual-required** | 177 |
| F | helper / beacon / nginx / pardal | 6 | the six `G1?` ids in §3 | — |

Cluster A is the largest hole and was not in the original G2 list. Cluster F should shrink to zero if G1 types it; author no plans there until the classifier runs.

## 5. G3 — backend actions (5 planned + 2 fail)

| id | action | dut |
|----|--------|-----|
| `add_external_nmea_gps_socket` | mutating-smoke + socket restore | 177 |
| `configure_camera_stream` | POST `/streams` + `McmStreamRestore` | 87 |
| `configure_uvc_device_controls` | POST `/v4l` snapshot/restore | 87 |
| `create_serial_to_udp_bridge` | serial fixture, else `usb_serial_device` skip | 177 |
| `enable_ping1d_rangefinder_mavlink` | typed skip `no_sonar` (ABSENT ×4) | n/a |
| `switch_local_blueos_version` | F-063 triage | 177 |
| `connect_to_wifi_network` | F-075 triage | 87 |

## 6. Standing flags

- **F-075 (`connect_to_wifi_network`, fail@87).** The first observed failure **sticks** as a product failure. It is downgraded to a harness defect only if a later QA pass says so explicitly; G0 does not reclassify it, and G3 triage must not silently rewrite it.
- **F-069 (`level_horizon` presence contradiction).** The journey is git-absent on 1.4-dev yet has a live UI pass@177. It stays **flagged**, not hand-edited: neither the presence map nor the report is adjusted here. It is also the source of the §1 count discrepancy against `G0_INVENTORY.md`.

## 7. Dual-required status

| Journey / group | backend | ui | both green? |
|-----------------|---------|-----|-------------|
| `apply_parameter_file` | ok (skip:no_route) | pass@124 | **yes** |
| `view_camera_streams` | ok (pass@87) | pass@87 | **yes** |
| `configure_video_stream`, `remove_camera_stream` | ok | pass@87 | **yes** |
| `configure_camera_stream`, `configure_uvc_device_controls` | **planned** | pass@87 | no — G3 backend |
| `rename_vehicle` | ok (pass@2.2) | planned (H4 plan) | no — needs G8 `--ui` on 177 |
| `modify_bag_database` | ok (pass@2.2) | **empty** | no — needs G2 plan (cluster E) + G8 |
| Extension trio (`install_extension`, `configure_installed_extension`, `uninstall_extension`) | ok (fixture skip) | planned (H4 plans) | no — needs extension fixture + G8 |

## 8. Invariants honored

No new `JourneyId`. No `--ui` on 2.2 and no journey stranded there. `enable_legacy_camera_support` never toggled. No H4-accepted `ui_plan` rewritten. Playwright interprets `UiAction` only. Local, offline, no commit or PR.

## Refused this phase

- No edits to `catalog/src/**`, `catalog/e2e/**`, `G0_INVENTORY.md`, `FINDINGS.md`, reports, or sister-campaign files.
- No live DUT run, no `--ui`, no G1–G8 implementation, no H0–H7 harness work.
- Did not hand-reconcile the `G0_INVENTORY.md` UI pass/skip counts or the F-069 contradiction; both are flagged instead.
