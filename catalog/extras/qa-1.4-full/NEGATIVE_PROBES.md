# NEGATIVE_PROBES — implementable Track B table for a future `catalog/src/negative_probes.rs`

**94 probes.** Every probe here is (a) HTTP-automatable through the nginx front door, (b) `safe` or `reversible`, and (c) attached to an existing `JourneyId` — no new journey ids are introduced.

## Inclusion rules applied

- **No B6 service-down.** Nothing here stops a tmux session or a backend. Those 19 modes stay in `FAILURE_MODE_LEDGER.md` as `harness_gap`, 177-only.
- **No stranding.** No probe reconfigures the interface, route, DNS, or wifi association that the runner is talking over. Cable-guy and wifi probes use `nonexistent0` / bogus SSIDs / malformed bodies so the apply path is never reached on a live link.
- **No shutdown / poweroff / reboot.** `NP-02` and `NP-03` hit `/commander/v1.0/shutdown` with `i_know_what_i_am_doing=false`, which returns 400 **before** anything is scheduled (`commander/main.py:52`).
- **Every status is sourced.** `expected_status` is either read out of BlueOS Python/Rust source, or asserted by a `failure_modes` entry, or written `UNKNOWN_LIVE`. Nothing is guessed. 47 probes are pinned, 47 are `UNKNOWN_LIVE`.

## Column meanings

| column | meaning |
|---|---|
| `id` | stable probe id, referenced from `SAD_PATH_MATRIX.md` and `FAILURE_MODE_LEDGER.md` |
| `journey_id` | the existing `JourneyId` this probe attaches to (Track B — never a new journey) |
| `class` | B1–B9 from the plan |
| `nginx path` | full front-door path including query string, exactly as the harness should send it |
| `body` | `—` for none; multipart fixtures are named `catalog/fixtures/<name>` |
| `expected_status` | pinned value **in bold**, or `UNKNOWN_LIVE` for first-capture |
| `restore` | `none` means the probe provably changes no state |
| `blast` | `safe` = no state change; `reversible` = state change with the restore in the same row |

Ids `NP-10`, `NP-34`, `NP-81` are intentionally unused (reserved during drafting). `NP-72`, `NP-75`, `NP-76`, `NP-77` exist in `SAD_PATH_MATRIX.md` but are **excluded** here — see the last section.

---

## commander — the confirmation gate (11 probes)

All 400s come from one function, `check_what_i_am_doing` (`core/services/commander/main.py:49-55`, raise at `:52`). This is the densest safe-probe cluster in BlueOS and it works on all four DUTs including the physical ROV.

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-01 | `run_host_command` | B2 | POST | `/commander/v1.0/command/host?command=true&i_know_what_i_am_doing=false` | — | **400** | none | safe |
| NP-02 | `reboot_onboard_computer` | B2 | POST | `/commander/v1.0/shutdown?shutdown_type=reboot&i_know_what_i_am_doing=false` | — | **400** | none | safe |
| NP-03 | `shutdown_onboard_computer` | B2 | POST | `/commander/v1.0/shutdown?shutdown_type=poweroff&i_know_what_i_am_doing=false` | — | **400** | none | safe |
| NP-04 | `sync_system_time` | B2 | POST | `/commander/v1.0/set_time?unix_time_seconds=1700000000&i_know_what_i_am_doing=false` | — | **400** | none | safe |
| NP-05 | `reset_blueos_settings` | B2 | POST | `/commander/v1.0/settings/reset?i_know_what_i_am_doing=false` | — | **400** | none | safe |
| NP-06 | `inspect_raspberry_eeprom_bootloader` | B2 | GET | `/commander/v1.0/raspi/vcgencmd?command=version&i_know_what_i_am_doing=false` | — | **400** | none | safe |
| NP-07 | `inspect_raspberry_eeprom_bootloader` | B2 | GET | `/commander/v1.0/raspi/eeprom_update?i_know_what_i_am_doing=false` | — | **400** | none | safe |
| NP-08 | `update_raspberry_eeprom_bootloader` | B2 | POST | `/commander/v1.0/raspi/eeprom_update?i_know_what_i_am_doing=false` | — | **400** | none | safe |
| NP-09 | `run_host_command` | B1 | POST | `/commander/v1.0/command/host?i_know_what_i_am_doing=false` | — | UNKNOWN_LIVE | none | safe |
| NP-11 | `reboot_onboard_computer` | B1 | POST | `/commander/v1.0/shutdown?shutdown_type=banana&i_know_what_i_am_doing=false` | — | UNKNOWN_LIVE | none | safe |
| NP-98 | `enable_legacy_camera_support` | B1 | POST | `/commander/v1.0/raspi_config/camera_legacy?enable=banana` | — | UNKNOWN_LIVE | none | safe |

**NP-04 caveat (must be honoured in the implementation):** `set_time` has no gate of its own. For a timestamp within 5 minutes of now it returns 200 "not updating" *before* delegating to the gated `command_host` (`commander/main.py:72-84`). The far-past `1700000000` is required to reach the 400. Do not "simplify" this to `unix_time_seconds=$(date +%s)`.

**NP-09 / NP-11 / NP-98** are framework-level rejections (missing required query param, bad enum, bad bool). FastAPI's default is a 422, but commander installs a custom handler, so the real code must be captured before it is pinned.

---

## customization — validation, suffix allowlist, path traversal, size limit (12 probes)

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-12 | `change_ui_theme_color` | B1 | PUT | `/customization/v1.0/theme` | `{"primary":"#ZZZZZZ"}` | **400** | none | safe |
| NP-13 | `change_ui_theme_color` | B1 | PUT | `/customization/v1.0/theme` | `{"primary":"#fff"}` | **400** | none | safe |
| NP-14 | `change_ui_theme_color` | B1 | PUT | `/customization/v1.0/theme` | `{}` | UNKNOWN_LIVE | none | safe |
| NP-15 | `upload_custom_logo` | B1 | POST | `/customization/v1.0/branding/logo` | multipart `file=@catalog/fixtures/np-bad-suffix.txt` | **400** | none | safe |
| NP-16 | `upload_custom_vehicle_image` | B1 | POST | `/customization/v1.0/branding/vehicle-image` | multipart `file=@catalog/fixtures/np-bad-suffix.txt` | **400** | none | safe |
| NP-17 | `upload_3d_model_override` | B1 | POST | `/customization/v1.0/models?name=np-bad.txt` | multipart `file=@catalog/fixtures/np-model.glb` | **400** | none | safe |
| NP-18 | `upload_3d_model_override` | B1 | POST | `/customization/v1.0/models?name=../../etc/np-evil.glb` | multipart `file=@catalog/fixtures/np-model.glb` | **400** | none | safe |
| NP-19 | `delete_3d_model_override` | B3 | DELETE | `/customization/v1.0/models/np-definitely-missing.glb` | — | **404** | none | safe |
| NP-20 | `remove_custom_logo` | B4 | DELETE | `/customization/v1.0/branding/logo` (second consecutive call) | — | **204** | `PUT /customization/v1.0/branding/logo` from snapshot | reversible |
| NP-21 | `remove_custom_vehicle_image` | B4 | DELETE | `/customization/v1.0/branding/vehicle-image` (second consecutive call) | — | **204** | re-upload snapshot | reversible |
| NP-22 | `reset_ui_theme_color` | B4 | DELETE | `/customization/v1.0/theme` (second consecutive call) | — | **204** | `PUT /customization/v1.0/theme` from snapshot | reversible |
| NP-23 | `upload_custom_logo` | B5 | POST | `/customization/v1.0/branding/logo` | multipart `file=@catalog/fixtures/np-oversize-21mib.png` | **413** | none | safe |

Anchors: `parse_hex` `customization/theme.py:9-16` → 400 at `main.py:77`; image suffix allowlist `storage.py:16` → 400 at `main.py:247-251`; model allowlist `{.glb}` `storage.py:15` → 400 at `main.py:205-209`; `safe_join` traversal `storage.py:24-30`; `MAX_UPLOAD_BYTES` `main.py:37` → 413 at `main.py:118-120`; model 404 at `main.py:222-224`.

**NP-18 is the security probe of this group.** A 2xx means `safe_join` let a write escape the models directory — blocker severity, report immediately, do not batch it into the summary.

**NP-23 needs a fixture that does not exist yet** (`np-oversize-21mib.png`, >20 MiB). Generate it as an incompressible PNG-headered blob rather than committing 21 MiB. Also confirm the 413 comes from the app and not from nginx's own `client_max_body_size` (`core/tools/nginx/nginx.conf:11,149,289`) — same code, different origin, and only the app's version proves `save_upload`.

**NP-20/21/22 ordering:** run them immediately after the corresponding mutating-smoke entry (`catalog/src/mutating_smoke.rs:72-76`), so the first delete is the smoke's and this is provably the second call.

---

## bag_of_holding (3 probes)

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-24 | `modify_bag_database` | B1 | GET | `/bag/v1.0/get/np/definitely/missing` | — | **400** | none | safe |
| NP-25 | `modify_bag_database` | B1 | POST | `/bag/v1.0/overwrite?path=np-catalog-probe` | `{"not":` (truncated JSON) | UNKNOWN_LIVE | none | safe |
| NP-96 | `modify_bag_database` | B9 | GET | `/bag/v1.0/get/np/definitely/missing` (no pirate-mode fixture set) | — | **400** | none | safe |

`NP-24` anchor: `core/services/bag_of_holding/main.py:102`. It is Track A candidate **A5**.

`NP-96` is the B9 control: the journey carries a `Software(PirateMode)` precondition, but bag_of_holding has no authorization check, so the response is identical to NP-24 with no fixture at all. The assertion is *"same status as NP-24"* — i.e. pirate mode is a frontend-only gate. That is a product finding to record, not a Fail.

---

## disk_usage (6 probes)

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-26 | `inspect_disk_usage` | B3 | GET | `/disk-usage/v1.0/disk/usage?path=/np/definitely/missing` | — | **404** | none | safe |
| NP-27 | `inspect_disk_usage` | B1 | GET | `/disk-usage/v1.0/disk/usage?path=/usr/blueos/userdata/../../../etc` | — | **404** | none | safe |
| NP-28 | `free_disk_space` | B3 | DELETE | `/disk-usage/v1.0/disk/paths/etc` | — | **400** | none | safe |
| NP-29 | `free_disk_space` | B3 | DELETE | `/disk-usage/v1.0/disk/paths/usr%2Fblueos%2Fuserdata%2Fnp-no-such-file.txt` | — | **404** | none | safe |
| NP-30 | `run_single_disk_speed_test` | B5 | GET | `/disk-usage/v1.0/disk/speed?size_bytes=9007199254740992` | — | **507** | none | safe |
| NP-97 | `inspect_disk_usage` | B1 | GET | `/disk-usage/v1.0/disk/usage?path=/usr/blueos/userdata&depth=-1` | — | UNKNOWN_LIVE | none | safe |

Anchors: 404 at `disk_usage/main.py:97,105`; protected-root 400 at `main.py:278` with the root list at `main.py:109-126`; 507 at `main.py:335-338`; `depth` guard `ge=0` at `main.py:256`.

**NP-27 expects 404, not 400** — deliberately. `FM:disk_usage/invalid_or_missing_path` asserts "404 or 400", but `FILESYSTEM_ROOT = Path("/")` (`main.py:27`) makes the 400 branch at `main.py:100-103` unreachable. If NP-27 ever returns 400, the root changed and the failure-mode card needs revisiting.

**NP-28 is a guard probe: run it before any real delete.** It proves the protected-root refusal fires ahead of `shutil.rmtree`. **NP-30** is fully safe because the size check precedes any file write.

---

## wifi (7 probes)

Radio-free unless marked. The three gold-standard wifi *journeys* are not duplicated here.

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-31 | `forget_saved_wifi_network` | B3 | POST | `/wifi-manager/v1.0/remove?ssid=__np_no_such_ssid__` | — | **400** | none | safe |
| NP-32 | `disconnect_from_wifi_network` | B4 | GET | `/wifi-manager/v1.0/disconnect` (while already idle) | — | **500** | `POST /wifi-manager/v1.0/connect` with the prior association | reversible |
| NP-33 | `connect_to_wifi_network` | B1 | POST | `/wifi-manager/v1.0/connect?hidden=false` | `{}` | UNKNOWN_LIVE | none | safe |
| NP-35 | `toggle_hotspot` | B1 | POST | `/wifi-manager/v1.0/hotspot?enable=banana` | — | UNKNOWN_LIVE | none | safe |
| NP-36 | `configure_hotspot_credentials` | B1 | POST | `/wifi-manager/v1.0/hotspot_credentials` | `{"ssid":"np-probe","password":"short"}` | UNKNOWN_LIVE | `POST /wifi-manager/v1.0/hotspot_credentials` with the snapshot | reversible |
| NP-37 | `toggle_smart_hotspot` | B4 | POST | `/wifi-manager/v1.0/smart_hotspot?enable=banana` | — | UNKNOWN_LIVE | none | safe |
| NP-38 | `connect_to_wifi_network` | B5 | GET | `/wifi-manager/v1.0/scan` ×2 concurrently | — | **425** | none | safe |

Anchors: remove-400 `core/services/wifi/main.py:96-98`; scan-425 `wifi/main.py:71`; disconnect route `wifi/main.py:102` with the idle-500 already asserted by the harness (`catalog/src/runner.rs:1154,1571-1576`).

**NP-38 is the highest-value probe in this group:** `FM:wifi/scan_busy` (425) is currently exercised by no journey, needs no radio association, and works on all four DUTs. Implement it as two overlapping requests, not two sequential ones.

**NP-31** is Track A candidate **A6**. **NP-36** uses a 7-character password on purpose: the question is whether WPA2's 8-character minimum is enforced at the API or only when `hostapd` starts — capture, then restore the real credentials.

**NP-32 is `not-2.2`** even though the restore is declared: on the physical ROV a wifi disconnect plus a failed restore is a stranding risk.

---

## beacon (2 probes)

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-39 | `rename_vehicle` | B1 | POST | `/beacon/v1.0/vehicle_name` (required `name` omitted) | — | UNKNOWN_LIVE | none | safe |
| NP-40 | `change_mdns_hostname` | B1 | POST | `/beacon/v1.0/hostname?hostname=` | — | UNKNOWN_LIVE | `POST /beacon/v1.0/hostname?hostname=blueos` | reversible |

Anchors: `core/services/beacon/main.py:298` (vehicle_name), `:286` (hostname). beacon installs no exception handler, so both are first-capture.

**NP-40 is `not-2.2`.** If the empty hostname is *accepted*, `blueos.local` discovery breaks for the rest of the campaign; the restore is mandatory and is already the allowlist restore (`runner.rs:1087-1098`).

---

## cable_guy (7 probes)

Every probe targets `nonexistent0` or sends a malformed body. **No probe here may ever name a real interface on a DUT** — `FM:cable_guy/network_reconfiguration_lockout` is precisely the stranding the plan forbids.

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-41 | `assign_static_ip_address` | B1 | POST | `/cable-guy/v1.0/address?interface_name=nonexistent0` (required `ip_address` omitted) | — | UNKNOWN_LIVE | none | safe |
| NP-42 | `assign_static_ip_address` | B3 | POST | `/cable-guy/v1.0/address?interface_name=nonexistent0&ip_address=10.99.99.99` | — | UNKNOWN_LIVE | none | safe |
| NP-43 | `acquire_dynamic_ip_address` | B3 | POST | `/cable-guy/v1.0/dynamic_ip?interface_name=nonexistent0` | — | UNKNOWN_LIVE | none | safe |
| NP-44 | `enable_onboard_dhcp_server` | B3 | POST | `/cable-guy/v1.0/dhcp?interface_name=nonexistent0&ipv4_gateway=10.99.99.1&is_backup_server=false` | — | UNKNOWN_LIVE | `DELETE /cable-guy/v1.0/dhcp?interface_name=nonexistent0` | reversible |
| NP-45 | `disable_onboard_dhcp_server` | B3 | DELETE | `/cable-guy/v1.0/dhcp?interface_name=nonexistent0` | — | UNKNOWN_LIVE | none | safe |
| NP-46 | `configure_host_dns` | B1 | POST | `/cable-guy/v1.0/host_dns` | `{"nameservers":"notalist","lock":false}` | UNKNOWN_LIVE | none | safe |
| NP-47 | `set_network_interface_priority` | B1 | POST | `/cable-guy/v1.0/set_interfaces_priority` | `{}` (object where a list is required) | UNKNOWN_LIVE | none | safe |

Anchors: `core/services/cable_guy/main.py:69` (address), `:115` (dynamic_ip), `:99` (dhcp), `:107` (dhcp delete), `:130` (host_dns), `:62` (priority). cable_guy has no error mapping, so all seven are first-capture by construction.

**NP-46 caveat:** `lock:false` is mandatory. A locked `resolv.conf` needs `chattr -i` to undo, and the existing smoke already carries a repair query (`runner.rs:460`) that would mask the result.

---

## helper (3 probes)

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-48 | `probe_interface_internet_connectivity` | B1 | GET | `/helper/v1.0/ping` (required `host` omitted) | — | UNKNOWN_LIVE | none | safe |
| NP-49 | `probe_interface_internet_connectivity` | B3 | GET | `/helper/v1.0/ping?host=np-no-such-host.invalid` | — | **200** (body `false`) | none | safe |
| NP-50 | `monitor_internet_connectivity` | B8 | GET | `/helper/v1.0/check_internet_access` (WAN blocked) | — | **200** (every site `offline`) | restore WAN | safe |

Anchors: `core/services/helper/main.py:583-590` (ping), `:540` (check_internet_access).

**NP-49 and NP-50 are deliberate "200 on a negative input" probes.** The route returns a boolean/map, so the negative *content* is the rejection. Filing either as "missing validation" would be wrong. NP-50 only runs in the B8 wave and its absence must be a typed skip, never a Fail.

---

## kraken (9 probes)

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-51 | `install_extension` | B3 | POST | `/kraken/v2.0/extension/np.no.such.extension/v0.0.0/install` | — | **404** | none | safe |
| NP-52 | `uninstall_extension` | B3 | DELETE | `/kraken/v2.0/extension/np.no.such.extension/v0.0.0` | — | **404** | none | safe |
| NP-53 | `configure_installed_extension` | B3 | POST | `/kraken/v2.0/extension/np.no.such.extension/restart` | — | **404** | none | safe |
| NP-54 | `configure_installed_extension` | B3 | GET | `/kraken/v2.0/container/np_no_such_container/log` | — | **404** | none | safe |
| NP-55 | `edit_extension_dev_version` | B3 | PUT | `/kraken/v2.0/extension/np.no.such.extension/v0.0.0` | — | UNKNOWN_LIVE | none | safe |
| NP-56 | `add_custom_manifest` | B1 | POST | `/kraken/v2.0/manifest/?validate_url=true` | `{"name":"np-probe","url":"http://127.0.0.1:1/np-no-such-manifest.json"}` | UNKNOWN_LIVE (FM says 502) | `DELETE /kraken/v2.0/manifest/{identifier}` if created | reversible |
| NP-57 | `add_custom_manifest` | B4 | POST | `/kraken/v2.0/manifest/` (same manifest twice) | `{"name":"np-probe","url":"https://.../Bluerobotics.json"}` | **409** | `DELETE /kraken/v2.0/manifest/{identifier}` (204) | reversible |
| NP-58 | `browse_extension_store` | B3 | GET | `/kraken/v2.0/manifest/np.no.such.manifest/details` | — | UNKNOWN_LIVE (map says 404) | none | safe |
| NP-59 | `install_custom_extension` | B1 | POST | `/kraken/v2.0/extension/` | `{}` | UNKNOWN_LIVE | `DELETE /kraken/v2.0/extension/{identifier}/{tag}` if anything landed | reversible |

Anchors: the exception map at `core/services/kraken/api/v2/routers/manifest.py:37-43` (502/404/409/500) and `extension.py:33,35,37,39` (404/400/507/400); container 404s at `core/services/kraken/harbor/container.py:120,132`.

**NP-56 and NP-59 must inspect the body, not only the status.** Both routes return a `StreamingResponse`, so an in-band error can arrive after a 200 header — that is exactly why `install_custom_extension` records 200 despite declaring 201 (`extension.py:78`).

**NP-57 needs the existing manifest cleanup** to run after it (`runner.rs:479` already strips the probe manifest out of `settings-2.json`). **NP-58/NP-51** turn into `FM:kraken/manifest_fetch_failure` (502) under B8 — typed skip, not Fail.

---

## versionchooser (7 probes)

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-60 | `switch_local_blueos_version` | B3 | POST | `/version-chooser/v1.0/version/current` | `{"repository":"bluerobotics/blueos-core","tag":"np-no-such-tag"}` | **412** | none (no switch occurs) | safe |
| NP-61 | `delete_local_blueos_version` | B3 | DELETE | `/version-chooser/v1.0/version/delete` | `{"repository":"bluerobotics/blueos-core","tag":"np-no-such-tag"}` | **404** | none | safe |
| NP-62 | `delete_local_blueos_version` | B3 | DELETE | `/version-chooser/v1.0/version/delete` | `{"repository":"bluerobotics/blueos-core","tag":"<currently running tag>"}` | **500** | none (refused) | safe |
| NP-63 | `update_bootstrap_image` | B3 | POST | `/version-chooser/v1.0/bootstrap/current` | `{"tag":"np-no-such-tag"}` | **412** | none | safe |
| NP-64 | `pull_blueos_version_without_switch` | B3 | POST | `/version-chooser/v1.0/version/pull` | `{"repository":"bluerobotics/np-no-such-repo","tag":"np"}` | UNKNOWN_LIVE (500 `chooser.py:91` / 501 `:98`) | `DELETE /version/delete` if anything landed | reversible |
| NP-65 | `docker_registry_login` | B9 | POST | `/version-chooser/v1.0/docker/login` | `{"username":"np-bad","password":"np-bad","registry":"","root":true}` | UNKNOWN_LIVE | snapshot `GET /docker/accounts/`, then re-login or logout | reversible |
| NP-66 | `update_blueos_version` | B3 | GET | `/version-chooser/v1.0/version/available/bluerobotics/np-no-such-image` | — | UNKNOWN_LIVE | none | safe |

Anchors: 412 at `core/services/versionchooser/utils/chooser.py:283-285` (version) and `:221-223` (bootstrap); 404 at `chooser.py:340`; in-use 500 at `chooser.py:331-333`; pull 500/501 at `chooser.py:91,98`.

**NP-60 is Track A candidate A7** — the happy path is destructive, the sad path is completely safe and source-pinned.

**NP-62 is the campaign's own safety net: run it in the first wave.** It proves the "image is in use and cannot be deleted" refusal still fires. If that guard ever regresses, a delete probe could remove the running core image and brick a DUT. Read the running tag from `GET /version-chooser/v1.0/version/current` rather than hardcoding it.

**NP-64 is 177-only** — a pull attempt against a nonexistent repo should land nothing, but pulls touch the image store and stream progress.

---

## recorder_extractor (5 probes)

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-67 | `download_video_recording` | B3 | GET | `/recorder-extractor/v1.0/recorder/files/np-no-such.mp4` | — | **404** | none | safe |
| NP-68 | `download_video_recording` | B1 | GET | `/recorder-extractor/v1.0/recorder/files/np-bad.txt` | — | **400** | none | safe |
| NP-69 | `delete_video_recording` | B1 | DELETE | `/recorder-extractor/v1.0/recorder/files/..%2F..%2Fetc%2Fpasswd` | — | **400** | none | safe |
| NP-70 | `delete_video_recording` | B3 | DELETE | `/recorder-extractor/v1.0/recorder/files/np-no-such.mp4` | — | **404** | none | safe |
| NP-71 | `browse_video_recordings` | B3 | GET | `/recorder-extractor/v1.0/recorder/files/np-no-such.mp4/thumbnail` | — | **404** | none | safe |

Anchors: traversal/suffix 400s at `core/services/recorder_extractor/main.py:70-72,75,78`; 404 at `:81`; delete route at `:395`.

**NP-69 is the second security probe of the suite** and the highest-value one: it proves `resolve_recording` rejects an escaping path *before* `unlink()`. A 204 here means arbitrary file deletion as root — blocker severity, stop the wave. Send the path percent-encoded so nginx does not normalise the traversal away; if it is normalised anyway, record that as a harness limitation rather than a pass.

---

## ardupilot_manager (3 probes)

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-73 | `restore_default_firmware` | B3 | POST | `/ardupilot-manager/v1.0/restore_default_firmware?board_name=NoSuchBoard` | — | **404** | none (autopilot restarts in `finally`) | reversible |
| NP-74 | `upload_custom_firmware` | B1 | POST | `/ardupilot-manager/v1.0/install_firmware_from_file` | multipart `binary=@catalog/fixtures/np-not-firmware.txt` | **415** | none (no flash on reject) | reversible |
| NP-78 | `run_sitl_simulation` | B1 | POST | `/ardupilot-manager/v1.0/sitl_frame?frame=np_no_such_frame` | — | UNKNOWN_LIVE | none | safe |

Anchors: 404 at `core/services/ardupilot_manager/api/v1/routers/index.py:286`; 415 at `index.py:211`; sitl_frame at `index.py:133`.

**NP-73 and NP-74 are `reversible`, not `safe`, and they are 177-first.** Both handlers kill and restart the autopilot in a `finally` block even when the request is rejected, so a live vehicle briefly loses its link. Run them on 177 (SITL) first, then once on 87 for the Pixhawk1 contrast, and never on 2.2 while it matters. NP-73 doubles as the `FM:ardupilot_manager/hardware_device_unavailable` probe: on 87 the legitimate Navigator-targeted call 404s too, which is platform contrast rather than a Fail.

**NP-74 is Track A candidate A8**; **NP-73 is A9**. NP-74 needs the `np-not-firmware.txt` fixture (any small non-ELF text file).

---

## bridget (2 probes)

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-79 | `remove_serial_bridge` | B3 | DELETE | `/bridget/v1.0/bridges` | `{"serial_path":"/dev/ttyNOPE","baud":115200,"ip":"127.0.0.1","udp_target_port":14559,"udp_listen_port":14558}` | UNKNOWN_LIVE | none | safe |
| NP-80 | `create_serial_to_udp_bridge` | B1 | POST | `/bridget/v1.0/bridges` | `{}` | UNKNOWN_LIVE | none | safe |

Anchors: `core/services/bridget/main.py:48` (create), `:56` (delete). No exception handler, so both are first-capture. `/dev/ttyNOPE` guarantees no real serial device is touched.

---

## nmea_injector (3 probes)

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-82 | `remove_configured_nmea_socket` | B3 | DELETE | `/nmea-injector/v1.0/socks` | `{"kind":"UDP","port":9999,"component_id":220}` (not configured) | UNKNOWN_LIVE | none | safe |
| NP-83 | `add_external_nmea_gps_socket` | B1 | POST | `/nmea-injector/v1.0/socks` | `{"kind":"BANANA","port":9999,"component_id":220}` | UNKNOWN_LIVE | none | safe |
| NP-84 | `add_external_nmea_gps_socket` | B5 | POST | `/nmea-injector/v1.0/socks` | `{"kind":"UDP","port":80,"component_id":220}` (port 80 held by nginx) | UNKNOWN_LIVE | `DELETE /nmea-injector/v1.0/socks` with the same body if created | reversible |

Anchors: `core/services/nmea_injector/main.py:48` (create, declares 201), `:59` (delete). `FM:nmea_injector/remove_nonexistent_socket` asserts "returns error" with **no status**, so NP-82 exists specifically to find out what that is and then tighten the service card.

**No probe here ever sends NMEA data.** `FM:nmea_injector/injecting_wrong_position_affects_navigation` is a documented safety limitation, not something to exercise — spoofing position on a real vehicle is out of bounds.

---

## mavlink-camera-manager (4 probes)

MCM is Rust with its own error mapping — do not assume FastAPI codes.

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-85 | `remove_camera_stream` | B3 | DELETE | `/mavlink-camera-manager/delete_stream?name=__np_no_such_stream__` | — | UNKNOWN_LIVE | none | safe |
| NP-86 | `configure_camera_stream` | B1 | POST | `/mavlink-camera-manager/streams` | `{}` | UNKNOWN_LIVE | `DELETE /mavlink-camera-manager/delete_stream?name=<name>` if created | reversible |
| NP-87 | `configure_uvc_device_controls` | B3 | POST | `/mavlink-camera-manager/v4l` | `{"device":"/dev/videoNOPE","v4l_controls":[]}` | UNKNOWN_LIVE | re-apply snapshot controls | reversible |
| NP-100 | `view_camera_streams` | B3 | GET | `/mavlink-camera-manager/v4l` (no camera attached) | — | **200** (empty list) | none | safe |

Route anchors come from the frontend store: `core/frontend/src/store/video.ts:102,120,144,168` and `core/frontend/src/components/video-manager/VideoControlsDialog.vue:189`.

**NP-100 is a Track D correctness check, not a failure probe.** An empty list is the documented no-camera contract (`FM:mavlink-camera-manager/camera_device_unavailable`). If the harness *skips* while a camera is present, that is a harness bug per the plan.

---

## pardal (2 probes)

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-89 | `run_lan_speed_test` | B1 | POST | `/network-test/post_file` | empty body | UNKNOWN_LIVE | none | safe |
| NP-90 | `run_internet_speed_test` | B3 | GET | `/network-test/internet_download_speed` (before any `/internet_best_server`) | — | UNKNOWN_LIVE | none | safe |

Anchors: `core/services/pardal/main.py:149,150` (LAN transfer), `:95,107` (internet). **NP-90 is ordering-sensitive:** `FM:pardal/speedtest_not_initialized` only reproduces while the `SPEED_TEST` global is unset, so it must run before any successful server search in the same service lifetime. Schedule it first within the pardal group, or skip it with a typed reason.

No full-size transfer probe is included: `FM:pardal/large_transfer_resource_pressure` saturates the link (limits at `nginx.conf:11,149,289`), which is unacceptable while 2.2 carries live video.

---

## ping (2 probes)

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-88 | `enable_ping1d_rangefinder_mavlink` | B1 | POST | `/ping/v1.0/sensors` | `{}` | UNKNOWN_LIVE | re-post the snapshot settings | reversible |
| NP-99 | `view_detected_sonar_devices` | B3 | GET | `/ping/v1.0/sensors` (no sonar attached) | — | **200** (empty list) | none | safe |

Anchors: `core/services/ping/main.py:38,46`. **NP-88 is `not-2.2`:** enabling the Ping1D MAVLink driver forwards `DISTANCE_SENSOR` and can skew depth-hold (`FM:ping/wrong_distance_affects_depth_hold`). The empty body should be rejected before anything is enabled, but on the physical ROV we do not rely on "should".

---

## front-door and third-party prefixes (6 probes)

Cheap 404-shape probes that verify nginx routing and that unknown paths do not silently 200. All are first-capture because the responses come from nginx or third-party binaries, not from BlueOS Python.

| id | journey_id | class | method | nginx path | body | expected_status | restore | blast |
|---|---|---|---|---|---|---|---|---|
| NP-91 | `view_system_information` | B1 | GET | `/system-information/np_no_such_endpoint` | — | UNKNOWN_LIVE | none | safe |
| NP-92 | `access_blueos_web_interface` | B1 | GET | `/np-no-such-route` | — | UNKNOWN_LIVE | none | safe |
| NP-93 | `inspect_zenoh_network` | B1 | GET | `/zenoh-api/np_no_such` | — | UNKNOWN_LIVE | none | safe |
| NP-94 | `access_web_terminal` | B1 | GET | `/terminal/np_no_such` | — | UNKNOWN_LIVE | none | safe |
| NP-95 | `manage_blueos_files` | B1 | GET | `/file-browser/api/resources/np-no-such` | — | UNKNOWN_LIVE | none | safe |
| NP-101 | `access_blueos_web_interface` | B1 | POST | `/status` (GET-only route) | — | UNKNOWN_LIVE | none | safe |

Prefixes are all in `core/tools/nginx/nginx.conf` (`/` at `:268`, `/status` at `:62`). These six are the only probes attached to journeys with **zero grounded HTTP routes** — they do not make those journeys automatable, they just give the front door a negative assertion.

---

## Summary

### Probe counts

| | count |
|---|---|
| Total probes | **94** |
| `expected_status` pinned from source / `failure_modes` | 48 |
| `expected_status` = `UNKNOWN_LIVE` (first capture) | 46 |
| blast `safe` | 76 |
| blast `reversible` (restore in-row) | 18 |
| blast `disruptive` / `destructive` | **0** (excluded by design) |

### Pinned status distribution

| status | count | probes |
|---|---|---|
| 400 | 19 | NP-01…08, 12, 13, 15, 16, 17, 18, 24, 28, 68, 69, 96 |
| 404 | 13 | NP-19, 26, 27, 29, 51, 52, 53, 54, 61, 67, 70, 71, 73 |
| 200 | 4 | NP-49, 50, 99, 100 |
| 204 | 3 | NP-20, 21, 22 |
| 412 | 2 | NP-60, 63 |
| 500 | 2 | NP-32, 62 |
| 413 / 415 / 425 / 507 / 409 | 1 each | NP-23, 74, 38, 30, 57 |

### Per-class distribution

| class | probes |
|---|---|
| B1 invalid input | 41 |
| B2 confirm / gate | 8 |
| B3 missing precondition | 31 |
| B4 double-apply | 7 |
| B5 resource / limit | 4 |
| B8 offline | 1 (NP-50) |
| B9 frontend-only gate | 2 (NP-65, NP-96) |
| B6 / B7 | 0 — excluded, see below |

### DUT affinity

- **all four DUTs (including 2.2):** 71 probes. The entire commander gate cluster, all of customization, bag, disk, recorder_extractor, kraken, versionchooser (except NP-64), and the front-door probes.
- **`not-2.2`:** NP-32, NP-40, NP-88, NP-89 — wifi disconnect, hostname change, Ping1D enable, LAN transfer.
- **`177-only`:** NP-44, NP-64, NP-73, NP-74, NP-78.
- **`rf-serial`:** none. Every wifi probe here is deliberately radio-free; real association stays in the existing wifi journeys.

### Suggested wave order

1. **Guard probes first:** NP-62 (in-use image refused), NP-28 (protected root refused), NP-69 + NP-18 (traversal refused). If any of these does *not* refuse, stop and report before running anything mutating.
2. **The gate cluster:** NP-01…NP-08 — 8 pinned 400s, zero state change, all four DUTs. Fastest confidence per minute in the whole plan.
3. **Pinned 4xx/5xx elsewhere:** customization, bag, disk, recorder_extractor, kraken, versionchooser.
4. **`UNKNOWN_LIVE` capture wave:** cable_guy, bridget, nmea_injector, MCM, front-door. Output feeds the `failure_modes` cards, not a pass/fail gate.
5. **Ordering-sensitive and fixture-dependent last:** NP-90 (must precede a successful server search), NP-23 (needs the oversize fixture), NP-20/21/22 (must follow the corresponding mutating-smoke delete).

### Deliberately excluded

| what | why |
|---|---|
| NP-72 (`POST /ardupilot-manager/v1.0/board` with an unknown board) | Board switch restarts the autopilot subprocess and needs a snapshotted board to restore — disruptive. |
| NP-75 (firmware from an unreachable URL) | Attempts a real flash path; 177-only and disruptive. |
| NP-76 / NP-77 (double `POST /start`, double `POST /stop`) | Dropping the autopilot is the B7 disruptor, not a safe probe. Keep them in the ledger as `harness_gap` on 177. |
| All B6 service-down modes (19 in the ledger) | Require tmux stop/start; excluded per the plan. |
| Any cable_guy or wifi call naming a real interface / SSID | Stranding risk (`FM:cable_guy/network_reconfiguration_lockout`, `FM:wifi/wireless_reconfiguration_lockout`). |
| Confirmed `POST /commander/v1.0/shutdown`, `/settings/reset`, `/raspi/eeprom_update` | Destructive. Only the *rejected* form appears here. |
| Full-size pardal transfers, real NMEA injection, sonar depth-hold skew, arbitrary MAVLink send | Link saturation or vehicle-safety limitations; documented in the ledger, never exercised. |

### Implementation notes for `catalog/src/negative_probes.rs`

- A `NegativeProbe` needs, at minimum: `id`, `journey: JourneyId`, `class`, `method`, `path` (already front-door resolved), `body: Option<ProbeBody>` (json / multipart / none), `expected: ExpectedStatus` (`Pinned(u16)` or `UnknownLive`), `restore: Option<RestoreAction>`, `blast`, `dut_affinity`.
- Reuse the existing restore machinery rather than inventing a second one: `mutating_smoke.rs` already models setup/restore actions and `runner.rs:1087-1168` already implements hostname, wifi, and manifest restores.
- `UnknownLive` must **record** the observed status rather than pass or fail on it, so the first `--negative` run is a capture run that produces the numbers to pin.
- `NP-38` (concurrent scan) and `NP-20/21/22` (second consecutive call) are the only probes that need more than one request; everything else is a single request and fits the existing single-step HTTP path.
