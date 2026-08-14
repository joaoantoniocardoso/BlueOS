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

