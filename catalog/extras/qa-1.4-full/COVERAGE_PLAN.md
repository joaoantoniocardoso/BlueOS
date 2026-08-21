# Coverage plan — 100% of catalog journeys (HTTP + UI)

100% is a **per-journey matrix**, not every Vue click. HTTP 200 does not mean the operator UI was tested. `Actor::Frontend` steps are dropped by `http_steps`.

**Pin:** `bluerobotics/blueos-core:1.4-dev @ sha256:5b50dfafc3114651d459993c0c65639d7205634613fb62e0dea5ee04de8eebb1`

## Matrix

Every `JourneyId` has two cells:

| Cell | Pass | Typed skip (counts as covered) |
|---|---|---|
| **Backend** | live HTTP matches the `RouteRef` (GET smoke, mutating+restore, or negative probe) | no route, `not_on_<channel>`, hard-exclude, would-strand 2.2 |
| **UI** | Playwright `ui_plan` (clicks + visible result), or page-load if the journey is “open this page” | no operator UI, hardware missing, not on this image |

Dual journeys (rename, bag, extensions, video, parameters) need **both** cells green or typed skip.

Oracle: `journey_matrix` (`catalog/src/journey_matrix.rs`). Two independent booleans per id — `has_known_route` (any step, Frontend included) and `has_frontend_step` / `ui_plan` / page-load allowlist. `derive_automatable()` is a **hint only**: it returns a single class and would skip a real `RouteRef` on Hardware/Frontend journeys (`create_serial_to_udp_bridge`). Do **not** add a UI field to `JourneyStep`.

Every cell is `{state, dut, report_path, utc}`. States: `empty`, `planned` (plan exists, never ran), typed `skip`, `pass` / `pass_with_finding` / `fail` from `--merge-report`. Planned is not Pass.

A unit test / `cargo run -q --bin journey_matrix -- --merge-report extras/qa-1.4-full/reports` fails if any present journey has both cells empty except `Deploy` and `shutdown_onboard_computer`.

Page-load may satisfy the UI cell **only** for `view_configured_serial_bridges`, `browse_available_web_services`, `view_system_information`. Mutating journeys need `ui_plan`. `calibrate_compass` UI pass is `pass_with_finding(F-068)`.

## Layers

1. **Coverage oracle** — `journey_matrix` emits `{backend, ui}` per id; merge live `reports/**/*.json`.
2. **Backend** — fill `--smoke` / `--mutating-smoke` / `--negative` holes; provision fixtures instead of skip; W5/W6 disruptive with restore. EEPROM/firmware/settings-reset stay skip-with-reason unless explicitly in scope.
3. **Page-load** — expand `frontend_smoke` from 3 pages to all 24 `PageId`s (SPA: `goto /` then `router.push`).
4. **UI journeys** — grow `ui_plan()`; Playwright stays a dumb interpreter. Vehicle Setup leftovers (quick accel, large-vehicle mag, Compass Learn) as plans or page landmarks, not new ids without a doc/source anchor. Then parameters, video (DUT 87 camera), terminal, MAVLink inspector, file browser, bag, extensions, version chooser, records, zenoh, ping, bridges, NMEA.
5. **DUT affinity** — SITL/`--ui` arming: 177 only. Never SITL/arm on 2.2. 124 Navigator replicate. 87 Pixhawk + USB camera.

## What 100% may claim

| Claim | Bar |
|---|---|
| 100% of **1.4-dev catalog journeys** | every present id has http and/or ui green, or a typed skip |
| 100% of **catalog on a later image** | re-run after presence flips (21 ids are 1.4.4+/1.5.0+) |
| 100% of **Vue components** | out of scope |
| **Product-green release** | all `fail` cells resolved or accepted; presence contradictions reconciled — **not met** |

Do not invent ~80 JourneyIds for sad paths. Track B probes + page landmarks cover those.

## Current hole (2026-08-14 closeout)

**Coverage oracle: none.** Merged matrix:

```
Journeys: 100 (present on 1.4-dev: 79)
Backend measured pass: 39   UI measured pass: 37
UI empty (needs plan): 0   presence contradictions: 1
Both cells empty (gate): 0
Backend planned: 0
```

**Green (G0–G8):** typed UI skips (G1); ui_plans for bag/rename/extensions/NMEA/bridges/version chooser + cable_guy internet cluster (G2/G8); backend planned filled (G3); calibration `--ui` (F-066); page-load 19/24 + pirate/scoped landmarks (F-070); no-hardware `--ui` 177/124 (F-071/F-074); camera `--ui` 87 5/5 (F-073); W5 (F-072); `install_custom_extension` PASS (`ClickSelectorIfVisible`); wifi 177 UI fixture skips; `disable_onboard_dhcp_server` UI skip (dhcp inactive); `switch_local_blueos_version` UI skip (no local-version fixture).

**Product fails (not coverage holes):**

| id | cell | journey |
|---|---|---|
| F-063 | backend fail @177 | `switch_local_blueos_version` (412; harness `:master` tag) |
| F-075 | backend fail @87 | `connect_to_wifi_network` |
| F-077 | backend pass + B6 finding | `view_configured_serial_bridges` — 200 `[]` when linux2rest down |
| F-069 | presence contradiction | `level_horizon` — do not hand-edit `journey_presence.rs` |
| F-068 | UI pass_with_finding | `calibrate_compass` |

**Harness landed (not product):** F-078–F-084 cable_guy `--ui` navigation fixes.

**Still out of scope:** Track A new ids (G6 none promoted); extra B6; `--ui` on 2.2; shutdown; 21 `not_on_1.4-dev` ids until image bump.

`smoke-catalog` vehicle_name is W3 restore, not a hostname bug.
