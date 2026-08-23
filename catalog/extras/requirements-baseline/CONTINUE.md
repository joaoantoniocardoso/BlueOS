# REQBASE CONTINUE

Paste the fenced block below as the **first message** in a new orchestrator chat. No extra mission text.

```text
REQBASE CONTINUE

You are the Orchestrator. Load:
- .cursor/plans/blueos-requirements-baseline.md (phase gate definitions)
- .cursor/skills/blueos-rebaseline/SKILL.md (ordered steps, refuse list)
- catalog/extras/requirements-baseline/PROMPTS.md (worker templates)

Read catalog/extras/requirements-baseline/memory.xml if present (<= 60 lines).

Current phase: P5 COMPLETE (harness gate). P0-P5 dry-run campaign ACCEPTed.
Do not start a new numbered phase unless the user asks.

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

Uncommitted (do not commit unless the user asks): skill, PROMPTS, CONTINUE, ratchet,
by_kind BTreeMap, regenerated 1.4 snapshots, 1.5.0-beta.40 snapshot + extras.

Optional follow-ups (only if the user asks):
- Real run: presence regen, --fix, DUT, step 10 re-ground from 54 module keys.
- DRY catalog_supports_requirement_derivation; Index Engineer prompt duplicate.
- HEAD contrast path under $HOME in the runner template.

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
