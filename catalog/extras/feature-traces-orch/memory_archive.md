# Feature traces orchestrator archive

## Iteration 0 — BOOTSTRAP
- **Hypothesis**: Schema v2 + gh/git discovery closes all 7 gaps.
- **Agent tasks**: none yet
- **Raw results**: ORCHESTRATOR_PROMPT.xml + memory.xml created
- **Decision**: Start Phase 1 with parallel design + discovery probe agents

## Iteration 1 — PHASE1 DESIGN
- **Hypothesis**: Schema v2 + path discovery closes 7 gaps.
- **Agent tasks**: schema design; discovery probe
- **Raw results**: SCHEMA_V2.md + DISCOVERY_PROBE.md written
- **Decision**: Freeze design; implement enricher in Phase 2

## Iteration 3 — FULL RUN + TIGHTEN + VALIDATE
- **Hypothesis**: Module paths + backport title filter kills FPs; full run meets 7 gaps.
- **Agent tasks**: direct implement/run (Task tool flaky); validate script
- **Raw results**: VALIDATION_REPORT.md all PASS; zenoh/custom/disk/speed/horizon spot-checks OK
- **Decision**: Acceptance met; stop.
