# Continuation contract — QA-HARNESS-IMPROVE

Use this phrase in a **new Agent chat** (or an enqueued follow-up) if this session dies. Paste it **verbatim**. Do not add extra mission language. Queue as many copies as you like; after DONE they are no-ops.

```
QAHARNESS CONTINUE
Read catalog/extras/qa-harness-improve/memory.xml and follow ONLY catalog/extras/qa-harness-improve/ORCHESTRATOR_PROMPT.xml
(plan detail: .cursor/plans/QA harness improvement-ccf7f122.plan.md).
Resume next_steps. Do not expand scope. Do not commit or push.
If catalog/extras/qa-harness-improve/DONE.md exists AND memory next_steps is STOP: report status in a few lines and HALT. Do not spawn agents. Do not start H7 again. Do not start B6/B7 recovery. Do not invent follow-on work.
```

## Allowed

- Catalog/harness work under `catalog/src/**`, `catalog/gate.sh`, `catalog/harness/**`
- Campaign artifacts under `catalog/extras/qa-harness-improve/`
- Live tests on `192.168.0.177` only for H7 (ssh `pi`/`raspberry`), and only if DONE.md does not exist
- Restoring `192.168.2.2` default route if it was already broken: `sshpass -p raspberry ssh pi@192.168.2.2 "sudo ip route add default via 192.168.2.1 dev eth0"`

## Forbidden (rogue-stop)

- New features, refactors, or BlueOS product patches beyond the plan phases H0–H7
- Git commit / push / PR
- `ShutdownOnboardComputer` / poweroff
- Network mutate on `192.168.2.2` (DHCP, static IP, interface priority, hotspot that drops USB)
- Starting a different campaign or “while we’re here” work
- New `JourneyId`s
- B6/B7/B8 recovery-window injection (explicitly out of this sequence)
- After STOP/DONE: any further implementation or live run

## Done (halt even if dozens of CONTINUE messages remain queued)

All of:

1. `catalog/extras/qa-harness-improve/DONE.md` exists
2. `memory.xml` `<next_steps>` is exactly `STOP`
3. H0–H6 ACCEPTed (H7 done **or** skipped with Pi-unreachable / not-in-scope reason in DONE.md)

Then: short status, **no further agents, no further tests, no further edits**.
