# BlueOS 2.0 — Frontend Re-home Candidates

**Branch:** `2.0-dev/model`
**Status:** M4 input (human decision). Generated from the catalog; **proposals are not decisions.**
**Audience:** Architecture-decision workshop (M4).

---

## Purpose

BlueOS 1.x implements real capability and whole workflows **in the browser**, invisible to
a backend-only view. The F2 frontend layer surfaced two concrete things a 2.0 redesign
should consider lifting server-side so *any* client (or API consumer) can use/resume them:

1. **Frontend features** — 14 capabilities implemented client-side with no backend owner.
2. **Category-B FrontendOwned state** — multi-step operations held as browser-side state
   machines (lost on tab close; no second client can observe or drive them).

This report lists both, per page, and **proposes** the backend service that would most
naturally own each. It is an *input* to the human workshop, not an architecture decision.

### How to read this (facts vs proposals)

| Column | Source | Trust |
|--------|--------|-------|
| Page, Feature/State, Aggregate | catalog (`export` / `pages/*.rs` / `FRONTEND_CAPABILITIES`) | **fact** (provenance-backed) |
| Already sends to | page `consumes` edges (catalog) | **fact** |
| **Proposed 2.0 owner** | judgment (aggregate + existing edge) | **proposal — decide in M4** |

Each proposed owner is grounded two ways: the capability's `Aggregate`, and the fact that
the page **already** calls that service (the data already flows there). Regenerate the
factual columns any time with `cargo run --bin export` (+ `--bin features`).

---

## Part 1 — Frontend features → propose a backend owner

All 14 are in `catalog/src/capability.rs::FRONTEND_CAPABILITIES`. If re-homed, the page
becomes a thin client of a new/extended backend capability.

### Autopilot aggregate — owning page `vehicle_setup` / `parameter_editor`

Already sends to: `ardupilot_manager`, `mavlink2rest`. **Proposed owner: `ardupilot_manager`**
(it already owns firmware/lifecycle/endpoint capabilities and the MAVLink router; calibration
and parameter application are autopilot operations driven over MAVLink).

| # | Feature | Why it's client-side today (1.x) |
|---|---------|----------------------------------|
| 1 | `calibrate_accelerometer` | quick + position-wizard accel flows orchestrated in Vue |
| 2 | `calibrate_barometer` | ground-pressure calibration triggered/tracked client-side |
| 3 | `calibrate_compass` | compass/large-vehicle calibration sequence in the browser |
| 4 | `calibrate_gyroscope` | gyro calibration flow in the browser |
| 5 | `derive_sensor_calibration_status` | "is it calibrated?" computed from params client-side |
| 6 | `detect_motor_directions` | motor-direction detection sequence in the browser |
| 7 | `level_horizon` | horizon-leveling flow in the browser |
| 8 | `edit_autopilot_parameters` | param edit surface (also on `vehicle_setup`) |
| 9 | `apply_parameter_set` | batch param apply orchestrated as repeated MAVLink writes |

### Camera aggregate — owning page `video_manager`

Already sends to: `mavlink-camera-manager`, `commander`. **Proposed owner: `mavlink-camera-manager`**
(it owns camera streams; these are stream-management operations).

| # | Feature | Why it's client-side today (1.x) |
|---|---------|----------------------------------|
| 10 | `configure_stream_endpoints` | RTSP/UDP endpoint editing assembled client-side |
| 11 | `diagnose_stream_accessibility` | reachability check computed in the browser |
| 12 | `filter_displayable_devices` | device-list filtering logic in the browser |
| 13 | `manage_thumbnail_preview` | thumbnail fetch/cache/preview handled client-side |
| 14 | `replace_stream_configuration` | delete+recreate orchestration in the browser |

---

## Part 2 — Category-B FrontendOwned state → propose a backend owner

These are **workflow/operation** states held only in the browser (a subset of the 72
`FrontendOwned` entries; the rest are benign UI chrome — see Part 3). Lost on refresh;
no other client can observe or resume them. Re-homing means the backend owns the
operation's progress/lifecycle and the page merely subscribes.

| Page | FrontendOwned workflow state | Already sends to | Proposed 2.0 owner |
|------|------------------------------|------------------|--------------------|
| `autopilot` | firmware install wizard state | `ardupilot_manager` | `ardupilot_manager` |
| `vehicle_setup` | calibration progress; per-component calibration status text; accelerometer/compass calibration wizard state; motor-detection sequence; PWM motor-test targets; selected parameter set | `ardupilot_manager`, `mavlink2rest` | `ardupilot_manager` |
| `parameter_editor` | parsed parameter-file draft; batch-load selection | `ardupilot_manager`, `mavlink2rest` | `ardupilot_manager` |
| `version_chooser` | pull/upload progress; backend restart-wait flag | `versionchooser` | `versionchooser` |
| `extension_manager` | docker-pull progress; active install operation; tar sideload wizard | `kraken` | `kraken` |
| `network_test` | LAN test state machine; LAN upload buffer | `pardal` | `pardal` |
| `records` | processing-status poller | `recorder_extractor` | `recorder_extractor` |
| `settings` | log-deletion streaming progress; pending log-clear type | `commander` | `commander` |

**Pattern:** every proposed owner is a service the page **already** talks to — re-homing is
mostly "the operation the client currently drives step-by-step becomes a backend job the
client observes." The calibration/firmware/param cluster (all → `ardupilot_manager`) is the
densest and highest-value target; it overlaps 1:1 with the Part-1 features.

---

## Part 3 — Explicitly NOT re-home targets (stay client-side)

The other ~50 `FrontendOwned` entries are UI chrome and belong in the client: dialog
visibility, active tab, iframe source/overlay, sort/filter preferences, viewport dimensions,
dashboard widget grid, expand states, search boxes. Listed here only so the workshop does
not mistake them for domain state. (Full list: `export` → `pages[].client_state`.)

---

## Caveats

- **Proposals, not decisions.** Owner assignments are judgment; M4 decides (and may split a
  capability across a new service, e.g. a dedicated calibration/commissioning service).
- **The A/B state classification is asserted** (rationale-backed, not compiler-checked).
  Spend human review on Part 2 before acting.
- **Shared state (69 entries)** is a secondary concern: cached-from-backend but derived
  client-side. Not listed here; revisit if a boundary decision depends on where the
  derivation should live.
