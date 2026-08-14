# Continuation contract — QA-1.4-FULL

Use this phrase in a **new chat** (or an enqueued follow-up) if this session dies. Paste it **verbatim**. Do not add extra mission language.

```
QA14FULL CONTINUE
Read catalog/extras/qa-1.4-full/memory.xml and follow ONLY .cursor/plans/qa-1.4-dev-full-feature-test.md
Resume next_steps. Do not expand scope. Do not commit or push. Do not shutdown any DUT.
If 192.168.2.2 cannot ping 8.8.8.8: sshpass -p raspberry ssh pi@192.168.2.2 "sudo ip route add default via 192.168.2.1 dev eth0"
If RELEASE_READINESS.md exists and memory next_steps say STOP, report status and halt.
```

## Allowed

- Live catalog tests on `192.168.0.177` `192.168.0.87` `192.168.2.2` `192.168.0.124` (ssh `pi`/`raspberry`)
- Catalog/harness fixes required to run those tests (`catalog/src/**`, `catalog/harness/**`)
- Campaign artifacts under `catalog/extras/qa-1.4-full/`
- Restoring 192.168.2.2 default route via `192.168.2.1` on `eth0`
- RF wifi **serial**, one DUT at a time
- Destructive (firmware/EEPROM/reboot/core-switch/settings-reset) **177 only**, per plan W6

## Forbidden (rogue-stop)

- New features, refactors, or BlueOS product patches
- Git commit / push / PR
- `ShutdownOnboardComputer` / poweroff
- Network mutate on `192.168.2.2` (DHCP, static IP, interface priority, hotspot that drops USB)
- Changing the plan’s DUT roles
- Starting a different campaign or “while we’re here” work
- Inventing ~80 new JourneyIds (Track B probes only unless Track A already listed)

## Done

`catalog/extras/qa-1.4-full/RELEASE_READINESS.md` exists **and** `memory.xml` `<next_steps>` is `STOP`. Then: short status, **no further tests**.
