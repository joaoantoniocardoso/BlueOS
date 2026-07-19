# Feature-traces next-11 — plan

Orchestrator: `next11/ORCHESTRATOR_PROMPT.xml`  
Memory: `next11/memory.xml`  
Baselines: `precision/PRECISION_QA.md`, `improve/IMPROVE_QA.md` (both PASS)

## Tracks (all in scope)
| ID | Track | Deliverable |
|---|---|---|
| N1 | ConfigureVideoStream hub | File-level OVERRIDES + HUBS; sibling ratios vs other video journeys |
| N2 | --journey enrich merge | Scoped enrich merges into existing feature_traces.json |
| N3 | Local gate wiring | check-traces.sh / gate.sh runs drift + sibling + report goldens |
| N4 | Shared-legit policy | Title/body classifier OR documented accept-shared model + tests |
| N5 | Pickaxe relevance score | Score follow-ups by term hit in title/files; expose in report/matrix |
| N6 | Expand sibling cargo gates | More inventory pairs as hard asserts |
| N7 | Report polish | Richer MD/HTML option; GitHub links; merge_method/squash in timeline |
| N8 | Vue provenance surface | Minimal panel in existing frontend debug/report path if feasible |
| N9 | Presence UI parity | Show present_in_tags/channels with timeline |
| N10 | Enricher resume | Partial progress persist + resume after mid-run failure |
| N11 | Golden corpus growth | +2–3 strict goldens (calibration backport, MODULE_S, …) |

## Phases & QA gates
| Phase | Work | QA gate |
|---|---|---|
| 1 Design | Freeze NEXT11_DESIGN.md | Design accepted |
| 2 Core engine | N1, N2, N10, N5 | cargo test; scoped merge demo; resume smoke |
| 3 Gates & goldens | N3, N6, N11 | check-traces.sh green; new goldens in tests+strict |
| 4 Shared-legit | N4 | NEXT11_AUDIT.md policy + any classifier |
| 5 Surfaces | N7, N8, N9 | report samples + Vue if in-scope |
| 6 Final QA | Full enrich if needed; NEXT11_QA.md | All tracks PASS; prior goldens intact |

Fail Phase 6 → loop Phase 2–5. Max 22 orchestrator iterations.
No commit/PR unless user asks. Local only.
