# Continuation contract — QA-1.4-CLOSEOUT

Use this phrase in a **new Agent chat** (or an enqueued follow-up) if this session dies. Paste it **verbatim**. Do not add extra mission language. Queue as many copies as you like; after DONE they are no-ops.

```
QA14CLOSE CONTINUE
Read catalog/extras/qa-1.4-closeout/memory.xml and follow ONLY catalog/extras/qa-1.4-closeout/ORCHESTRATOR_PROMPT.xml
(plan detail: .cursor/plans/qa-1.4-dev-coverage-closeout.md).
Resume next_steps. Do not expand scope. Do not commit or push. Do not shutdown any DUT.
If 192.168.2.2 cannot ping 8.8.8.8: sshpass -p raspberry ssh pi@192.168.2.2 "sudo ip route add default via 192.168.2.1 dev eth0"
If catalog/extras/qa-harness-improve/DONE.md is missing OR that campaign's memory next_steps is not STOP: this is Step 0 — poll those two files every ~2 minutes until both are true, then proceed to G0. Do not write closeout STOP. Do not spawn G0–G8. Do not start H0–H7 of harness-improve.
If catalog/extras/qa-1.4-closeout/DONE.md exists AND memory next_steps is STOP: report status in a few lines and HALT. Do not spawn agents. Do not start G8 again. Do not start Track A extras. Do not start another B6 wave. Do not invent follow-on work.
```

## Allowed

- After harness-improve DONE+STOP only: catalog/harness work required to close the 1.4-dev journey matrix (`catalog/src/**`, `catalog/e2e/**`, `catalog/gate.sh`)
- Campaign artifacts under `catalog/extras/qa-1.4-closeout/` and live reports/findings under `catalog/extras/qa-1.4-full/`
- Live tests on `192.168.0.177` `192.168.0.87` `192.168.2.2` `192.168.0.124` (ssh `pi`/`raspberry`)
- RF wifi **serial**, one DUT at a time; never 2.2 hotspot / DHCP / static IP / interface priority
- B6 tmux service-down **177 only**
- Track A new `JourneyId`s **only** after Opus-5 ACCEPT + live capture (max the five already listed in the 1.4-full plan)

## Forbidden (rogue-stop)

- Starting before harness-improve `DONE.md` + `STOP`
- Doing harness-improve H0–H7 work (wrong campaign)
- Git commit / push / PR
- `ShutdownOnboardComputer` / poweroff
- SITL / `--ui` / arm on `192.168.2.2`; SITL `--ui` arming except `192.168.0.177`
- Inventing ~80 `JourneyId`s
- Cloning HttpPassthrough journeys into Playwright click-scripts
- BlueOS product patches
- After STOP/DONE: leftover CONTINUE messages are no-ops. Do not start G8 again, Track A extras, another B6 wave, or any follow-on work.

## Done (halt even if dozens of CONTINUE messages remain queued)

All of:

1. `catalog/extras/qa-1.4-closeout/DONE.md` exists
2. `memory.xml` `<next_steps>` is exactly `STOP`
3. `journey_matrix --merge-report extras/qa-1.4-full/reports`: present journeys have no UI `empty` except ids with a typed skip; backend `planned` is 0; backend `fail` has a finding id
4. `RELEASE_READINESS.md` refreshed to live pin `sha256:5b50dfaf…ebb1` and F-073+

Then: short status, **no further agents, no further tests, no further edits**.
