//! CDR codec round-trip property tests (L2, D-33).

#[path = "common/cdr_proptest.rs"]
mod cdr_proptest_helpers;

mod cdr_proptest_generated {
    include!("generated/cdr_proptest_strategies.rs");
    include!("generated/cdr_proptest_cases.rs");
}
