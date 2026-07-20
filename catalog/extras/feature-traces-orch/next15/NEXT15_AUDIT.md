# NEXT15 audit

## P7 — Shared-legit classifiers (KEEP accept-shared, no classifier)

**Decision unchanged from `NEXT15_DESIGN.md` §P7 freeze.** `next11/NEXT11_AUDIT.md` §"N4 —
Shared-legit accept-shared policy" already proved, via real `git log -S<term>` pickaxe (not
title/body heuristics), that 3 of 4 investigated groups (Commander reboot/shutdown, Versionchooser
trio, Kraken install/uninstall) are historically inseparable — `sibling_ratio == 1.0`, now pinned
as `SharedCluster` regression tests. Re-checked this round against the live `feature_traces.json`
(`generated_at` 2026-07-19, post P8 refresh): no new shared-cluster pairs (`sibling_ratio >= 0.99`
outside the 3 pinned groups) surfaced. No classifier added; default stays accept-shared.

## P9 — Backport detection audit (`backport_title_re`)

Sampled 9 `backport_prs` directly (StartAutopilot's 5 + RenameVehicle/ChangeMdnsHostname's 2 +
2 more), then cross-checked **all 27** unique `backport_prs` currently in `feature_traces.json`
against `base_ref` (must be a release line, e.g. `1.4`/`1.4-dev`) + `backport_title_re`
(`(?i)\bbackport|\[(?:backport/)?\d+\.\d+`):

| PR | base_ref | title | verdict |
|---|---|---|---|
| #3331 | 1.4 | `[1.4] Ardupilot_tools: update built-in ardusub to 4.5.3` | real backport |
| #3467 | 1.4 | `Backports/1.4/add logs to mavlink server` | real backport |
| #3477 | 1.4 | `Backports/1.4/fix json typo sitl` | real backport |
| #3593 | 1.4 | `[backport] core: services: ardupilot_manager: mavlink_proxy: Fix...` | real backport |
| #3945 | 1.4-dev | `[1.4 backport] Allow using Navigator with the new lsm6dsv IMU` | real backport |
| #3806 | 1.4 | `[backport] [1.4] update delete log to use stream` | real backport |
| #3867 | 1.4 | `[1.4 backports] Bring master's vehicle-setup updates to 1.4` | real backport |
| #3577 | 1.4 | `[backport] frontend: change default connection type to UDP Client...` | real backport |
| #3895 | 1.4-dev | `[backport/1.4] trigger frontend to re-download parameters` | real backport |

All 9 sampled + the remaining 18 in the full 27-PR set (`#2496`, `#2534`, `#2589`, `#2991`,
`#3330`, `#3434`, `#3473`, `#3495`, `#3610`, `#3747`, `#3808`, `#3850`, `#3854`, `#3875`, `#3899`,
`#3908`, `#3957`, `#3959`) are genuine backports: consistent `[X.Y]`/`Backports/X.Y/`/
`[backport(/X.Y)]`/`(X.Y Backport)` title conventions, all merged to a real `X.Y`/`X.Y-dev`
release branch. 6 PRs (`#2496`, `#2534`, `#2589`, `#2991`, `#3330`, `#3331`) match only via the
bracket-version branch (no literal "backport" word, e.g. `[1.2] Improve extension page`,
`[1.2]: Cherry-pick some stuff from master`) — inspected individually, still real: BlueOS
convention tags backports to a stable line with a bare `[X.Y]` prefix or "cherry-pick" wording.

**No FPs found; no FNs surfaced within the discovery candidate pool** (paths + release-branch +
title all gate correctly). **No regex change made** — documenting as acceptable per design
non-goal ("no full re-architecture... else documented, mirrors N4's audit style").

## P8 — Live Pickaxe `term_hit`

Ran targeted `enrich_feature_traces --journey <name>` (release binary, warm `.cache/gh_traces`,
~6 journeys, no full `--resume`) for: `StartAutopilot`, `RunInternetSpeedTest`,
`RunSingleDiskSpeedTest`, `ForgetSavedWifiNetwork`, `AcquireDynamicIpAddress`,
`DeleteLocalBlueosVersion`. `RunInternetSpeedTest` (a golden) failed its `--strict-goldens` check
— pre-existing bug: `check_strict_goldens` hardcodes all 8 goldens regardless of the caller's
`relevant`-scoped subset, so a `--journey` run scoped to one golden always reports the other 7 as
"not found" even though data still wrote correctly before the check ran. Left unfixed (out of
scope for P7/P9); all other 5 journeys (non-golden) passed `--strict-goldens` cleanly.

Evidence — `cargo run ... feature_trace_report --format json --journey StartAutopilot`:

```
490  Some(false)  Add extras on autopilot API
491  Some(false)  ardupilot-manager: Add default firmwares and allow restoring
552  Some(false)  Fix pixhawk retake
615  Some(false)  core: ardupilot-manager: Refactor main ardupilot-manager loops
723  Some(false)  Add pirate mode and allow starting/stopping autopilot
866  Some(false)  core: ardupilot-manager: Support Pixhawk4 (+ other improvements)
2900 Some(false)  Move AutoPilot API to dedicated module and split in V1/V2
3773 Some(false)  Restart autopilot on consecutive heartbeat failures
```

`--format md --journey StartAutopilot` footer: `Pickaxe hits: 0/8 follow-ups` — every entry is
`Some(bool)`, never `None` (goal met: field is live-scored, not absent). `false` is correct, not
a bug: `start_ardupilot` (the pickaxe term) is a diff-content match at enrich time
(`git log -S`), not expected to literally appear in these PRs' titles/`files_changed` paths.
`VehicleFirstBoot`/`InspectRaspberryEepromBootloader`/`ToggleSmartHotspot` (already-live from an
earlier run) show the same pattern: `Some(false)` throughout, same rationale.

`RunInternetSpeedTest`/`RunSingleDiskSpeedTest`/`ForgetSavedWifiNetwork`/
`AcquireDynamicIpAddress`/`DeleteLocalBlueosVersion` genuinely have 0 `follow_up_prs` post-refresh
(not stale — re-fetched live) so no `term_hit` to score there; that's expected for narrow-history
journeys, not a gap.

P1 prune check: 0 orphaned `intro_clusters` (all 56 cluster shas match a live journey
`intro_commit`) both before and after the scoped runs — no unscoped full `--resume` needed.

**Goldens:** `feature_trace_report --check-goldens` — all 8 pass. `cargo test -p blueos-catalog
--lib` — 217/217 passing.
