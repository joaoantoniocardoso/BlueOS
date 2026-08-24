//! Diffs two `feature_traces.json` snapshots for changelog provenance:
//! journeys added/removed, `intro_commit` changes, `follow_up_prs`/
//! `backport_prs` set diffs per journey, and new/orphaned `intro_clusters`.
//! Text/markdown output only — no HTML diff view, no git-blame integration
//! (P15's non-goals).
//!
//! Invoked by:
//! `cargo run -p blueos-catalog --bin feature_trace_diff -- old.json new.json`

use std::collections::{BTreeSet, HashMap};
use std::fs;

use catalog_data::feature_trace::IntroTraces;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JourneyChange {
    pub journey_id: String,
    pub intro_commit_change: Option<(String, String)>,
    pub follow_up_added: BTreeSet<u64>,
    pub follow_up_removed: BTreeSet<u64>,
    pub backport_added: BTreeSet<u64>,
    pub backport_removed: BTreeSet<u64>,
}

impl JourneyChange {
    fn is_empty(&self) -> bool {
        self.intro_commit_change.is_none()
            && self.follow_up_added.is_empty()
            && self.follow_up_removed.is_empty()
            && self.backport_added.is_empty()
            && self.backport_removed.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceDiff {
    pub journeys_added: Vec<String>,
    pub journeys_removed: Vec<String>,
    pub journey_changes: Vec<JourneyChange>,
    pub clusters_added: Vec<String>,
    pub clusters_removed: Vec<String>,
}

/// This journey's own `follow_up_prs`/`backport_prs` within its intro-commit
/// cluster, looked up by `(traces, journey_id, intro_commit)` rather than
/// `crate::feature_trace::discovery_for_journey` since that helper only
/// reads the process-wide static `feature_traces()`, not an arbitrary parsed
/// snapshot (old/new here are both ad hoc `IntroTraces` values).
fn discovery_sets(
    traces: &IntroTraces,
    journey_id: &str,
    intro_commit: &str,
) -> (BTreeSet<u64>, BTreeSet<u64>) {
    let discovery = traces
        .intro_clusters
        .get(intro_commit)
        .and_then(|cluster| cluster.by_journey.get(journey_id));
    let follow_up = discovery
        .map(|d| d.follow_up_prs.iter().copied().collect())
        .unwrap_or_default();
    let backport = discovery
        .map(|d| d.backport_prs.iter().copied().collect())
        .unwrap_or_default();
    (follow_up, backport)
}

/// Pure diff of two parsed snapshots — no I/O, safe to unit test directly.
pub fn diff(old: &IntroTraces, new: &IntroTraces) -> TraceDiff {
    let old_journeys: HashMap<&str, &str> = old
        .journeys
        .iter()
        .map(|j| (j.journey.as_str(), j.intro_commit.as_str()))
        .collect();
    let new_journeys: HashMap<&str, &str> = new
        .journeys
        .iter()
        .map(|j| (j.journey.as_str(), j.intro_commit.as_str()))
        .collect();

    let mut journeys_added: Vec<String> = new_journeys
        .keys()
        .filter(|id| !old_journeys.contains_key(*id))
        .map(|id| id.to_string())
        .collect();
    journeys_added.sort_unstable();

    let mut journeys_removed: Vec<String> = old_journeys
        .keys()
        .filter(|id| !new_journeys.contains_key(*id))
        .map(|id| id.to_string())
        .collect();
    journeys_removed.sort_unstable();

    let mut common_ids: Vec<&str> = old_journeys
        .keys()
        .filter(|id| new_journeys.contains_key(*id))
        .copied()
        .collect();
    common_ids.sort_unstable();

    let journey_changes = common_ids
        .into_iter()
        .filter_map(|journey_id| {
            let old_intro = old_journeys[journey_id];
            let new_intro = new_journeys[journey_id];
            let (old_follow, old_backport) = discovery_sets(old, journey_id, old_intro);
            let (new_follow, new_backport) = discovery_sets(new, journey_id, new_intro);
            let change = JourneyChange {
                journey_id: journey_id.to_string(),
                intro_commit_change: (old_intro != new_intro)
                    .then(|| (old_intro.to_string(), new_intro.to_string())),
                follow_up_added: &new_follow - &old_follow,
                follow_up_removed: &old_follow - &new_follow,
                backport_added: &new_backport - &old_backport,
                backport_removed: &old_backport - &new_backport,
            };
            (!change.is_empty()).then_some(change)
        })
        .collect();

    let mut clusters_added: Vec<String> = new
        .intro_clusters
        .keys()
        .filter(|sha| !old.intro_clusters.contains_key(*sha))
        .cloned()
        .collect();
    clusters_added.sort_unstable();

    let mut clusters_removed: Vec<String> = old
        .intro_clusters
        .keys()
        .filter(|sha| !new.intro_clusters.contains_key(*sha))
        .cloned()
        .collect();
    clusters_removed.sort_unstable();

    TraceDiff {
        journeys_added,
        journeys_removed,
        journey_changes,
        clusters_added,
        clusters_removed,
    }
}

fn format_pr_set_diff(
    label: &str,
    added: &BTreeSet<u64>,
    removed: &BTreeSet<u64>,
) -> Option<String> {
    if added.is_empty() && removed.is_empty() {
        return None;
    }
    Some(format!(
        "  {label}: +{:?} -{:?}",
        added.iter().collect::<Vec<_>>(),
        removed.iter().collect::<Vec<_>>()
    ))
}

/// Human-readable text report. Pure formatting — no I/O.
pub fn format_diff(trace_diff: &TraceDiff) -> String {
    let mut out = String::from("# Feature trace diff\n\n## Journeys\n");
    for id in &trace_diff.journeys_added {
        out.push_str(&format!("+ {id}\n"));
    }
    for id in &trace_diff.journeys_removed {
        out.push_str(&format!("- {id}\n"));
    }
    if trace_diff.journeys_added.is_empty() && trace_diff.journeys_removed.is_empty() {
        out.push_str("(none)\n");
    }

    out.push_str("\n## Journey changes\n");
    if trace_diff.journey_changes.is_empty() {
        out.push_str("(none)\n");
    }
    for change in &trace_diff.journey_changes {
        out.push_str(&format!("### {}\n", change.journey_id));
        if let Some((old_sha, new_sha)) = &change.intro_commit_change {
            out.push_str(&format!("  intro_commit: {old_sha} -> {new_sha}\n"));
        }
        if let Some(line) = format_pr_set_diff(
            "follow_up_prs",
            &change.follow_up_added,
            &change.follow_up_removed,
        ) {
            out.push_str(&line);
            out.push('\n');
        }
        if let Some(line) = format_pr_set_diff(
            "backport_prs",
            &change.backport_added,
            &change.backport_removed,
        ) {
            out.push_str(&line);
            out.push('\n');
        }
    }

    out.push_str("\n## Intro clusters\n");
    for sha in &trace_diff.clusters_added {
        out.push_str(&format!("+ {sha}\n"));
    }
    for sha in &trace_diff.clusters_removed {
        out.push_str(&format!("- {sha}\n"));
    }
    if trace_diff.clusters_added.is_empty() && trace_diff.clusters_removed.is_empty() {
        out.push_str("(none)\n");
    }

    out
}

fn load(path: &str) -> Result<IntroTraces, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("{path}: failed to parse: {e}"))
}

pub fn run(args: &[String]) -> Result<(), String> {
    let [old_path, new_path] = args else {
        return Err("usage: feature_trace_diff <old.json> <new.json>".to_string());
    };
    let old = load(old_path)?;
    let new = load(new_path)?;
    print!("{}", format_diff(&diff(&old, &new)));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(journeys_and_shas: &[(&str, &str)], clusters: &[&str]) -> IntroTraces {
        let json = serde_json::json!({
            "schema_version": 2,
            "repo": "bluerobotics/BlueOS",
            "source_presence": "feature_presence_map.json",
            "tool": "feature_trace_diff test fixture",
            "generated_at": "2024-01-01T00:00:00Z",
            "commits": {},
            "pull_requests": {},
            "issues": {},
            "intro_clusters": clusters.iter().map(|sha| (sha.to_string(), serde_json::json!({
                "intro_commit": sha,
                "landing_prs": [],
                "squash_merge": false,
                "intro_sha_in_pr_commits": false,
                "merge_commit_sha": null,
                "by_journey": {},
            }))).collect::<serde_json::Map<_, _>>(),
            "journeys": journeys_and_shas.iter().map(|(id, sha)| serde_json::json!({
                "journey": id,
                "module": "test",
                "method": "test",
                "intro_commit": sha,
                "first_tag": null,
                "present_on_master": true,
                "present_on_1_4_dev": false,
                "present_in_tags": [],
            })).collect::<Vec<_>>(),
        });
        serde_json::from_value(json).expect("valid fixture IntroTraces")
    }

    fn fixture_with_discovery(
        journey_id: &str,
        intro_commit: &str,
        follow_up_prs: &[u64],
        backport_prs: &[u64],
    ) -> IntroTraces {
        let json = serde_json::json!({
            "schema_version": 2,
            "repo": "bluerobotics/BlueOS",
            "source_presence": "feature_presence_map.json",
            "tool": "feature_trace_diff test fixture",
            "generated_at": "2024-01-01T00:00:00Z",
            "commits": {},
            "pull_requests": {},
            "issues": {},
            "intro_clusters": {
                intro_commit: {
                    "intro_commit": intro_commit,
                    "landing_prs": [],
                    "squash_merge": false,
                    "intro_sha_in_pr_commits": false,
                    "merge_commit_sha": null,
                    "by_journey": {
                        journey_id: {
                            "discovery_paths": [],
                            "follow_up_prs": follow_up_prs,
                            "backport_prs": backport_prs,
                            "issues": [],
                        },
                    },
                },
            },
            "journeys": [{
                "journey": journey_id,
                "module": "test",
                "method": "test",
                "intro_commit": intro_commit,
                "first_tag": null,
                "present_on_master": true,
                "present_on_1_4_dev": false,
                "present_in_tags": [],
            }],
        });
        serde_json::from_value(json).expect("valid fixture IntroTraces")
    }

    #[test]
    fn diff_reports_added_and_removed_journeys() {
        let old = fixture(&[("Stays", "sha1"), ("Removed", "sha2")], &[]);
        let new = fixture(&[("Stays", "sha1"), ("Added", "sha3")], &[]);
        let trace_diff = diff(&old, &new);
        assert_eq!(trace_diff.journeys_added, vec!["Added"]);
        assert_eq!(trace_diff.journeys_removed, vec!["Removed"]);
    }

    #[test]
    fn diff_reports_intro_commit_change() {
        let old = fixture(&[("Journey", "sha_old")], &[]);
        let new = fixture(&[("Journey", "sha_new")], &[]);
        let trace_diff = diff(&old, &new);
        assert_eq!(trace_diff.journey_changes.len(), 1);
        assert_eq!(
            trace_diff.journey_changes[0].intro_commit_change,
            Some(("sha_old".to_string(), "sha_new".to_string()))
        );
    }

    #[test]
    fn diff_reports_follow_up_and_backport_set_diffs() {
        let old = fixture_with_discovery("Journey", "sha1", &[100, 200], &[]);
        let new = fixture_with_discovery("Journey", "sha1", &[200, 300], &[400]);
        let trace_diff = diff(&old, &new);
        assert_eq!(trace_diff.journey_changes.len(), 1);
        let change = &trace_diff.journey_changes[0];
        assert_eq!(change.follow_up_added, BTreeSet::from([300]));
        assert_eq!(change.follow_up_removed, BTreeSet::from([100]));
        assert_eq!(change.backport_added, BTreeSet::from([400]));
        assert!(change.backport_removed.is_empty());
    }

    #[test]
    fn diff_omits_unchanged_journeys() {
        let old = fixture_with_discovery("Journey", "sha1", &[100], &[]);
        let new = fixture_with_discovery("Journey", "sha1", &[100], &[]);
        let trace_diff = diff(&old, &new);
        assert!(trace_diff.journey_changes.is_empty());
    }

    #[test]
    fn diff_reports_new_and_orphaned_intro_clusters() {
        let old = fixture(&[], &["sha_kept", "sha_orphaned"]);
        let new = fixture(&[], &["sha_kept", "sha_new"]);
        let trace_diff = diff(&old, &new);
        assert_eq!(trace_diff.clusters_added, vec!["sha_new"]);
        assert_eq!(trace_diff.clusters_removed, vec!["sha_orphaned"]);
    }

    #[test]
    fn format_diff_renders_added_removed_and_changes() {
        let old = fixture_with_discovery("Journey", "sha1", &[100], &[]);
        let new = fixture_with_discovery("Journey", "sha1", &[200], &[]);
        let text = format_diff(&diff(&old, &new));
        assert!(text.contains("### Journey"));
        assert!(text.contains("follow_up_prs: +[200] -[100]"));
    }

    #[test]
    fn format_diff_shows_none_placeholders_when_no_changes() {
        let old = fixture(&[("Journey", "sha1")], &[]);
        let new = fixture(&[("Journey", "sha1")], &[]);
        let text = format_diff(&diff(&old, &new));
        assert!(text.contains("## Journeys\n(none)"));
        assert!(text.contains("## Journey changes\n(none)"));
        assert!(text.contains("## Intro clusters\n(none)"));
    }

    #[test]
    fn run_errors_on_wrong_argument_count() {
        assert!(run(&["only-one.json".to_string()]).is_err());
    }
}
