//! Sibling-ratio matrix scorer — Rust port of
//! `extras/feature-traces-orch/improve/score_sibling_matrix.sh`.
//!
//! Prints `|intersection|/|union|` of `follow_up_prs` for each pair in the T1
//! matrix (`IMPROVE_DESIGN.md` "T1 sibling-pair matrix"). Exits non-zero if the
//! primary gate (`ConfigureCameraStream` vs `ViewCameraStreams`) is >= 0.40 —
//! the regression this tool exists to catch.
//!
//! Hub path-intersection is never used as a gate here (see
//! `crate::feature_trace::sibling_ratio` doc comment and `PRECISION_QA.md` §2);
//! sibling ratio of `follow_up_prs` is the real over-broad-hint signal.
//!
//! Invoked by: `cargo run -p blueos-catalog --bin sibling_matrix`

use std::collections::HashSet;

use crate::feature_trace::discovery_for_journey;

/// Pairs from `IMPROVE_DESIGN.md` "T1 sibling-pair matrix" (10 pairs; #1 carries
/// the explicit numeric gate, the rest are regression-watch).
pub const PAIRS: &[(&str, &str)] = &[
    ("ConfigureCameraStream", "ViewCameraStreams"),
    ("ConnectToWifiNetwork", "ForgetSavedWifiNetwork"),
    ("ConfigureHotspotCredentials", "ToggleSmartHotspot"),
    (
        "InspectRaspberryEepromBootloader",
        "UpdateRaspberryEepromBootloader",
    ),
    ("AcquireDynamicIpAddress", "DisableOnboardDhcpServer"),
    ("RenameVehicle", "ChangeMdnsHostname"),
    ("BrowseAvailableWebServices", "MonitorInternetConnectivity"),
    (
        "VerifyInternetConnectivity",
        "ProbeInterfaceInternetConnectivity",
    ),
    ("UpdateBootstrapImage", "DeleteLocalBlueosVersion"),
    ("StartAutopilot", "UpdateFirmwareOnline"),
];

const CAMERA_GATE_PAIR: (&str, &str) = ("ConfigureCameraStream", "ViewCameraStreams");
const CAMERA_GATE_THRESHOLD: f64 = 0.40;

pub struct PairScore {
    pub journey_a: &'static str,
    pub journey_b: &'static str,
    pub intersection: usize,
    pub union: usize,
    pub ratio: f64,
}

fn follow_up_set(journey_id: &str) -> HashSet<u64> {
    discovery_for_journey(journey_id)
        .map(|d| d.follow_up_prs.iter().copied().collect())
        .unwrap_or_default()
}

pub fn score_pair(journey_a: &'static str, journey_b: &'static str) -> PairScore {
    let a = follow_up_set(journey_a);
    let b = follow_up_set(journey_b);
    let intersection = a.intersection(&b).count();
    let union = a.union(&b).count();
    let ratio = if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    };
    PairScore {
        journey_a,
        journey_b,
        intersection,
        union,
        ratio,
    }
}

pub fn score_matrix() -> Vec<PairScore> {
    PAIRS.iter().map(|(a, b)| score_pair(a, b)).collect()
}

pub fn format_row(score: &PairScore) -> String {
    format!(
        "{:<40} {:<40} {:>6} {:>6} {:>8.3}",
        score.journey_a, score.journey_b, score.intersection, score.union, score.ratio
    )
}

/// `Some(ratio)` when the camera gate pair is present in `scores` and its
/// ratio breached the threshold.
pub fn camera_gate_violation(scores: &[PairScore]) -> Option<f64> {
    scores
        .iter()
        .find(|s| (s.journey_a, s.journey_b) == CAMERA_GATE_PAIR)
        .filter(|s| s.ratio >= CAMERA_GATE_THRESHOLD)
        .map(|s| s.ratio)
}

/// Runs the matrix, printing the table to stdout. Returns the process exit code.
pub fn run() -> i32 {
    let scores = score_matrix();
    println!(
        "{:<40} {:<40} {:>6} {:>6} {:>8}",
        "journey A", "journey B", "inter", "union", "ratio"
    );
    for score in &scores {
        println!("{}", format_row(score));
    }
    match camera_gate_violation(&scores) {
        Some(ratio) => {
            eprintln!(
                "GATE FAILED: {}/{} ratio {ratio:.3} >= {CAMERA_GATE_THRESHOLD}",
                CAMERA_GATE_PAIR.0, CAMERA_GATE_PAIR.1
            );
            1
        }
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_has_all_ten_designed_pairs() {
        assert_eq!(PAIRS.len(), 10);
        assert_eq!(score_matrix().len(), 10);
    }

    #[test]
    fn camera_pair_passes_gate_on_real_data() {
        let scores = score_matrix();
        assert!(camera_gate_violation(&scores).is_none());
    }

    #[test]
    fn camera_gate_violation_detects_breach() {
        let scores = vec![PairScore {
            journey_a: "ConfigureCameraStream",
            journey_b: "ViewCameraStreams",
            intersection: 4,
            union: 10,
            ratio: 0.40,
        }];
        assert_eq!(camera_gate_violation(&scores), Some(0.40));
    }
}
