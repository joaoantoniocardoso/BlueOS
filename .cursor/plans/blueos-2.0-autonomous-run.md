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
- [x] **Phase 2 — Fix harness**: DONE. (2) `extract` binary — parses `core/start-blueos-core`, emits `ExtractedService` (`--json`), and self-checks the observed layer vs source (test `bootstrap_observed_matches_source`). (3) `drift` — real asserted↔observed (resource-evidence) + asserted↔runtime (state-machine/state existence); `diff_catalog_runtime` wired into the bin. (4) inter-rater rubric FROZEN v1.0 in `blueos-service-card/SKILL.md` — calibration 13/14 (93%) hard-field agreement; codified 4 decision rules (user_confirmation⇐dangerous_ops non-empty; dangerous_ops = irreversible/untrusted-code/unsafe-state only; authorities = exclusive rights others depend on; bounded_context = semantic). No catalog data changes needed — committed cards already comply.
- [x] **Phase 3 — Harness eval & polish**: DONE. Audited validator/skills/rules/bins. Fixes: added `catalog/gate.sh` (one-shot fmt+clippy+test+extract+drift+export, fail-fast) as the canonical per-slice/resume gate; QA skill now runs `extract` and enforces FROZEN RUBRIC v1.0 on judgment fields. Verified: no stale type/IP refs; validator covers authority uniqueness, edge/journey/runtime cross-refs, and state existence. See "PHASE 3 AUDIT" section for loose ends + deliberate deferrals.
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
| 8 | beacon | beacon | 9111 /beacon/ | python | DONE | DONE | DONE | DONE | FULLY MODELED; mDNS = harness gap (no Interface variant) |
| 9 | cable_guy | cable_guy | 9090 /cable-guy/ | python | DONE | DONE | DONE | DONE | FULLY MODELED; netlink/D-Bus mutation = harness gap |
| 10 | wifi | wifi | 9000 /wifi-manager/ | python | DONE | DONE | DONE | DONE | FULLY MODELED; wpa ctrl-socket/D-Bus = harness gap |
| 11 | versionchooser | versionchooser | 8081 /version-chooser/ | python | DONE | DONE | DONE | DONE | FULLY MODELED; core-image updater; dangerous ops = Upgrade/delete/pull |
| 12 | bag_of_holding | bag_of_holding | 9101 /bag/ | python | DONE | DONE | DONE | DONE | FULLY MODELED; JSON store; 1 journey (Bag Editor), rest is infra |
| 13 | customization | customization | 9152 /customization/ | python | DONE | DONE | DONE | DONE | FULLY MODELED; Auxiliary branding; no dangerous ops |
| 14 | nmea_injector | nmea_injector | 2748 /nmea-injector/ | python | DONE | DONE | DONE | DONE | FULLY MODELED; ext-GPS→GPS_INPUT via m2r; dynamic listener = gap |
| 15 | pardal | pardal | 9120 /network-test/ | python(aiohttp) | DONE | DONE | DONE | DONE | FULLY MODELED; unversioned; LAN throughput + WAN speedtest; measurement-only |
| 16 | ping | ping | 9110 /ping/ | python(v1.0) | DONE | DONE | DONE | DONE | FULLY MODELED; FIRST non-root py svc (run_as blueos → PrivilegeLevel::Regular); BR sonar mgr; spawns `bridges` binary; dynamic UDP = gap |
| 17 | bridget | bridget | 27353 /bridget/ | python(v1.0) | DONE | DONE | DONE | DONE | FULLY MODELED; non-root (Regular); user-configured serial↔UDP bridges; spawns `bridges` binary; OutboundHttp→linux2rest:6030; dynamic UDP = gap |
| 18 | recorder_extractor | recorder-extractor | 9150 /recorder-extractor/ | python(v1.0) | DONE | DONE | DONE | DONE | FULLY MODELED; root; MCAP→MP4 extract + video gallery; 5 subprocs (mcap/gst); DELETE=dangerous(delete_recording); shares recorder dir w/ recorder svc |
| 19 | mavlink2rest | mavlink2rest | 6040 /mavlink2rest/ | **BINARY** (rust) | DONE | DONE | DONE | DONE | FULLY MODELED; FIRST BINARY + FIRST CONCRETE EDGE (→ardupilot_manager via paired udpout/udpin:14001); Priority tier; MAVLink↔REST/WS bridge; tier Important; dangerous_ops=[] (transparent relay) |
| 20 | linux2rest | linux2rest | 6030 /system-information/ | **BINARY** (rust) | DONE | DONE | DONE | DONE | FULLY MODELED; read-only host system-info REST; unversioned; tier Important (bridget/helper depend); dangerous_ops=[]; authorities system_information_provider; edges=[] |
| 21 | video | mavlink-camera-manager | 6020 /mavlink-camera-manager/ (+/webrtc/ws/ :6021) | **BINARY** (rust) | DONE | DONE | DONE | DONE | FULLY MODELED; tmux `video` ≠ id `mavlink-camera-manager`; PRIORITY tier; 4 journeys; tier Important; dangerous_ops=[] (stream removal reversible); authorities camera_stream_manager; **2nd concrete edge → ardupilot_manager via tcpout/tcpin:5777**; runtime: no cameras attached (v4l=ISP nodes, streams=[]) |
| 9 | bridget | bridget | ? /bridget/ | python | TODO | TODO | TODO | TODO | |
| 10 | commander | commander | 9100 /commander/ | python | DONE | DONE | DONE | DONE | FULLY MODELED (RSS~35MB; read-only SLO; dangerous POSTs Unknown by design) |
| 11 | nmea_injector | nmea_injector | ? /nmea-injector/ | python | TODO | TODO | TODO | TODO | |
| 12 | helper | helper | 81 /helper/ | python | DONE | DONE | DONE | DONE | FULLY MODELED (RSS~62MB, cpu~1.06%; v1.0 GETs) |
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
| 25 | disk_usage | disk_usage | 9151 /disk-usage/ | python | DONE | DONE | DONE | DONE | FULLY MODELED (RSS~35MB, cpu~0.29%; du/ heavy) |
| 26 | customization | customization | ? /bootstrap? | python | TODO | TODO | TODO | TODO | |

> Ports/prefixes marked `?` must be confirmed from `nginx.conf` + argparse during that service's recon.
> Suggested order: cheap python services with clear nginx routes first (disk_usage, helper, commander,
> beacon, cable_guy, wifi, versionchooser, bag_of_holding, customization, nmea_injector, pardal, ping,
> bridget, recorder_extractor), then binaries (mavlink2rest, linux2rest, video, zenohd, nginx, ttyd,
> iperf3, filebrowser, recorder), then user_terminal (trivial).

---

## >>> RESUME POINTER (update every service) <<<
- Phases 1-3: DONE. Harness is built + hardened (gate.sh, extract, drift, frozen rubric).
- Phase 4 progress: FULLY MODELED (21; 18 py + 3 binaries) = ...python18..., mavlink2rest, linux2rest, mavlink-camera-manager.
- IMAGE NOTE (CORRECTED): the live Pi image did NOT change. sha256:cdccc744 = the image CONFIG ID (`docker inspect .Image`); sha256:0406983a = the same image's RepoDigest (registry `@sha256:` manifest). Both identify ONE image. The linux2rest artifact recorded the RepoDigest, all others the config ID — same image throughout. Earlier "image advanced" note was wrong (config-id vs repo-digest confusion), now retracted.
- BINARY PATTERN ESTABLISHED (mavlink2rest, ACCEPTED first-pass): kind=Binary; anchor deployment in start-blueos-core tuple + nginx.conf; id==tmux name (so extract gate cross-checks tier/mem/cpu); zenoh_log_topic/sentry/logs_path = Unknown (rust binary, no python init_logger); versions=[] (don't fabricate); MavlinkRole out→Consumer, in→Endpoint (interface.rs:51-52); sample_resource.sh now supports `--pid` for binaries. Edges CAN be asserted concretely when both sides' observed interfaces pair up (verify BOTH sides in source).
- **NEXT UP: `zenohd`** (then nginx, ttyd, iperf3, filebrowser, recorder) then user_terminal.
- BINARY launch tuples (start-blueos-core): video/mcm L120 (:6020 /mavlink-camera-manager/), mavlink2rest L121 DONE, zenohd L128 (/zenoh/,/zenoh-api/), iperf3 L135 (:5201 server, mem250), linux2rest L136 (:6030 /system-information/, mem250), filebrowser L137 (/file-browser/, --baseurl, mem250), ttyd L142 (:8088 /terminal/, tmux attach user_terminal, mem250), nginx L143 (the proxy itself, mem250, nice-18), recorder L145 (blueos-recorder --recorder-path /usr/blueos/userdata/recorder, mem250). nginx prefixes: mcm=/mavlink-camera-manager/→6020, linux2rest=/system-information/→6030, ttyd=/terminal/→8088, filebrowser=/file-browser/, zenoh=/zenoh/ + /zenoh-api/. (mavlink2rest, linux2rest, video, zenohd, nginx, ttyd, iperf3, filebrowser, recorder) then user_terminal. NOTE: binaries are NOT python services — no main.py/observed-from-python; they are external binaries launched by start-blueos-core. Extraction anchors come from start-blueos-core (entrypoint/tier/port), nginx.conf (prefix), and the binary's upstream repo/docs. ServiceKind will be a non-python variant. Reassess the observed rubric per-binary.
- Per-service loop (each layer committed separately, ledger updated): Fact Extractor(self-recon)→QA(observed)→Docs Specialist(journeys)→Card Author(wires all_journeys + service_def)→QA(card+journeys)→Runtime Specialist(live Pi)→commit. Run `bash catalog/gate.sh` before every commit. Pi at 192.168.0.177 (pi:raspberry). NEVER call destructive endpoints during capture.

## DECISIONS LOG (append-only; newest last)

- 2026-07-11: HARNESS FIX — extract self-check (find_extracted) now falls back to the observed tmux_name so binaries whose catalog id differs from the start-blueos-core tmux name still get tier/mem/cpu cross-checked. Motivated by `video` (tmux) vs `mavlink-camera-manager` (id). Committed standalone before modeling video.
- 2026-07-11: video/mavlink-camera-manager FULLY MODELED (3rd binary). tmux `video`, catalog id `mavlink-camera-manager`, aliases=["video"]. start-blueos-core:120 PRIORITY block, mem/cpu 0, nice --19, run_as root. Observed: Rest :6020 (/mavlink-camera-manager/, unversioned), Mavlink Consumer tcpout:127.0.0.1:5777, OutboundHttp stun://stun.l.google.com:19302, logs_path /var/logs/blueos/services/mavlink-camera-manager (REAL --log-path, citable — unlike other binaries). zenoh_topic/sentry/git_path/resources/openapi Unknown. /webrtc/ws/->:6021 NOT in observed (nginx doesn't name MCM there) — asserted-layer only. 4 journeys (view_camera_streams, configure_camera_stream, remove_camera_stream, configure_uvc_device_controls) grounded in docs + video-manager frontend; view_webrtc_stream OMITTED (BlueOS docs point to Cockpit, no in-app WebRTC player). Asserted: tier Important (operator-critical video, autopilot flies without it), dangerous_ops=[] (stream removal is reversible config, unlike recorder_extractor's irreversible file delete), privilege Root, offline_required true (STUN optional), authorities camera_stream_manager, +2 machine caps (provide_webrtc_signalling, advertise_cameras_over_mavlink). **2nd concrete edge: mavlink-camera-manager → ardupilot_manager via paired tcpout:5777 / tcpin:5777 (ardupilot_manager observed endpoint @ autopilot_manager.py:111).** QA BOUNCED 3 journey provenance anchors (POST /v4l cited store:190 [thumbnail sizing] not VideoControlsDialog.vue:189; device-list step VideoManager.vue:8 [css class] not :12 [v-for]; bundled nickname/encoding/resolution/framerate step at dialog:27 [encoding only] → split into 4 steps @ :20/:27/:35/:42). Orchestrator verified each against source and fixed directly (added file-parametrized sourced_route_in helper). Tier-1 runtime: PID 795, RSS ~37 MB, CPU ~0.6% (no active pipelines); NO USB/external cameras attached → GET /v4l 200 lists 4 onboard nodes (bcm2835-isp x2, Fake source, Redirect source) p50 8.5ms, GET /streams 200 empty [] p50 1.7ms; /info=201 (version 0.2.4), /xml=400 (needs file param); mutating routes left Unknown (no camera hw). Re-grounded the 2 read-only journey outcomes to runtime, left 3 mutating ones Unknown.

- 2026-07-11: linux2rest FULLY MODELED (2nd binary). Rust binary (start-blueos-core:136, SERVICES block → Normal tier, mem 250, run_as root), nginx /system-information/ :6030, UNVERSIONED (routes directly under prefix, versions=[]). Read-only host system-info REST (cpu/memory/disk/network/temperature/process/platform/serial/netstat/usb). Observed binary adaptation: kind=Binary, zenoh/sentry/logs_path=Unknown, sole Rest interface (no OutboundHttp/Mavlink/File — /proc reads are upstream internals). 1 journey (view_system_information, System Information page, doc+frontend-grounded). Asserted tier Important (bridget serial-enum + helper depend on it; NOT flight-critical), dangerous_operations=[] (read-only), authorities Other("system_information_provider") (exclusive host-info consolidator), edges=[] (reads local host only; bridget/helper edges are THEIR outbound, not linux2rest's), 2nd capability provide_system_information_over_rest (machine consumers). QA ACCEPT first-pass. Interrupted/resumed cleanly (observed+journeys were in-flight uncommitted; removed a stray catalog/examples/print_validate_errors.rs debug file a subagent left). Tier-1: PID 855 (rust, RSS flat 28 MB, CPU ~2.5% spiky on sampler ticks), 9 read-only GETs all 200; /system p50 10ms + /system/process p50 9ms heaviest, rest ~1.5ms. No mutating routes exist. Capture image = sha256:0406983a (master moved on).

- 2026-07-11: mavlink2rest FULLY MODELED — FIRST BINARY, FIRST CONCRETE EDGE. Rust binary (start-blueos-core:121, PRIORITY_SERVICES block → tier Priority), nginx /mavlink2rest/ :6040. MAVLink↔REST/WS bridge: connects out to router udpout:127.0.0.1:14001 (MavlinkRole Consumer), serves REST on [::]:6040. Observed binary adaptation: kind=Binary, zenoh/sentry/logs_path=Unknown (not python), versions=[] (no fabrication). 1 journey (MAVLink Inspector, UI-only steps, doc-grounded ADV:496-506). Asserted tier Important (data-plane bridge; many dependents break but vehicle still controllable via router/GCS → NOT VehicleCritical), dangerous_operations=[] (transparent relay — POST-send risk captured in blast_radius + mavlink_send_surface_abuse failure_mode, not a service-owned dangerous op), authorities Other("mavlink_rest_bridge"). **EDGE mavlink2rest→ardupilot_manager** (Mavlink, udpout/udpin:14001, required_at_boot true, failure_impact ServiceUnavailable) — verified BOTH sides in source: mavlink2rest Consumer udpout:14001 ↔ ardupilot_manager UDPServer udpin:14001 (autopilot_manager.py:74-81, protected). QA ACCEPT first-pass (no bounce). Tier-1: PID 243 (rust binary, RSS flat 8.8 MB — vs ~40 MB python; CPU ~1.3%), GET /v1/mavlink p50 3.9ms → 200 message tree, GET / p50 1.3ms → 200 html (built-in Vue UI), GET /v1/mavlink/HEARTBEAT → 200 body "None" (no live heartbeat, no vehicle armed). POST /v1/mavlink + WS NOT exercised (injects MAVLink). Runtime Specialist extended sample_resource.sh with --pid. FOLLOW-UP: ardupilot_manager/nmea_injector/ping edges to mavlink2rest can now be resolved from Unknown (deferred edges-pass).

- 2026-07-11: recorder_extractor FULLY MODELED. Root FastAPI v1.0 (nginx /recorder-extractor/ :9150, router prefix /recorder). Serves recorded MP4 for playback/download + manages /usr/blueos/userdata/recorder; background loop (every 10s) extracts MP4 from MCAP via 5 subprocs: mcap doctor, mcap recover, mcap-foxglove-video-extract, gst-discoverer-1.0, gst-play-1.0. 3 journeys: browse/download/delete. **FIRST recent card with NON-empty dangerous_operations**: DELETE /files → path.unlink() = Other("delete_recording") (irreversible data loss; precedent = disk_usage's Other("delete_filesystem_paths")); user_confirmation Required (rubric policy; frontend currently has NO confirm dialog → framed as 2.0 policy assertion, honestly noted). Asserted Auxiliary, Root, authorities Other("video_recording_server"), edges Unknown (shares recorder dir w/ uncataloged `recorder` svc — SharedWrite resource, deferral convention). QA BOUNCE ×1: delete journey summary composite string ("MP4...Records gallery...destructive and irreversible") grounded at main.py:397 which only says "Delete a recording." → shortened summary to match line literally. Tier-1: PID 1323 (root), RSS flat 35.6 MB, CPU ~0.3% (10s MCAP loop idle, no files); GET /files p50 6ms → 200 [], GET /status p50 6ms → 200 {"processing":[]}, GET / p50 5ms. Thumbnail/download/DELETE NOT exercised (no recordings present / destructive).
- 2026-07-11: MILESTONE — all 18 PYTHON/aiohttp services FULLY MODELED. Remaining Phase-4 work = 9 external BINARIES (mavlink2rest, linux2rest, video/mavlink-camera-manager, zenohd, nginx, ttyd, iperf3, filebrowser, recorder) + user_terminal (trivial: `cat /etc/motd`). Binaries need a different extraction approach (no python main.py): anchor observed facts in start-blueos-core (entrypoint/args/tier/port/run_as), nginx.conf (prefix), and upstream repo/docs for the binary's behavior. Several are Rust binaries maintained by BR (mavlink2rest, mavlink-camera-manager, and zenohd is eclipse). Journeys for infra binaries may be thin/None (they are plumbing consumed by other services) — model honestly, use Unknown/None where no operator-facing journey exists.

- 2026-07-11: bridget FULLY MODELED. User-configured serial↔UDP bridge manager ("Serial Bridges", nginx /bridget/, VersionedFastAPI v1.0 on :27353). Non-root (run_as blueos → PrivilegeLevel::Regular, like ping). Spawns the same `bridges` BINARY per operator-defined bridge; enumerates serial ports by proxying linux2rest (OutboundHttp localhost:6030/serial). Persists bridge list to settings-2.json. 3 journeys (Advanced/pirate visibility): view/create/remove. Asserted Auxiliary, dangerous_operations=[] (reversible operator config, consistent w/ ping/cable_guy; serial-exposure + port-contention risk → failure_modes/blast_radius), authorities Other("serial_bridge_manager") (distinct from ping's ping_sonar_manager — arbitrary operator serial devices vs BR sonar autodetect, NO overlap), edges Unknown (linux2rest not cataloged — deferred, same convention as m2r). GAP: dynamic per-bridge UDP ports + serial devices have no Interface variant. QA BOUNCE ×3 (journey-only UI anchor precision): (1) card step claimed serial-path+baud+UDP from BridgeCard.vue:22 (only name+baud) → reworded to name+baud; (2) "select serial+baud" from CreateDialog:18 (serial select only; baud at :81) → split into serial(:17)+baud(:81) steps; (3) "choose UDP mode/IP/ports" cited ADV:586 (prose about 0.0.0.0 server, not a UI action) → re-anchored to CreateDialog tabs :87 + IP field :111. Tier-1: PID 633 (python3, blueos), RSS flat 38 MB, CPU ~1.7% (spiky to 79%); GET /v1.0/bridges p50 9ms → 200 [], GET /v1.0/serial_ports p50 24ms → 200 [ttyAMA0..3,ttyS0] (latency incl. linux2rest round-trip), GET / p50 6ms. POST/DELETE NOT exercised (mutating, no serial HW).

- 2026-07-11: ping FULLY MODELED. Blue Robotics Ping-family sonar manager (nginx /ping/, VersionedFastAPI v1.0 on :9110). **FIRST non-root Python service** — wrapped in RUN_AS_REGULAR_USER_BEGIN/END → run_as=blueos → asserted PrivilegeLevel::Regular (all prior py svcs were Root). Auto-detects Ping1D/Ping360 over serial + Ping360-over-ethernet (UDP discovery :30303); spawns the external `bridges` BINARY (which("bridges"), NOT the bridget service) per device → UDP endpoint (9090↓ Ping1D / 9092↑ Ping360) for surface Ping Viewer. Optional Ping1D→MAVLink DISTANCE_SENSOR via OutboundHttp localhost:6040 (m2r). 3 journeys. Asserted Auxiliary, dangerous_operations=[] (MAVLink distance toggle reversible, consistent w/ nmea_injector GPS_INPUT; depth-hold risk → failure_modes/blast_radius), authorities Other("ping_sonar_manager"), edges Unknown (m2r not cataloged, nmea convention). GAP: dynamic per-device UDP bridge ports have no Interface variant (same class as nmea_injector). QA BOUNCE ×2 (journey-only): (1) connect journey visibility cited OVERVIEW:107 (Feature Comparison heading) → re-anchored to menus.ts:94 `advanced:false`; (2) device-card step claimed type+UDP+serial from ADV:563 (serial only) → re-anchored/reworded to ping1d.vue:15 (Bridge UDP endpoint). Tier-1: PID 1072 (python3, blueos user), RSS flat 40 MB, CPU ~1% (1s serial poll + eth discovery, spikes to 26% during discovery bursts); GET /v1.0/sensors p50 11ms → 200 `[]` (no sonar HW attached), GET / p50 6ms. POST /sensors + UDP bridge + Ping Viewer + MAVLink fwd NOT exercised (mutating / no hardware).

- 2026-07-11: pardal FULLY MODELED. Network speed-test service (nginx /network-test/, NOT /pardal/; unversioned aiohttp on :9120). Two mechanisms: LAN throughput via self-hosted HTTP /get_file (download) + /post_file (upload) + /ws echo (latency); WAN via speedtest-cli library (/internet_* routes) — NO iperf3 subprocess (iperf3 is a SIBLING tmux service) and NO literal OutboundHttp URL (speedtest-cli hides them). Asserted Auxiliary, dangerous_operations=[] (bandwidth saturation is transient/reversible, captured in blast_radius), authorities=[] (measurement producer, no exclusive right — consistent with disk_usage), offline_required true (LAN test offline-capable). QA BOUNCE ×2, both fixed: (1) /ws step used Actor::Service with a frontend client anchor → re-anchored to service handler main.py:45; (2) Interface::Rest anchored at web.Application() (line 144, shows neither prefix/port/version) → re-anchored to the TCPSite bind main.py:155 (carries port; prefix corroborated by nginx.conf:197). NOTE: unversioned service → versions:[]; the "cite prefix_format line" convention doesn't apply, so anchor the bind line. Tier-1 capture: PID 1008 `python` (NOT python3 — extended sample_resource.sh to match both), RSS flat 38.8 MB, CPU ~0.02% idle; GET / p50 2.6ms, internet_test_previous_result p50 2.7ms (returned 200 not 500 — SPEED_TEST initialized at boot on this host), get_file?size=1MB p50 82ms (transfer-dominated). WAN speedtest routes + POST upload + /ws NOT exercised (link-saturation hazard / mutating).

- 2026-07-11: nmea_injector FULLY MODELED. External NMEA/GPS ingest → MAVLink GPS_INPUT via HTTP POST to mavlink2rest (NOT a MAVLink connect string; OutboundHttp localhost:6040). Asserted Auxiliary (opt-in external GPS, not required for flight), nmea_gps_injector authority (producer, NOT router owner), dangerous_operations=[] (socket config reversible; wrong-position is a failure_mode). edges Unknown (mavlink2rest not yet cataloged — deferred). Extractor fixed nothing (clean); 5 dns-style anchors N/A. Harness gap: dynamic per-socket NMEA TCP/UDP listeners have no Interface variant. Tier-1 capture: RSS ~40 MB, CPU ~0.26%; GET /socks p50 8ms (returned [], no sockets configured). Mutations NOT exercised.

- 2026-07-11: customization FULLY MODELED. White-labeling (theme color, logo/vehicle-image branding, .glb 3D model overrides) writing /usr/blueos/userdata/{styles,branding,modeloverrides}. Asserted Auxiliary (cosmetic, non-critical), ui_branding_manager authority (scoped to userdata assets; bag stores sidebar images separately — noted overlap), dangerous_operations=[] (.glb upload is static data served to renderer, NOT executed code ⇒ NotRequired). 8 journeys. Self-QA (simple service): verified theme route + doc + frontend paths + authority uniqueness. Tier-1 read-only capture: RSS ~35 MB, CPU ~0.26%; all GETs ~5-6ms. Mutations NOT exercised.

- 2026-07-11: bag_of_holding FULLY MODELED. Generic JSON key-value store (set/get/overwrite, appdirs db.json). Only 1 first-class operator journey (modify_bag_database via advanced Bag Editor); all other frontend use (settings/wizard/vehicle-image/cloud-token) is INDIRECT infra — journeys belong to consuming features, not the store (Docs Specialist correctly declined to invent journeys). Asserted Important (many features depend on it; flight-independent), json_document_store authority, dangerous_operations=overwrite_entire_datastore (whole-db replacement, ⇒ Required; incremental /set excluded). Self-QA (simple service): verified journey anchors + /overwrite route. Tier-1 read-only capture: RSS ~35.8 MB (lightest Python service), CPU ~0.49%; GET /get/* p50 7ms. set/overwrite NOT exercised.

- 2026-07-11: versionchooser FULLY MODELED. Core BlueOS Docker-image updater (pull/switch/delete/restart, bootstrap startup.json, docker login) via docker.sock. Asserted Important (system-integrity critical, not flight-critical), is_platform=false (manages core OS image, not a third-party extension host — contrast kraken=true). dangerous_operations = Upgrade + delete_core_image + pull_untrusted_image ⇒ user_confirmation Required (QA accepted; pull mirrors kraken's install_arbitrary_docker_image). Authorities blueos_version_controller / bootstrap_image_controller / UserdataWriter(startup.json) — distinct from kraken's extension orchestrator (core image vs extension image split). QA bounced 1 anchor (zenoh_log_topic main.py:12 → logs.py:78); fixed. Tier-1 read-only capture: RSS ~53 MB, CPU ~0.82%; version/bootstrap GETs ~90-105ms (docker.sock). Mutating/destructive routes NOT exercised.

- 2026-07-11: wifi FULLY MODELED. WLAN manager (wpa_supplicant Bullseye / NetworkManager Bookworm handlers). Asserted Important, 2 authorities (wireless_network_controller, wifi_hotspot_operator) — distinct from cable_guy's wired authorities. dangerous_operations=[] (reversible reconfig, same rule as cable_guy). QA PASS on first submit. Real cross-service finding: wifi + cable_guy BOTH run dnsmasq and edit /etc/dhcpcd.conf (wifi=uap0 hotspot, cable_guy=wired); authority strings distinct so validator OK; narrowed cable_guy's dnsmasq authority rationale to "WIRED interfaces" for accuracy. Harness gaps: wpa_supplicant control-socket protocol, NetworkManager D-Bus, TCP fallback have no Interface variant. Tier-1 read-only capture: RSS ~46 MB, CPU ~0.37%; GET /status ~30ms (wpa ctrl socket). Mutating routes NOT exercised.

- 2026-07-11: cable_guy FULLY MODELED. Priority-tier wired-network manager; asserted Important, 3 authorities (wired_network_controller, host_dns_writer, onboard_dhcp_server_operator), dangerous_operations=[] (network reconfig is REVERSIBLE per frozen rubric rule 2; lockout is a failure_mode, not a dangerous op — QA confirmed). Modeling gaps logged: pyroute2 netlink + NetworkManager D-Bus in-process mutation have no Interface variant (only subprocess/file captured). Fixed 5 mis-anchored dns.py subprocess lines (extractor cited wrapper-call lines, not the run_command lines: correct = 56/64/74/80/86). QA over-strictly bounced the Interface::Rest anchor (main.py:163 prefix_format) — but that matches the QA-passed helper.rs gold standard; kept it and documented the convention in the extraction skill. Tier-1 read-only capture: RSS ~53 MB, CPU ~2.47%; GET /host_dns is slow (~1.2s, shells out to cat/lsattr resolv.conf). Mutating routes NOT exercised (lockout hazard).

- 2026-07-11: beacon FULLY MODELED. mDNS/zeroconf advertisement has no `Interface` variant — modeled honestly as capability `advertise_mdns_domains` + `discover_blueos_on_network` journey; logged as harness gap (consider `Interface::Mdns` if more network-advertisement services appear). QA bounced 2 anchors (Settings evidence → pykson_manager.py:69; discover precondition → getting-started:29); both fixed. Tier-1 read-only capture: RSS ~39.9 MB flat, CPU ~0.79% mean; GET SLOs p50 5–10.5 ms. POST /vehicle_name + /hostname NOT exercised (would rename live vehicle); those journey outcomes stay Unknown.

- 2026-07-11: Kraken committed `e5ffdaf8a`. Runtime captured Tier 1+2 with Example 1; vehicle restored. Fixed `probe_http.sh` (ARG_MAX + SIGPIPE). 4 journey outcomes left Unknown → **Phase 1 target**.
- 2026-07-11: `SloBaseline` is latency-only; `ResourceUsage` holds cpu/mem Distributions; kraken has NO state machine (state_contracts Unknown is correct).
- 2026-07-11: Journey validation requires participating service + capabilities to exist → wire `all_journeys()` together with the Card Author step, not before.
- 2026-07-11: Phase 3 done. Added `catalog/gate.sh`; hardened QA skill (extract + frozen rubric). Audit found no stale refs; validator is strong. Deferred coverage-threshold calibration and route-inventory cross-check with rationale (see PHASE 3 AUDIT). Harness is ready for Phase 4 batch.
- 2026-07-11: Phase 2 (4) done. Inter-rater calibration: independent rater vs committed gold on ardupilot+kraken. Hard enum/bool agreement 13/14 (only kraken user_confirmation flipped). All disagreements (that flip + dangerous_ops/authorities label variance) were rubric under-specification, not data errors. Froze rubric v1.0 with 4 deterministic decision rules; committed cards already comply, so no data changes. Also fixed stale tier menu + user_journeys→journey_refs in the card skill.
- 2026-07-11: Phase 2 (2)+(3) done. extract self-check + real drift landed. Adjudicated a REAL drift finding: drift flagged ardupilot `api_stable:true` vs runtime 500 in `stopped`. Ruled the CHECK unsound (api_stable = versioned surface stability; contractual 5xx are documented StateContracts, not surface churn). Removed that check; strengthened the state-machine/state existence check instead. Reworded the artifact note that conflated the two meanings of "stable". Lesson: runtime↔asserted checks must compare like-for-like semantics.
- 2026-07-11: Phase 1 done. Closed kraken's 4 gaps with a reversible Tier-2 capture of Example 1: custom install (POST /extension/ 200), add/del manifest (POST 201 / DELETE 204), tag edit v1.0.0→v1.0.1 (PUT 200), container log (GET 200). Artifact `transitions` extended; 4 journey outcomes re-grounded. Values orchestrator-captured + verified against artifact; gates green.

## PHASE 3 AUDIT — bulletproofing findings

Bulletproofed (done):
- **One-shot gate** `catalog/gate.sh` — deterministic full gate; run it after every slice and on every resume. `extract` now hard-fails if a newly-modeled observed layer diverges from `core/start-blueos-core` (catches copy/paste errors automatically).
- **QA skill** runs `extract` + applies the frozen rubric to judgment fields.
- **Completeness enforcement** is via mandatory `Unknown{reason}` + QA provenance spot-check (one fabricated citation = BOUNCE), not a numeric gate.

Deliberate deferrals (documented; low ROI until more services exist):
- **Coverage threshold** stays advisory (10_000). Phase-4 services legitimately carry Unknowns (`team`, `adr_refs`, `compatibility_policy`). Revisit AFTER all services are modeled to set a data-driven threshold; completeness is meanwhile enforced by mandatory Unknown reasons + QA.
- **Route-inventory cross-check**: `RouteRef`s in journeys/runtime are not validated against each service's real OpenAPI routes (validate.rs notes this). Mitigation today: runtime-capture skill grounds routes from live OpenAPI + QA spot-check. Future: store per-service route inventory in `observed.openapi_refs` and cross-check.
- **`Interface::Settings`** carries only `path` (root shape lives in prose/rationale).
- **No mDNS/zeroconf `Interface` variant**: beacon advertises BlueOS over mDNS (zeroconf lib) but the `Interface` enum has no network-advertisement variant. Currently captured via the service purpose + a capability, not an interface. Future: consider an `Interface::Mdns`/`NetworkAdvertisement` variant (schema change).

## OPEN HARNESS GAPS (superseded by PHASE 3 AUDIT above; kept for history)

- DONE: `extract` binary implemented (start-blueos-core parser + observed self-check). Future: could also mechanize nginx-prefix / listen-port extraction (deferred: nginx→service mapping is ambiguous by port/upstream).
- DONE: `drift` now covers asserted↔observed (resource evidence) and asserted↔runtime (state-machine/state existence). Note: `api_stable` has no sound runtime status falsifier (surface stability ≠ runtime robustness) — intentionally not checked.
- Coverage gate threshold is `10_000` (effectively disabled) — calibrate after rubric freeze.
- Inter-rater reliability not yet measured; rubric not frozen (Phase 2 item 4).
- `Interface::Settings` carries only `path` (no root-shape field) — settings root shape currently lives in prose/rationale.
