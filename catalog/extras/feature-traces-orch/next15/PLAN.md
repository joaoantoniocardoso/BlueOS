# Feature-traces next15 — plan

Orchestrator: `next15/ORCHESTRATOR_PROMPT.xml`  
Memory: `next15/memory.xml`  
Baselines: precision + improve + next11 QA (all PASS)

## Tracks (all in scope)
| ID | Track | Deliverable |
|---|---|---|
| P1 | Stale cluster cleanup | Drop/orphan intro_clusters when journey intro_commit moves |
| P2 | Scratch cleanup | Remove catalog/examples/n4_check.rs (or promote real examples) |
| P3 | Gate wiring | catalog/gate.sh invokes check-traces.sh |
| P4 | Resume UX | Progress log + skip/resume summary after enrich |
| P5 | Cache schema version | Version stamp for .cache/gh_traces; invalidate on mismatch |
| P6 | Parallel enrich | Safe limited concurrency for cluster enrich + rate-limit awareness |
| P7 | Shared-legit classifiers | Optional title/body classifier OR document keep-accept-shared (design pick) |
| P8 | Live Pickaxe term_hit | Full enrich so Pickaxe journeys have scored follow-ups; report shows hits |
| P9 | Backport audit | Sample/audit backport FPs/FNs; fix or document in AUDIT |
| P10 | HTML artifact bundle | Post-enrich (or script) writes report HTML under extras or catalog/out |
| P11 | Vue / static serve | Static HTML via tools path OR document blocked until API; no fake SPA |
| P12 | Tag cross-links | Report links presence tags ↔ GitHub/git tags where possible |
| P13 | Runner skip by presence | Journey runner skips when feature absent on DUT tag/channel |
| P14 | Skip explanation | Human-readable why-skipped from intro_commit + presence |
| P15 | Traces diff | Bin/script diffs two feature_traces.json for changelog provenance |

## Phases & QA gates
| Phase | Work | QA gate |
|---|---|---|
| 1 Design | NEXT15_DESIGN.md freezes ACs + P7/P11 decisions | Design accepted |
| 2 Hygiene+engine | P1–P6 | cargo test; enrich smoke; cache version test |
| 3 Semantics+audit | P7–P9 + full enrich as needed | NEXT15_AUDIT.md; goldens intact |
| 4 Surfaces+diff | P10–P12, P15 | HTML artifact + diff bin demo |
| 5 Runner | P13–P14 | Runner skip+reason tests |
| 6 Final QA | NEXT15_QA.md; check-traces + gate.sh | All tracks PASS |

Max 22 iterations. No commit/PR unless user asks. Local only.
