# BlueOS 1.4.5 since 1.4.3 -- shipped binaries

## Scope

Tags: 1.4.3 (`c38aa1eef`) -> 1.4.5 (`14722557`). Binary API, CLI, and protocol
are the product surface. `blueos_ui: true` means Vue/nginx also wires the
capability (may also appear in INVENTORY.md / BEHAVIOR.md).

**Status** (see INVENTORY.md legend): on | opt-out (pirate) | pirate-only |
no UI | not launched | off.

## Pin table

| binary | pin 1.4.3 | pin 1.4.5 | delta |
|--------|-----------|-----------|-------|
| mavlink-camera-manager | t3.19.2 | t3.26.3 | bumped |
| mavlink-server | 0.3.1 | 0.5.9 | bumped |
| linux2rest | v0.6.2 | v0.6.5 | bumped |
| zenoh | 1.0.0 | 1.0.0 | unchanged (x86 gnu toolchain) |
| mavlink2rest | t0.11.23 | t0.11.23 | unchanged (strip) |
| filebrowser | v2.30.0 | v2.30.0 | unchanged |
| ttyd | 1.6.3 | 1.6.3 | unchanged |
| mavp2p | v1.1.1 | v1.1.1 | unchanged (strip) |
| mavlink-router | v4 | v4 | unchanged (strip) |
| bridges | 0.10.3 | 0.10.3 | unchanged (strip) |
| machineid | 0.2.3 | 0.2.3 | unchanged (strip) |
| logviewer | v1.0.1 | v1.0.1 | unchanged |

## mavlink-camera-manager (t3.19.2 -> t3.26.3)

### New functions

- block_video_source [UI] **pirate-only** -- Block or unblock a camera video source via POST
  /block_source and /unblock_source; blocked sources stay in the block list and
  are omitted from streaming. Video Streams switch only in Pirate Mode.
- zenoh_video_publish [binary-only] **not launched** -- Publish per-stream H264/H265
  CompressedVideo on Zenoh topics (Foxglove CDR with metadata) for Foxglove,
  zenoh clients, and zenohd on the vehicle. MCM t3.26.3 `--zenoh` is opt-in
  (default off). 1.4.5 start-blueos-core does not pass it, so the pirate Extra
  configuration "Disable Zenoh" toggle never takes effect. No Zenoh Inspector
  page on 1.4.5.
- pipeline_dot_debug [binary-only] **no UI** -- Inspect live GStreamer pipeline DOT graphs
  via MCM bundled UI and websocket at /mavlink-camera-manager/ (nginx-proxied).
- external_recorder [binary-only] **not launched** -- Delegate recording to an external sink via
  --recorder=external CLI flag instead of the built-in recorder path. BlueOS
  start-blueos-core does not pass this flag.
- status_mavlink_ids [binary-only] **no UI** -- Read mavlink system and camera component
  IDs from the MCM status REST API for integrator discovery.

Pirate Extra configuration (defaults all false = those stream options on):
Disable Lazy, Disable Mavlink, Disable Thumbnails, Disable Zenoh. Disable
Zenoh only matters if MCM is started with `--zenoh` (it is not on 1.4.5).

### Behavioral changes

- stream_lifecycle_idle [UI] -- Streams reported coarse running vs not-running;
  pipelines stayed active or showed errors whenever not running ->
  Lifecycle state machine exposes running, idle (lazy), and stopped; idle keeps
  pipeline warm without full encode until a viewer connects.
- stream_extended_config [UI] -- No per-stream disable_lazy, disable_thumbnails,
  or disable_zenoh options when creating or editing streams ->
  Pirate Extra configuration toggles persist and change lazy startup, thumbnail
  generation, and MAVLink camera msgs per stream. Defaults false (those stay
  on). Disable Zenoh is stored the same way but MCM Zenoh is off unless
  `--zenoh` is passed (not on 1.4.5). Non-pirate operators cannot toggle them.
- zenoh_cdr_encoding [binary-only] -- Zenoh video topics used JSON-encoded
  CompressedVideo payloads ->
  Zenoh video uses Foxglove CDR encoding with per-stream topic suffix and
  metadata side channel.
- device_formats_monitor [UI] -- Camera format and resolution lists came from
  legacy caps probing with incomplete discovery on some devices ->
  Formats enumerated via GStreamer Device Monitor on first use for fuller
  encode and resolution lists in REST /v4l.
- mcm_rt_threads [UI] -- MCM ran without --enable-realtime-threads in BlueOS
  1.4.3 start-blueos-core ->
  BlueOS 1.4.5 passes --enable-realtime-threads; MCM sets GStreamer realtime
  scheduling on pipeline threads.
- udp_even_ports [UI] -- Custom UDP and UDP265 video sources could bind odd port
  numbers ->
  Video source ports use even numbers only to align with RTP conventions and
  QGC expectations.
- config_on_idr [binary-only] -- H264/H265 RTP payloader sent codec config only
  at stream start ->
  Parser and RTP payloader re-send config on every IDR frame for faster
  mid-stream client join.
- low_latency_queues [binary-only] -- Default GStreamer queue limits added
  buffering latency on live streams ->
  Queue limits tuned for lower-latency live streaming with less backpressure.
- thumbnail_capture_pipeline [UI] -- Thumbnail image sink used a delayed capture
  path that contended with the live stream thread ->
  GStreamer-native image sink and backpressure fixes reduce thumbnail
  interference with live video.
- webrtc_server_teardown_notify [binary-only] -- WebRTC clients were not
  notified when MCM tore down the server-side session ->
  Clients receive server-side session teardown notification over the
  signalling websocket.
- rtsp_retransmission [binary-only] -- RTSP ingest sources could not enable RTP
  retransmission for lossy links ->
  rtspsrc retransmission is allowed and non-video RTSP channels are filtered
  during discovery.
- mavlink_broadcast_id_zero [binary-only] -- MAVLink messages with target system
  or component ID 0 were not honored as broadcast ->
  Broadcast target IDs of 0 are accepted for mavlink routing.
- onvif_discovery_reliability [binary-only] -- ONVIF discovery could fail on auth
  URI mismatches, premature task exit, or late stream-manager start ->
  Discovery strips credentials before compare, survives run failures, and
  starts only after stream manager is ready.
- radcam_secondary_hidden [UI] -- RadCam UnderwaterCam /stream_1 appeared as a
  separate video device in lists ->
  Secondary RadCam stream filtered from BlueOS device list; primary stream
  visible in QGC via ONVIF fix.

## mavlink-server (0.3.1 -> 0.5.9)

BlueOS launches mavlink-server with serial/UDP/TCP/zenoh proxy endpoints only.
REST/TLOG-condition/direction functions below are **not launched**.

### New functions

- rest_helper [binary-only] **not launched** -- REST v1 helper endpoint and built-in helper page
  for MAVLink message lookup and autocomplete.
- rest_vehicle_control [binary-only] **not launched** -- REST v1 vehicle control endpoints (arm,
  disarm, mode, yaw, guided limits) via the rest driver.
- tlog_creation_conditions [binary-only] **not launched** -- TLOG driver file-creation-condition
  CLI and per-endpoint options (e.g. log only when armed).
- endpoint_direction [binary-only] **not launched** -- UDP and TCP endpoint send-only,
  receive-only, or both direction control on proxy endpoints.
- rest_parameters [binary-only] **not launched** -- REST and websocket parameter get/set with
  autopilot abstraction and metadata.
- resources_api [binary-only] **not launched** -- REST resources API exposing driver and endpoint
  resource information.
- control_webpage [binary-only] **not launched** -- Built-in control webpage with vehicle info,
  component names, and parameters table.
- zenoh_metadata [binary-only] **on*** -- Zenoh driver publishes metadata information
  alongside MAVLink topics when an operator adds a Zenoh MAVLink endpoint.

### Behavioral changes

- versionized_rest_v1 [binary-only] -- Unversioned REST routes with inconsistent
  404 and trailing-slash handling ->
  Versioned /v1 router with --default-api-version CLI, unified 404, permissive
  tracing.
- permissive_cors [binary-only] -- Browser clients (Cockpit) hit CORS errors on
  mavlink-server REST and webpages ->
  Permissive CORS and trace layers on all v1 routers.
- vehicle_components_layout [binary-only] -- Vehicle REST and control webpage
  returned a flat vehicle object ->
  Vehicle broken into components with per-component names and information.
- zenoh_per_field_topics [binary-only] -- Zenoh driver published one topic per
  MAVLink message ->
  Zenoh creates topics per message field for finer-grained subscription.
- zenoh_client_mode [binary-only] -- Zenoh endpoints used embedded defaults only
  ->
  CLI zenoh_config_file or run as a zenoh client against an external router.
- zenoh_put_encoding [binary-only] -- Zenoh put operations omitted payload
  encoding metadata ->
  Zenoh put includes proper encoding for Foxglove and zenoh clients.
- mavlink_server_service_logs [binary-only] -- mavlink-server on BlueOS 1.4.3
  launched without --log-path; no service log files on disk ->
  BlueOS 1.4.5 passes --log-path=/var/logs/blueos/services/mavlink-server/;
  binary writes rotating service logs.
- mavlink_system_id_from_env [binary-only] -- mavlink-server launched without
  explicit system/component ID flags ->
  ardupilot_manager passes --mavlink-system-id and --mavlink-component-id from
  MAV_SYSTEM_ID env.
- resources_api_improved [binary-only] -- Resources API returned basic driver
  listing ->
  Improved resources API with richer endpoint and driver resource detail.
- log_folder_write_check [binary-only] -- Logger attempted writes without
  checking destination folder permissions ->
  Logger verifies log folder write permission before creating log files.

## linux2rest (v0.6.2 -> v0.6.5)

No new functions.

### Behavioral changes

- network_probe_deltas [UI] -- Tray and System Information derived upload/download
  speeds from frontend total-byte snapshots ->
  Throughput uses linux2rest received_B/transmitted_B probe deltas.
- sampler_system_actor [UI] -- On-demand mutex+cached sysinfo refresh for cpu,
  disk, memory, network, process, and temperature endpoints ->
  Background Tokio Sampler with monotonic 5s cadence, fewer stale or spurious
  samples.
- per_core_cpu_metrics [UI] -- /system/cpu returned a single global aggregate ->
  Each logical CPU core returned separately for per-core usage bars plus clock
  and temperature in the tray CPU widget.
- platform_library_cm5 [binary-only] -- rppal and netstat2 versions limited Pi
  board detection ->
  v0.6.3 bumps rppal (0.18->0.22.1) and netstat2 for newer Pi boards including
  CM5; /platform JSON shape unchanged.

## Unchanged binaries

- filebrowser v2.30.0 -- no pin or bootstrap change 1.4.3->1.4.5.
- ttyd 1.6.3 -- no pin or bootstrap change 1.4.3->1.4.5.
- logviewer v1.0.1 -- no pin or bootstrap change 1.4.3->1.4.5.
- zenoh 1.0.0 -- version unchanged; x86_64 bootstrap toolchain musl->gnu only.
- mavlink2rest t0.11.23 -- version unchanged; bootstrap adds strip only.
- mavp2p v1.1.1 -- version unchanged; bootstrap adds strip only.
- mavlink-router v4 -- version unchanged; bootstrap adds strip only.
- bridges 0.10.3 -- version unchanged; bootstrap adds strip only.
- machineid 0.2.3 -- version unchanged; bootstrap adds strip only.

## Packaging-only (no binary pin change)

- create_ap_wifi_bootstrap -- create_ap install moved from
  core/services/wifi/setup.py to core/tools/wifi/bootstrap.sh; same
  lakinduakash/linux-wifi-hotspot commit 4627e3c; hotspot already on 1.4.3.
- zenoh_x86_toolchain -- zenoh bootstrap x86_64-unknown-linux-musl ->
  x86_64-unknown-linux-gnu; VERSION 1.0.0 unchanged.
- static_binary_strip -- binutils apt + strip on downloaded static binaries
  (bridges, machineid, mavlink2rest, mavp2p, mavlink-router) in Dockerfile
  download-binaries stage and install-system-tools.
- ardupilot_firmware_bootstrap_reloc -- Default ArduSub firmware pre-download
  moved from ardupilot_manager/setup.py to ardupilot_tools/bootstrap.sh; same
  stable-4.5.3 URLs.
- ardupilot_python_libs_reloc -- ArduPilot uploader/decoder/apj_tool scripts
  moved from ardupilot_tools/bootstrap.sh to setup-python-libs.sh via
  install-python-libs.sh; same ArduPilot commit pins.
- install_system_tools_wifi -- wifi added to install-system-tools.sh parallel
  bootstrap list; apt clean after system packages.

## Firmware

ArduSub stable-4.5.3 bundled firmware URLs unchanged from 1.4.3 (bootstrap path
relocated only).
