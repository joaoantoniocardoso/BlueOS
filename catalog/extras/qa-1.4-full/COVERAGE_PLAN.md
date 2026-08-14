# Coverage plan — 100% of catalog journeys (HTTP + UI)

100% is a **per-journey matrix**, not every Vue click. HTTP 200 does not mean the operator UI was tested. `Actor::Frontend` steps are dropped by `http_steps`.

## Matrix

Every `JourneyId` has two cells:

| Cell | Pass | Typed skip (counts as covered) |
|---|---|---|
| **Backend** | live HTTP matches the `RouteRef` (GET smoke, mutating+restore, or negative probe) | no route, `not_on_<channel>`, hard-exclude, would-strand 2.2 |
| **UI** | Playwright `ui_plan` (clicks + visible result), or page-load if the journey is “open this page” | no operator UI, hardware missing, not on this image |

Dual journeys (rename, bag, extensions, video, parameters) need **both** cells green.

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

Do not invent ~80 JourneyIds for sad paths. Track B probes + page landmarks cover those.

## Current hole (177 `--ui` 2026-08-14)

Green: wizard skip + gyro, baro, level horizon, full accel, onboard compass, motor detect. Navigator restored.

Still open: `--ui` only on 177; compass Dismiss product bug (F-068); Level Horizon presence map vs live image; other frontend journeys; W5; W6 version-switch 412; W4 RF on 87/2.2; Track A new ids; B6 service-down.
