# FAILURE_MODE_LEDGER — every asserted `failure_modes` id, classified

**Source of truth:** the `failure_modes` `AssertedSet::established` block of each of the 26 service cards in `catalog/src/services/*.rs` (exported via `cargo run --bin export | jq '.services[].definition.failure_modes'`).
**Total:** 111 failure modes across 26 services. Every service card has an `established` set — none is `Asserted::unknown`.

**Status is the PLAN, not a result.** Live outcomes go in `FINDINGS.md`; this file is append-only and its `status` column is what we intend to do:

| status | meaning |
|---|---|
| `probed` | a Track A journey or a Track B probe (`NP-nn`) exercises it and the result will be attached |
| `skipped` | deliberately not exercised — reason given (no hardware, hard-exclude, would strand a DUT, lab-wide impact) |
| `limitation` | product/platform ceiling; we can describe it but cannot make it a pass/fail gate |
| `harness_gap` | we *should* probe it and currently cannot — the harness needs work (missing fixture, needs body inspection, needs tmux control) |

`NP-nn` ids resolve in `NEGATIVE_PROBES.md`. DUT column uses the same affinity vocabulary as `SAD_PATH_MATRIX.md`.

---

## ardupilot_manager (5)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| ardupilot_manager | mavlink_router_crash | `stop_autopilot` / NP-77 (as generator) | harness_gap | `tmux kill-session -t autopilot`, then `GET /mavlink2rest/v1/mavlink` and MCM `GET /streams`; expect degraded, not silent 200 | 177-only | B6/B7. Needs the tmux-control harness step the plan defers; `auto_restart_router` may mask it within seconds, so it also needs a timed observation window. |
| ardupilot_manager | flight_controller_heartbeat_loss | `restart_autopilot` | skipped | physically unplug the FC USB/serial while `GET /ardupilot-manager/v1.0/firmware_info` is polled | 177-only | Requires physical access mid-run; 87's Pixhawk1 is on `/dev/ttyACM0` so unplugging it is the cleanest reproduction, but it is not automatable this campaign. |
| ardupilot_manager | firmware_flash_failure | `upload_custom_firmware` / **A8** / NP-74 | probed | `POST /ardupilot-manager/v1.0/install_firmware_from_file` with `binary=@np-not-firmware.txt` → **415** (`index.py:211`) | 177-only, pixhawk-87 | The safe half of the mode: rejected before any write. The *bricking* half stays `skipped` — we will not deliberately flash a corrupt-but-accepted image. |
| ardupilot_manager | autopilot_subprocess_crash | `start_autopilot` / `stop_autopilot` / NP-76, NP-77 | probed | `POST /stop` then poll `GET /firmware_info`; then `POST /start` and confirm recovery; double-`POST /start` for the 423 shape (`index.py:88`) | 177-only, pixhawk-87 | This is the controlled version of the crash. SITL on 177 gives a crash surface without touching a real FC. |
| ardupilot_manager | hardware_device_unavailable | `restore_default_firmware` / **A9** / NP-73 | probed | `POST /ardupilot-manager/v1.0/restore_default_firmware?board_name=NoSuchBoard` → **404** (`index.py:286`); also naturally true on any DUT with no FC | all | On 87 (Pixhawk1) the *legitimate* Navigator-targeted variant also 404s — that is platform contrast, not a Fail. |

## bag_of_holding (3)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| bag_of_holding | database_file_missing | `modify_bag_database` | limitation | delete `db.json` and re-`GET /bag/v1.0/get/*`; `read_db` returns `{}` | 177-only | Contract is "empty object, no error" — there is no observable failure status, so it cannot be a pass/fail gate. Deleting the store also destroys real settings ⇒ 177 only. |
| bag_of_holding | json_decode_error | `modify_bag_database` | harness_gap | corrupt `db.json` on disk, then `GET /bag/v1.0/get/*` | 177-only | Needs filesystem write access via `POST /commander/v1.0/command/host` plus a snapshot/restore of `db.json`. Same silent-`{}` shape as above, so the assertion has to be on the log, not the status. |
| bag_of_holding | invalid_get_path | `modify_bag_database` / **A5** / NP-24 | probed | `GET /bag/v1.0/get/np/definitely/missing` → **400** (`bag_of_holding/main.py:102`) | all | The one bag failure mode with a real status. Track A candidate A5. |

## beacon (4)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| beacon | mdns_registration_failure | `discover_blueos_on_network` | harness_gap | `avahi-browse -rt _http._tcp` from the runner before/after `POST /beacon/v1.0/hostname` | not-2.2 | Needs an mDNS observer on the runner; there is no HTTP status to assert. |
| beacon | interface_runner_creation_failure | `discover_blueos_on_network` | limitation | bring an interface down while beacon is running | 177-only | beacon logs and skips the interface; no API surface. |
| beacon | service_info_value_error | `discover_blueos_on_network` | limitation | requires a malformed registered service entry | 177-only | Internal `ValueError` path, log-only. |
| beacon | stale_mdns_after_hostname_change | `change_mdns_hostname` / NP-40 | harness_gap | `POST /beacon/v1.0/hostname?hostname=np-catalog`, then poll mDNS until the new domain appears; restore `hostname=blueos` | not-2.2 | The mode is explicitly about the diff-on-next-cycle delay, so it needs a timed mDNS observation. Restore is already in the allowlist (`runner.rs:1087-1098`). |

## bridget (5)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| bridget | linux2rest_unreachable | `view_configured_serial_bridges` | probed | stop the `linux2rest` tmux session, then `GET /bridget/v1.0/serial_ports` | 177-only | B6 177: `pkill -x linux2rest` → **200** `[]` (F-077); not 502. `/system-information/system` → 502 in a separate pkill cycle. |
| bridget | bridge_subprocess_crash | `create_serial_to_udp_bridge` | skipped | kill the `bridges` child process for a configured bridge | 177-only | Needs a real serial device *and* process control; no USB serial fixture is guaranteed on any DUT this campaign. |
| bridget | serial_port_contention | `create_serial_to_udp_bridge` | skipped | create a bridge on the autopilot's own serial device | 177-only | Deliberately breaks the autopilot link. Not worth the blast radius during a release campaign. |
| bridget | udp_endpoint_conflict | `create_serial_to_udp_bridge` / NP-80 variant | harness_gap | `POST /bridget/v1.0/bridges` twice with the same `udp_listen_port` | 177-only | Automatable in principle, but bridget has no error mapping so the status is `UNKNOWN_LIVE`; needs a real serial path to get far enough. |
| bridget | settings_persistence_failure | `create_serial_to_udp_bridge` | limitation | make the userdata settings path unwritable | 177-only | Would require breaking the filesystem; product ceiling, not a gate. |

## cable_guy (5)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| cable_guy | network_reconfiguration_lockout | `assign_static_ip_address`, `set_network_interface_priority`, `configure_host_dns` | skipped | apply a static IP / priority change that moves the default route off the management link | 177-only, never 2.2 | This *is* the stranding risk the plan forbids. NP-41…NP-47 deliberately use `nonexistent0` and malformed bodies so the lockout is never reached. |
| cable_guy | dhclient_acquisition_timeout | `acquire_dynamic_ip_address` / NP-43 | probed | `POST /cable-guy/v1.0/dynamic_ip?interface_name=nonexistent0` → `UNKNOWN_LIVE`; the real timeout needs a DHCP-less segment | 177-only | The mode's own text says the route may *return* the timeout output with 200 — so the assertion is on the body, not the status. |
| cable_guy | dnsmasq_start_failure | `enable_onboard_dhcp_server` / NP-44 | probed | `POST /cable-guy/v1.0/dhcp?interface_name=nonexistent0&ipv4_gateway=10.99.99.1&is_backup_server=false` → `UNKNOWN_LIVE` | 177-only | A real second DHCP server on the shared lab LAN affects other machines, so only the invalid-interface variant runs unattended. |
| cable_guy | resolv_conf_write_failure | `configure_host_dns` | harness_gap | make `/etc/resolv.conf` immutable in a way `chattr`/`tee` cannot fix | 177-only | Needs host-level file manipulation plus a guaranteed restore; the existing smoke already has a `resolv.conf` repair query (`runner.rs:460`) which would mask it. |
| cable_guy | interface_watchdog_reconciliation | `assign_static_ip_address` | limitation | observe an interface state mismatch between watchdog cycles | 177-only | Explicitly a transient-until-next-cycle behaviour; not a deterministic gate. |

## commander (5)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| commander | i_know_what_i_am_doing_rejected | **A1**, **A2** / NP-01…NP-08 | probed | every gated route with `i_know_what_i_am_doing=false` → **400** (`commander/main.py:52`) | all | The most probed mode in the campaign: 8 probes across `/command/host`, `/shutdown`, `/set_time`, `/raspi/vcgencmd`, `/raspi/eeprom_update`, `/settings/reset`. **Caveat:** `/set_time` only reaches the gate for a timestamp >5 min from now (`main.py:76`). |
| commander | host_command_subprocess_failure | `run_host_command` | probed | `POST /commander/v1.0/command/host?command=false&i_know_what_i_am_doing=true` → 200 with `return_code != 0` | all | Contract is a **200 carrying a non-zero `return_code`**, not an HTTP error. A probe asserting 4xx here would be wrong. |
| commander | raspi_config_legacy_failure | `enable_legacy_camera_support` / NP-98 | probed | `POST /commander/v1.0/raspi_config/camera_legacy?enable=false` on a host where `raspi-config` exits non-zero → **400** (`main.py:124`) | navigator, not-2.2 | On a Pi this is the *happy* path; the 400 appears exactly where `raspi-config` is unavailable. Expect a platform split between 177/124/2.2 and 87. |
| commander | ssh_setup_failure | — (startup path, no journey) | limitation | make key generation or `authorized_keys` write fail at boot | 177-only | Log-and-continue at startup; no route and no journey references it. |
| commander | settings_reset_partial_failure | `reset_blueos_settings` | skipped | run a confirmed `POST /commander/v1.0/settings/reset` and inspect leftovers | 177-only | The confirmed reset is destructive (`FilesystemReplace` + `ContainerRestart`). The gate rejection (NP-05) is probed instead; the partial-failure half stays skipped for W6. |

## customization (5)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| customization | invalid_theme_color | `change_ui_theme_color` / **A3** / NP-12, NP-13, NP-14 | probed | `PUT /customization/v1.0/theme` body `{"primary":"#ZZZZZZ"}` → **400** (`theme.py:9-16` → `main.py:77`) | all | Three input shapes: bad hex digits, wrong length (`#fff`), missing field. |
| customization | upload_size_exceeded | `upload_custom_logo` / NP-23 | harness_gap | `POST /customization/v1.0/branding/logo` with a >20 MiB PNG → **413** (`main.py:118-120`, limit `main.py:37`) | all | Needs a new oversize fixture under `catalog/fixtures/`. Also verify nginx's own `client_max_body_size` (`nginx.conf:11,149,289`) does not shadow the app's 413 with a 413 of its own. |
| customization | invalid_file_extension | `upload_custom_logo`, `upload_custom_vehicle_image`, `upload_3d_model_override` / **A4** / NP-15, NP-16, NP-17 | probed | upload a `.txt` → **400** (images `main.py:250`, models `main.py:205-209`) | all | Allowed sets are explicit: images `{.png,.jpg,.jpeg,.webp,.svg,.gif}`, models `{.glb}` (`storage.py:15-16`). |
| customization | model_not_found | `delete_3d_model_override` / NP-19 | probed | `DELETE /customization/v1.0/models/np-definitely-missing.glb` → **404** (`main.py:222-224`) | all | Contrast with branding deletes, which are idempotent 204 — the asymmetry is intentional per source. |
| customization | path_traversal_blocked | `upload_3d_model_override` / NP-18 | probed | `POST /customization/v1.0/models?name=../../etc/np-evil.glb` → **400** (`storage.py:24-30` → `main.py:77`) | all | Security-relevant. A 200 here is a blocker-severity product finding. |

## disk_usage (5)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| disk_usage | du_subprocess_failure | `inspect_disk_usage` | limitation | make `du` return non-zero on a subtree (e.g. permission-denied paths) | all | The mode says the service logs but may return a *partial tree with 200* — so it is a body-completeness observation, not a status gate. |
| disk_usage | disktest_binary_missing | `run_single_disk_speed_test` | probed | `GET /disk-usage/v1.0/disk/speed` on a DUT without `disktest` → **503** (`main.py:324-327`) | all | This is the Track D typed skip (`disktest binary is available on PATH` precondition). A 503 is the *expected* result where the binary is absent — never a Fail. |
| disk_usage | insufficient_storage_for_benchmark | `run_single_disk_speed_test` / NP-30 | probed | `GET /disk-usage/v1.0/disk/speed?size_bytes=9007199254740992` → **507** (`main.py:335-338`) | all | Cleanest 507 in the codebase: the size check happens before any file is written, so it is fully safe. |
| disk_usage | protected_path_deletion_refused | `free_disk_space` / NP-28 | probed | `DELETE /disk-usage/v1.0/disk/paths/etc` → **400** (`main.py:278`, roots `main.py:109-126`) | all | High-value guard test — run it before any real delete probe. |
| disk_usage | invalid_or_missing_path | `inspect_disk_usage`, `free_disk_space` / NP-26, NP-27, NP-29 | probed | `GET /disk-usage/v1.0/disk/usage?path=/np/missing` → **404**; `DELETE .../disk/paths/<missing>` → **404** | all | **Contradiction:** the mode claims 400 for "paths outside `/`", but `FILESYSTEM_ROOT = Path("/")` (`main.py:27`) makes that branch (`main.py:100-103`) unreachable. Expect 404 only. |

## filebrowser (3)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| filebrowser | filebrowser_process_down | `manage_blueos_files` | harness_gap | stop the `filebrowser` tmux session, then `GET /file-browser/` | 177-only | B6; needs tmux control. |
| filebrowser | rest_listener_unavailable | `manage_blueos_files` | harness_gap | block port 7777, then `GET /file-browser/` and expect an nginx 502, not a blank 200 | 177-only | Overlaps `FM:nginx/backend_proxy_unreachable`; one B6 wave can cover both. |
| filebrowser | filebrowser_database_unavailable | `manage_blueos_files` | limitation | corrupt or remove `/etc/filebrowser/filebrowser.db` | 177-only | Third-party state; restore requires a file snapshot. Product ceiling. |

## helper (5)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| helper | nginx_reload_failure | (extension install → `reload_nginx`) | limitation | make `nginx -s reload` fail while installing an extension | 177-only | `subprocess.run(check=False)` means the failure is invisible to the caller; the observable symptom is a stale extension route. Not a gate. |
| helper | internet_probe_unreachable | `monitor_internet_connectivity`, `verify_internet_connectivity` / NP-50 | probed | `GET /helper/v1.0/check_internet_access` with WAN blocked → 200 with every site `offline` | all | Contract is a 200 with negative content. B8 skip must be typed. |
| helper | service_scan_timeout | `browse_available_web_services` | harness_gap | stop a scanned backend, then `GET /helper/v1.0/web_services` and look for an invalid `ServiceInfo` | 177-only | B6-adjacent; needs body inspection plus tmux control. (B6 helper-down: `GET /helper/v1.0/web_services` → 502 when helper stopped — probed B6_177.) |
| helper | factory_mode_notification_failure | — (startup path, no journey) | limitation | make version-chooser unreachable during helper startup | 177-only | Log-and-skip; no journey references it. |
| helper | uuid_read_failure | — (`GET /hardware_id`, `GET /software_id`; no journey) | harness_gap | remove or corrupt `/etc/blueos/hardware-uuid`, then `GET /helper/v1.0/hardware_id` → **400** (`helper/main.py:558`, `:575`) | 177-only | **Coverage hole:** these two routes have an explicit 400 contract but no journey and no capability journey_ref reaches them. Worth a Track B probe once a snapshot/restore of the UUID files exists. |

## iperf3 (3)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| iperf3 | iperf3_process_down | — (no journey; iperf3 has no nginx prefix) | harness_gap | stop the `iperf3` tmux session, then `iperf3 -c <dut>` from the runner | 177-only | iperf3 is reached on TCP 5201 directly, not through nginx, so `journey_http` cannot see it at all. |
| iperf3 | listener_unavailable | — (no journey) | harness_gap | bind-conflict or firewall port 5201, then `iperf3 -c <dut>` | 177-only | Same as above — needs a non-HTTP probe path in the harness. |
| iperf3 | active_test_link_saturation | — (no journey) | skipped | run a sustained `iperf3` while video/telemetry are live | not-2.2 | Deliberately saturates the link; unacceptable on the physical ROV and disruptive on the shared LAN. |

## kraken (7)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| kraken | docker_daemon_unavailable | `install_extension`, `configure_installed_extension` | skipped | stop the Docker daemon on the host | 177-only | Stopping Docker takes down `blueos-core` itself, so this is effectively a whole-DUT outage rather than a service-down probe. |
| kraken | manifest_fetch_failure | `add_custom_manifest`, `browse_extension_store` / NP-56 | probed | `POST /kraken/v2.0/manifest/?validate_url=true` with an unreachable URL → `UNKNOWN_LIVE` (map says **502** at `manifest.py:37`) | all | Also occurs naturally under B8 (WAN blocked) — that variant must be a typed skip. |
| kraken | image_pull_failure | `install_extension` | probed | install an extension with WAN blocked, or NP-51 for the not-found shape | all | The pull streams progress, so the failure may arrive in-band rather than as a status; `streamed_fragment_error` inspects the commonwealth fragment envelope and is wired into `evaluate_http_response`, so every suite fails on an in-band error. |
| kraken | extension_crash_loop | `install_extension`, `configure_installed_extension` | limitation | install an extension whose entrypoint exits immediately | 177-only | Exponential-backoff retry loop; observable only over time via `GET /kraken/v2.0/container/{name}/log`. |
| kraken | insufficient_storage | `install_extension` | harness_gap | install an extension whose `expanded_size` exceeds free disk → **507** (`extension.py:37`) | 177-only | Needs either a genuinely full disk or a crafted manifest with an inflated `expanded_size`. Filling the disk on purpose is a bad idea mid-campaign. |
| kraken | incompatible_extension | `install_extension` | harness_gap | install an extension with no image digest for `linux/arm64` → **400** (`extension.py:39`) | all | Needs a crafted manifest entry; safe once the fixture exists. |
| kraken | extension_lifecycle | `--extension-lifecycle` | probed | `journey_http --base <url> --extension-lifecycle --allow-mutating`: install of an absent extension, upgrade, same-version reinstall, downgrade, uninstall, each with effect reads against `GET /extension/` and `GET /container/` | all | Opt-in mutating; not part of `gate.sh`. A first-time install of a not-yet-installed extension was broken in `1.4.4-beta.19` and the previous harness scored it green. |

## linux2rest (3)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| linux2rest | rest_listener_down | `view_system_information` | probed | stop the `linux2rest` tmux session, then `GET /system-information/system` | 177-only | B6 177: `pkill -x linux2rest` → **502**; restore via `run-service`. See B6_177.md. |
| linux2rest | privileged_proc_read_failure | `view_system_information` | limitation | run with reduced privileges so `/proc` queries return partial data | 177-only | Partial-data-with-200 shape; body completeness, not a status. |
| linux2rest | serial_port_enumeration_stale | `view_configured_serial_bridges` | probed | `pkill -x linux2rest` in its own stop cycle (C-c leaves listener up), then `GET /bridget/v1.0/serial_ports` | 177-only | B6 177: returns **200** `[]` (F-077), not error/502. |

## mavlink-camera-manager (4)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| mavlink-camera-manager | mavlink_router_endpoint_unreachable | `view_camera_streams` | harness_gap | `POST /ardupilot-manager/v1.0/stop` (NP-77), then `GET /mavlink-camera-manager/streams` | 177-only | B7 analogue of the wifi AP-loss journey; automatable once `--negative` can sequence a disruptor step. |
| mavlink-camera-manager | camera_device_unavailable | `view_camera_streams`, `configure_camera_stream` / NP-100 | probed | `GET /mavlink-camera-manager/v4l` with no camera → 200 with an empty list | all | Track D: absence is a typed skip. A *wrong* skip (camera present, harness skipped) is a harness bug per the plan. |
| mavlink-camera-manager | stream_pipeline_failure | `configure_camera_stream` / NP-86 | harness_gap | create a stream with an unusable encoder/endpoint and watch it die while the REST API stays up | 177-only | Needs a camera plus stream-state inspection; the REST layer stays 200 so this cannot be asserted on status alone. |
| mavlink-camera-manager | webrtc_stun_unreachable | `configure_video_stream` | skipped | block the external STUN server and attempt a cross-NAT WebRTC session | 177-only | Frontend/browser-driven and explicitly a graceful degradation (local UDP/RTSP keeps working). Not worth a campaign slot. |

## mavlink2rest (4)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| mavlink2rest | mavlink_router_endpoint_unreachable | `inspect_mavlink_messages_in_browser` | harness_gap | `POST /ardupilot-manager/v1.0/stop`, then read `/mavlink2rest/` | 177-only | Same B7 disruptor as MCM; bundle them. |
| mavlink2rest | rest_bridge_down | `inspect_mavlink_messages_in_browser`, all calibration journeys | probed | stop the `mavlink2rest` tmux session, then load the calibration page | 177-only | B6 177: `GET /mavlink2rest/v1/mavlink` → **502** when stopped (HTTP proxy). Frontend cal journeys not re-run. |
| mavlink2rest | websocket_stream_stale | `inspect_mavlink_messages_in_browser` | harness_gap | drop the WebSocket mid-view while REST still answers | 177-only | Needs a WebSocket client in the harness; `journey_http` is curl-only. |
| mavlink2rest | mavlink_send_surface_abuse | `apply_parameter_file` | skipped | `POST` an arbitrary MAVLink message (param write / mode change) with no authentication | 177-only | This is a **security limitation to document, not to exercise** — sending arbitrary MAVLink to a real vehicle is exactly what we must not do. Record it as a finding of kind `limitation`. |

## nginx (4)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| nginx | master_process_down | `access_blueos_web_interface` | skipped | kill the nginx master | 177-only | Removes the only ingress, including the harness's own recovery probe (`GET /status`). Recovery would need SSH; not worth it. |
| nginx | port_80_bind_failure | `access_blueos_web_interface` | skipped | occupy port 80 before nginx starts | 177-only | Same total-ingress-loss problem. |
| nginx | frontend_spa_unavailable | `access_blueos_web_interface` | harness_gap | move `/home/pi/frontend` aside, then `GET /` | 177-only | Reversible via `FilesystemReplace`, but needs a file snapshot; `GET /status` (204) stays up, which is itself the assertion. |
| nginx | backend_proxy_unreachable | every proxied journey | harness_gap | stop any one backend tmux session, then hit its prefix; expect **502/504**, not a silent 200 | 177-only | The single highest-value B6 test: it validates that *all* B6 probes produce a real status. Run it once per backend in W5/W6. |

## nmea_injector (6)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| nmea_injector | rest_listener_down | `view_configured_nmea_sockets` | probed | stop the `nmea_injector` tmux session, then `GET /nmea-injector/v1.0/socks` | 177-only | B6 177: tmux C-c → **502**; restore via `run-service`. See B6_177.md. |
| nmea_injector | injecting_wrong_position_affects_navigation | `add_external_nmea_gps_socket` | skipped | send spoofed NMEA to a configured socket and watch `GPS_INPUT` reach the autopilot | never 2.2 | Deliberately corrupting the position estimate of a real ROV is out of bounds. Document as a `limitation`-class safety note. |
| nmea_injector | mavlink2rest_unreachable | `add_external_nmea_gps_socket` | harness_gap | stop `mavlink2rest`, then feed the socket; NMEA is dropped silently | 177-only | B6; silent drop means the assertion must be on logs or on absent `GPS_INPUT`, not on a status. |
| nmea_injector | invalid_nmea_parse_failure | `add_external_nmea_gps_socket` | harness_gap | send a non-NMEA datagram to the configured UDP port | 177-only | Needs a UDP sender in the harness; safe, and a good cheap addition. |
| nmea_injector | port_conflict_on_socket_create | `add_external_nmea_gps_socket` / NP-84 | probed | `POST /nmea-injector/v1.0/socks` with `port: 80` (already bound by nginx) → `UNKNOWN_LIVE` | all | nmea_injector has no error mapping, so the status must be captured. Restore with the matching `DELETE /socks`. |
| nmea_injector | remove_nonexistent_socket | `remove_configured_nmea_socket` / NP-82 | probed | `DELETE /nmea-injector/v1.0/socks` body `{"kind":"UDP","port":9999,"component_id":220}` when absent → `UNKNOWN_LIVE` | all | The failure mode says "returns error" with **no status**, and the service declares none (`nmea_injector/main.py:59`). Capture first, then tighten the card. |

## pardal (4)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| pardal | speedtest_not_initialized | `run_internet_speed_test` / NP-90 | probed | `GET /network-test/internet_download_speed` on a fresh service, before `GET /internet_best_server` → `UNKNOWN_LIVE` | all | Ordering-dependent global (`SPEED_TEST`); requires the probe to run before any successful server search, so schedule it first in the pardal group. |
| pardal | speedtest_cli_unavailable_offline | `run_internet_speed_test` | probed | block WAN, then `GET /network-test/internet_best_server` | all | B8: must produce a typed skip. Overlaps `speedtest_not_initialized` because a failed server search leaves the global unset. |
| pardal | websocket_echo_disconnect | `run_lan_speed_test` | harness_gap | close the `/ws` echo session mid-latency-test | all | Needs a WebSocket client; curl-only harness cannot do it. |
| pardal | large_transfer_resource_pressure | `run_lan_speed_test` | skipped | run a full-size `GET /network-test/get_file` / `POST /post_file` pair | not-2.2 | Saturates the link (limits: 2 GiB global, 200 M and 100 M per-location — `nginx.conf:11,149,289`). Unacceptable while 2.2 carries live video/telemetry. |

## ping (5)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| ping | sonar_not_detected | `view_detected_sonar_devices` / NP-99 | probed | `GET /ping/v1.0/sensors` with no sonar → 200 with an empty list | all | Track D: the empty list *is* the contract. A Fail here would be a harness bug. |
| ping | bridge_subprocess_crash | `connect_ping_viewer_to_sonar` | skipped | kill the `bridges` child for a detected sonar | 177-only | Needs real Ping hardware; none is guaranteed on any DUT this campaign. |
| ping | wrong_distance_affects_depth_hold | `enable_ping1d_rangefinder_mavlink` | skipped | enable the MAVLink driver and forward a stale/incorrect `DISTANCE_SENSOR` | never 2.2 | Safety: skewing depth-hold on a physical ROV. Document as a `limitation`-class note. |
| ping | mavlink2rest_unreachable | `enable_ping1d_rangefinder_mavlink` | harness_gap | stop `mavlink2rest` while the Ping1D driver is enabled | 177-only | B6; needs both a sonar and tmux control. |
| ping | udp_bridge_port_conflict | `connect_ping_viewer_to_sonar` | harness_gap | occupy a port in the 9090–9092 range before detection | 177-only | Needs sonar hardware to reach the bridge-allocation path. |

## recorder (3)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| recorder | recorder_process_down | `browse_video_recordings` (indirect) | harness_gap | stop the `recorder` tmux session, then `GET /recorder-extractor/v1.0/recorder/status` | 177-only | recorder has **no nginx prefix and no journey of its own**; it is only observable through recorder_extractor. B6. |
| recorder | recorder_path_unavailable | `browse_video_recordings` (indirect) | harness_gap | make `/usr/blueos/userdata/recorder` unwritable | 177-only | Needs host filesystem manipulation plus restore. |
| recorder | disk_full_from_recording | `free_disk_space` | skipped | let continuous MCAP capture fill userdata | 177-only | Filling the disk mid-campaign would poison every other wave. Related safe probe: NP-30 (507 on disk speed). |

## recorder_extractor (4)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| recorder_extractor | recording_not_found | `download_video_recording`, `delete_video_recording` / NP-67, NP-70, NP-71 | probed | `GET`/`DELETE /recorder-extractor/v1.0/recorder/files/np-no-such.mp4` → **404** (`main.py:81`) | all | Three probes share this one mode (download, delete, thumbnail). |
| recorder_extractor | delete_recording_failure | `delete_video_recording` | harness_gap | make a recording file undeletable, then `DELETE` it → **500** (`main.py:405-409`) | 177-only | Needs a read-only mount or immutable flag; reversible only with care. |
| recorder_extractor | mcap_extraction_failure | `browse_video_recordings` | harness_gap | seed a truncated/garbage `.mcap` and wait for the background extractor | 177-only | Background task: log-only, no status. Needs a fixture plus log inspection. |
| recorder_extractor | thumbnail_generation_failure | `browse_video_recordings` | harness_gap | seed a corrupt `.mp4`, then `GET .../recorder/files/<name>/thumbnail` | 177-only | Needs an intentionally-broken MP4 fixture; the existing smoke seeds a 3-byte `printf mp4` file (`runner.rs:455`), which may already trigger it — check that first, it may be free. |

## ttyd (3)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| ttyd | ttyd_process_down | `access_web_terminal` | harness_gap | stop the `ttyd` tmux session, then `GET /terminal/` | 177-only | B6. |
| ttyd | websocket_listener_unavailable | `access_web_terminal` | harness_gap | block port 8088, then `GET /terminal/` and expect an nginx 502 | 177-only | Overlaps `FM:nginx/backend_proxy_unreachable`. |
| ttyd | user_terminal_tmux_unavailable | `access_web_terminal` | harness_gap | kill the `user_terminal` tmux session, then attach via `/terminal/` | 177-only | ttyd recreates a session on attach (see `FM:user_terminal/user_terminal_tmux_down`), so the observable difference is the missing MOTD, not an error. |

## user_terminal (2)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| user_terminal | user_terminal_tmux_down | `access_web_terminal` | harness_gap | `tmux kill-session -t user_terminal`, then attach through `/terminal/` | 177-only | The recovery (ttyd's `tmux new -s user_terminal`) is the interesting part — confirm the shell still comes up. |
| user_terminal | motd_banner_lost | `access_web_terminal` | limitation | kill the boot session and let ttyd recreate it; the MOTD is not replayed | 177-only | Cosmetic, explicitly documented in the card as expected behaviour. Not a gate. |

## versionchooser (6)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| versionchooser | docker_daemon_unavailable | `update_blueos_version` and siblings | skipped | make `/var/run/docker.sock` unreachable | 177-only | Same problem as `FM:kraken/docker_daemon_unavailable`: it takes `blueos-core` down with it. |
| versionchooser | image_pull_failure | `pull_blueos_version_without_switch` / NP-64 | probed | `POST /version-chooser/v1.0/version/pull` body `{"repository":"bluerobotics/np-no-such-repo","tag":"np"}` → `UNKNOWN_LIVE` (500 at `chooser.py:91`, 501 at `:98`) | 177-only | Pull streams progress, so the error may be in-band. Two distinct codes exist in source — capture which one fires. |
| versionchooser | invalid_startup_json | `update_blueos_version` | harness_gap | corrupt `startup.json`, then `GET /version-chooser/v1.0/version/current` → 500 (`chooser.py:57`) | 177-only | Very high blast: `startup.json` drives the boot. Needs a proven file snapshot before it can be attempted, and only on 177. |
| versionchooser | registry_auth_failure | `docker_registry_login`, `update_blueos_version` / NP-65 | harness_gap | pull a private image with no prior login | all | NP-65 covers the bad-credentials half safely; the private-image half needs a private repo we do not have. |
| versionchooser | bootstrap_switch_failure | `update_bootstrap_image` / NP-63 | probed | `POST /version-chooser/v1.0/bootstrap/current` body `{"tag":"np-no-such-tag"}` → **412** (`chooser.py:221-223`) | all | Fully safe: the abort happens before the bootstrap container is touched. |
| versionchooser | switch_to_missing_image | `switch_local_blueos_version` / **A7** / NP-60 | probed | `POST /version-chooser/v1.0/version/current` body `{"repository":"bluerobotics/blueos-core","tag":"np-no-such-tag"}` → **412** (`chooser.py:283-285`) | all | Track A candidate A7. Also run NP-62 (delete the *running* tag → **500**, `chooser.py:331-333`) early: that guard is what keeps the delete probes from bricking a DUT. |

## wifi (5)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| wifi | wireless_reconfiguration_lockout | `toggle_hotspot`, `connect_to_wifi_network`, `disconnect_from_wifi_network` | skipped | enable the hotspot or disconnect while the operator path is the wlan | never 2.2, rf-serial elsewhere | Wifi-side twin of `FM:cable_guy/network_reconfiguration_lockout`. All four DUTs are managed over ethernet, which is what makes the RF journeys safe at all — keep it that way. |
| wifi | wpa_supplicant_connection_failure | `reject_invalid_wifi_credentials` (**already a journey**) | probed | `POST /wifi-manager/v1.0/connect` with `WRONG_PASSWORD_SMOKE_BODY` → **500** | rf-serial | The gold-standard Track A case. Contract pinned at `runner.rs:406`, restore at `runner.rs:1155-1168`. |
| wifi | hotspot_start_failure | `toggle_hotspot` | harness_gap | make `hostapd` / virtual-interface creation / `dnsmasq` fail (e.g. no radio, or radio busy as a station) | rf-serial | Naturally reproducible on a DUT with no wifi radio — but that is a Track D skip, not this mode. A genuine start failure needs radio manipulation. |
| wifi | scan_busy | `connect_to_wifi_network` / NP-38 | probed | two concurrent `GET /wifi-manager/v1.0/scan` → **425** (`wifi/main.py:71`) | all | **Currently uncovered by any journey** and completely radio-free. Cheapest missing wifi contract; add it to `--negative` first. |
| wifi | smart_hotspot_watchdog_mismatch | `toggle_smart_hotspot` / NP-37 | limitation | toggle smart-hotspot and observe asynchronous state flips | all | Explicitly transient-until-next-cycle. A B4 probe must re-read state rather than assert an immediate value. |

## zenohd (4)

| service | mode_id | mapped_journey_or_probe | status | how_to_exercise | dut | notes |
|---|---|---|---|---|---|---|
| zenohd | router_process_down | `inspect_zenoh_network` | harness_gap | stop the `zenohd` tmux session, then `GET /zenoh/` | 177-only | B6. Zenoh carries inter-service pub/sub, so expect collateral effects in MCM/recorder — capture them rather than treating them as separate Fails. |
| zenohd | pubsub_transport_partition | `inspect_zenoh_network` | limitation | make the router unreachable while local processes keep running | 177-only | Message-delivery loss with no status surface. |
| zenohd | rest_admin_plugin_down | `inspect_zenoh_network` / NP-93 | harness_gap | block port 7117, then `GET /zenoh/` → expect nginx 502 | 177-only | NP-93 only proves the happy prefix answers; the down case needs tmux/port control. |
| zenohd | websocket_remote_api_down | `inspect_zenoh_network` | harness_gap | block port 7118, then open the Zenoh Inspector | 177-only | Needs a WebSocket client. |

---

## Ledger summary

| status | count | share |
|---|---|---|
| `probed` | 33 | 30% |
| `harness_gap` | 45 | 41% |
| `skipped` | 18 | 16% |
| `limitation` | 15 | 14% |
| **total** | **111** | 100% |

### What the shape means

- **`probed` (33)** is almost entirely Track B safe probes plus the wifi Track A journey. These are the modes with an explicit status in source, and they are the ones we can claim as tested for 1.4-dev.
- **`harness_gap` (45)** is dominated by two missing harness capabilities:
  1. **tmux service stop/start or port blocking** (B6) — 19 modes across `bridget`, `linux2rest`, `nginx`, `filebrowser`, `ttyd`, `user_terminal`, `zenohd`, `recorder`, `mavlink2rest`, `ping`, `nmea_injector`, `iperf3`. One harness feature unlocks all of them, 177-only.
  2. **non-curl transports and fixtures** — WebSocket probes (4 modes), a UDP sender (1), and three missing fixtures (oversize upload, corrupt MP4, crafted manifest).
- **`skipped` (18)** are all justified by the plan's own safety rules: stranding (`cable_guy`, `wifi` lockout), vehicle safety (`nmea_injector` spoofing, `ping` depth-hold, `mavlink2rest` send abuse, motor spin), total-ingress loss (`nginx` master/port 80), whole-container loss (Docker daemon), or missing hardware (serial, sonar).
- **`limitation` (15)** are silent-degradation modes with no observable status: log-and-continue paths, partial-data-with-200 responses, and transient watchdog reconciliation. They should be described in `RELEASE_READINESS.md` as known ceilings, not chased.

### Coverage holes worth a harness change (W9 candidates)

1. **tmux stop/start harness step** — unlocks 19 `harness_gap` B6 modes on 177 in a single wave. Highest return of anything in this ledger.
2. **`FM:helper/uuid_read_failure`** — `GET /helper/v1.0/hardware_id` and `/software_id` have an explicit **400** contract (`helper/main.py:558,575`) but **no journey reaches them**. This is a genuine journey-coverage hole, not just a probe hole.
3. **`FM:recorder/*`** — the `recorder` service has no nginx prefix and no journey; it is only observable through recorder_extractor. Consider whether it needs a journey or an explicit "not operator-visible" note on the card.
4. **`FM:iperf3/*`** — iperf3 is TCP 5201 with no nginx prefix, so `journey_http` structurally cannot reach it. Either add a non-HTTP probe path or mark all three modes `limitation` by design.
5. **Three missing fixtures** — oversize image (>20 MiB) for the 413, corrupt `.mp4` for the thumbnail failure, crafted manifest for `incompatible_extension`/`insufficient_storage`. All three are cheap and safe.
6. **`FM:nmea_injector/remove_nonexistent_socket`** asserts an error with no code, against a service that declares no handler. Capture live, then tighten the card so the asserted layer stops implying a contract that does not exist.
