# Feature-traces ABC — plan (UX + parallel + release workflow)

Orchestrator: `abc/ORCHESTRATOR_PROMPT.xml`  
Memory: `abc/memory.xml`  
Baselines: precision / improve / next11 / next15 QA (all PASS)

## Tracks
| ID | Track | Deliverable |
|---|---|---|
| A1 | Catalog read API (minimal) | Serve presence+traces+skip-reason JSON without inventing a full microservice if a thin static/bin server fits; else nginx-static reports + JSON snapshot |
| A2 | Skip-reason surface | Expose journey_availability_skip reasons to a consumable API/JSON for UI |
| A3 | Vue provenance + skip panel | Minimal tools view: timeline + presence chips + “why skipped” when DUT context available |
| B1 | Live --jobs>1 smoke | Run enrich --jobs 2..N with resume; document rate-limit behavior |
| B2 | Parallel harden | Fix races/cache thrash/gh 403s found in B1; backoff shared across workers |
| C1 | Release diff workflow | Script: feature_trace_diff between two tags/snapshots + HTML report regen |
| C2 | QA checklist | Documented release/QA steps using check-traces, reports, diff |

## Phases & QA gates
| Phase | Work | QA gate |
|---|---|---|
| 1 Design | ABC_DESIGN.md freezes A API vs static, Vue mount, B jobs target, C scripts | Design accepted |
| 2 Track B | B1 smoke + B2 harden | Live jobs≥2 completes or documented safe fallback; tests green |
| 3 Track C | C1 scripts + C2 docs | Diff+report demo on two snapshots; checklist in ABC_QA |
| 4 Track A | A1–A3 API/JSON + Vue | Panel renders; skip reason shown in fixture/mock; yarn lint |
| 5 Final QA | ABC_QA.md | A+B+C PASS; prior goldens intact |

Max 20 iterations. Subagents: **composer-2.5**. Local only; no commit/PR unless user asks.
