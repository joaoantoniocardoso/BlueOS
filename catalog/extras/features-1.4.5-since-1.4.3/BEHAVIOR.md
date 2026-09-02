# BlueOS 1.4.5 since 1.4.3 -- behavioral changes

## network

- cable_guy_routing: cable_guy adopted kernel routes into settings, reapplied static routes without priority metric or DHCP-marker preservation, and left orphan or stale routes after IP removal -> ignores kernel routes, applies user priority to metrics, reconciles desired routes on apply, preserves DHCP Client markers, cleans orphan /24 and /64 on IP delete, tolerates duplicate adds, and blacklists ZeroTier interfaces
- static_ip_watchdog_recovery: backup-server static IP removed externally was not re-applied by the watchdog -> static IP watchdog restores the configured backup address after external loss
- network_probe_deltas: tray and System Information derived upload/download speeds from frontend total-byte snapshots -> throughput uses linux2rest received_B/transmitted_B probe deltas
- internet_tcp_ping_fallback: per-interface internet checks used ICMP only and marked hosts unreachable when ICMP was filtered -> filtered ICMP triggers TCP connect to ports 80/443 so globe icons match real HTTP reachability
- ping_disappearing_interfaces: ping device discovery could error when an interface vanished mid-scan -> discovery skips disappearing interfaces without failing the scan
- wifi_nm_failed_connect_cleanup: failed initial WiFi join left an orphaned NetworkManager profile after the 30s timeout -> timeout calls remove_network before raising so failed joins do not persist
- wifi_nm_forget_saved_network: Forget saved network on the NetworkManager backend raised NotImplemented -> deletes the NM connection by UUID so Forget works on Bookworm

## video

- video_thumbnail_preview: registered thumbnails auto-polled continuously; still sources used live-stream icons -> manual snapshot or continuous controls by default; still sources use image icons; polling gated on healthy streams and disable_thumbnails
- stream_extended_config: no disable_lazy, disable_thumbnails, or disable_zenoh toggles when creating or editing a stream -> pirate Extra configuration can opt out of lazy, MAVLink camera msgs, and thumbnails (defaults on); Disable Zenoh is stored but MCM Zenoh stays off unless start-blueos-core passes `--zenoh` (1.4.5 does not)
- video_stream_lifecycle_state: stream cards showed Running or Not running and displayed errors whenever not running -> cards show Running, Idle, or Stopped and surface error details only when stopped
- hide_radcam_secondary_stream: RadCam UnderwaterCam /stream_1 listed as a separate video device -> secondary RadCam stream filtered from the device list
- filter_video_overview_by_streams: vehicle-setup video overview listed every encode-capable device and auto-registered thumbnails for all -> omits devices with no streams and registers thumbnails only when streams exist
- video_stream_diagnostics: RTSP reachability check flagged default-route streams bound only to the vehicle IP as unreachable -> diagnostic also accepts rtsp://0.0.0.0 endpoints so default-route cameras clear the no-stream banner
- video_udp_even_default_ports: auto-filled UDP/UDP265 stream ports used 5600 + index (5600, 5601, 5602, ...) -> ports use 5600 + 2*index (5600, 5602, 5604, ...)
- video_setup_thumbnail_autoplay_hover: vehicle-setup overview tooltip thumbnails required manual play like Video Manager -> tooltip thumbnails autoplay on hover in vehicle setup overview

## mavlink

- mavlink_unknown_endpoints: unsupported connection types were dropped from settings and could not be viewed or removed in the UI -> unknown types kept as disabled records, shown in Endpoint Manager, skippable for routing, and deletable
- mavlink_message_rate_claims: message refresh rates set per listener without tracking other subscribers; rate 0 still sent SET_MESSAGE_INTERVAL -> per-message claims refcounted with max(claims), idle at 1 Hz when none remain, and rate 0 skips SET_MESSAGE_INTERVAL
- mavlink_endpoint_udp_client_default: new endpoint dialog defaulted to UDP Server (udpin) on 0.0.0.0:14550 -> defaults to UDP Client (udpout) on 0.0.0.0:14550
- mavlink_router_single_tcp_server: multiple TCPServer endpoints passed to mavlink-routerd, which only honors the last --tcp-port -> additional TCP server endpoints skipped with an error log
- mavlink_system_id_from_env: MAV_SYSTEM_ID hardcoded to 1; mavlink-server launched without explicit ID flags -> MAV_SYSTEM_ID from env (default 1); mavlink-server passes --mavlink-system-id and --mavlink-component-id
- ping_baud_timeout_as_failure: ping baud-rate autodetection request timeout could throw uncaught and abort detection -> timeout caught and counted as failure toward max_failures threshold

## settings

- confirm_destructive_resets: reset BlueOS settings and wipe all parameters ran immediately on button click -> WarningDialog requires explicit confirmation before either reset proceeds
- keep_ssh_on_reset: settings reset deleted /root/.config/.ssh, ardupilot-manager, and kraken directories -> reset skips .ssh, ardupilot-manager, bag-of-holding, bootstrap, and kraken paths
- settings_atomic_save: settings JSON written directly to target path (partial file on interrupted write) -> write to .tmp, fsync, then atomic os.replace
- settings_invalid_file_reset: after exhausting backups only SettingsFromTheFuture was caught; corrupt primary could block load -> instantiates factory defaults, saves them, and continues

## extensions

- open_extensions_relative_paths: v2 extensions with relative-path support iframe-loaded at absolute hostname:port URLs -> works_in_relative_paths extensions route through /extensionv2/:name under the BlueOS origin
- disable_bazaar: Bazaar marketplace tab reachable from Extension Manager -> Bazaar entry disabled and not usable from Extension Manager
- extension_nginx_404: requests to unregistered extension paths could fall through without a clear 404 -> nginx explicitly returns 404 when the extension is not registered
- extension_websocket_upgrade: extension WebSocket connections could fail through helper nginx proxy -> Upgrade headers forwarded so extension WebSockets establish reliably
- fresh_install_no_major_tom: set_default_extensions pulled and installed Major Tom alongside Cockpit on fresh install -> Major Tom image pull and install entry skipped
- kraken_permission_pull_no_uninstall: any atomic image pull failure uninstalled the extension and re-raised -> permission-only unique_entry change survives without uninstall or re-raise
- kraken_container_logs_404: streaming logs for a non-running container could hang or error opaquely -> log stream checks container existence first and returns 404 when absent
- kraken_default_settings_json: set_default_extensions.sh wrote invalid trailing-comma JSON that could break kraken bootstrap -> default kraken settings JSON parses cleanly on first boot
- kraken_extension_mav_system_id_inject: extension containers started without host MAV_SYSTEM_ID in Env -> MAV_SYSTEM_ID injected into container Env when set on host and not already present

## vehicle

- configure_servo_output_function: SERVO_FUNCTION used one generic min/trim/max editor for every function value -> Motor, GPIO, Lights, or Actuator sub-editors with role-specific parameters, near-track clicks, visible trim handle, and new enum values
- failsafe_card_disabled_state: failsafe fields stayed editable when parent action was off; low-battery DISABLED could lock thresholds incorrectly -> non-control fields grey out when parent failsafe is disabled; DISABLED only when all four volt and mAh pairs are zero
- blueboat_power_presets: battery preset list had no BlueBoat120 Power Module v1 or v2 entries -> BlueBoat120 Power Module v1/v2 presets auto-fill monitor scaling parameters
- autopilot_params_board_change: board change left stale parameters in the frontend store until manual refresh -> autopilot_data.reset() after board change reloads parameters for the new board
- parameter_repository_4_7: parameter editor used pre-4.7 ArduPilot-Parameter-Repository metadata -> 4.7 metadata so labels, units, ranges, and options match 4.7 firmware
- sensor_device_identification: newer compass, INS, and barometer hardware showed generic or unknown labels -> additional device IDs decode to specific hardware type names in Vehicle Setup
- accelerometer_orientation_display: full accelerometer calibration had no orientation preview; unknown state showed blank -> live vehicle orientation visualization during calibration; unknown state labeled Unknown
- autopilot_reset_message_filter: stale parameter messages could repopulate UI right after a reset -> parameter fetcher ignores autopilot messages for 500ms after reset
- wizard_vehicle_type_options: wizard could expose boat/UNDEFINED when only rover/BOAT exists -> wizard lists rover/BOAT without a spurious boat/UNDEFINED entry
- navigator_compass_flex: Navigator FC detection required a single fixed compass hardware pairing -> either ak09915 or IIS2MDC compass satisfies Navigator detection
- lights_ardusub_detection: lights setup failed to detect lights on ArduSub >= 4.5.6 -> lights detection works for ArduSub 4.5.6 and newer builds
- rcin_param_names: UI renamed RCIN9/RCIN10 for ArduSub versions above 4.5.4 -> RCIN9 and RCIN10 keep their original names on newer ArduSub
- compass_calibrate_requires_position: compass Calibrate enabled when compass mask selected -> Calibrate disabled until rough vehicle position coordinates are set
- ardupilot_start_failure_cap: failed board starts retried every 5s indefinitely -> auto-restart stops after ten consecutive failures until user starts autopilot or changes board
- ardupilot_restart_timeout_60s: autopilot restart UI timed out sooner and could show failure on slow reconnects -> restart progress waits 60s before reporting timeout
- autopilot_auto_restart_race_guard: race could trigger overlapping auto_restart_autopilot sequences -> guard serializes restart so only one sequence runs
- digital_twin_no_default_model: Vehicle Setup Digital Twin showed a default BlueBoat 3D model -> no default BlueBoat model preloaded in Digital Twin viewer
- model_override_priority: connected vehicle type could override a custom model path; overrides missed on mount -> global override checked first; overrides applied on mount; model path can be overridden
- model_axis_orientation: several vehicle GLB models appeared misoriented in GenericViewer -> model assets reoriented to match viewer-model axis convention
- paramsets_board_fetch: ParamSets could show no sets if board info was never loaded elsewhere -> ParamSets fetches current board on mount to filter available sets

## logs

- download_system_logs: Settings and Self Health Test downloaded service logs via filebrowser folder fetch -> commander /services/download_system_logs zip prepends host diagnostics and includes extension logs
- stream_service_log_deletion: POST /services/remove_log blocked with a generic spinner until finished -> POST /services/remove_log_stream shows per-file deletion progress in Settings
- stream_mavlink_log_deletion: POST /services/remove_mavlink_log blocked with a generic spinner until finished -> POST /services/remove_mavlink_log_stream shows per-file deletion progress in Settings
- log_zipper_single_archive: each rotated log input overwrote the same gzip output, leaving only the last file -> all rotated logs appended into one gzip file before originals are removed
- log_timestamp_iso8601_utc: log files used ambiguous local-time stamps and non-ISO filename patterns -> file sink and log_zipper emit UTC ISO 8601 timestamps in content and filenames
- delete_gz_logs_while_open: .gz files skipped when file_is_open check failed during log wipe -> .gz files always eligible for deletion regardless of open check

## frontend

- wizard_webgl_fallback: setup wizard assumed WebGL/model-viewer worked and showed broken 3D vehicle pickers otherwise -> detects missing WebGL/model-viewer and renders icon-based vehicle cards instead
- lazy_mount_system_info_tabs: every System Information tab stayed mounted; kernel WebSocket subscribed at SPA boot; network polled continuously -> only active tab mounted; kernel WS opens on Kernel tab; FetchType refcount gates network and platform polling
- pirate_mode_system_info_tabs: tab selection by numeric index left Kernel/Firmware panes blank after those tabs were inserted -> tabs bind by page name so all System Information panes mount correctly in Pirate Mode
- pwa_upgrade_cache_bust: gzip_static, try_files, and leftover service workers could serve an old UI shell after upgrade -> nginx serves shell/PWA from exact locations, keeps sw.js uncompressed, and registers a self-destroying worker
- spa_nginx_fallback_routing: unknown frontend paths could return bare nginx 404 without loading the SPA -> @fallback location serves index.html for client-side routes
- parameter_loader_dialog_once: loading a parameter set could open ParameterLoader dialog twice -> parameter set load opens the loader dialog a single time

## other

- api_validation_422: malformed API bodies could be wrapped as 500 by GenericErrorHandlingRoute -> RequestValidationError propagates as standard FastAPI 422 responses
- helper_speedtest_null_previous: GET /internet_test_previous_result raised RuntimeError when no prior speedtest existed -> endpoint returns JSON null when there is no prior speedtest result
- helper_website_check_error_reporting: website_status.error stayed None on failure; GitHub check used wrong port/timeout -> error populated from response; GitHub check port and timeout corrected
- version_chooser_unlimited_memory: versionchooser ran with nice 250 under a memory-capped cgroup during image load/switch -> runs with nice 0 without memory cap to load large side-loaded images
- version_chooser_dockerhub_empty_images: Docker Hub tags with empty images[] array excluded from available versions list -> empty-images tags included with placeholder digest so arch filter is bypassed
- ssh_setup_at_startup: SSH authorized_keys setup happened later in the core boot path -> startup_blueos_update generates keys and installs authorized_keys before main startup tasks
- ardupilot_empty_serials_nonlinux: requesting serials on a non-Linux board could assert-fail the API -> non-Linux boards get an empty serial list instead of an internal error
