//! Add a Criterion benchmark beside the code it measures (D-33).

// qual:allow(test_quality, no_sut) reason: "Points at the IDL CDR benchmark the cookbook documents"
#[test]
fn idl_cdr_benchmark_is_the_reference_example() {
    let manifest = include_str!("../../../../libs/idl/Cargo.toml");
    assert!(
        manifest.contains("[[bench]]\nname = \"cdr_codec\""),
        "blueos-idl should declare the cdr_codec benchmark"
    );
    let benchmark = include_str!("../../../../libs/idl/benches/cdr_codec.rs");
    assert!(
        benchmark.contains("criterion_group!"),
        "cdr_codec benchmark should use Criterion"
    );
    assert!(
        benchmark.contains("CommandAck::decode"),
        "cdr_codec benchmark should cover decode"
    );
}
