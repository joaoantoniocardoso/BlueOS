//! Compact feature-provenance snapshot for the Vue tools page.
//!
//! Invoked by: `cargo run -p blueos-catalog --bin export_feature_provenance`

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::feature_trace_report::{build_timeline, IssueRef, PrRef, GOLDEN_JOURNEY_IDS};
use catalog_data::feature_intro::presence_for_journey;
use catalog_data::feature_trace::feature_traces;
use catalog_kernel::version::{availability_skip, format_availability_skip_reason, Availability};

pub const SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_REFERENCE_TAGS: &[&str] = &["master", "1.4-dev"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CompactPr {
    pub number: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CompactIssue {
    pub number: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JourneyPresence {
    pub present_in_tags: Vec<String>,
    pub present_on_master: bool,
    pub present_on_1_4_dev: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JourneyAvailability {
    pub intro_commit: String,
    pub present_in_tags: Vec<String>,
    pub present_on_master: bool,
    pub present_on_1_4_dev: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JourneyProvenance {
    pub id: String,
    pub intro_commit: String,
    pub discovery_paths: Vec<String>,
    pub landing_prs: Vec<CompactPr>,
    pub follow_up_prs: Vec<CompactPr>,
    pub backport_prs: Vec<CompactPr>,
    pub issues: Vec<CompactIssue>,
    pub presence: JourneyPresence,
    pub availability: JourneyAvailability,
    pub skip_reasons: BTreeMap<String, Option<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProvenanceSnapshot {
    pub schema_version: u32,
    pub generated_at: String,
    pub reference_dut_tags: Vec<String>,
    pub journeys: Vec<JourneyProvenance>,
}

pub fn collect_reference_dut_tags() -> Vec<String> {
    DEFAULT_REFERENCE_TAGS
        .iter()
        .map(|tag| (*tag).to_string())
        .collect()
}

fn pr_url(number: u64, url: Option<&String>) -> String {
    url.cloned().unwrap_or_else(|| {
        format!(
            "https://github.com/{}/pull/{}",
            feature_traces().repo,
            number
        )
    })
}

fn issue_url(number: u64, url: Option<&String>) -> String {
    url.cloned().unwrap_or_else(|| {
        format!(
            "https://github.com/{}/issues/{}",
            feature_traces().repo,
            number
        )
    })
}

fn compact_pr(pr: &PrRef) -> CompactPr {
    CompactPr {
        number: pr.number,
        title: pr.title.clone(),
        url: pr_url(pr.number, pr.url.as_ref()),
    }
}

fn compact_issue(issue: &IssueRef) -> CompactIssue {
    CompactIssue {
        number: issue.number,
        title: issue.title.clone(),
        url: issue_url(issue.number, issue.url.as_ref()),
    }
}

pub fn skip_reasons_for_tags(
    availability: Availability,
    reference_tags: &[String],
) -> BTreeMap<String, Option<String>> {
    reference_tags
        .iter()
        .map(|tag| {
            let reason = availability_skip(tag, &availability)
                .map(|skip| format_availability_skip_reason(&skip, tag));
            (tag.clone(), reason)
        })
        .collect()
}

pub fn journey_provenance(
    journey_id: &str,
    reference_tags: &[String],
) -> Option<JourneyProvenance> {
    let timeline = build_timeline(journey_id)?;
    let availability = presence_for_journey(journey_id)?;
    let presence = JourneyPresence {
        present_in_tags: timeline.present_in_tags.clone(),
        present_on_master: timeline.present_on_master,
        present_on_1_4_dev: timeline.present_on_1_4_dev,
    };
    Some(JourneyProvenance {
        id: journey_id.to_string(),
        intro_commit: timeline.intro_commit.clone(),
        discovery_paths: timeline.discovery_paths.clone(),
        landing_prs: timeline.landing_prs.iter().map(compact_pr).collect(),
        follow_up_prs: timeline.follow_up_prs.iter().map(compact_pr).collect(),
        backport_prs: timeline.backport_prs.iter().map(compact_pr).collect(),
        issues: timeline.issues.iter().map(compact_issue).collect(),
        availability: JourneyAvailability {
            intro_commit: timeline.intro_commit.clone(),
            present_in_tags: presence.present_in_tags.clone(),
            present_on_master: presence.present_on_master,
            present_on_1_4_dev: presence.present_on_1_4_dev,
        },
        presence,
        skip_reasons: skip_reasons_for_tags(availability, reference_tags),
    })
}

pub fn build_snapshot_for_journeys(journey_ids: &[&str]) -> ProvenanceSnapshot {
    let reference_dut_tags = collect_reference_dut_tags();
    let mut journeys: Vec<JourneyProvenance> = journey_ids
        .iter()
        .filter_map(|id| journey_provenance(id, &reference_dut_tags))
        .collect();
    journeys.sort_by(|a, b| a.id.cmp(&b.id));
    ProvenanceSnapshot {
        schema_version: SCHEMA_VERSION,
        generated_at: feature_traces().generated_at.clone(),
        reference_dut_tags,
        journeys,
    }
}

pub fn build_snapshot() -> ProvenanceSnapshot {
    let journey_ids: Vec<&str> = feature_traces()
        .journeys
        .iter()
        .map(|journey| journey.journey.as_str())
        .collect();
    build_snapshot_for_journeys(&journey_ids)
}

pub fn build_golden_snapshot() -> ProvenanceSnapshot {
    build_snapshot_for_journeys(GOLDEN_JOURNEY_IDS)
}

pub fn write_snapshot(path: &Path, snapshot: &ProvenanceSnapshot) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("create {}: {err}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(snapshot)
        .map_err(|err| format!("serialize feature provenance: {err}"))?;
    fs::write(path, json).map_err(|err| format!("write {}: {err}", path.display()))
}

fn default_out_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../core/frontend/public/assets/feature-provenance.json")
}

fn default_full_out_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../core/frontend/public/assets/feature-provenance-full.json")
}

pub fn run(args: &[String]) -> Result<(), String> {
    let mut out = default_out_path();
    let mut full_out = default_full_out_path();
    let write_golden = true;
    let mut write_full = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--out" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--out requires a path".to_string())?;
                out = PathBuf::from(value);
            }
            "--full-out" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "--full-out requires a path".to_string())?;
                full_out = PathBuf::from(value);
            }
            "--full" => {
                write_full = true;
            }
            "--help" | "-h" => {
                eprintln!(
                    "usage: export_feature_provenance [--out PATH] [--full] [--full-out PATH]\n\
                     default: golden journeys → feature-provenance.json;\n\
                     --full: also write all journeys → feature-provenance-full.json (gitignored)"
                );
                return Ok(());
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        index += 1;
    }
    if write_golden {
        let golden = build_golden_snapshot();
        write_snapshot(&out, &golden)?;
        eprintln!(
            "wrote {} ({} golden journeys)",
            out.display(),
            golden.journeys.len()
        );
    }
    if write_full {
        let full = build_snapshot();
        write_snapshot(&full_out, &full)?;
        eprintln!(
            "wrote {} ({} journeys)",
            full_out.display(),
            full.journeys.len()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use catalog_data::feature_trace::feature_traces;
    use catalog_kernel::id::journey::JourneyId;

    #[test]
    fn build_snapshot_has_nonempty_journeys() {
        let snapshot = build_snapshot();
        assert!(!snapshot.journeys.is_empty());
        assert_eq!(snapshot.journeys.len(), feature_traces().journeys.len());
        assert_eq!(snapshot.schema_version, SCHEMA_VERSION);
        assert_eq!(snapshot.reference_dut_tags, vec!["master", "1.4-dev"]);
    }

    #[test]
    fn golden_snapshot_has_eight_journeys() {
        let snapshot = build_golden_snapshot();
        assert_eq!(snapshot.journeys.len(), GOLDEN_JOURNEY_IDS.len());
    }

    #[test]
    fn inspect_disk_usage_golden_fields() {
        let snapshot = build_snapshot();
        let journey = snapshot
            .journeys
            .iter()
            .find(|entry| entry.id == JourneyId::InspectDiskUsage.as_str())
            .expect("inspect_disk_usage");
        assert_eq!(
            journey.intro_commit,
            "c29e24679e3dbe083ab6a3f21bb1459e66bb4dd5"
        );
        assert!(journey.presence.present_on_master);
        assert!(!journey.presence.present_on_1_4_dev);
        assert!(journey
            .availability
            .present_in_tags
            .contains(&"1.4.4-beta.16".to_string()));
        assert_eq!(
            journey
                .follow_up_prs
                .iter()
                .map(|pr| pr.number)
                .collect::<Vec<_>>(),
            vec![3681, 3691, 3743]
        );
        assert!(!journey.landing_prs.is_empty());
        assert!(!journey.discovery_paths.is_empty());
        assert!(journey
            .landing_prs
            .iter()
            .all(|pr| pr.url.starts_with("https://github.com/")));
        let skip_14dev = journey.skip_reasons.get("1.4-dev").expect("1.4-dev key");
        assert!(skip_14dev.is_some(), "expected skip on 1.4-dev");
        assert!(journey
            .skip_reasons
            .get("master")
            .expect("master key")
            .is_none());
    }

    #[test]
    fn skip_reasons_shape_matches_reference_tags() {
        let snapshot = build_snapshot();
        for journey in &snapshot.journeys {
            assert_eq!(journey.skip_reasons.len(), 2);
            assert!(journey.skip_reasons.contains_key("master"));
            assert!(journey.skip_reasons.contains_key("1.4-dev"));
        }
    }
}
