# Feature-traces improvement tracks — plan

Orchestrator: `improve/ORCHESTRATOR_PROMPT.xml`  
Memory: `improve/memory.xml`  
Prior gate: `precision/PRECISION_QA.md` (PASS) — do not regress goldens/siblings.

## Tracks (all in scope)
| ID | Track | Primary deliverable |
|---|---|---|
| T1 | Measurable precision | Sibling-ratio matrix + optional Pickaxe score; non-tautological gate |
| T2 | Enricher harden | gh 504 retry/backoff; golden fail-loud; cache write-through |
| T3 | Shared-legit semantics | Route/method/title separation for marked inseparable groups |
| T4 | MODULE_S / noisy hubs | Audit + fix thin/noisy journeys (fu empty or fu≫30 with broad paths) |
| T5 | Product surface | Catalog report/API/export timeline (no Vue unless trivial hook exists) |
| T6 | Local hygiene | Drift check script; expand cargo golden+sibling tests; trim process memory from tree or gitignore |
| T7 | Presence ↔ traces | Regenerate presence after OVERRIDES; assert intro_commit parity in tests |

## Phases & QA gates
| Phase | Work | QA gate |
|---|---|---|
| 1 Design | Freeze acceptance criteria per track | `IMPROVE_DESIGN.md` |
| 2 Foundation | T2 + T6 + T1 scoring harness + T7 sync | `cargo test` green; drift script runs; design metrics computed |
| 3 Audit+fix | T4 MODULE_S/hubs; T3 shared-legit experiments | `IMPROVE_AUDIT.md` + targeted enrich spot-checks |
| 4 Surface | T5 report/export | Sample export validates against goldens |
| 5 Final QA | Full enrich if needed; `IMPROVE_QA.md` | All track acceptance criteria PASS; precision goldens still 5/5 |

Fail Phase 5 → loop to Phase 2/3. Max 20 orchestrator iterations.
Do not commit unless user asks. Local project only — no PR.
