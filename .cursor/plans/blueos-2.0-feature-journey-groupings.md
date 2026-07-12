# BlueOS 2.0 — Feature & Journey Grouping: Multiple Approaches + Consensus

**Branch:** `2.0-dev/model`
**Status:** M4 input (human decision). **Generated from the catalog; these are candidate signals, not boundary decisions.**
**Regenerate:** `cargo run --quiet --bin groupings` (deterministic).

The service split (`blueos-2.0-split-approaches.md`) asked *"which services belong together?"*. This report asks the same question one level down, over the two units a feature-first architecture actually reasons about:

- **Features** — the 143 capabilities (129 backend + 14 frontend-origin), independent of which service hosts them today.
- **Journeys** — the 94 user workflows.

Each unit is grouped through **four independent lenses**, then reconciled by a **cross-lens consensus**: a pair is "consensus-joined" only when a **majority (≥ 3 of 4)** of lenses place it together. Boundaries that survive multiple, differently-motivated lenses are the defensible ones.

---

## Part 1 — Feature grouping (143 features)

| Lens | Kind | "Together" means… |
|---|---|---|
| `aggregate` | label | act on the **same entity** (View A) |
| `origin` | label | hosted by the **same service/page today** |
| `journey_cooccurrence` | clustering (Q≈0.307) | **used together in a workflow** (View B) |
| `page_reachability` | clustering (Q≈0.109) | **surfaced together through one frontend page** |

`aggregate` and `origin` are deterministic labels. `journey_cooccurrence` builds a feature×feature affinity from journeys' `capability_refs` and clusters by modularity. `page_reachability` links features reachable through the same page (its `frontend_features` ∪ the capabilities of every service it `consumes`) — a *coarse* lens (page consumption is at service granularity), so its Q is deliberately low and it mostly reinforces rather than leads.

**Modularity note:** unlike the *service* co-occurrence lenses (which were negative — a hub-and-spoke around the MAVLink plane), feature `journey_cooccurrence` is **positive (Q≈0.31)**. Features referenced together in a workflow are genuinely cohesive; the hub effect lives in services, not capabilities.

### Feature consensus — 23 multi-feature clusters, 44 singletons

Clusters where **all four lenses agree (4/4)** are the strongest re-home units. Highlights:

```
autopilot core (all 4/4):
  manage_autopilot_lifecycle, flash_firmware, select_flight_controller_board,
  detect_flight_controllers, query_vehicle_firmware_info, configure_sitl_frame

extensions (4/4):
  browse_extension_store, configure_extension, install_extension,
  manage_extension_lifecycle, manage_manifests, uninstall_extension

disk usage:
  delete_disk_paths, inspect_disk_usage, navigate_disk_usage,
  run_disk_speed_test, run_multi_size_disk_speed_test
```

Other consensus clusters (≥ 3/4): branding+theme (12), BlueOS version management (9), host/system ops (`configure_legacy_camera, inspect_raspberry_eeprom, reboot_onboard_computer, run_host_command, setup_ssh, shutdown_onboard_computer, sync_system_time, update_raspberry_eeprom`), **sensor calibration** (`calibrate_{accelerometer,barometer,compass,gyroscope}, derive_sensor_calibration_status, detect_motor_directions, level_horizon` — 7), video streams (two clusters: live-view vs. stream-config), mDNS/discovery, bag store, video recordings, ping/sonar, NMEA sockets, serial bridges, MAVLink (rest/inspect and endpoints/router), and small dyads (parameters, internet-connectivity, docker accounts, zenoh/pubsub, system-information).

**What the calibration cluster tells us:** the seven calibration features hold together across *every* lens, yet today they are split between `ardupilot_manager` (backend) and the Vehicle Setup **frontend page** (`derive_sensor_calibration_status` and the calibration wizards are frontend-origin). That is the clearest re-home signal in the set — a candidate "Calibration" capability owner (see `blueos-2.0-rehome-candidates.md`).

---

## Part 2 — Journey grouping (94 journeys)

| Lens | Kind | "Together" means… |
|---|---|---|
| `shared_service` | clustering (Q≈0.247) | journeys **touch the same services** |
| `shared_capability` | clustering (Q≈0.135) | journeys **reference the same capabilities** |
| `chain` | structural | linked by `chains_from` (one is a step in the other) |
| `dominant_aggregate` | label | the **entity a journey mostly acts on** |

`shared_capability` is intentionally sparse (85 groups): most journeys reference distinct capability sets, so it rarely *reinforces* a pair — which is why journey consensus is conservative and leans on `shared_service`, `chain`, and `dominant_aggregate`.

### Journey consensus — 7 multi-journey clusters, 65 singletons

The pairs every lens agrees on (4/4) are the workflow "spines":

```
4/4  change_board            ↔ run_sitl_simulation
4/4  update_firmware_online  ↔ upload_custom_firmware ↔ restore_default_firmware
4/4  inspect_disk_usage      ↔ free_disk_space
4/4  configure_installed_extension ↔ edit_extension_dev_version
```

Consensus clusters (≥ 3/4):

```
autopilot lifecycle + firmware (8):
  change_board, restart_autopilot, restore_default_firmware, run_sitl_simulation,
  start_autopilot, stop_autopilot, update_firmware_online, upload_custom_firmware

extensions (7):
  add_custom_manifest, browse_extension_store, configure_installed_extension,
  edit_extension_dev_version, install_custom_extension, install_extension, uninstall_extension

camera streams (4): configure_camera_stream, configure_uvc_device_controls,
                    remove_camera_stream, view_camera_streams
version management (4): delete_local_blueos_version, switch_local_blueos_version,
                        update_blueos_version, update_bootstrap_image
disk (2):     free_disk_space, inspect_disk_usage
eeprom (2):   inspect_raspberry_eeprom_bootloader, update_raspberry_eeprom_bootloader
internet (2): monitor_internet_connectivity, verify_internet_connectivity
```

---

## Cross-validation between the two views

The 7 journey consensus clusters map **cleanly** onto the strongest feature consensus clusters — autopilot/firmware, extensions, camera streams, version management, disk, eeprom, internet. When the workflow view and the capability view independently draw the same boundary, that boundary is real, not an artifact of one metric.

## Caveats (read before using)

- **These are inputs, not decisions.** M4 owns the boundaries; the algorithms only surface candidates.
- `page_reachability` is coarse (page→service, not page→capability); treat it as a reinforcing signal, never a leading one.
- `shared_capability` is sparse; its silence on a pair is not evidence of separation.
- Determinism: same catalog ⇒ byte-identical output. Re-run after any feature/journey change.
