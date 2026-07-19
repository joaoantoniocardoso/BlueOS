//! Product-surface timeline export: intro commit → landing PRs → follow-ups →
//! backports → issues → discovery paths, per journey, sourced entirely from
//! `feature_traces.json` (schema_version 2, no schema change).
//!
//! Reuses [`crate::report::ReportTrace`] for the PR/issue number sets; adds
//! intro-commit metadata and `discovery_paths`, which `ReportTrace` doesn't
//! carry (it's scoped to the HTTP journey report schema).
//!
//! Invoked by: `cargo run -p blueos-catalog --bin feature_trace_report`

use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::feature_trace::{
    commit, discovery_for_journey, intro_commit_for_journey, issue, pull_request,
};
use crate::report::ReportTrace;

/// The 5 precision-baseline goldens (`IMPROVE_DESIGN.md` / `PRECISION_QA.md`
/// §1); default journey selection when `--journey` is not given.
pub const GOLDEN_JOURNEY_IDS: &[&str] = &[
    "InspectZenohNetwork",
    "ChangeUiThemeColor",
    "InspectDiskUsage",
    "RunInternetSpeedTest",
    "LevelHorizon",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrRef {
    pub number: u64,
    pub title: Option<String>,
    pub url: Option<String>,
    pub merged_at: Option<String>,
    pub closed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IssueRef {
    pub number: u64,
    pub title: Option<String>,
    pub url: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JourneyTimeline {
    pub journey_id: String,
    pub intro_commit: String,
    pub intro_subject: String,
    pub intro_date: Option<String>,
    pub landing_prs: Vec<PrRef>,
    pub follow_up_prs: Vec<PrRef>,
    pub backport_prs: Vec<PrRef>,
    pub issues: Vec<IssueRef>,
    pub discovery_paths: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Md,
    Both,
}

fn pr_ref(number: u64) -> PrRef {
    match pull_request(number) {
        Some(pr) => PrRef {
            number,
            title: pr.title.clone(),
            url: pr.url.clone(),
            merged_at: pr.merged_at.clone(),
            closed_at: pr.closed_at.clone(),
        },
        None => PrRef {
            number,
            title: None,
            url: None,
            merged_at: None,
            closed_at: None,
        },
    }
}

fn issue_ref(number: u64) -> IssueRef {
    match issue(number) {
        Some(entry) => IssueRef {
            number,
            title: entry.title.clone(),
            url: entry.url.clone(),
            created_at: entry.created_at.clone(),
        },
        None => IssueRef {
            number,
            title: None,
            url: None,
            created_at: None,
        },
    }
}

/// Timeline for one journey id, or `None` if it isn't in `feature_traces.json`.
pub fn build_timeline(journey_id: &str) -> Option<JourneyTimeline> {
    let trace = ReportTrace::for_journey(journey_id)?;
    let intro_sha = intro_commit_for_journey(journey_id).unwrap_or_default();
    let intro = commit(intro_sha);
    let discovery_paths = discovery_for_journey(journey_id)
        .map(|d| d.discovery_paths.clone())
        .unwrap_or_default();
    Some(JourneyTimeline {
        journey_id: journey_id.to_string(),
        intro_commit: intro_sha.to_string(),
        intro_subject: intro.map(|c| c.subject.clone()).unwrap_or_default(),
        intro_date: intro.and_then(|c| c.authored_at.clone()),
        landing_prs: trace.landing_prs.iter().map(|&n| pr_ref(n)).collect(),
        follow_up_prs: trace.follow_up_prs.iter().map(|&n| pr_ref(n)).collect(),
        backport_prs: trace.backport_prs.iter().map(|&n| pr_ref(n)).collect(),
        issues: trace.issue_numbers.iter().map(|&n| issue_ref(n)).collect(),
        discovery_paths,
    })
}

fn short_sha(sha: &str) -> String {
    sha.chars().take(12).collect()
}

fn pr_line(label: &str, pr: &PrRef) -> String {
    format!(
        "- **{label}** #{} — {}",
        pr.number,
        pr.title.as_deref().unwrap_or("(no title)")
    )
}

fn issue_line(entry: &IssueRef) -> String {
    format!(
        "- **Issue** #{} — {}",
        entry.number,
        entry.title.as_deref().unwrap_or("(no title)")
    )
}

fn pr_date(pr: &PrRef) -> Option<String> {
    pr.merged_at.clone().or_else(|| pr.closed_at.clone())
}

/// Ordered `(date, markdown line)` events for one journey's timeline.
/// Kept separate from formatting so the sort is a pure, testable step.
fn timeline_events(timeline: &JourneyTimeline) -> Vec<(Option<String>, String)> {
    let mut events = vec![(
        timeline.intro_date.clone(),
        format!(
            "- **Intro commit** `{}` — {}",
            short_sha(&timeline.intro_commit),
            timeline.intro_subject
        ),
    )];
    events.extend(
        timeline
            .landing_prs
            .iter()
            .map(|pr| (pr_date(pr), pr_line("Landing PR", pr))),
    );
    events.extend(
        timeline
            .follow_up_prs
            .iter()
            .map(|pr| (pr_date(pr), pr_line("Follow-up PR", pr))),
    );
    events.extend(
        timeline
            .backport_prs
            .iter()
            .map(|pr| (pr_date(pr), pr_line("Backport PR", pr))),
    );
    events.extend(
        timeline
            .issues
            .iter()
            .map(|entry| (entry.created_at.clone(), issue_line(entry))),
    );
    // Chronological, undated events last; stable sort keeps each kind's
    // relative insertion order (intro → landing → follow-up → backport → issue)
    // as the tie-break among events sharing a date (or lacking one).
    events.sort_by(|(a, _), (b, _)| match (a, b) {
        (Some(x), Some(y)) => x.cmp(y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    events
}

/// Human-readable markdown timeline for one journey. Pure formatting helper —
/// no I/O, safe to unit test without `feature_traces.json` data.
pub fn format_timeline_markdown(timeline: &JourneyTimeline) -> String {
    let mut out = format!("## {}\n\n", timeline.journey_id);
    for (date, line) in timeline_events(timeline) {
        match date {
            Some(d) => out.push_str(&format!("{line} ({d})\n")),
            None => out.push_str(&format!("{line}\n")),
        }
    }
    out.push_str(&format!(
        "\nDiscovery paths: {}\n",
        if timeline.discovery_paths.is_empty() {
            "(none)".to_string()
        } else {
            timeline.discovery_paths.join(", ")
        }
    ));
    out
}

/// Precision-regression gate (`IMPROVE_DESIGN.md` "Precision regression
/// invariants" / `PRECISION_QA.md` §1): fails loudly if any of the 5 golden
/// journeys' timeline drifts from the locked-in precision baseline. Always
/// checks all 5, independent of `--journey`.
pub fn check_goldens() -> Result<(), String> {
    let mut violations: Vec<String> = vec![];

    match build_timeline("InspectZenohNetwork") {
        None => violations.push("InspectZenohNetwork: journey not found".to_string()),
        Some(t) => {
            let landing: Vec<u64> = t.landing_prs.iter().map(|p| p.number).collect();
            if landing != [3300] {
                violations.push(format!(
                    "InspectZenohNetwork: expected landing_prs == [3300], got {landing:?}"
                ));
            }
            let follow: Vec<u64> = t.follow_up_prs.iter().map(|p| p.number).collect();
            for pr in [3313, 3953] {
                if !follow.contains(&pr) {
                    violations.push(format!("InspectZenohNetwork: follow_up_prs missing #{pr}"));
                }
            }
        }
    }

    match build_timeline("ChangeUiThemeColor") {
        None => violations.push("ChangeUiThemeColor: journey not found".to_string()),
        Some(t) => {
            if !t.follow_up_prs.is_empty() || !t.backport_prs.is_empty() {
                violations.push(
                    "ChangeUiThemeColor: expected empty follow_up_prs and backport_prs".to_string(),
                );
            }
        }
    }

    match build_timeline("InspectDiskUsage") {
        None => violations.push("InspectDiskUsage: journey not found".to_string()),
        Some(t) => {
            let mut follow: Vec<u64> = t.follow_up_prs.iter().map(|p| p.number).collect();
            follow.sort_unstable();
            if follow != [3681, 3691, 3743] {
                violations.push(format!(
                    "InspectDiskUsage: expected follow_up_prs == [3681, 3691, 3743], got {follow:?}"
                ));
            }
        }
    }

    match build_timeline("RunInternetSpeedTest") {
        None => violations.push("RunInternetSpeedTest: journey not found".to_string()),
        Some(t) => {
            let landing: Vec<u64> = t.landing_prs.iter().map(|p| p.number).collect();
            let follow: Vec<u64> = t.follow_up_prs.iter().map(|p| p.number).collect();
            let backport: Vec<u64> = t.backport_prs.iter().map(|p| p.number).collect();
            let issues: Vec<u64> = t.issues.iter().map(|p| p.number).collect();
            if !landing.contains(&3602) {
                violations.push("RunInternetSpeedTest: landing PR #3602 missing".to_string());
            }
            if follow.contains(&3686) || backport.contains(&3686) {
                violations.push(
                    "RunInternetSpeedTest: PR #3686 must be absent from follow_up/backport"
                        .to_string(),
                );
            }
            if !issues.contains(&2146) {
                violations.push("RunInternetSpeedTest: issue #2146 missing".to_string());
            }
        }
    }

    match build_timeline("LevelHorizon") {
        None => violations.push("LevelHorizon: journey not found".to_string()),
        Some(t) => {
            let follow: Vec<u64> = t.follow_up_prs.iter().map(|p| p.number).collect();
            let backport: Vec<u64> = t.backport_prs.iter().map(|p| p.number).collect();
            if !backport.contains(&3867) {
                violations.push("LevelHorizon: backport PR #3867 missing".to_string());
            }
            if follow.contains(&3930) || backport.contains(&3930) {
                violations.push(
                    "LevelHorizon: PR #3930 must be absent from follow_up/backport".to_string(),
                );
            }
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} golden check(s) failed:\n  - {}",
            violations.len(),
            violations.join("\n  - ")
        ))
    }
}

pub fn run(args: &[String]) -> Result<(), String> {
    let mut journey_filter: Option<String> = None;
    let mut format = Format::Json;
    let mut output: Option<String> = None;
    let mut check_goldens_flag = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--journey" => {
                i += 1;
                journey_filter = Some(
                    args.get(i)
                        .ok_or_else(|| "--journey requires a value".to_string())?
                        .clone(),
                );
            }
            "--format" => {
                i += 1;
                let value = args
                    .get(i)
                    .ok_or_else(|| "--format requires a value".to_string())?;
                format = match value.as_str() {
                    "json" => Format::Json,
                    "md" => Format::Md,
                    "both" => Format::Both,
                    other => return Err(format!("unknown --format value: {other}")),
                };
            }
            "--output" => {
                i += 1;
                output = Some(
                    args.get(i)
                        .ok_or_else(|| "--output requires a value".to_string())?
                        .clone(),
                );
            }
            "--check-goldens" => check_goldens_flag = true,
            other => return Err(format!("unknown argument: {other}")),
        }
        i += 1;
    }

    let journey_ids: Vec<&str> = match &journey_filter {
        Some(j) => vec![j.as_str()],
        None => GOLDEN_JOURNEY_IDS.to_vec(),
    };

    let mut timelines = Vec::new();
    for id in &journey_ids {
        let timeline = build_timeline(id)
            .ok_or_else(|| format!("journey not found in feature_traces.json: {id}"))?;
        timelines.push(timeline);
    }

    let json_text = serde_json::to_string_pretty(&timelines).map_err(|e| e.to_string())? + "\n";
    let md_text = timelines
        .iter()
        .map(format_timeline_markdown)
        .collect::<Vec<_>>()
        .join("\n");

    match &output {
        Some(dir) => {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            if matches!(format, Format::Json | Format::Both) {
                let path = Path::new(dir).join("feature_trace_report.json");
                fs::write(&path, &json_text).map_err(|e| e.to_string())?;
                println!("Wrote {}", path.display());
            }
            if matches!(format, Format::Md | Format::Both) {
                let path = Path::new(dir).join("feature_trace_report.md");
                fs::write(&path, &md_text).map_err(|e| e.to_string())?;
                println!("Wrote {}", path.display());
            }
        }
        None => {
            if matches!(format, Format::Json | Format::Both) {
                println!("{json_text}");
            }
            if matches!(format, Format::Md | Format::Both) {
                println!("{md_text}");
            }
        }
    }

    if check_goldens_flag {
        check_goldens().map_err(|e| format!("--check-goldens: {e}"))?;
        println!("--check-goldens: all 5 golden journeys match PRECISION_QA.md §1");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_timeline() -> JourneyTimeline {
        JourneyTimeline {
            journey_id: "InspectDiskUsage".to_string(),
            intro_commit: "abcdef0123456789".to_string(),
            intro_subject: "core: services: disk_usage: Add service".to_string(),
            intro_date: Some("2023-01-01T00:00:00Z".to_string()),
            landing_prs: vec![PrRef {
                number: 3300,
                title: Some("Land disk usage".to_string()),
                url: None,
                merged_at: Some("2023-01-02T00:00:00Z".to_string()),
                closed_at: None,
            }],
            follow_up_prs: vec![PrRef {
                number: 3691,
                title: Some("Fix disk usage bug".to_string()),
                url: None,
                merged_at: Some("2023-01-03T00:00:00Z".to_string()),
                closed_at: None,
            }],
            backport_prs: vec![],
            issues: vec![IssueRef {
                number: 2146,
                title: Some("Disk usage slow".to_string()),
                url: None,
                created_at: Some("2022-12-31T00:00:00Z".to_string()),
            }],
            discovery_paths: vec!["core/services/disk_usage".to_string()],
        }
    }

    #[test]
    fn markdown_orders_events_chronologically() {
        let md = format_timeline_markdown(&sample_timeline());
        let issue_pos = md.find("Issue").expect("issue line present");
        let intro_pos = md.find("Intro commit").expect("intro line present");
        let landing_pos = md.find("Landing PR").expect("landing line present");
        let follow_pos = md.find("Follow-up PR").expect("follow-up line present");
        assert!(
            issue_pos < intro_pos,
            "issue (2022-12-31) should sort before intro (2023-01-01)"
        );
        assert!(intro_pos < landing_pos);
        assert!(landing_pos < follow_pos);
    }

    #[test]
    fn markdown_includes_journey_heading_and_discovery_paths() {
        let md = format_timeline_markdown(&sample_timeline());
        assert!(md.starts_with("## InspectDiskUsage\n"));
        assert!(md.contains("Discovery paths: core/services/disk_usage"));
    }

    #[test]
    fn markdown_handles_missing_dates_and_titles() {
        let mut timeline = sample_timeline();
        timeline.intro_date = None;
        timeline.follow_up_prs[0].title = None;
        timeline.discovery_paths = vec![];
        let md = format_timeline_markdown(&timeline);
        assert!(md.contains("(no title)"));
        assert!(md.contains("Discovery paths: (none)"));
    }

    #[test]
    fn check_goldens_passes_on_real_feature_traces_json() {
        assert_eq!(check_goldens(), Ok(()));
    }

    #[test]
    fn build_timeline_returns_none_for_unknown_journey() {
        assert!(build_timeline("NotARealJourney").is_none());
    }
}
