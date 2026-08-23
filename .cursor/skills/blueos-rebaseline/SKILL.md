---
name: blueos-rebaseline
description: >-
  Orchestrate one BlueOS requirements re-baseline campaign for a named immutable
  tag: presence/traces regen, provenance repair + extract verification, impact
  work order, per-module re-ground, requirements snapshot/diff, DELTA.md. Use as
  the orchestrator spawning composer-2.5 workers and an independent Opus-5 QA
  Reviewer. Never checks out the target tag onto the model branch.
disable-model-invocation: true
---

# BlueOS Re-baseline (Orchestrator)

You are the **Orchestrator**. Run one named-tag campaign. Spawn workers from `catalog/extras/requirements-baseline/PROMPTS.md`. QA each artifact with a **fresh** Opus-5 using `blueos-catalog-validate`. Merge only on ACCEPT. You do not implement catalog code or run cargo as the author of the artifact being QAd.

**Who runs each step:** 1-2 orchestrator; 3-8 and 11-13 Rebaseline Runner (spawned); 9 orchestrator dispatches Extractor/Runtime workers from impact.json `modules` object keys; 10 those workers; 14 Docs Engineer; 15 independent Opus-5 QA.

## Environment (every gate)

```bash
export TMPDIR="$HOME/.cache/reqbase-tmp" SCCACHE_DIR="$HOME/.cache/sccache"
export CARGO_TARGET_DIR="$HOME/.cache/reqbase-target-$(date +%s)"
mkdir -p "$TMPDIR" "$SCCACHE_DIR" "$CARGO_TARGET_DIR"
cd catalog
```

Never use `/tmp` (quota). TMPDIR and CARGO_TARGET_DIR must be under `$HOME`. Use a **fresh** `CARGO_TARGET_DIR` per gate pass. One `cd catalog` per shell, then relative commands (repeating `cd catalog &&` nests `catalog/catalog/`).

## Ordered steps

1. Pick immutable `<target>` tag and `<prev>` git ref. Record HEAD SHA. **Never** use `HEAD` as `--until`.
2. **Do not** `git checkout <target>`. Citations and extractors read current `core/` on the model branch.
3. `cargo run -q --bin generate_feature_presence` (full run; skip in dry run).
4. `cargo run -q --bin enrich_feature_traces` (full run; skip in dry run).
5. `cargo run -q --bin provenance_lint -- --fix` (full run; dry run may skip `--fix`).
6. `cargo run -q --bin extract`.
7. `cargo run -q --bin provenance_lint` (bare read-only; never trust step 5 without 6-7).
8. `cargo run -q --bin impact -- --since <prev> --until <target> --json` -> `catalog/extras/requirements-baseline/<target>/impact.json`.
9. Dispatch **only** the object keys of `impact.json` `modules` (e.g. `journeys/helper.rs`, `services/helper.rs`). `modules` is an object of arrays, not a count; entry count is the sum of those array lengths (`jq '[.modules[] | length] | add'`). Adjacent betas may share one file (e.g. helper). Never hand out the whole catalog. Do not invent a different JSON shape.
10. Per dispatched key: Extractor Engineer re-grounds `line`/`anchor` only (values are facts; never change a value to fix a pointer). Runtime Capture Runner when DUT reachable (skip in dry run; no invented status codes).
11. `cargo run -q --bin requirements -- --version <target> --json` -> `catalog/requirements-baselines/<target>.json`.
12. `cargo run -q --bin requirements -- --version <target> --diff <stem> --json`. `--diff` is the **persisted snapshot stem** (file `requirements-baselines/<stem>.json`). If that file is missing, stop; do not pass a git tag that has no snapshot. `<prev>` for `impact --since` is a git ref; it may differ from the `--diff` stem. (P4 used `--diff 1.4-dev` because that file exists; `1.4.4-beta.20` is a git ref, not a snapshot stem unless the file is present.)
13. `bash gate.sh` (omit `BLUEOS_BASE` in dry run).
14. Docs Engineer writes `catalog/extras/requirements-baseline/<target>/DELTA.md`.
15. Opus-5 QA Reviewer: ACCEPT or BOUNCE each worker output.

## `--diff` identity trap

`--diff` compares persisted snapshots, not git trees. When both `--version` filters admit the same requirement IDs on the **same HEAD catalog** (e.g. `1.4-dev` + a beta newer than the presence table), 0/0/0 is **identity**, not a demonstrated loss check. DELTA.md must state whether the two sides are different snapshots/trees. `--diff` still reports added/changed when snapshots actually differ.

## DELTA.md required sections

- Target: tag, SHA, commit message, `<prev>` git ref, previous snapshot path (`requirements-baselines/<stem>.json`).
- Impact: `--since <prev> --until <target>` counts **and** a measured contrast with implicit `HEAD` (same command without `--until`); report the observed ratio. Do not assume a fixed overshoot.
- Requirements snapshot table (totals, Unknown counts, contamination).
- Diff vs snapshot `<stem>`: added/removed/changed; **identity warning** when applicable. FUN ids are function-keyed; journeys appear as verification only. Mass FUN churn vs pre-rehang snapshot is FUN re-key, not loss.
- Extract/lint: extract exit, bare `provenance_lint`, whether `--fix` ran.
- Gate: `gate.sh`, `harness_ratchet`, feature-traces gate.
- Holes: DUT, skipped regen, doc citations not diffed by `impact`, untested two-tree diff, **HashMap `by_kind` byte-stability** (required line whenever a snapshot is written).
- Refused: explicit list.

## Refuse list

- `git checkout <target>` onto the model branch.
- `impact --since <prev>` without `--until <target>` (defaults to HEAD; oversized vs named-tag until; measure the contrast, do not assume a ratio).
- Hand-edit `journey_presence.rs`, `feature_traces.json`, `catalog/observed/**`.
- Trust `provenance_lint --fix` without `extract` + bare lint.
- Treat 0/0/0 `--diff` as pass without proving different snapshots/trees.
- Pass `--diff` a git tag with no file `requirements-baselines/<stem>.json`.
- Dispatch agents for keys not in the impact `modules` object.
- Invent runtime status codes or `Known` journey outcomes without DUT capture.
- Silently bump citation pins; every pin change must name what was added and why.
- Subtract disjoint populations to claim a ratchet improved (metrics that improve are not evidence).
- Treat mass FUN churn in `--diff` vs pre-rehang snapshots as capability loss (FUN re-key from journey to function is expected once).
- TMPDIR or CARGO_TARGET_DIR under `/tmp` (must be under `$HOME`).
- Commit, push, or open a PR unless the user asks.
- GitHub comments (issues, reviews, discussions) even if the user asks -- draft in chat instead.

## Worker spawn

Use templates in `catalog/extras/requirements-baseline/PROMPTS.md`. Spawn Rebaseline Runner for steps 3-8 and 11-13; spawn Extractor/Runtime workers in step 9 from `modules` keys; spawn Docs Engineer for step 14; spawn a fresh Opus-5 QA Reviewer per artifact. End every worker prompt with: `Return ONLY: [deliverable]. Max 40 lines.`

## Resume

Dead session: paste `REQBASE CONTINUE` from `catalog/extras/requirements-baseline/CONTINUE.md`.
