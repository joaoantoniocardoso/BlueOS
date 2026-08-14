# Findings catalog — BlueOS 1.4-dev full QA

Append-only. Supersede with `F-NNN.1` if a re-run changes the result. Never delete.

Core pin: `bluerobotics/blueos-core:1.4-dev @ sha256:f615d7caef4d3e99f1c068e082350c1af43d5fcc0f45379ed97ea27c6dc89805`

| id | dut | journey / probe | kind | severity | expected vs got | restore | next |
|---|---|---|---|---|---|---|---|
| F-001 | 177, 124 | inspect_disk_usage | environment | major | disk-usage :9151 down / HTTP 404 via nginx | n/a | W1 will Fail or Skip; confirm service; harness must not hide |
| F-002 | 87, 124 | inspect_mavlink_messages_in_browser | environment | minor | mavlink2rest nginx front-door 404; localhost OK on 124 | n/a | nginx path vs direct; catalog RouteRef |
| F-003 | 177, 2.2 | start_autopilot / MAVLink | environment | major | board present; mavlink2rest vehicles empty / no heartbeat | n/a | autopilot mutating will skip or fail; not a product 1.4 regression until confirmed running |
| F-004 | catalog | JourneyId::Deploy | harness | note | ALL has 101; bootstrap() registers 100 — no journey module | n/a | do not claim 101 live journeys |
| F-005 | catalog | FM:disk_usage/invalid_or_missing_path | contract | minor | card says 400-or-404; FILESYSTEM_ROOT=/ makes 400 branch dead | n/a | NP-27 expects 404; update card if live agrees |
| F-006 | catalog | sync_system_time B2 | contract | note | set_time returns 200 "not updating" within 5 min before gate | n/a | NP-04 must use far timestamp |
| F-007 | catalog | kraken install 201 vs 200 | contract | note | source annotates 201; StreamingResponse live 200 | n/a | catalog 200 is right |
| F-008 | all | disconnect_from_wifi_network | contract | minor | GET /disconnect mutates; 500 when idle (harness special-cases) | n/a | API-shape finding; NP-32 captures it |
| F-009 | catalog | pirate/advanced | product | minor | B9: bag/cable_guy DNS/priority not backend-gated | n/a | assert "not 403", never Fail for missing 403 |
| F-010 | 87 | modify_bag_database | environment | note | pirate/advanced ABSENT in bag | n/a | pirate fixture skip vs frontend-only (F-009) |
| F-011 | 87 | disk | environment | note | 7.9G free (44% used) vs ~96–106G on others | n/a | avoid large pulls / disk speed huge writes |
| F-012 | 2.2 | version-chooser | environment | note | docker RepoDigests null; HTTP sha matches | n/a | identity from HTTP current, not inspect |
| F-013 | 124 | platform | environment | note | Raspberry Pi 5 + Navigator (others Pi 4) | n/a | platform_matrix contrast |
| F-014 | 177, 87, 124, 2.2 | wifi RF | environment | note | wlan present, DOWN/disconnected; hotspot off | n/a | RF W4 must associate first; serial host radio |
| F-015 | 87 | view_camera_streams | environment | note | USB H264 camera; MCM stream to udp://192.168.2.1:5600 | n/a | camera journeys on 87; 124 cameras ABSENT |
| F-016 | catalog | helper hardware_id/software_id | harness_gap | note | explicit 400 contract, no journey | n/a | optional extra probes, not Track A |
| F-017 | catalog | recorder, iperf3 | limitation | note | failure_modes but no nginx prefix / no journey | n/a | ledger limitation |
| F-018 | 2.2 | host | environment | note | throttled=0x50000 history | n/a | do not treat as current undervoltage unless live |
| F-019 | 87 | ardupilot /serials | product | minor | GET /ardupilot-manager serials → 500 | n/a | capture in W1/W2 |
| F-020 | 2.2 | network | limitation | note | management eth0 192.168.2.2/24 via 192.168.2.1; strand ifaces eth0, usb0, wlan0 | n/a | never DHCP/static/priority/hotspot on 2.2 |
| F-021 | 177 | inspect_disk_usage | harness | minor | smoke Skip "not present on 1.4-dev" (first tag 1.4.4-beta.16); masks F-001 disk-usage down | n/a | version gate hides env failure; probe disk-usage directly |
| F-022 | 177 | W1 smoke 1.4-dev | limitation | note | 16 journeys Skip "not present on 1.4-dev" (1.4.4+/1.5.0+): disk, recorder, reset_settings, theme/logo, bootstrap, etc. | n/a | smoke on pinned 1.4-dev cannot cover 1.4.4 catalog |
| F-023 | 177 | modify_bag_database | harness | minor | pirate fixture loaded; Skip "no smoke-eligible GET steps" (see F-009 product) | n/a | add read-only bag GET probe or accept smoke gap |
| F-024 | 124 | W1 smoke 1.4-dev | note | 19 pass / 0 fail / 72 skip exit 0; 19 journeys Skip "not present on 1.4-dev"; F-001 masked (inspect_disk_usage); F-002 not in HTTP smoke; modify_bag_database Skip (F-023 pattern) | n/a | Tier-1 green does not clear F-001/F-002; direct probes |
| F-025 | 87 | W1 smoke 1.4-dev | note | 19 pass / 0 fail / 72 skip exit 0; 19 journeys Skip "not present on 1.4-dev"; F-002 not in HTTP smoke; modify_bag_database Skip (F-010/F-023); F-019 /serials not probed (bridges only); view_camera_streams Pass (F-015) | n/a | Tier-1 green does not clear F-002/F-019; direct probes |
| F-026 | 2.2 | NP-12 / NP-13 change_ui_theme_color | limitation | note | PUT /customization/v1.0/theme → 405; catalog 400; route absent on pinned 1.4-dev (1.5.0+) | n/a | not 1.4-dev regression; re-probe on 1.5.0+ |
| F-027 | 2.2 | NP-19 delete_3d_model_override | limitation | note | DELETE missing model → 405; catalog 404; customization absent on 1.4-dev | n/a | same as F-026 |
| F-028 | 2.2 | NP-28 / NP-29 free_disk_space | limitation | note | DELETE disk paths → 405; catalog 400/404; disk-usage mutators 1.4.4+ absent on 1.4-dev | n/a | aligns F-001/F-021; not live regression |
| F-029 | 2.2 | NP-30 run_single_disk_speed_test | limitation | note | GET speed size_bytes=max → 404; catalog 507; endpoint 1.4.4+ absent on 1.4-dev | n/a | re-probe when disk-usage live |
| F-030 | 2.2 | NP-31 forget_saved_wifi_network | product | minor | POST remove unknown SSID → 200; catalog 400 | n/a | wifi-manager accepts missing SSID silently |
| F-031 | 2.2 | NP-53 configure_installed_extension | product | minor | POST restart missing extension → 400; catalog 404 | n/a | kraken error shape vs missing-id 404 |
| F-032 | 2.2 | NP-54 configure_installed_extension | product | minor | GET container log missing → 200; catalog 404 | n/a | kraken returns 200 empty log for unknown container |
| F-033 | 2.2 | NP-68 / NP-69 / NP-70 recorder | limitation | note | recorder-extractor GET bad ext / DELETE traversal / DELETE missing → 404/405; catalog 400/404; service 1.4.4+ absent on 1.4-dev | n/a | re-probe on 1.4.4+ image |
| F-026 | 177 | W2 NP-12 | environment | minor | PUT /customization/v1.0/theme invalid hex; exp 400 got 405; route absent on 1.4-dev (1.5.0+) | n/a | harness should env-skip or accept 405 on pinned 1.4-dev |
| F-027 | 177 | W2 NP-13 | environment | minor | PUT /customization/v1.0/theme short hex; exp 400 got 405; customization absent 1.4-dev | n/a | same as F-026 |
| F-028 | 177 | W2 NP-19 | environment | minor | DELETE /customization/v1.0/models/missing; exp 404 got 405; customization absent 1.4-dev | n/a | same as F-026 |
| F-029 | 177 | W2 NP-28 | environment | minor | DELETE /disk-usage/v1.0/disk/paths/etc; exp 400 got 405; disk-usage paths absent 1.4-dev (F-001) | n/a | env not product regression |
| F-030 | 177 | W2 NP-29 | environment | minor | DELETE disk-usage missing path; exp 404 got 405; disk-usage absent 1.4-dev | n/a | env not product regression |
| F-031 | 177 | W2 NP-30 | environment | minor | GET disk/speed huge size; exp 507 got 404; disk-usage speed route absent 1.4-dev | n/a | env not product regression |
| F-032 | 177 | W2 NP-31 | product | minor | POST /wifi-manager/v1.0/remove missing SSID; exp 400 got 200 | n/a | idempotent remove? update FM or probe |
| F-033 | 177 | W2 NP-53 | product | minor | POST /kraken/v2.0/extension/missing/restart; exp 404 got 400 | n/a | kraken error shape on 1.4-dev |
| F-034 | 177 | W2 NP-54 | product | major | GET /kraken/v2.0/container/missing/log; exp 404 got 200 | n/a | false success on missing container |
| F-035 | 177 | W2 NP-68 | environment | minor | GET recorder bad .txt; exp 400 got 404; recorder-extractor absent 1.4-dev | n/a | env not product regression |
| F-036 | 177 | W2 NP-69 | environment | minor | DELETE recorder traversal ..%2F..%2Fetc%2Fpasswd; exp 400 got 405; recorder absent 1.4-dev (405 not 2xx) | n/a | no traversal blocker |
| F-037 | 177 | W2 NP-70 | environment | minor | DELETE recorder missing mp4; exp 404 got 405; recorder absent 1.4-dev | n/a | env not product regression |
| F-038 | 177 | W2 negative | note | 22 pass / 13 fail / 0 skip / 28 unasserted exit 1; NP-62 500 safe; no traversal 2xx blocker | n/a | 13 fails mostly 1.4.4+/1.5.0+ absent routes |
| F-026 | 87 | NP-12 change_ui_theme_color | limitation | PUT /customization/v1.0/theme invalid hex → 400; got 405 (route 1.5.0-beta.38+, absent on 1.4-dev pin) | n/a | version gate; not 1.4-dev regression |
| F-027 | 87 | NP-13 change_ui_theme_color | limitation | PUT /customization/v1.0/theme short hex → 400; got 405 (1.5.0-beta.38+) | n/a | version gate |
| F-028 | 87 | NP-19 delete_3d_model_override | limitation | DELETE missing .glb → 404; got 405 (1.5.0-beta.38+) | n/a | version gate |
| F-029 | 87 | NP-28 free_disk_space | limitation | DELETE protected path /disk/paths/etc → 400; got 405 (1.4.4+) | n/a | version gate |
| F-030 | 87 | NP-29 free_disk_space | limitation | DELETE missing userdata file → 404; got 405 (1.4.4+) | n/a | version gate |
| F-031 | 87 | NP-30 run_single_disk_speed_test | limitation | GET disk/speed huge size_bytes → 507; got 404 (1.4.4+) | n/a | version gate |
| F-032 | 87 | NP-31 forget_saved_wifi_network | product | minor | POST /wifi-manager/remove missing SSID → 400; got 200 | n/a | idempotent remove? catalog contract |
| F-033 | 87 | NP-53 configure_installed_extension | product | minor | POST kraken restart missing extension → 404; got 400 | n/a | error shape vs 404 |
| F-034 | 87 | NP-54 configure_installed_extension | product | minor | GET kraken container log missing → 404; got 200 | n/a | empty log vs 404 |
| F-035 | 87 | NP-68 download_video_recording | limitation | GET recorder bad extension .txt → 400; got 404 (1.4.4+) | n/a | version gate |
| F-036 | 87 | NP-69 delete_video_recording | limitation | DELETE recorder path traversal → 400; got 405 (1.4.4+) | n/a | version gate |
| F-037 | 87 | NP-70 delete_video_recording | limitation | DELETE missing .mp4 → 404; got 405 (1.4.4+) | n/a | version gate |
| F-038 | 87 | W2 negative 1.4-dev | note | 22 pass / 13 fail / 0 skip / 28 unasserted exit 1; NP-62→500 Pass (no blocker); fails F-026–F-037 | n/a | 3 product (F-032–F-034); rest version-gated |
| F-039 | 124 | W2 NP-12 | environment | minor | PUT /customization/v1.0/theme invalid hex; exp 400 got 405; route absent on 1.4-dev (1.5.0+) | n/a | harness should env-skip or accept 405 on pinned 1.4-dev |
| F-040 | 124 | W2 NP-13 | environment | minor | PUT /customization/v1.0/theme short hex; exp 400 got 405; customization absent 1.4-dev | n/a | same as F-039 |
| F-041 | 124 | W2 NP-19 | environment | minor | DELETE /customization/v1.0/models/missing; exp 404 got 405; customization absent 1.4-dev | n/a | same as F-039 |
| F-042 | 124 | W2 NP-28 | environment | minor | DELETE /disk-usage/v1.0/disk/paths/etc; exp 400 got 405; disk-usage paths absent 1.4-dev (F-001) | n/a | env not product regression |
| F-043 | 124 | W2 NP-29 | environment | minor | DELETE disk-usage missing path; exp 404 got 405; disk-usage absent 1.4-dev | n/a | env not product regression |
| F-044 | 124 | W2 NP-30 | environment | minor | GET disk/speed huge size; exp 507 got 404; disk-usage speed route absent 1.4-dev (F-001) | n/a | env not product regression |
| F-045 | 124 | W2 NP-31 | product | minor | POST /wifi-manager/v1.0/remove missing SSID; exp 400 got 200 | n/a | idempotent remove? update FM or probe |
| F-046 | 124 | W2 NP-38 | product | minor | concurrent GET /wifi-manager/scan; exp 425 scan_busy; got [200,200] | n/a | FM:wifi/scan_busy not enforced |
| F-047 | 124 | W2 NP-53 | product | minor | POST /kraken/v2.0/extension/missing/restart; exp 404 got 400 | n/a | kraken error shape on 1.4-dev |
| F-048 | 124 | W2 NP-54 | product | major | GET /kraken/v2.0/container/missing/log; exp 404 got 200 | n/a | false success on missing container |
| F-049 | 124 | W2 NP-68 | environment | minor | GET recorder bad .txt; exp 400 got 404; recorder-extractor absent 1.4-dev | n/a | env not product regression |
| F-050 | 124 | W2 NP-69 | environment | minor | DELETE recorder traversal ..%2F..%2Fetc%2Fpasswd; exp 400 got 405; recorder absent 1.4-dev (405 not 2xx) | n/a | no traversal blocker |
| F-051 | 124 | W2 NP-70 | environment | minor | DELETE recorder missing mp4; exp 404 got 405; recorder absent 1.4-dev | n/a | env not product regression |
| F-052 | 124 | W2 negative 1.4-dev | note | 22 pass / 13 fail / 0 skip / 28 unasserted exit 1; NP-62→500 Pass (no blocker); fails F-039–F-051 | n/a | 4 product (F-045–F-048); rest version-gated/env |
| F-053 | 87 | W3 mutating 1.4-dev | note | 7 pass / 0 fail / 1 skip exit 0; remove_configured_nmea_socket Skip (configured NMEA socket required); RF skipped __qa_no_rf__; hotspot creds + mdns restored | n/a | pre-configure NMEA to exercise remove journey; F-010/F-014 unchanged |
| F-054 | 177 | W3 mutating 1.4-dev | note | 7 pass / 0 fail / 1 skip exit 0; remove_configured_nmea_socket Skip (configured NMEA socket required); RF skipped __qa_no_rf__; hotspot creds + mdns restored | n/a | same as F-053; pre-configure NMEA; F-003/F-014 unchanged |
| F-054 | 2.2 | W3 mutating 1.4-dev | note | 6 pass / 0 fail / 1 skip exit 0; change_mdns_hostname excluded (USB F-020); remove_configured_nmea_socket Skip (configured NMEA socket required); RF __qa_no_rf__; hotspot creds restored; default route via 192.168.2.1 intact | n/a | pre-configure NMEA; modify_bag_database Pass (contrast F-010 on 87) |
| F-055 | 124 | W3 mutating 1.4-dev | note | 7 pass / 0 fail / 1 skip exit 0; remove_configured_nmea_socket Skip (configured NMEA socket required); RF skipped __qa_no_rf__; hotspot creds + mdns restored; GET /status 204 | n/a | pre-configure NMEA to exercise remove journey; F-001/F-014 unchanged |
| F-056 | harness | remove_configured_nmea_socket | harness | minor | W3 `--fixtures internet,pirate,advanced` overrode mutating defaults; omitted `nmea-socket` so setup never ran | ok | run_w3.sh fixed; retry Pass on all 4 DUTs |
| F-057 | 177 | W4 RF connect_to_wifi_network | harness | note | DUT scan still lists `BlueOS-Hotspot` 30s after host AP down (stale scan); harness WARN and continues | ok | benign race; optional scan refresh before connect |
| F-058 | 177 | W4 RF static_ip | harness | note | POST /network/v1.0/address on `wlan0` → 500 "No interface with name 'wlan0' is present"; harness skips static IP, uses DHCP lease 10.42.0.118 | ok | DUT wlan iface name may differ from harness assumption |
| F-059 | 177 | W4 mutating 1.4-dev | note | 10 pass / 0 fail / 0 skip exit 0; host wlp8s0 AP torn down; GET /status 204 | ok | F-014 exercised with real RF; toggle_hotspot via DUT hotspot 192.168.42.x |
| F-060 | 124 | W4 RF connect_to_wifi_network | harness | note | DUT scan still lists `BlueOS-Hotspot` 30s after host AP down (stale scan); harness WARN and continues | ok | benign race; optional scan refresh before connect (F-057 on 177) |
| F-061 | 124 | W4 RF static_ip | harness | note | POST /network/v1.0/address on `wlan0` → 500 "No interface with name 'wlan0' is present"; harness skips static IP, uses DHCP lease 10.42.0.173 | ok | DUT wlan iface name may differ from harness assumption (F-058 on 177) |
| F-062 | 124 | W4 mutating 1.4-dev | note | 10 pass / 0 fail / 0 skip exit 0; host wlp8s0 AP torn down; GET /status 204 | ok | F-014 exercised with real RF; toggle_hotspot via DUT hotspot 192.168.42.x |
| F-063 | 177 | switch_local_blueos_version | harness | major | POST /version/current → 412; teardown restore to tag `master` (digest ae50d2…) also 412; DELETE 404 | n/a | Harness assumes local `master` alias; DUT is `1.4-dev` @ f615d7ca. 412 prevented a channel change. Need 1.4-dev-aware switch smoke |
| F-064 | 177 | reboot_onboard_computer | contract | note | Pass; wait-for-BlueOS recovered; still 1.4-dev @ sha256:f615d7ca…; /status 204 | ok | W6 reboot path good |
| F-065 | 177 | W6 EEPROM/firmware/settings-reset | limitation | note | Not executed: EEPROM brick risk; firmware flash needs restore pin; settings reset restarts core. Shutdown hard-excluded | n/a | reboot+switch attempted; switch blocked by F-063 |
| F-066 | 177 | `--ui` SITL calibration | note | 7 pass / 0 fail (wizard_skip + gyro, baro, level_horizon, accel, compass, motor detect); Navigator restored | ok | SITL `calibration` then `vectored`; never on 2.2 |
| F-067 | 177 | sitl_cal DO_SET_SERVO | harness | note | ArduSub motor SERVOn_FUNCTION blocks DO_SET_SERVO (outputs stay 1500; model stays in angvel). Harness sets SERVO5–8_FUNCTION=0 on the calibration frame | ok | required for pose PWM; restore Navigator after suite |
| F-068 | 177 | calibrate_compass Dismiss | product | minor | MAG_CAL_REPORT fitness 0 for UAVCAN/LSM303D/AK8963 at 98–99%; Dismiss never shown (Cancel stays). `all_compasses_calibrated` compares reports to all COMPASS_DEV_ID≠0, not the mask | n/a | harness treats 3 report rows at ≥90% as done |
| F-069 | 177 | level_horizon presence | harness | note | Git `present_on_1_4_dev=false` (first tag 1.4.4-beta.100) but live pin `sha256:5b50dfaf…` bundle contains Level Horizon (UI Pass, F-066). Image tip ≠ seeded git 1.4-dev ancestry | n/a | do not hand-edit generated presence; matrix flags presence_contradiction |
| F-070 | 177 | page-load 24 PageId | note | 19 pass / 0 fail / 5 skip (disk, records, zenoh not_on_1.4-dev; extensions needs port; settings is gear dialog on 1.4-dev) | n/a | landmarks are menu titles + live 1.4-dev strings; 5xx fail the test |
| F-071 | 177 | no-hardware `--ui` | note | access_web_terminal, manage_blueos_files, inspect_mavlink_messages_in_browser, apply_parameter_file Pass; Navigator stayed Navigator | ok | ttyd has no accessible text (xterm); plan uses ExpectIframe |
| F-072 | 177 | W5 autopilot/board/SITL | note | restart/stop/start_autopilot, change_board, run_sitl_simulation Pass; Navigator restored | ok | change_board smoke POSTs Navigator JSON on Navigator; SITL restore via mutating teardown |
