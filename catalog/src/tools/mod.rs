//! Offline generators for feature presence and GitHub provenance traces.
//!
//! Invoked by bins:
//! - `cargo run -p blueos-catalog --bin generate_feature_presence`
//! - `cargo run -p blueos-catalog --bin enrich_feature_traces`

pub mod feature_presence;
pub mod feature_trace_enrich;
pub mod feature_trace_report;
pub mod shell;
pub mod sibling_matrix;
