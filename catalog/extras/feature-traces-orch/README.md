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
