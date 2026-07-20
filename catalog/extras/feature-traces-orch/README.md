# Feature traces — Rust generators

Pipeline (pure Rust):

```bash
cargo run -p blueos-catalog --bin generate_feature_presence
cargo run -p blueos-catalog --bin enrich_feature_traces
```

Runtime: `catalog/src/feature_trace.rs` loads `feature_traces.json` via `include_str!`.

Implementation lives in `catalog/src/tools/feature_{presence,trace_enrich}.rs`.

To regenerate the static HTML report bundle in `reports/` (gitignored, P10/P11):
`cargo run -p blueos-catalog --bin feature_trace_report -- --format html --output catalog/extras/feature-traces-orch/reports`

## Release & QA (ABC phase)

Operator checklist and final QA gates: [`abc/RELEASE_CHECKLIST.md`](abc/RELEASE_CHECKLIST.md), [`abc/ABC_QA.md`](abc/ABC_QA.md).

Tag-to-tag diff workflow: `bash abc/release_traces_workflow.sh --old snapshots/traces-<old>.json`
