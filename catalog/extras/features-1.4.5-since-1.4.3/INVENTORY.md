# BlueOS 1.4.5 since 1.4.3 -- merged inventory

## Scope

Tags: 1.4.3 (`c38aa1eef`) -> 1.4.5 (`14722557`). Includes BlueOS Vue/API
jobs and capabilities of shipped binaries (MCM, mavlink-server, linux2rest,
zenohd, ...). FUNCTION = distinct job absent on 1.4.3.

**Status** (how 1.4.5 ships it):

| status | meaning |
|--------|---------|
| on | Enabled by default; no extra flag |
| opt-out (pirate) | On by default; Pirate Mode Extra configuration can disable |
| pirate-only | BlueOS UI only when Pirate Mode is on |
| no UI | Running or reachable (binary/API/nginx), no Vue page |
| API only | REST exists; no Networking/settings screen |
| not launched | In the binary, BlueOS does not start or configure it |
| off | Explicitly disabled in this release |

MCM Extra configuration (`disable_lazy`, `disable_mavlink`,
`disable_thumbnails`, `disable_zenoh`) is pirate-only. Defaults are all
false, so lazy pipelines, MAVLink camera protocol, and thumbnails are
**on** unless a pirate operator opts out. Zenoh video publish is a
separate process-level switch: MCM t3.26.3 `--zenoh` is opt-in (clap
bool, default off). 1.4.5 `start-blueos-core` does not pass it, so
per-stream Disable Zenoh is a no-op. zenohd still runs; there is no
Zenoh Inspector page on 1.4.5 (1.5-only).

## New functions

### BlueOS

| id | status | summary |
|----|--------|---------|
| configure_vehicle_body | on | Select vehicle frame and board orientation with 3D previews in Vehicle Setup Configure |
| configure_battery_monitor | on | Configure battery monitor presets, scaling, and dual-battery params on the Power Configure tab |
| calibrate_level_horizon | on | Run level-horizon accelerometer trim with live attitude preview |
| manage_interface_routes | API only | Add, list, or remove static IP routes on a wired interface via cable_guy REST (no Networking UI) |
| block_video_source | pirate-only | Block or unblock a camera video source (Video Streams switch; MCM `/block_source`) |
| support_navigator_pi5 | on | Run BlueOS Navigator on Raspberry Pi 5 with Pi5 device-tree overlays at startup |

### mavlink-camera-manager (t3.19.2 -> t3.26.3)

| id | status | summary |
|----|--------|---------|
| zenoh_video_publish | not launched | Publish per-stream H264/H265 CompressedVideo on Zenoh (Foxglove CDR). MCM requires `--zenoh`; 1.4.5 does not pass it. zenohd still runs; no Inspector UI |
| pipeline_dot_debug | no UI | GStreamer pipeline DOT graphs via MCM UI/websocket under `/mavlink-camera-manager/` (nginx) |
| status_mavlink_ids | no UI | MCM status REST reports mavlink system and camera component IDs |
| external_recorder | not launched | `--recorder=external` exists; BlueOS start-blueos-core does not pass it (built-in recorder path) |

`block_video_source` is listed under BlueOS (pirate-only UI on the same MCM API).

### mavlink-server (0.3.1 -> 0.5.9)

BlueOS launches mavlink-server with serial/UDP/TCP/zenoh **proxy** endpoints
only (`MAVLinkServer.assemble_command`). It does not add `rest:`, TLOG
creation-condition, or send/receive-only direction flags.

| id | status | summary |
|----|--------|---------|
| rest_helper | not launched | REST v1 helper + built-in message-lookup page |
| rest_vehicle_control | not launched | REST v1 arm/disarm/mode/yaw/guided |
| rest_parameters | not launched | REST/websocket parameter get/set |
| resources_api | not launched | REST resources API for drivers/endpoints |
| control_webpage | not launched | Built-in control webpage |
| tlog_creation_conditions | not launched | TLOG file-creation-condition CLI/endpoint options |
| endpoint_direction | not launched | UDP/TCP send-only or receive-only proxy direction |
| zenoh_metadata | on* | Zenoh driver metadata topics when a Zenoh MAVLink endpoint is enabled (*endpoint is operator-added, not a default) |

### linux2rest (v0.6.2 -> v0.6.5)

No new functions. Behavioral only (Sampler, per-core CPU, CM5 platform libs). See TOOLS.md.

## Behavioral changes (68)

Grouped before/after list: BEHAVIOR.md and inventory/BEHAVIOR.json.

### network (6)

cable_guy_routing; static_ip_watchdog_recovery; network_probe_deltas;
internet_tcp_ping_fallback; ping_disappearing_interfaces;
wifi_nm_failed_connect_cleanup; wifi_nm_forget_saved_network.

### video (8)

video_thumbnail_preview; stream_extended_config; video_stream_lifecycle_state;
hide_radcam_secondary_stream; filter_video_overview_by_streams;
video_stream_diagnostics; video_udp_even_default_ports;
video_setup_thumbnail_autoplay_hover.

MCM per-stream flags (pirate Extra configuration, default **on**):
lazy pipelines, MAVLink camera msgs, thumbnails, Zenoh publish. Check
Disable Lazy / Disable Mavlink / Disable Thumbnails / Disable Zenoh to
opt out. See TOOLS.md.

### mavlink (6)

mavlink_unknown_endpoints; mavlink_message_rate_claims;
mavlink_endpoint_udp_client_default; mavlink_router_single_tcp_server;
mavlink_system_id_from_env; ping_baud_timeout_as_failure.

### settings (4)

confirm_destructive_resets; keep_ssh_on_reset; settings_atomic_save;
settings_invalid_file_reset.

### extensions (9)

open_extensions_relative_paths; disable_bazaar; extension_nginx_404;
extension_websocket_upgrade; fresh_install_no_major_tom;
kraken_permission_pull_no_uninstall; kraken_container_logs_404;
kraken_default_settings_json; kraken_extension_mav_system_id_inject.

### vehicle (20)

configure_servo_output_function; failsafe_card_disabled_state;
blueboat_power_presets; autopilot_params_board_change;
parameter_repository_4_7; sensor_device_identification;
accelerometer_orientation_display; autopilot_reset_message_filter;
wizard_vehicle_type_options; navigator_compass_flex; lights_ardusub_detection;
rcin_param_names; compass_calibrate_requires_position;
ardupilot_start_failure_cap; ardupilot_restart_timeout_60s;
autopilot_auto_restart_race_guard; digital_twin_no_default_model;
model_override_priority; model_axis_orientation; paramsets_board_fetch.

### logs (6)

download_system_logs; stream_service_log_deletion; stream_mavlink_log_deletion;
log_zipper_single_archive; log_timestamp_iso8601_utc; delete_gz_logs_while_open.

### frontend (6)

wizard_webgl_fallback; lazy_mount_system_info_tabs;
pirate_mode_system_info_tabs; pwa_upgrade_cache_bust;
spa_nginx_fallback_routing; parameter_loader_dialog_once.

### other (7)

api_validation_422; helper_speedtest_null_previous;
helper_website_check_error_reporting; version_chooser_unlimited_memory;
version_chooser_dockerhub_empty_images; ssh_setup_at_startup;
ardupilot_empty_serials_nonlinux.

## Shipped binaries

Pin table and per-binary before/after: TOOLS.md and inventory/TOOLS.json.

Unchanged pins: zenohd 1.0.0, mavlink2rest t0.11.23, filebrowser v2.30.0,
ttyd 1.6.3, mavp2p v1.1.1, mavlink-router v4, bridges 0.10.3, machineid
0.2.3, logviewer v1.0.1.

**Explicitly off in 1.4.5 product UI:** Bazaar (`is_bazaar_enabled` is
hard-false); Major Tom not installed on fresh images.

**Not launched** despite being in the bumped binary: MCM
`external_recorder`; mavlink-server REST helper/control/parameters/
resources/control webpage, TLOG creation conditions, endpoint direction.

## Notable enhancements

See also; many overlap with behavioral changes above. Full grouped
before/after list: BEHAVIOR.md.

## Not included

**1.5-only catalog functions** (20 absent on 1.4.5): disk usage browser,
video recorder/extractor, zenoh inspector, branding/theme customization,
bootstrap image update, commander reset_blueos_settings, and related journeys.
See FUNCTIONS.md / functions.json.

**Not a drop:** mavlink-server REST/control/helper, TLOG conditions, endpoint
direction, MCM Zenoh video and DOT debug, linux2rest CM5 platform libs remain
in the inventory (see New functions status). REST/TLOG/direction are
**not launched**; Zenoh video is **not launched** (no `--zenoh` on the MCM cmdline); DOT is **no UI**.

**Resolved as non-functions**: motor remapping (display fix only);
LeakSetup/RelaySetup widgets (servo-function enhancement); log download/delete
(streaming/progress enhancement of existing Settings jobs).

## Disputes resolved

14 items adjudicated in MERGED.json disputed_resolved (relay-configuration,
widget collapse, Pi5 overlays vs LSM6DSV, motor remap, logs, extensionv2,
forget-WiFi, keep-SSH, CPU per-core). No remaining open disputes.
