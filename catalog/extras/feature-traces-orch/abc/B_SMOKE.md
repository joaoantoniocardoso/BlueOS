# ABC F1 — Multi-cluster `--jobs 2` soak

**Date:** 2026-07-19

## Approach

Option 2: backup `feature_traces.json`, surgically remove 2 non-golden `intro_clusters` entries on different intro commits, run one full enrich with `--jobs 2 --resume --strict-goldens` (forces reprocessing without touching goldens).

## Pre-soak backup

```bash
cp catalog/feature_traces.json catalog/feature_traces.json.bak-f1-soak
```

## Removed clusters (non-golden)

| intro_commit (short) | journey |
|---|---|
| `0bec1d64d9b1` | RestoreDefaultFirmware |
| `0efc22001a40` | RunSitlSimulation |

Clusters before removal: 56 → 54.

## Soak command

```bash
cd catalog
cargo run -p blueos-catalog --bin enrich_feature_traces -- --jobs 2 --resume --strict-goldens
```

## Results

| Metric | Value |
|---|---|
| Workers | 2 (`Using parallel enrich with 2 workers…`) |
| Clusters processed | **2** |
| Journeys processed | 2 |
| Clusters skipped (resume) | 54 (92 journeys) |
| Exit code | 0 |
| Runtime | ~142s |
| Rate-limit hits | **0** (no `rate_limit` in stdout) |
| `--strict-goldens` | 8/8 passed |
| Post-soak `intro_clusters` | 56 (both removed clusters restored) |

## Post-soak gates

```bash
cargo run -q -p blueos-catalog --bin feature_trace_report -- --check-goldens
# → all 8 golden journeys match PRECISION_QA.md §1 + NEXT11_DESIGN.md N11
```

## Notes

- Merge path intact: 54 golden/sibling clusters skipped via `--resume`; only the 2 deleted clusters re-enriched in parallel.
- Backup retained at `catalog/feature_traces.json.bak-f1-soak` for rollback if needed.
