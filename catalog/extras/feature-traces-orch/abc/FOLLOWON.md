# ABC follow-on — multi-cluster jobs soak + fuller UI snapshot

Orchestrator continues from ABC_QA open items.
Subagents: composer-2.5

## Tracks
| ID | Work |
|---|---|
| F1 | Multi-cluster `--jobs 2` soak (force process ≥2 clusters; no goldens wipe) |
| F2 | Optional full provenance in UI (toggle or load feature-provenance-full.json when present) |
| F3 | Document soak results in abc/B_SMOKE.md; keep ABC_QA addendum |

## QA
- check-traces green after soak
- yarn eslint on touched Vue/TS
- cargo test green
