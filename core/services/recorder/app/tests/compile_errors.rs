//! Each endpoint mistake is a compile error that names the endpoint (D-26). Each case's expected error is the
//! `.stderr` next to it; after a deliberate change, or a Rust release that rewords an error, regenerate them with
//! `TRYBUILD=overwrite cargo test -p blueos-recorder-app --test compile_errors`.

#[test]
fn endpoint_mistakes_are_compile_errors_that_name_the_endpoint() {
    trybuild::TestCases::new().compile_fail("tests/compile_fail/*.rs");
}
