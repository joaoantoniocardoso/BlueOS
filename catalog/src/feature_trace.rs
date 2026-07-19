//! GitHub feature provenance traces (landing / backport / follow-up PRs).
//!
//! Data lives in `catalog/feature_traces.json` (schema_version 2), produced by:
//! `cargo run -p blueos-catalog --bin enrich_feature_traces`
//! (after `cargo run -p blueos-catalog --bin generate_feature_presence`).
//! Loaded as first-class typed data via [`feature_traces`].

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde::Deserialize;

const FEATURE_TRACES_JSON: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/feature_traces.json"));

#[derive(Debug, Clone, Deserialize)]
pub struct FeatureTraces {
    pub schema_version: u32,
    pub repo: String,
    pub source_presence: String,
    pub tool: String,
    pub generated_at: String,
    pub commits: HashMap<String, TraceCommit>,
    pub pull_requests: HashMap<String, TracePullRequest>,
    pub issues: HashMap<String, TraceIssue>,
    pub intro_clusters: HashMap<String, IntroCluster>,
    pub journeys: Vec<TraceJourneyRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TraceCommit {
    pub sha: String,
    pub short: String,
    pub subject: String,
    pub author_name: Option<String>,
    pub author_email: Option<String>,
    pub authored_at: Option<String>,
    #[serde(default)]
    pub body: String,
    pub url: String,
    #[serde(default)]
    pub files_changed: Vec<String>,
    #[serde(default)]
    pub files_changed_truncated: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TracePullRequest {
    pub number: u64,
    pub title: Option<String>,
    pub url: Option<String>,
    pub state: Option<String>,
    pub author: Option<String>,
    pub merged_at: Option<String>,
    pub closed_at: Option<String>,
    pub base_ref: Option<String>,
    pub head_ref: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub body_truncated: bool,
    #[serde(default)]
    pub commit_shas: Vec<String>,
    #[serde(default)]
    pub commit_headlines: Vec<String>,
    #[serde(default)]
    pub files_changed: Vec<String>,
    #[serde(default)]
    pub files_changed_truncated: bool,
    pub merge_commit_sha: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TraceIssue {
    pub number: u64,
    pub title: Option<String>,
    pub url: Option<String>,
    pub state: Option<String>,
    pub author: Option<String>,
    pub created_at: Option<String>,
    pub closed_at: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub missing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSourceKind {
    Closing,
    Body,
    Commit,
    Timeline,
    Search,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IssueSource {
    pub kind: IssueSourceKind,
    pub pr: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClusterIssueRef {
    pub number: u64,
    #[serde(default)]
    pub sources: Vec<IssueSource>,
}

fn default_merge_method() -> String {
    "unknown".to_string()
}

/// Per-journey discovery within a shared `IntroCluster`: two journeys can
/// share an `intro_commit` (bootstrap-era multi-module commits, or several
/// journeys landing via one module-wide commit) while needing independent
/// follow-up/backport/issue discovery scoped to each journey's own module —
/// see `catalog/extras/feature-traces-orch/BOOTSTRAP_SPLIT.md`.
#[derive(Debug, Clone, Deserialize)]
pub struct JourneyDiscovery {
    #[serde(default)]
    pub discovery_paths: Vec<String>,
    #[serde(default)]
    pub follow_up_prs: Vec<u64>,
    #[serde(default)]
    pub backport_prs: Vec<u64>,
    #[serde(default)]
    pub issues: Vec<ClusterIssueRef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntroCluster {
    pub intro_commit: String,
    #[serde(default)]
    pub landing_prs: Vec<u64>,
    pub squash_merge: bool,
    #[serde(default = "default_merge_method")]
    pub merge_method: String,
    pub intro_sha_in_pr_commits: bool,
    pub merge_commit_sha: Option<String>,
    #[serde(default)]
    pub by_journey: HashMap<String, JourneyDiscovery>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TraceJourneyRef {
    pub journey: String,
    pub module: String,
    pub method: String,
    pub intro_commit: String,
    pub first_tag: Option<String>,
    pub present_on_master: bool,
    pub present_on_1_4_dev: bool,
    #[serde(default)]
    pub present_in_tags: Vec<String>,
}

static FEATURE_TRACES: OnceLock<FeatureTraces> = OnceLock::new();

/// Parsed `feature_traces.json` (loaded once).
pub fn feature_traces() -> &'static FeatureTraces {
    FEATURE_TRACES.get_or_init(|| {
        serde_json::from_str(FEATURE_TRACES_JSON).unwrap_or_else(|error| {
            panic!(
                "invalid catalog/feature_traces.json (run: cargo run -p blueos-catalog --bin enrich_feature_traces): {error}"
            )
        })
    })
}

/// Intro-commit cluster for a journey id (e.g. `"InspectZenohNetwork"`).
pub fn cluster_for_journey(journey_id: &str) -> Option<&'static IntroCluster> {
    let traces = feature_traces();
    let journey = traces.journeys.iter().find(|j| j.journey == journey_id)?;
    traces.intro_clusters.get(&journey.intro_commit)
}

/// This journey's own discovery (paths/follow-ups/backports/issues) within
/// its intro-commit cluster.
pub fn discovery_for_journey(journey_id: &str) -> Option<&'static JourneyDiscovery> {
    cluster_for_journey(journey_id)?.by_journey.get(journey_id)
}

/// Intro commit sha recorded in the traces map for a journey.
pub fn intro_commit_for_journey(journey_id: &str) -> Option<&'static str> {
    feature_traces()
        .journeys
        .iter()
        .find(|j| j.journey == journey_id)
        .map(|j| j.intro_commit.as_str())
}

pub fn pull_request(number: u64) -> Option<&'static TracePullRequest> {
    feature_traces().pull_requests.get(&number.to_string())
}

pub fn issue(number: u64) -> Option<&'static TraceIssue> {
    feature_traces().issues.get(&number.to_string())
}

pub fn commit(sha: &str) -> Option<&'static TraceCommit> {
    feature_traces().commits.get(sha)
}

/// Primary (first) landing PR for a journey, if any.
pub fn landing_pr_for_journey(journey_id: &str) -> Option<&'static TracePullRequest> {
    let cluster = cluster_for_journey(journey_id)?;
    let number = *cluster.landing_prs.first()?;
    pull_request(number)
}

/// Sibling separation ratio for a journey pair at risk of over-broad
/// discovery hints re-merging their follow-up history: `|∩|/|∪|` of
/// `follow_up_prs`. See the T1 matrix in
/// `catalog/extras/feature-traces-orch/improve/IMPROVE_DESIGN.md`.
///
/// Hub path-intersection (whether a follow-up PR's `files_changed` overlaps
/// `discovery_paths`) is never used as a gate here or anywhere in this crate
/// — every discovered follow-up trivially intersects its own hint set by
/// construction (it was found via `git log -- <discovery_paths>`), so that
/// check only validates internal consistency, not hint precision. Sibling
/// ratio is the real over-broad-hint signal (see `PRECISION_QA.md` §2).
pub fn sibling_ratio(journey_a: &str, journey_b: &str) -> f64 {
    let a: HashSet<u64> = discovery_for_journey(journey_a)
        .map(|d| d.follow_up_prs.iter().copied().collect())
        .unwrap_or_default();
    let b: HashSet<u64> = discovery_for_journey(journey_b)
        .map(|d| d.follow_up_prs.iter().copied().collect())
        .unwrap_or_default();
    let union: HashSet<u64> = a.union(&b).copied().collect();
    if union.is_empty() {
        return 0.0;
    }
    a.intersection(&b).count() as f64 / union.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_schema_v2() {
        let traces = feature_traces();
        assert_eq!(traces.schema_version, 2);
        assert_eq!(traces.repo, "bluerobotics/BlueOS");
        assert!(!traces.generated_at.is_empty());
        assert_eq!(traces.journeys.len(), 94);
        assert_eq!(traces.intro_clusters.len(), 55);
    }

    #[test]
    fn zenoh_landing_and_follow_ups() {
        let cluster = cluster_for_journey("InspectZenohNetwork").expect("zenoh cluster");
        assert_eq!(cluster.landing_prs, vec![3300]);
        let discovery = discovery_for_journey("InspectZenohNetwork").expect("zenoh discovery");
        assert!(discovery.backport_prs.is_empty());
        assert!(discovery.follow_up_prs.contains(&3313));
        assert!(discovery.follow_up_prs.contains(&3953));
        assert!(!cluster.intro_sha_in_pr_commits);
        let pr = landing_pr_for_journey("InspectZenohNetwork").expect("landing pr");
        assert!(pr.title.as_deref().unwrap_or("").contains("zenoh"));
        assert!(!pr.body.is_empty());
        assert!(!pr.files_changed.is_empty());
    }

    #[test]
    fn customization_has_no_backport() {
        let cluster = cluster_for_journey("ChangeUiThemeColor").expect("customization");
        assert_eq!(cluster.landing_prs, vec![3930]);
        let discovery =
            discovery_for_journey("ChangeUiThemeColor").expect("customization discovery");
        assert!(discovery.backport_prs.is_empty());
        assert!(discovery.follow_up_prs.is_empty());
    }

    #[test]
    fn internet_speed_links_closing_issue() {
        let cluster = cluster_for_journey("RunInternetSpeedTest").expect("pardal cluster");
        assert_eq!(cluster.landing_prs, vec![3602]);
        let discovery = discovery_for_journey("RunInternetSpeedTest").expect("pardal discovery");
        assert!(discovery.issues.iter().any(|i| i.number == 2146));
        assert!(!discovery.follow_up_prs.contains(&3686));
        let issue = issue(2146).expect("issue 2146");
        assert!(issue
            .title
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .contains("speed"));
    }

    #[test]
    fn disk_usage_follow_ups_exact_set() {
        let discovery = discovery_for_journey("InspectDiskUsage").expect("disk usage discovery");
        let follow_ups: HashSet<u64> = discovery.follow_up_prs.iter().copied().collect();
        assert_eq!(follow_ups, HashSet::from([3681, 3691, 3743]));
    }

    #[test]
    fn level_horizon_backport_no_customization_leak() {
        let discovery = discovery_for_journey("LevelHorizon").expect("level horizon discovery");
        assert!(discovery.backport_prs.contains(&3867));
        assert!(!discovery.follow_up_prs.contains(&3930));
        assert!(!discovery.backport_prs.contains(&3930));
    }

    #[test]
    fn camera_pair_sibling_ratio_below_gate() {
        let ratio = sibling_ratio("ConfigureCameraStream", "ViewCameraStreams");
        assert!(ratio < 0.40, "camera sibling ratio {ratio} >= 0.40 gate");
    }

    #[test]
    fn autopilot_pair_sibling_ratio_below_gate() {
        let ratio = sibling_ratio("StartAutopilot", "UpdateFirmwareOnline");
        assert!(ratio < 0.20, "autopilot sibling ratio {ratio} >= 0.20 gate");
    }

    #[test]
    fn helper_pair_sibling_ratio_below_gate() {
        let ratio = sibling_ratio("BrowseAvailableWebServices", "MonitorInternetConnectivity");
        assert!(ratio < 0.40, "helper sibling ratio {ratio} >= 0.40 gate");
    }

    #[test]
    fn presence_and_traces_share_intro_commits() {
        use crate::feature_intro::presence_for_journey;

        for journey_id in [
            "InspectZenohNetwork",
            "InspectDiskUsage",
            "LevelHorizon",
            "ViewCameraStreams",
            "AccessWebTerminal",
        ] {
            let presence = presence_for_journey(journey_id).expect("presence");
            let intro = intro_commit_for_journey(journey_id).expect("trace intro");
            assert_eq!(presence.intro_commit, intro);
        }
    }
}
