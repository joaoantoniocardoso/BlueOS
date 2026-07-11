# BlueOS 2.0 Catalog — Autonomous Orchestrator Run Book

> **This file is the single source of truth for the autonomous run.** It is written so that a
> *fresh* instance of the orchestrator (after a crash / context loss) can read this file + `git log`
> and resume EXACTLY where the previous instance stopped, without re-asking the user.
> **Update this file (the ledger + decisions log) as the FIRST action after each artifact is accepted
> and committed.** Never let it go stale.

## Mission

Model every BlueOS core service into the `blueos-catalog` Rust crate across its four layers
(**Observed → Journeys → Asserted → Runtime**), each independently QA-gated, then harden the harness.
The catalog is the machine-verifiable single source of truth for the BlueOS 2.0 service rework.

## Hard guardrails (DO NOT GO ROGUE)

1. **Only model services that exist** in `core/start-blueos-core` (the 26 in the ledger). Do not invent services, routes, or capabilities.
2. **Never fabricate a fact.** Every observed fact cites `file:line`; every asserted value has a rationale; every runtime value traces to a live-capture artifact. No evidence → `Unknown{reason}`.
3. **Runtime values come ONLY from the live BlueOS Pi** at `192.168.0.177` (user `pi`, pass `raspberry`, container `blueos-core`, image `bluerobotics/blueos-core:master @ sha256:cdccc744...`). The POC `../microservices_core_prototype` is design-reference only, never a value source. If the Pi is unreachable, mark runtime `Unknown` and move on — do NOT block.
4. **Follow the harness**: dispatch composer-2.5 subagents per role; the orchestrator QAs (independent, fresh subagent) and adjudicates but does not author artifacts it will QA. Every artifact passes `cargo fmt` + `clippy -D warnings` + `test` + `drift` + `Catalog::validate()` before acceptance.
5. **One service per unit of work.** Keep the crate green at every commit. Commit after each service (or each harness fix) with the repo commit-style (`catalog: ...` / path-prefixed, capitalized).
6. **Mutating runtime captures must be reversible** and the vehicle restored to pre-capture state. Do not run destructive ops (firmware bricking, factory reset) without a restore path.
7. **Do not touch git config, force-push, or amend pushed commits.** Never modify files outside this repo except reading `../BlueOS-docs` and the live Pi.
8. **When uncertain, prefer `Unknown{reason}` over a guess.** Truthfulness beats coverage.

## Resume protocol (run this on every fresh start)

1. Read this file top to bottom.
2. Run `git -C . log --oneline -15` and `git status` to see what is committed vs in-flight.
3. Find the **first ledger row / phase step that is not `DONE`** and continue from there.
4. If an artifact is half-written (uncommitted), run the gates; fix or finish it; QA; commit; update ledger.
5. Never restart completed work. Never ask the user — proceed autonomously per the guardrails.

## Harness recipe (condensed — full detail in `blueos-2.0-service-catalog.md`)

Model slug for all subagents: **composer-2.5**. Roles + skills:
- **Fact Extractor** → `.cursor/skills/blueos-service-extraction/SKILL.md` → `observed_facts()` in `catalog/src/services/<id>.rs`
- **Docs Specialist** → `.cursor/skills/blueos-journey-extraction/SKILL.md` → `catalog/src/journeys/<id>.rs` (doc-grounded)
- **Card Author** → `.cursor/skills/blueos-service-card/SKILL.md` → `service_definition()` in `catalog/src/services/<id>.rs`
- **Runtime Specialist** → `.cursor/skills/blueos-runtime-capture/SKILL.md` → `runtime_facts()` + journey outcomes, from a live-Pi capture artifact under `catalog/runtime-captures/<id>__pi4_*.json`
- **QA Reviewer** (fresh, per artifact) → `.cursor/skills/blueos-catalog-validate/SKILL.md` → ACCEPT/BOUNCE

**Per-service order** (keeps crate green; wire mod.rs incrementally):
1. Recon (orchestrator): nginx route, start-blueos-core tuple, service dir, docs section.
2. Fact Extractor → observed → wire `all_observed()` → QA → fix bounces → commit-ready.
3. Docs Specialist → journeys (declare `mod`, defer `all_journeys()` wiring) → QA.
4. Card Author → `service_definition()` → wire `all_service_definitions()` + `all_journeys()` (journeys need service+capabilities to validate) → QA.
5. Runtime Specialist → `runtime_facts()` → wire `all_runtime()` → QA. (Skip/Unknown if Pi unreachable or low value.)
6. Orchestrator: full gates, update ledger, commit.

Templates to copy: `catalog/src/services/ardupilot_manager.rs`, `catalog/src/services/kraken.rs`, `catalog/src/journeys/{ardupilot_manager,kraken}.rs`.
Reusable capture tools: `catalog/runtime-captures/tools/{sample_resource.sh,probe_http.sh}` (params documented in their headers).

## Reusable capture facts
- Pi: `192.168.0.177`, ssh `pi:raspberry`, container `blueos-core`, board Navigator, image sha256:cdccc744...
- Resource sample: `bash tools/sample_resource.sh --match "<id>/main.py" --samples 60 --label <state> --out <file>`
- HTTP SLO probe: `bash tools/probe_http.sh --base http://192.168.0.177/<prefix>/v2.0 --gets "/a /b" --repeats 40 --out <file>`
- Get routes: `curl -s http://192.168.0.177/<prefix>/v{ver}/openapi.json | jq ...`

---

## PHASE CHECKLIST

- [x] **Phase 1 — Finish kraken**: DONE. Captured all 4 gaps live (custom install POST /extension/ 200; add-manifest POST /manifest/ 201 + DELETE 204; tag edit PUT /extension/{id}/{tag} 200; container log GET .../log 200). All 10 kraken journey route-steps now runtime-grounded; no Unknown outcomes remain. Vehicle restored.
- [ ] **Phase 2 — Fix harness**: (2) implement `extract` binary (mechanical observed-layer regen); (3) extend `drift` to diff RuntimeFacts vs asserted; (4) inter-rater rubric freeze (two independent passes on 2-3 services, measure disagreement on authorities/tiers/bounded_context, fix rubric, then freeze).
- [ ] **Phase 3 — Harness eval & polish**: (5) re-evaluate the whole harness for loose ends + how to bulletproof; (6) fix what's needed.
- [ ] **Phase 4 — The hard work**: (7) orchestrate the remaining services one by one (ledger below).

---

## SERVICE LEDGER (Phase 4 + calibration)

Status values: `TODO` / `WIP` / `DONE`. Columns: Observed / Journeys / Card / Runtime. Runtime `n/a`
allowed for services with no meaningful runtime or unreachable. Update after each commit.

| # | Service (tmux) | catalog id | port/prefix | kind | Obs | Jrn | Card | Rt | Committed |
|---|---|---|---|---|---|---|---|---|---|
| 1 | autopilot | ardupilot_manager | 8000 /ardupilot-manager/ | python | DONE | DONE | DONE | DONE | ad5950b93 |
| 2 | kraken | kraken | 9134 /kraken/ | python | DONE | DONE | DONE | DONE | e5ffdaf8a (+Phase1) |
| 3 | cable_guy | cable_guy | ? /cable-guy/ | python | TODO | TODO | TODO | TODO | |
| 4 | video | mavlink-camera-manager | 6020? | binary | TODO | TODO | TODO | TODO | |
| 5 | mavlink2rest | mavlink2rest | 6040 | binary | TODO | TODO | TODO | TODO | |
| 6 | wifi | wifi | ? /wifi-manager/ | python | TODO | TODO | TODO | TODO | |
| 7 | zenohd | zenohd | 7447 | binary | TODO | TODO | TODO | TODO | |
| 8 | beacon | beacon | ? | python | TODO | TODO | TODO | TODO | |
| 9 | bridget | bridget | ? /bridget/ | python | TODO | TODO | TODO | TODO | |
| 10 | commander | commander | ? /commander/ | python | TODO | TODO | TODO | TODO | |
| 11 | nmea_injector | nmea_injector | ? /nmea-injector/ | python | TODO | TODO | TODO | TODO | |
| 12 | helper | helper | ? /helper/ | python | TODO | TODO | TODO | TODO | |
| 13 | iperf3 | iperf3 | 5201 | binary | TODO | TODO | TODO | TODO | |
| 14 | linux2rest | linux2rest | ? | binary | TODO | TODO | TODO | TODO | |
| 15 | filebrowser | filebrowser | ? /file-browser | binary | TODO | TODO | TODO | TODO | |
| 16 | versionchooser | versionchooser | ? /version-chooser/ | python | TODO | TODO | TODO | TODO | |
| 17 | pardal | pardal | 9120? /pardal/ | python | TODO | TODO | TODO | TODO | |
| 18 | ping | ping | ? /ping/ | python | TODO | TODO | TODO | TODO | |
| 19 | user_terminal | user_terminal | - | shell(motd) | TODO | TODO | TODO | n/a | trivial: cat /etc/motd |
| 20 | ttyd | ttyd | 8088 /terminal/ | binary | TODO | TODO | TODO | TODO | |
| 21 | nginx | nginx | 80 | binary | TODO | TODO | TODO | TODO | reverse proxy (authority) |
| 22 | bag_of_holding | bag_of_holding | ? /bag/ | python | TODO | TODO | TODO | TODO | |
| 23 | recorder | recorder | ? | binary | TODO | TODO | TODO | TODO | blueos-recorder |
| 24 | recorder_extractor | recorder_extractor | ? | python | TODO | TODO | TODO | TODO | |
| 25 | disk_usage | disk_usage | ? /disk-usage/ | python | TODO | TODO | TODO | TODO | reference PR #3669 |
| 26 | customization | customization | ? /bootstrap? | python | TODO | TODO | TODO | TODO | |

> Ports/prefixes marked `?` must be confirmed from `nginx.conf` + argparse during that service's recon.
> Suggested order: cheap python services with clear nginx routes first (disk_usage, helper, commander,
> beacon, cable_guy, wifi, versionchooser, bag_of_holding, customization, nmea_injector, pardal, ping,
> bridget, recorder_extractor), then binaries (mavlink2rest, linux2rest, video, zenohd, nginx, ttyd,
> iperf3, filebrowser, recorder), then user_terminal (trivial).

---

## DECISIONS LOG (append-only; newest last)

- 2026-07-11: Kraken committed `e5ffdaf8a`. Runtime captured Tier 1+2 with Example 1; vehicle restored. Fixed `probe_http.sh` (ARG_MAX + SIGPIPE). 4 journey outcomes left Unknown → **Phase 1 target**.
- 2026-07-11: `SloBaseline` is latency-only; `ResourceUsage` holds cpu/mem Distributions; kraken has NO state machine (state_contracts Unknown is correct).
- 2026-07-11: Journey validation requires participating service + capabilities to exist → wire `all_journeys()` together with the Card Author step, not before.
- 2026-07-11: Phase 1 done. Closed kraken's 4 gaps with a reversible Tier-2 capture of Example 1: custom install (POST /extension/ 200), add/del manifest (POST 201 / DELETE 204), tag edit v1.0.0→v1.0.1 (PUT 200), container log (GET 200). Artifact `transitions` extended; 4 journey outcomes re-grounded. Values orchestrator-captured + verified against artifact; gates green.

## OPEN HARNESS GAPS (Phase 2/3 backlog)

- `extract` binary is a stub (`eprintln!("extract: not yet implemented")`). Should regenerate the observed layer mechanically from start-blueos-core + nginx.conf + zenoh config.
- `drift` only diffs asserted↔observed; does NOT yet diff RuntimeFacts↔asserted (e.g. captured 500 vs `api_stable:true`).
- Coverage gate threshold is `10_000` (effectively disabled) — calibrate after rubric freeze.
- Inter-rater reliability not yet measured; rubric not frozen.
- `Interface::Settings` carries only `path` (no root-shape field) — settings root shape currently lives in prose/rationale.
