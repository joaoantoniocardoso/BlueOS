# QA 1.4-dev archive (append-only)

## Iteration 0 — Plan + kickoff
- **Hypothesis**: Track A/B/C/D sad strategy without mass new JourneyIds.
- **Agent tasks**: none yet (plan written by orchestrator).
- **Raw results**: four DUTs ping; same 1.4-dev digest; boards Navigator/Pixhawk1/Navigator/Navigator.
- **Decision**: start Part 1 matrix (Opus) and W0 inventory (Composer ×4) in parallel.

## Iteration 1 — W0–W2 + continuation phrase
- **Hypothesis**: Track B `--negative` finds 1.4-dev contracts vs absent-service 405s.
- **Agent tasks**: inventory ×4, W1 smoke ×4, `--negative` harness, W2 ×4, journey scope.
- **Raw results**: W1 19/0/72 all DUTs; W2 22/13/28 identical fails; 79 present / 21 absent.
- **Decision**: W3 safe mutating subset with `HOST_WIFI_IFACE=none`; 2.2 route via 192.168.2.1; CONTINUE.md is the only re-entry.

## Iteration 2 — W3 done, NMEA retry, W4 started
- **Hypothesis**: RF gold path on 177.
- **Agent tasks**: W3 already complete; NMEA retry Pass×4; W4 RF 177 background.
- **Raw results**: W3 FAIL=0; F-056 harness fixture override; NMEA retry Pass.
- **Decision**: W4 serial 177; skip 2.2 hotspot.

## Iteration 3 — W4 177 Pass, start 124
- **Hypothesis**: Pi5 Navigator 124 matches 177 RF.
- **Agent tasks**: W4 177 complete; W4 124 started.
- **Raw results**: 177 10/10 Pass; F-057 stale scan; F-058 wlan0 static IP 500.
- **Decision**: W4 124 then W6 177 reboot-only (no shutdown).

## Iteration 4 — W4 124 Pass, W6 reboot Pass, STOP
- **Hypothesis**: none.
- **Agent tasks**: W4 124 complete; W6 switch+reboot on 177.
- **Raw results**: 124 RF 10/10; switch 412 F-063; reboot Pass; pin unchanged.
- **Decision**: RELEASE_READINESS.md written. next_steps STOP.


## Iteration 6 — Resume coverage matrix (camera UI)
- **Hypothesis**: Camera `--ui` on 87 works if SITL/BoardRestore is skipped when no `sitl_frame`, and `--journey` falls back to `ui_plan()` outside `ui_suite_plans()`.
- **Agent tasks**: (1) harness SITL gate + camera ui_plans (2) matrix empty-cell inventory.
- **Raw results**: pending.
- **Decision**: spawn both; do not delete 87 live UDP Stream 0; SITL UI stays 177-only.


## Iteration 6b — Matrix inventory
- **Hypothesis**: remaining holes are UI-empty (65), not backend-empty (0).
- **Agent tasks**: journey_matrix --merge-report inventory (read-only).
- **Raw results**: 79 present; backend 37 pass / 36 skip / 5 planned / 1 fail (F-063); UI 13 pass / 65 empty. Camera ids still no ui_plan in binary. Next after camera: 124 no-hw --ui, RF 87, W6 412, extension fixture.
- **Decision**: wait for harness agent; then execute 87 camera UI (no Remove).


## Iteration 7 — Camera mutate with restore
- **Hypothesis**: GET /streams snapshot + Drop recreate is sufficient undo for delete/reset on 87.
- **Agent tasks**: McmStreamRestore; RemoveCameraStream UI; ConfigureVideoStream → Reset Settings; never legacy camera toggle.
- **Raw results**: pending.
- **Decision**: operator authorized mutate-with-undo; do not skip remove.


## Iteration 8 — 87 camera UI Pass (F-073)
- **Hypothesis**: McmStreamRestore Drop restores UDP Stream 0 after remove/reset.
- **Agent tasks**: live --ui ×5 on 87; polled report JSON (callbacks broken).
- **Raw results**: all five Pass; GET /streams has UDP Stream 0 running udp://192.168.2.1:5600 /dev/video2.
- **Decision**: replicate no-hw --ui on 124 next. Poll shells/files, do not rely on Task callbacks.


## Iteration 8b — 124 no-hw --ui Pass (F-074)
- **Hypothesis**: 124 matches 177 F-071 without SITL.
- **Agent tasks**: four --ui journeys on 124; polled shell stdout.
- **Raw results**: all Pass; board Navigator.
- **Decision**: next W4 RF on 87 if host wifi lock free; never 2.2 hotspot.


## Iteration 9 — Poll shells; W4 87
- **Hypothesis**: Task callbacks broken; poll report JSON and shell logs.
- **Agent tasks**: polled 87 camera reports; ran 124 no-hw --ui; polled W4 87 via shell until W4_DONE.
- **Raw results**: camera 5/5; 124 4/4; W4 9/10 (connect_to_wifi_network 500 timeout F-075).
- **Decision**: do not RF 2.2; next remaining UI plans then F-063.

