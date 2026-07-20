# Feature-traces release checklist

Operator flow for catalog maintainers after enriching `feature_traces.json`.

## 1. Enrich (network + `gh` auth)

```bash
cd catalog
cargo run -p blueos-catalog --bin enrich_feature_traces -- --strict-goldens
```

Optional parallel smoke first (`--jobs 2 --resume` on a filtered `--journey` set) — see `abc/ABC_DESIGN.md` track B.

## 2. Optional snapshot copy

Before tagging a release, archive the committed traces file beside reports (gitignored):

```bash
mkdir -p extras/feature-traces-orch/snapshots
tag=1.4.0   # or: tag=$(git describe --tags --always)
cp feature_traces.json "extras/feature-traces-orch/snapshots/traces-${tag}.json"
```

Use the same `traces-<tag>.json` name when comparing across releases.

## 3. Release workflow (report + diff)

```bash
cd catalog
bash extras/feature-traces-orch/abc/release_traces_workflow.sh \
  --out-dir extras/feature-traces-orch/reports \
  --old extras/feature-traces-orch/snapshots/traces-1.3.2.json
```

Flags:

- `--out-dir` — HTML output directory (default: `extras/feature-traces-orch/reports`)
- `--old PATH` — diff `PATH` against current `feature_traces.json`
- `--journeys golden|all|list` — `golden` (8 default goldens, one HTML), `all` (per-journey HTML), or `list J1,J2`

Review `reports/feature_trace_report.html` and the diff stdout.

## 4. Local feature-traces gate

```bash
bash extras/feature-traces-orch/next11/check-traces.sh
```

Runs drift check, sibling matrix, and `feature_trace_report --check-goldens` (offline).

## 5. Full catalog gate

```bash
bash gate.sh
```

From `catalog/`: fmt, clippy, test, extract, drift, export, and check-traces when present.

## Tag example

```bash
tag=1.4.0
cp feature_traces.json "extras/feature-traces-orch/snapshots/traces-${tag}.json"
bash extras/feature-traces-orch/abc/release_traces_workflow.sh \
  --old "extras/feature-traces-orch/snapshots/traces-1.3.2.json"
git tag "$tag"
```
