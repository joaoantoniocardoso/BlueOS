# QA 1.4-dev coverage closeout — DONE

Pin: `bluerobotics/blueos-core:1.4-dev` @ `sha256:5b50dfafc3114651d459993c0c65639d7205634613fb62e0dea5ee04de8eebb1`

## Phases ACCEPTed

- G0 inventory + taxonomy — ACCEPT
- G1 typed UI skips — ACCEPT
- G2 leftover ui_plans — ACCEPT (with G8 loop-backs)
- G3 backend planned — ACCEPT (planned=0)
- G4 backend fails — ACCEPT (F-063, F-075 KEEP)
- G5 B6 177 — ACCEPT
- G6 Track A — ACCEPT skip-with-reason (0 new JourneyIds)
- G7 RELEASE_READINESS.md + COVERAGE_PLAN.md — ACCEPT
- G8 live `--ui` — ACCEPT (every new ui_plan is pass, pass_with_finding, or typed skip)

## G6 / G8 status

- G6: no Track A promotions.
- G8: cable_guy internet cluster PASS; wifi 177 fixture skips; extension/version landmarks PASS; `install_custom_extension` PASS (`ClickSelectorIfVisible` for 1.4-dev beta.15 plus FAB vs later speed-dial). Never `--ui` on 192.168.2.2. No extra B6.

## Oracle

`cargo run -q --offline --bin journey_matrix -- --merge-report extras/qa-1.4-full/reports`

- present on 1.4-dev: 79
- UI empty: 0
- backend planned: 0
- backend fail + finding: F-063 `switch_local_blueos_version` @177; F-075 `connect_to_wifi_network` @87
- presence contradiction: `level_horizon` F-069 (do not hand-edit `journey_presence.rs`)

## Remaining typed skips (non-exhaustive)

- wifi/hotspot: saved/known/active/hotspot-capability fixtures unmet on 177
- `disable_onboard_dhcp_server` (dhcp-active unmet)
- `remove_configured_nmea_socket` (NmeaSocketConfigured unmet)
- extension installed cluster (`configure_installed_extension`, `uninstall_extension`, `edit_extension_dev_version`)
- `switch_local_blueos_version` UI (no local-version fixture); backend KEEP F-063
- ping / no_sonar Track D (ABSENT)

## Remaining product findings (not harness)

F-063, F-075, F-069, F-077, F-068 (`calibrate_compass` pass_with_finding). F-073/F-074 are pass notes. Cable-guy F-078–F-084 are harness, landed.

Not product-green. Coverage matrix cells are filled.

next_steps: STOP
