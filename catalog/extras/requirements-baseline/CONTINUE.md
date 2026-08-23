# REQBASE CONTINUE

Paste the fenced block below as the **first message** in a new orchestrator chat. No extra mission text.

```text
REQBASE CONTINUE

You are the Orchestrator. Load:
- .cursor/plans/blueos-requirements-baseline.md (phase gate definitions)
- .cursor/skills/blueos-rebaseline/SKILL.md (ordered steps, refuse list)
- catalog/extras/requirements-baseline/PROMPTS.md (worker templates)

Read catalog/extras/requirements-baseline/memory.xml if present (<= 60 lines).

Current phase: P6 (function layer). P0-P5 COMPLETE.
P5 harness gate ACCEPTed. Next work is P6a unless the user says otherwise.

Latest ACCEPTed artifacts:
- impact: `cargo run -q --bin impact -- --since 1.5.0-beta.39 --until 1.5.0-beta.40 --json`
  -> catalog/extras/requirements-baseline/1.5.0-beta.40/impact.json
  (139 repo paths; 54 module keys; 631 entries = jq '[.modules[] | length] | add')
  implicit HEAD contrast: 608 / 41 / 74 / 729; measured ratio 1.15
- snapshot: catalog/requirements-baselines/1.5.0-beta.40.json (473 requirements)
- diff: catalog/extras/requirements-baseline/1.5.0-beta.40/diff.json
  (added 92 / removed 0 / changed 0 vs stem 1.4.4-beta.21; same-HEAD filter gap)
- report: catalog/extras/requirements-baseline/1.5.0-beta.40/DELTA.md
- persisted snapshot stems on disk: 1.4-dev, 1.4.4-beta.21, 1.5.0-beta.40

Key lessons (enforce in every spawn):
- Named `--until <tag>`; never HEAD; measure --until vs implicit HEAD; report observed ratio.
- Stay on model branch; never checkout target tag.
- `provenance_lint --fix` then `extract` then bare lint (full run; dry run may skip --fix).
- 0/0/0 `--diff` vs an older stem can be identity on HEAD, not a loss check.
  added 92 vs 1.4.4-beta.21 is a version-filter gap on the same HEAD catalog, not two-tree.
- Git `--since <prev>` may differ from `--diff` snapshot stem.
- Dispatch only impact.json `modules` object keys (not a numeric count).
- No DUT in dry run.
- TMPDIR/SCCACHE_DIR/CARGO_TARGET_DIR under $HOME (never /tmp). Do not write contrast JSON to /tmp.
- Record provenance_lint unresolved=0 explicitly, not only "ran".
- Write HEAD contrast JSON under $HOME, never /tmp.
- P6: do not rename UserJourney; do not 1:1 clone journeys as functions; FUN ids are function keys;
  I/O stays Unknown unless an extractor yields a type; Feature.rationale is not a function spec.

P5 is committed (tip 560dcfcdf). P6 plan is in the runbook; types have not moved.

Next work (P6a):
1. Spawn Requirements Engineer for `catalog/src/function.rs` + `FunctionCatalog::from_catalog`.
2. No `requirement.rs` re-hang in P6a.
3. QA with fresh Opus-5 + blueos-catalog-validate.

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
