# Feature-trace pristine precision — plan

Orchestrator: `precision/ORCHESTRATOR_PROMPT.xml`  
Memory: `precision/memory.xml`

## Goal
Per-journey discovery follow-ups that are thematically precise (≥90% file
intersection with journey hints on hub samples), without breaking goldens.

## Phases & QA gates
| Phase | Work | QA gate |
|---|---|---|
| 1 Inventory | Map wide journeys → distinct files; MODULE_S gaps | `PRECISION_INVENTORY.md` |
| 2 Design | Freeze hints table + MODULE_S + metrics | `PRECISION_DESIGN.md` |
| 3 Implement | Wire hints + unit tests | `cargo test` feature_trace + incidental |
| 4 Enrich+QA | Full enrich + precision scores | `PRECISION_QA.md` goldens + ≥90% hub score |
| 5 Harden | Fix FPs/FNs from QA | Final enrich; docs frozen |

## Loop
Fail Phase 4/5 → return to Phase 2/3. Max 15 orchestrator iterations.
