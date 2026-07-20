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

## Full soak

**Date:** 2026-07-19

### Pre-soak backup

```bash
cp catalog/feature_traces.json catalog/feature_traces.json.bak-full-soak
```

### Forced work (initial run was 0 processed)

Removed 5 non-golden `intro_clusters` (56 → 51) on different intro commits:

| intro_commit (short) | journey |
|---|---|
| `0bec1d64d9b1` | RestoreDefaultFirmware |
| `0efc22001a40` | RunSitlSimulation |
| `11b310ea575a` | ConfigureVideoStream |
| `120af6bfee7e` | UpdateRaspberryEepromBootloader |
| `11088c7b8afe` | DetectMotorDirections |

Golden clusters untouched (InspectZenohNetwork, ChangeUiThemeColor, InspectDiskUsage, RunInternetSpeedTest, LevelHorizon, AccessWebTerminal, InspectMavlinkMessagesInBrowser, CalibrateGyroscope).

### Soak command

```bash
cd catalog
cargo run -p blueos-catalog --bin enrich_feature_traces --release -- --jobs 2 --resume --strict-goldens
```

### Results

| Metric | Value |
|---|---|
| Workers | 2 (`Using parallel enrich with 2 workers…`) |
| Clusters processed | **5** |
| Journeys processed | 5 |
| Clusters skipped (resume) | 51 (89 journeys) |
| Exit code | 0 |
| Wall time | **238s** |
| Rate-limit hits | **0** (no `rate_limit` in stdout) |
| `--strict-goldens` | 8/8 passed |
| Post-soak `intro_clusters` | 56 (all 5 removed clusters restored) |

### Post-soak gates

```bash
cargo run -p blueos-catalog --bin feature_trace_report -- --check-goldens
# → all 8 golden journeys match PRECISION_QA.md §1 + NEXT11_DESIGN.md N11

bash catalog/extras/feature-traces-orch/next11/check-traces.sh
# → ALL FEATURE-TRACES GATES GREEN
```

### Notes

- Initial `--resume` pass: 56 skipped, 0 processed (everything already enriched).
- Parallel merge path clean: 51 golden/sibling clusters skipped; 5 deleted clusters re-enriched concurrently with `--jobs 2`.
- No merge/corruption issues; cluster count and goldens intact vs backup.
- Backup retained at `catalog/feature_traces.json.bak-full-soak`.
