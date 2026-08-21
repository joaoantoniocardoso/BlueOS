---
todos:
  - id: p0a
    content: "P0a Citation resolution gate: every cited file exists, every line in range; provenance_lint in gate.sh"
    status: completed
  - id: p0b
    content: "P0b Content anchors on Evidence/Provenance; bulk-generate from current sources; auto-relocate on line shift"
    status: completed
  - id: p1
    content: "P1 Reverse index source path -> catalog entities; impact --since <tag> work orders"
    status: completed
  - id: p2
    content: "P2 Requirement layer: statement/criterion split, derived baseline, contamination lint, SRS + RTM exports"
    status: pending
  - id: p3
    content: "P3 Extraction expansion: nginx routes/ports, FastAPI decorators, frontend router; shrink hand-authored observed"
    status: pending
  - id: p4
    content: "P4 Re-baseline loop: per beta/release campaign producing a requirements delta report"
    status: pending
  - id: p5
    content: "P5 Agent harness hardening: roles, skills, QA protocol, requirements ratchet"
    status: pending
isProject: true
---

# BlueOS as-built requirements baseline -- orchestrator runbook

**Audience:** Orchestrator spawning **composer-2.5** workers and an independent **Opus-5** QA Reviewer. Local branch only. No PR. Commits only if the user asks.

**Goal.** Project an **as-built requirements baseline** out of the catalog, and keep it true by re-baselining at **every BlueOS beta and release**. The requirements are the deliverable; per-release sync is how they stay true; the agent harness is what makes it affordable.

**Campaign root:** `catalog/extras/requirements-baseline/` (create in P0).
**Memory:** overwrite `catalog/extras/requirements-baseline/memory.xml` (<= 60 lines). Archive: append `memory_archive.md`.
**Continuation trigger (verbatim):** `REQBASE CONTINUE` -- see `CONTINUE.md` (written in P0).

```mermaid
flowchart LR
  p0a[P0a citation gate] --> p0b[P0b content anchors]
  p0b --> p1[P1 impact index]
  p1 --> p2[P2 requirement layer]
  p1 --> p3[P3 extraction expansion]
  p2 --> p4[P4 rebaseline loop]
  p3 --> p4
  p4 --> p5[P5 agent harness]
  p4 -->|drift found| p2
```

---

## Ground truth (measured 2026-08-21, do not re-derive by hand)

| Thing | Count | Where |
|---|---|---|
| **Citations total** (pinned `FILE_CITATION_COUNT`) | **2041** | walked by `provenance_lint` |
| `Observed` / `Evidence { file, line, anchor }` | 928 | `catalog/src/**` |
| `Provenance::source` | 512 | `catalog/src/**` |
| `Provenance::doc` | 395 | into `../BlueOS-docs` |
| `Provenance::runtime` | 206 | into `catalog/runtime-captures/` |
| `Provenance::asserted` (no file, pinned) | 214 | judgement, not a citation |
| Anchored citations | 1808 | 1 `Unanchored`, 26 `AnchorExempt` |
| Journeys | ~100 consts | `catalog/src/journeys/*.rs` (29 modules) |
| Features | 143 (129 backend + 14 frontend) | `catalog/src/feature.rs` |
| Capability defs | 145 | `catalog/src/capability.rs` |
| Services | 27 | `catalog/src/services/*.rs` |
| Aggregates / Domains | 20 / 6 | `catalog/src/capability.rs`, `catalog/src/domain.rs` |
| Negative probes | 64 | `catalog/src/negative_probes.rs` |

**Structural facts that shape this plan:**

- The catalog lives **in the BlueOS repo itself**. `core/` is real in-tree BlueOS source at the same commit -- not a submodule, no cross-repo pin.
- This branch is **almost purely additive** (33 files outside `catalog/`, nearly all `.cursor/`). Rebasing onto upstream will therefore **never conflict** -- the model goes stale **silently**. All sync pressure must be manufactured deliberately.
- `gate.sh` today checks: `extract` (only `core/start-blueos-core`, ~3 fields/service), `drift` (asserted vs observed **within the model**), `validate()` (referential integrity + runtime capture route-key match), `journey_http --smoke` (live DUT). **No check opens a cited source file.** That is the hole P0 closes.

---

## Hard rules (orchestrator)

- NEVER implement, edit catalog sources, run `cargo`, or explore the tree yourself. Spawn composer-2.5.
- NEVER QA an artifact you (or its author) produced. Spawn a **fresh Opus-5** QA Reviewer (`claude-opus-5-thinking-high`). QA is the one place worth the expensive model.
- Workers are `composer-2.5`. If a worker BOUNCEs twice on the same deliverable, escalate that worker to `cursor-grok-4.6-high` -- never to Opus. Opus stays on QA and orchestration.
- Merge only on **ACCEPT**. BOUNCE goes back to a new author instance with the bounce list.
- Every worker prompt is self-contained. Ends with: `Return ONLY: [deliverable]. Max 40 lines.`
- Spawn 1-3 workers per iteration. Prefer parallel independent clusters (P2 and P3 are parallel-safe: different files).
- Working memory is the source of truth. Re-read it at the start of every iteration.
- Work stays on this branch. Do not `git push`. Do not open a PR. Do not commit unless the user asks.
- Pushing a PR branch: personal fork only, matched by URL owner (`joaoantoniocardoso`), never the upstream org remote. `origin` here is `bluerobotics/BlueOS-docker` with push DISABLED.
- Never post GitHub comments, reviews, or reactions. Draft replies in chat for the user.
- Finding is never a stop. Drift found -> record it, continue.
- **No iteration cap.** NEVER stop because the turn count is high.
- Phase gates are commands, not opinions. A phase is done when its gate command exits 0.

---

## Roles

| Role | Model | Owns | Never |
|------|-------|------|-------|
| **Orchestrator** | Opus | Phase gates, spawn, ACCEPT/BOUNCE, memory | Code, cargo, QA of the artifact |
| **Provenance Engineer** | composer-2.5 | P0 `provenance_lint`, `Evidence.anchor`, bulk anchor migration | Editing journeys/services semantics; deleting a citation to silence a failure |
| **Index Engineer** | composer-2.5 | P1 reverse index + `impact` bin | Changing model types |
| **Requirements Engineer** | composer-2.5 | P2 `requirement.rs`, derivation, exports, contamination lint | Hand-writing per-feature requirement prose |
| **Extractor Engineer** | composer-2.5 | P3 nginx / FastAPI / frontend-router extractors | Hand-editing `catalog/observed/**` (generated) |
| **Rebaseline Runner** | composer-2.5 | P4 one tag: presence + traces regen, impact, delta report | Hand-editing `journey_presence.rs` or `feature_traces.json` |
| **Runtime Capture Runner** | composer-2.5 | Live DUT capture + `--smoke` for one cluster | Guessing a status code; stranding a DUT |
| **Docs Engineer** | composer-2.5 | `DONE.md`, delta reports, campaign notes | Changing DUT roles |
| **QA Reviewer** | Opus-5 **fresh** | One worker output -> ACCEPT/BOUNCE with evidence | Edit the artifact; review own work |

---

## What we will not build (bounce on sight)

ReqIF / SysML / XMI exporters. A requirements-management tool integration. A new crate. Hand-authored requirement prose per feature (the ~12-entry system-level overlay in P2 is the **only** exception). Gherkin. A DOORS-style GUI. Chasing `master` continuously. Rewriting journeys so they fit a requirement template. Deleting or loosening a citation to make `provenance_lint` pass. Requirement IDs derived from line numbers, file paths, or anything else that moves.

---

## Phases

Each phase: spawn workers -> QA each output -> merge ACCEPTs -> update memory -> next phase.

### P0a -- Citation resolution gate (1 Provenance Engineer + Opus-5)

New bin `catalog/src/bin/provenance_lint.rs`. Walk every `Evidence` and `Provenance::{Source,Doc,Runtime}` reachable from `Catalog::bootstrap()` plus `NEGATIVE_PROBES`, `MUTATING_SMOKE_ENTRIES`, `UI_*`, and page/feature registries. For each: file exists, line >= 1 and <= file length. Doc citations resolve against `../BlueOS-docs` (report `skipped` with a count, not a failure, when that checkout is absent). Runtime citations resolve `capture#key` against the JSON.

Report counts by provenance kind and by resolution status. Non-zero unresolved = exit 1.

**Gate:** `cd catalog && cargo run -q --bin provenance_lint` exits 0, and `gate.sh` invokes it. Any citation that genuinely cannot resolve is fixed by **re-grounding**, never by deletion.

**DONE (2026-08-21, ACCEPT).** 2041 citations walked, all resolving; 11 repaired by re-grounding (10 `source` -> `doc` in `commander.rs`, 1 placeholder -> `asserted`). `provenance_lint` is in `gate.sh` after `drift`. Fatal on `MissingFile`, `LineOutOfRange`, `MissingCaptureKey`, `Misclassified` (checked symmetrically in both directions). Took three QA rounds; the recurring failure was **citations the walk never reached** -- non-standard field names (`call_file`, `store_file`) and registries not hung off `Catalog::bootstrap()`. A registry coverage table now makes that explicit.

### P0b -- Content anchors (1 Provenance Engineer + Opus-5)

**DONE (2026-08-21, ACCEPT).** `anchor: &'static str` on `Evidence` and `Provenance::Doc`, carrying a literal copied from the cited line. 1808 anchored citations, all verified true by an independent checker. `Runtime` keeps `capture#key`; `Asserted` has no file.

Statuses: exact prefix match at the cited line -> `Resolved`; one match within +/-50 lines -> `Relocated`; several -> `AmbiguousAnchor`; elsewhere in file -> `AnchorMovedFar`; gone -> `AnchorLost`. **All four are fatal**, each with a live test proven non-vacuous (breaking `verify_anchor` fails eight tests). `Relocated` is fatal in a bare run -- a tree needing `--fix` does not pass the gate. `--fix` repairs all seven citation shapes idempotently, touching only the line number.

Six pins guard against silent loss: `FILE_CITATION_COUNT=2041`, `SOURCE_COUNT=512`, `ASSERTED_COUNT=214`, `UNANCHORED_COUNT=1`, `ANCHOR_EXEMPT_COUNT=26`, `NON_UNIQUE_IN_WINDOW_COUNT=171`. Exact `assert_eq!`, not floors, so a legitimate addition requires a deliberate bump. Non-fatal statuses are exactly `Resolved`, `Skipped`, `NoFileReference` (asserted), `Unanchored` (1 blank cited line), `AnchorExempt` (23 frontend `call_file`/`store_file` + 3 path constants); each proven structurally unreachable by a citation that has a verifiable anchor.

**Residual risk, carry into P1 and P4:**

- **171 anchors (9.5%) are not unique within their +/-50-line window.** They never let drift pass as `Resolved`, but on drift they degrade from auto-repairable `Relocated` to fatal `AmbiguousAnchor` needing a hand fix. Four repeated anchors account for 80 of them and already span the whole cited line, so truncation tuning cannot help -- only a different anchor design (multi-line context, narrower window) would, and that is not worth it yet. Pinned so it cannot grow silently.
- **`--fix` rewrites the worktree and its post-fix report reflects in-memory state.** Always follow it with a fresh, explicitly rebuilt bare run. Cargo's mtime freshness check hands back a stale binary after `--fix` writes to `src/`; this produced false results four times across QA and authoring.
- 14 citation lines exceed 120 columns, deliberately: at the 12-char anchor floor, correctness beat the column limit. Nothing in `gate.sh` enforces width.

**Lesson for later phases.** Five separate regex-construction defects landed in this tool, two of which passed a full self-check because the tool verified its own output. Anything that rewrites catalog source needs an *independent* checker that parses the Rust directly, plus a re-run of every previously-proven repair shape -- three consecutive rounds each broke a shape an earlier round had proven working.

### P1 -- Impact index (1 Index Engineer + Opus-5)

Reverse map: source path -> catalog entities citing it (`ServiceId`, `JourneyId`, `PageId`, `FeatureId`, negative probe, UI plan). Built from the same walk as `provenance_lint`, so the two share one traversal.

New bin `catalog/src/bin/impact.rs`:

- `impact --since <tag|sha>` -- diff `core/` between that ref and HEAD, map changed files to entities, emit a work order grouped by catalog module.
- `impact --path <file>` -- what cites this file.
- `--json` for agent consumption.

**Gate:** every file cited anywhere appears in the index (index file-set == lint file-set). A test asserts a known path (`core/services/nmea_injector/main.py`) maps to the expected journeys. `impact --since` against a real prior tag produces a non-empty, correctly-grouped work order.

**DONE (2026-08-21, ACCEPT).** `catalog/src/source_index.rs` + `catalog/src/bin/impact.rs`. Index built from the *same* `build_report()` traversal as `provenance_lint`, so the two cannot disagree about what is cited. 231 distinct cited files, verified identical to the linter's resolved file-set. Pinned per kind: 158 Observed / 104 Source / 7 Doc / 27 Runtime.

`impact --since v1.4.2` -> 179 impacted indexed paths, 1528 entries across 77 modules. `--path` answers the reverse. Entries are `needs-review` by default and `confirmed-drifted` when the linter reports that citation as `Relocated` or worse.

Attribution resolves the walk's `model_path` index back into `SERVICES` / `PAGES` / `all_journeys()`. That is only sound while `Catalog::bootstrap()` serializes those exact slices in that exact order, so `bootstrap_slices_match_attribution_index_sources` asserts it -- a filter or a reorder fails loudly instead of silently sending an agent to the wrong module.

**Known blind spots, stated in the tool's own output:**

- **Doc citations are not diffed.** They resolve against the sibling `../BlueOS-docs` checkout, which `--since` never looks at, so documentation drift produces an empty work order. P4 must re-check doc-grounded claims by another route.
- **An intact anchor is not a guarantee.** It proves the cited line survived, not that the surrounding behaviour still supports the claim -- hence changed-file-means-review rather than trusting the anchor.
- Between a pre-catalog tag and HEAD, ~14% of entries are the catalog's own birth (27 runtime captures + 3 path constants). Real evidence, but noise for this particular comparison; it shrinks to nothing once baselines are catalog-era.

**Lesson.** The first completeness test was tautological -- both sides called the same `filter_map`, so it passed for every possible input. Dropping all `Runtime` citations removed 27 files and 206 citations from the index with the test still green. Any "the index covers everything" claim must derive its expected set *without* the function under test, and pin the counts.

### P2 -- Requirement layer (1 Requirements Engineer + Opus-5; parallel with P3)

New `catalog/src/requirement.rs`. **Derived, not authored** -- a pure function of the catalog, in the spirit of `FeatureCatalog::from_catalog`.

**The statement/criterion split is mandatory.** Requirements-recovery literature calls the failure mode *contaminated requirements*: well-formed statements that encode implementation artifacts instead of system expectations. `"BlueOS shall return 200 from GET /nmea-injector/v1.0/socks"` is not a requirement; a 2.0 rewrite that renames the route would falsely appear to violate it.

- **Statement** -- implementation-free, derived from the Doc-grounded `UserJourney.summary` and `Feature.rationale`. This is what 2.0 is held to.
- **AcceptanceCriterion** -- implementation-bound, derived from `RouteRef` + `StepOutcome` + runtime capture. This is how 1.x is verified today, and it is *expected* to change in 2.0.

Levels, all derived:

| Kind | One per | Source |
|---|---|---|
| System | `Feature` | `Feature.rationale`, allocated to owning service/page |
| Functional | `UserJourney` | `summary`, refined by steps |
| Interface | distinct resolved route | `resolve_http_path` + capture |
| Performance | `SloBaseline` | `catalog/src/runtime.rs` |
| Robustness | `NegativeProbe` | `catalog/src/negative_probes.rs` |
| Constraint | distinct `Precondition` | journey preconditions |

Requirement IDs are stable and derived from `Domain` + `Aggregate` + capability/journey identity. **Never** from line numbers, file paths, or ordinal position.

Version scope comes free: each requirement inherits `FeatureAvailability`, so `requirements --version 1.4-dev` is the baseline for that tag, driven by the existing feature map in `catalog/src/feature_intro.rs`.

The **only** hand-authored content is a small overlay (target <= 12 entries) for system-level requirements no single journey can express -- boot-to-usable-UI, offline operability, and similar. It is `Asserted`, carries a rationale, and lives in one file.

New bin `catalog/src/bin/requirements.rs`: `--version <tag>`, `--json`, `--srs` (markdown), `--rtm` (CSV traceability matrix), `--diff <tag>`.

`validate()` additions: every `Feature` yields a system requirement; every system requirement has >= 1 functional requirement or a typed `Unknown { reason }`; every functional requirement has >= 1 acceptance criterion or typed `Unknown`; anything claiming `Automatable::Http` has real runtime evidence.

**Contamination lint:** statements must not contain route paths, HTTP verbs, status codes, port numbers, or service ids. Violations fail.

**Gate:** `cargo run -q --bin requirements -- --version 1.4-dev --srs` renders; contamination lint = 0; `cargo test` green; `unreferenced_features` surface as typed `Unknown` rather than silently vanishing. Opus-5 reads 20 sampled statements and judges them implementation-free.

### P3 -- Extraction expansion (1 Extractor Engineer + Opus-5; parallel with P2)

Every fact a parser derives is a fact nobody re-checks each release. Extend `catalog/src/extract.rs` beyond `start-blueos-core`, in churn order:

1. `core/tools/nginx/nginx.conf` -- routes, ports, prefixes (highest churn, feeds interface requirements).
2. FastAPI decorators in `core/services/*/main.py` -- method, path, version.
3. Frontend router table (`core/frontend/src/router/index.ts`) and `menus.ts`.

Extracted facts cross-check the observed layer via `check_against_observed`, exactly as the existing tier/limits check does.

**Gate:** `cargo run -q --bin extract` exits 0 with the new checks active; `drift` still 0; count of hand-authored observed fields **decreases**, recorded in memory as a ratchet.

### P4 -- Re-baseline loop (Rebaseline Runner + Runtime Capture Runners + Opus-5)

The per-beta/per-release campaign. Ordered, and each step is a gate:

1. Pick the target tag. **Do not chase HEAD** -- the catalog is true of a *named baseline*, never of a moving branch.
2. Regenerate presence: `cargo run --bin generate_feature_presence`. Never hand-edit `journey_presence.rs`.
3. Regenerate traces: `cargo run --bin enrich_feature_traces`.
4. `provenance_lint --fix` -- absorb line shifts; escalate real drift.
5. `impact --since <previous tag> --json` -- the work order. **Only** the listed clusters get agents.
6. Per affected cluster: re-ground citations, refresh runtime captures on a live DUT, re-run `journey_http --smoke`.
7. `requirements --diff <previous tag>` -- requirements gained, lost, and changed.
8. Write `catalog/extras/requirements-baseline/<tag>/DELTA.md` and refresh the ratchet baseline.

**Gate:** one full dry run against a real prior tag, end to end, producing a delta report a human can read. `gate.sh` green throughout. Any requirement that disappeared is explained (removed feature vs. lost citation) -- an unexplained disappearance is a bug in P0/P1, and loops back.

### P5 -- Agent harness hardening (Docs Engineer + Opus-5)

Only after P4 has run once, so we automate what actually happened rather than what we imagined.

- Skill `blueos-rebaseline` under `.cursor/skills/`, matching the existing skill conventions.
- Worker prompt templates per role, self-contained, in the campaign root.
- Extend `catalog/src/harness_ratchet.rs` with requirements-coverage counts so they can only improve.
- `CONTINUE.md` + memory template so a dead session resumes cleanly.

**Gate:** a second re-baseline against a different tag completes with the orchestrator spawning agents from the skill and prompts alone, no ad-hoc instructions.

---

## Worker prompt skeleton (orchestrator MUST use)

```text
You are the [Role]. Load [skill path] if listed. Follow catalog rules on glob match.

Mission: [one sentence]
Files you may edit: [exact paths]
Do not edit: [paths]
Invariants: derived requirements only (no hand-authored per-feature prose);
  statement stays implementation-free, criterion carries the route/status;
  never delete a citation to silence provenance_lint -- re-ground it;
  new check must be non-vacuous (break it, watch fail, restore);
  never hand-edit journey_presence.rs, feature_traces.json, or catalog/observed/**;
  Unknown { reason } over a guess; requirement ids never encode line/path/ordinal;
  baseline is a named tag, never HEAD; local only, no commit/PR unless asked HERE.
Done: [gate commands]
Forbidden: [phase forbidden list]

Return ONLY: files changed; gate command output; remaining holes; anything you refused.
Max 40 lines.
```

QA prompt: model `claude-opus-5-thinking-high`; skill `blueos-catalog-validate` when reviewing catalog diffs; review **one** author output; ACCEPT or BOUNCE with evidence; do not edit. Fresh Task spawn.

---

## Memory template (overwrite every iteration)

```xml
<working_memory last_updated="iteration N @ ISO-8601">
  <recovery>Read .cursor/plans/blueos-requirements-baseline.md -- ORCHESTRATOR only.</recovery>
  <phase>P0a|P0b|P1|P2|P3|P4|P5</phase>
  <baseline_tag>e.g. 1.4-dev</baseline_tag>
  <ratchets>unresolved_citations=N anchors_missing=N handauthored_observed=N reqs_without_criterion=N</ratchets>
  <in_flight>agent -> cluster -> status</in_flight>
  <last_results>ACCEPT/BOUNCE / gate output</last_results>
  <next_steps>- [Phase Px] spawn ...</next_steps>
  <blocked_on>Nothing | BOUNCE on X | DUT unreachable</blocked_on>
</working_memory>
```

---

## Invariants that must not move

- A citation is re-grounded, never deleted, to make a gate pass.
- **A pin is bumped deliberately, with the reason stated, or not at all.** The six citation pins (`FILE_CITATION_COUNT=2041`, `SOURCE_COUNT=512`, `ASSERTED_COUNT=214`, `UNANCHORED_COUNT=1`, `ANCHOR_EXEMPT_COUNT=26`, `NON_UNIQUE_IN_WINDOW_COUNT=171`) are exact equalities precisely so that adding or losing a citation cannot pass unnoticed. They will legitimately move whenever a phase adds citations -- P2 and P3 both will. That is the dangerous moment: a worker who treats a tripped pin as a failing test to be silenced, rather than a fact to be explained, reopens the hole the pin exists to close. A pin change must name what was added and why the new number is right. "Test failed, updated the number" is a BOUNCE.
- Generated files are regenerated, never hand-edited: `journey_presence.rs`, `feature_traces.json`, `catalog/observed/**`.
- Requirement **statements** are implementation-free; **criteria** carry the implementation detail. The split never collapses.
- Requirements are derived. The only authored requirements are the <= 12 system-level overlay entries.
- `Unknown { reason }` beats a plausible guess, everywhere.
- Requirement IDs are stable across refactors.
- The baseline is a named tag. The catalog is never claimed to be true of `master`.
- Findings never stop a wave.

## How to run

1. Open a **new Agent chat** as orchestrator.
2. Paste the fenced block in `catalog/extras/requirements-baseline/CONTINUE.md` as the first message (`REQBASE CONTINUE`). Written during P0a; until then, start at P0a from this file.
3. Queue several copies as follow-ups in case the session dies. Each copy resumes `next_steps`, or halts if `DONE.md` exists with `next_steps: STOP`.
4. Do not add extra mission text to those messages.

Cold start and resume use the same phrase. There is no iteration cap.
