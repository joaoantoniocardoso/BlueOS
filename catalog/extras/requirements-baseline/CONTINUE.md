# REQBASE CONTINUE

Paste the fenced block below as the **first message** in a new orchestrator chat. No extra mission text.

```text
REQBASE CONTINUE

You are the Orchestrator. Load:
- .cursor/plans/blueos-requirements-baseline.md (phase gate definitions)
- .cursor/skills/blueos-rebaseline/SKILL.md (ordered steps, refuse list)
- catalog/extras/requirements-baseline/PROMPTS.md (worker templates)

Read catalog/extras/requirements-baseline/memory.xml if present (<= 60 lines).

Current phase: P6 COMPLETE (function layer). P0-P6 COMPLETE.
P6a-c ACCEPTed. FUNCTION_COUNT=108 vs 100 journeys. FUN ids are function-keyed; journeys are verification.

Latest ACCEPTed artifacts:
- FunctionCatalog: catalog/src/function.rs (FUNCTION_COUNT=108; not 1:1 with journeys)
- FUN re-hang: catalog/src/requirement.rs (function_id + verifying_journeys; FUN journey_id is None)
- snapshots: catalog/requirements-baselines/{1.4-dev,1.4.4-beta.21,1.5.0-beta.40}.json
  1.4-dev / 1.4.4-beta.21: 388 total (88 FUN); 1.5.0-beta.40: 481 total (108 FUN)
  unknown statements/criteria at 1.4-dev: 16/50; contamination 10 (FUN re-key, not loss)
- WBS: domain -> aggregate -> capability -> function
- RTM: function -> verifying journeys (check_internet_connectivity verifies
  monitor_internet_connectivity; verify_internet_connectivity)
- impact: catalog/extras/requirements-baseline/1.5.0-beta.40/impact.json
  (139 repo paths; 54 module keys; 631 entries)
- persisted snapshot stems: 1.4-dev, 1.4.4-beta.21, 1.5.0-beta.40

Key lessons (enforce in every spawn):
- Named `--until <tag>`; never HEAD; measure --until vs implicit HEAD; report observed ratio.
- Stay on model branch; never checkout target tag.
- `provenance_lint --fix` then `extract` then bare lint (full run; dry run may skip --fix).
- 0/0/0 `--diff` vs an older stem can be identity on HEAD, not a loss check.
- Mass FUN remove+add vs pre-rehang snapshots is FUN re-key from journey to function, not capability loss.
- Git `--since <prev>` may differ from `--diff` snapshot stem.
- Dispatch only impact.json `modules` object keys (not a numeric count).
- No DUT in dry run.
- TMPDIR/SCCACHE_DIR/CARGO_TARGET_DIR under $HOME (never /tmp).
- A check that cannot fail is worse than no check.
- P6: do not rename UserJourney; do not 1:1 clone journeys as functions; FUN ids are function keys;
  journeys are verification only; I/O stays Unknown unless an extractor yields a type;
  Feature.rationale is not a function spec.

Next work: STOP unless the user schedules out-of-P6 work (typed I/O, MAVLink/Zenoh signatures,
function decomposition, domain activity model). Do not start those without evidence and a new plan wave.

Hard rules: composer-2.5 workers; Opus-5 QA only; merge ACCEPT only;
commit/push/PR only if the user asks (personal fork only if push ever requested);
GitHub comments forbidden even if the user asks -- draft in chat.

Return ONLY: phase status; next spawns; blocked_on. Max 40 lines.
```

## Resume checklist

1. Re-read `memory.xml` and latest `DELTA.md` under `catalog/extras/requirements-baseline/<tag>/`.
2. Confirm `CARGO_TARGET_DIR` is fresh under `$HOME` before any gate.
3. If `DONE.md` exists with `next_steps: STOP`, halt.
4. Queue additional `REQBASE CONTINUE` follow-ups if the session may die.
