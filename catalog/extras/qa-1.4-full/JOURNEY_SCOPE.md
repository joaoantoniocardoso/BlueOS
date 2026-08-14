# Journey scope — 1.4-dev QA campaign

Source: `catalog/src/journey_presence.rs` (`present_on_1_4_dev`, `first_tag` from `present_in_tags`); W1 from `reports/*/w1-smoke.log` (4 DUTs — identical skip patterns, only `base:` differs).

Target: `bluerobotics/blueos-core:1.4-dev` @ `sha256:f615d7caef4d3e99f1c068e082350c1af43d5fcc0f45379ed97ea27c6dc89805`

## Count table

| Scope | Count |
|-------|------:|
| Catalog journeys (`ALL_JOURNEY_PRESENCE`) | 100 |
| Present on 1.4-dev | 79 |
| Not present on 1.4-dev (F-022) | 21 |
| W1 HTTP smoke journeys | 81 |
| W1 pass | 15 |
| W1 skip — not present on 1.4-dev | 19 |
| W1 skip — present, fixtures/GET-only/other | 47 |
| Present on 1.4-dev, outside W1 HTTP set | 17 |

## 1. Present on 1.4-dev

79 catalog journeys (includes 6 wifi journeys aliased to `connect_to_wifi_network`). W1 HTTP subset: 62 journeys — 15 passed with `fixtures=internet,pirate,advanced`.

- `access_blueos_web_interface` — **W1 pass**
- `access_web_terminal` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `acquire_dynamic_ip_address` — W1 skip (fixture/GET-only)
- `add_custom_manifest` — W1 skip (fixture/GET-only)
- `add_external_nmea_gps_socket` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `apply_parameter_file` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `assign_static_ip_address` — W1 skip (fixture/GET-only)
- `autoconnect_to_saved_wifi_network` — W1 skip (fixture/GET-only)
- `browse_available_web_services` — **W1 pass**
- `browse_extension_store` — **W1 pass**
- `calibrate_accelerometer` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `calibrate_barometer` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `calibrate_compass` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `calibrate_gyroscope` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `change_board` — W1 skip (fixture/GET-only)
- `change_mdns_hostname` — W1 skip (fixture/GET-only)
- `configure_camera_stream` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `configure_host_dns` — W1 skip (fixture/GET-only)
- `configure_hotspot_credentials` — W1 skip (fixture/GET-only)
- `configure_installed_extension` — W1 skip (fixture/GET-only)
- `configure_uvc_device_controls` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `configure_video_stream` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `connect_ping_viewer_to_sonar` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `connect_to_hidden_wifi_network` — W1 skip (fixture/GET-only)
- `connect_to_wifi_network` — **W1 pass**
- `create_serial_to_udp_bridge` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `detect_motor_directions` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `detect_wifi_ap_loss` — W1 skip (fixture/GET-only)
- `disable_onboard_dhcp_server` — W1 skip (fixture/GET-only)
- `disconnect_from_wifi_network` — W1 skip (fixture/GET-only)
- `discover_blueos_on_network` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `docker_registry_login` — **W1 pass**
- `edit_extension_dev_version` — W1 skip (fixture/GET-only)
- `enable_legacy_camera_support` — W1 skip (fixture/GET-only)
- `enable_onboard_dhcp_server` — W1 skip (fixture/GET-only)
- `enable_ping1d_rangefinder_mavlink` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `force_wifi_network_password` — W1 skip (fixture/GET-only)
- `forget_saved_wifi_network` — W1 skip (fixture/GET-only)
- `inspect_mavlink_messages_in_browser` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `inspect_raspberry_eeprom_bootloader` — W1 skip (fixture/GET-only)
- `install_custom_extension` — W1 skip (fixture/GET-only)
- `install_extension` — W1 skip (fixture/GET-only)
- `manage_blueos_files` — non-HTTP (UI / HostWifiRf / no HTTP route)
- `modify_bag_database` — W1 skip (fixture/GET-only)
- `monitor_internet_connectivity` — **W1 pass**
- `probe_interface_internet_connectivity` — **W1 pass**
- `pull_blueos_version_without_switch` — W1 skip (fixture/GET-only)
- `reboot_onboard_computer` — W1 skip (fixture/GET-only)
- `reconnect_to_saved_wifi_network` — W1 skip (fixture/GET-only)
- `reject_invalid_wifi_credentials` — W1 skip (fixture/GET-only)
- `remove_camera_stream` — W1 skip (fixture/GET-only)
- `remove_configured_nmea_socket` — W1 skip (fixture/GET-only)
- `remove_serial_bridge` — W1 skip (fixture/GET-only)
- `rename_vehicle` — W1 skip (fixture/GET-only)
- `restart_autopilot` — W1 skip (fixture/GET-only)
- `restore_default_firmware` — W1 skip (fixture/GET-only)
- `run_host_command` — W1 skip (fixture/GET-only)
- `run_lan_speed_test` — **W1 pass**
- `run_sitl_simulation` — W1 skip (fixture/GET-only)
- `set_network_interface_priority` — W1 skip (fixture/GET-only)
- `shutdown_onboard_computer` — W1 skip (fixture/GET-only)
- `start_autopilot` — W1 skip (fixture/GET-only)
- `stop_autopilot` — W1 skip (fixture/GET-only)
- `switch_local_blueos_version` — W1 skip (fixture/GET-only)
- `sync_system_time` — W1 skip (fixture/GET-only)
- `toggle_hotspot` — W1 skip (fixture/GET-only)
- `toggle_smart_hotspot` — W1 skip (fixture/GET-only)
- `uninstall_extension` — W1 skip (fixture/GET-only)
- `update_blueos_version` — **W1 pass**
- `update_firmware_online` — W1 skip (fixture/GET-only)
- `update_raspberry_eeprom_bootloader` — W1 skip (fixture/GET-only)
- `upload_custom_firmware` — W1 skip (fixture/GET-only)
- `vehicle_first_boot` — W1 skip (fixture/GET-only)
- `verify_internet_connectivity` — **W1 pass**
- `view_camera_streams` — **W1 pass**
- `view_configured_nmea_sockets` — **W1 pass**
- `view_configured_serial_bridges` — **W1 pass**
- `view_detected_sonar_devices` — **W1 pass**
- `view_system_information` — **W1 pass**

## 2. Not present on 1.4-dev (F-022 — not 1.4-dev product bugs)

21 journeys land after the 1.4-dev tip. Findings here are **F-022** (future-release scope), not 1.4-dev regressions.

### first tag 1.4.4+ (13)
- `browse_video_recordings` — intro `6df9fd85a378`; first tag `1.4.4-beta.16`
- `delete_local_blueos_version` — intro `7d924e22376f`; first tag `1.4.4-beta.16`
- `delete_video_recording` — intro `6df9fd85a378`; first tag `1.4.4-beta.16`
- `download_video_recording` — intro `6df9fd85a378`; first tag `1.4.4-beta.16`
- `free_disk_space` — intro `c29e24679e3d`; first tag `1.4.4-beta.16`
- `inspect_disk_usage` — intro `c29e24679e3d`; first tag `1.4.4-beta.16`
- `inspect_zenoh_network` — intro `127f885b2daf`; first tag `1.4.4-beta.16`
- `level_horizon` — intro `06490f90ed01`; first tag `1.4.4-beta.100`. **Live pin `sha256:5b50dfaf…` includes it** (F-066/F-069); git presence bit stays false until regenerated against a 1.4-dev tip that contains the intro commit.
- `reset_blueos_settings` — intro `ebd3a1cac363`; first tag `1.4.4-beta.16`
- `run_internet_speed_test` — intro `757ce3c2f3dd`; first tag `1.4.4-beta.16`
- `run_multi_size_disk_speed_test` — intro `f860312f8b13`; first tag `1.4.4-beta.16`
- `run_single_disk_speed_test` — intro `f860312f8b13`; first tag `1.4.4-beta.16`
- `update_bootstrap_image` — intro `7d924e22376f`; first tag `1.4.4-beta.16`

### first tag 1.5.0+ (8)
- `change_ui_theme_color` — intro `2d797c10c681`; first tag `1.5.0-beta.38`
- `delete3d_model_override` — intro `2d797c10c681`; first tag `1.5.0-beta.38`
- `remove_custom_logo` — intro `2d797c10c681`; first tag `1.5.0-beta.38`
- `remove_custom_vehicle_image` — intro `2d797c10c681`; first tag `1.5.0-beta.38`
- `reset_ui_theme_color` — intro `2d797c10c681`; first tag `1.5.0-beta.38`
- `upload3d_model_override` — intro `2d797c10c681`; first tag `1.5.0-beta.38`
- `upload_custom_logo` — intro `2d797c10c681`; first tag `1.5.0-beta.38`
- `upload_custom_vehicle_image` — intro `2d797c10c681`; first tag `1.5.0-beta.38`

### W1 HTTP skip-presence

Absent journeys exercised by W1 HTTP smoke:

- `browse_video_recordings` — not present on 1.4-dev (intro 6df9fd85a378; first tag 1.4.4-beta.16)
- `change_ui_theme_color` — not present on 1.4-dev (intro 2d797c10c681; first tag 1.5.0-beta.38)
- `delete_3d_model_override` — not present on 1.4-dev (intro 2d797c10c681; first tag 1.5.0-beta.38)
- `delete_local_blueos_version` — not present on 1.4-dev (intro 7d924e22376f; first tag 1.4.4-beta.16)
- `delete_video_recording` — not present on 1.4-dev (intro 6df9fd85a378; first tag 1.4.4-beta.16)
- `download_video_recording` — not present on 1.4-dev (intro 6df9fd85a378; first tag 1.4.4-beta.16)
- `free_disk_space` — not present on 1.4-dev (intro c29e24679e3d; first tag 1.4.4-beta.16)
- `inspect_disk_usage` — not present on 1.4-dev (intro c29e24679e3d; first tag 1.4.4-beta.16)
- `remove_custom_logo` — not present on 1.4-dev (intro 2d797c10c681; first tag 1.5.0-beta.38)
- `remove_custom_vehicle_image` — not present on 1.4-dev (intro 2d797c10c681; first tag 1.5.0-beta.38)
- `reset_blueos_settings` — not present on 1.4-dev (intro ebd3a1cac363; first tag 1.4.4-beta.16)
- `reset_ui_theme_color` — not present on 1.4-dev (intro 2d797c10c681; first tag 1.5.0-beta.38)
- `run_internet_speed_test` — not present on 1.4-dev (intro 757ce3c2f3dd; first tag 1.4.4-beta.16)
- `run_multi_size_disk_speed_test` — not present on 1.4-dev (intro f860312f8b13; first tag 1.4.4-beta.16)
- `run_single_disk_speed_test` — not present on 1.4-dev (intro f860312f8b13; first tag 1.4.4-beta.16)
- `update_bootstrap_image` — not present on 1.4-dev (intro 7d924e22376f; first tag 1.4.4-beta.16)
- `upload_3d_model_override` — not present on 1.4-dev (intro 2d797c10c681; first tag 1.5.0-beta.38)
- `upload_custom_logo` — not present on 1.4-dev (intro 2d797c10c681; first tag 1.5.0-beta.38)
- `upload_custom_vehicle_image` — not present on 1.4-dev (intro 2d797c10c681; first tag 1.5.0-beta.38)

## 3. Present on 1.4-dev, W1 skipped (W3 mutating-smoke candidates)

47 HTTP journeys present on 1.4-dev but skipped in W1.

### GET-only (mutating W3) (17)
- `acquire_dynamic_ip_address` — no smoke-eligible GET steps with expected_status
- `add_custom_manifest` — no smoke-eligible GET steps with expected_status
- `assign_static_ip_address` — no smoke-eligible GET steps with expected_status
- `change_mdns_hostname` — no smoke-eligible GET steps with expected_status
- `configure_host_dns` — no smoke-eligible GET steps with expected_status
- `configure_hotspot_credentials` — no smoke-eligible GET steps with expected_status
- `connect_to_hidden_wifi_network` — no smoke-eligible GET steps with expected_status
- `enable_legacy_camera_support` — no smoke-eligible GET steps with expected_status
- `install_custom_extension` — no smoke-eligible GET steps with expected_status
- `install_extension` — no smoke-eligible GET steps with expected_status
- `modify_bag_database` — no smoke-eligible GET steps with expected_status
- `pull_blueos_version_without_switch` — no smoke-eligible GET steps with expected_status
- `reject_invalid_wifi_credentials` — no smoke-eligible GET steps with expected_status
- `rename_vehicle` — no smoke-eligible GET steps with expected_status
- `set_network_interface_priority` — no smoke-eligible GET steps with expected_status
- `toggle_hotspot` — no smoke-eligible GET steps with expected_status
- `toggle_smart_hotspot` — no smoke-eligible GET steps with expected_status

### fixture required (13)
- `autoconnect_to_saved_wifi_network` — saved Wi-Fi network required
- `configure_installed_extension` — installed extension required
- `detect_wifi_ap_loss` — active Wi-Fi connection required
- `disable_onboard_dhcp_server` — onboard DHCP server active required
- `disconnect_from_wifi_network` — active Wi-Fi connection required
- `edit_extension_dev_version` — installed extension required
- `force_wifi_network_password` — saved Wi-Fi network required
- `forget_saved_wifi_network` — saved Wi-Fi network required
- `reconnect_to_saved_wifi_network` — saved Wi-Fi network required
- `remove_configured_nmea_socket` — configured NMEA socket required
- `remove_serial_bridge` — configured serial bridge required
- `switch_local_blueos_version` — local BlueOS version required
- `uninstall_extension` — installed extension required

### fixture / unevaluable (11)
- `change_board` — unevaluable: Other::At least one flight controller board is connected or SITL simulation is available
- `enable_onboard_dhcp_server` — unevaluable: Other::The interface has at least one static IP address to use as the DHCP gateway
- `remove_camera_stream` — unevaluable: Other::At least one configured stream is listed on a device card
- `restart_autopilot` — unevaluable: Other::A flight controller board is selected
- `restore_default_firmware` — unevaluable: Other::A flight controller board is connected
- `run_sitl_simulation` — unevaluable: Other::Virtual SITL flight controller board is selected
- `start_autopilot` — unevaluable: Other::A flight controller board is selected
- `stop_autopilot` — unevaluable: Other::An autopilot is available to stop
- `update_firmware_online` — unevaluable: Other::A compatible flight controller board is connected
- `upload_custom_firmware` — unevaluable: Other::A compatible flight controller board is connected
- `vehicle_first_boot` — unevaluable: Other::Cold start after power-on; unevaluable: Other::BlueOS is newly installed and the configuration wizard is available

### dangerous-op confirm (6)
- `inspect_raspberry_eeprom_bootloader` — confirm dangerous operation required
- `reboot_onboard_computer` — confirm dangerous operation required
- `run_host_command` — confirm dangerous operation required
- `shutdown_onboard_computer` — confirm dangerous operation required
- `sync_system_time` — confirm dangerous operation required
- `update_raspberry_eeprom_bootloader` — confirm dangerous operation required
