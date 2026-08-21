use serde::Serialize;

use crate::feature_trace::{cluster_for_journey, discovery_for_journey};
use crate::id::JourneyId;
use crate::runner::{DutVersion, JourneyResult, RunCounts, StepResult};
use crate::version::FeatureAvailability;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReportDut {
    pub repository: String,
    pub tag: String,
    pub digest: String,
}

impl ReportDut {
    pub fn from_dut(dut: &DutVersion) -> Self {
        Self {
            repository: dut.repository.clone(),
            tag: dut.tag.clone(),
            digest: normalize_digest(dut.digest.as_deref()),
        }
    }
}

fn normalize_digest(digest: Option<&str>) -> String {
    match digest {
        Some(sha) if sha.starts_with("sha256:") => sha.to_string(),
        Some(sha) => format!("sha256:{sha}"),
        None => "sha256:unknown".to_string(),
    }
}

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictKind {
    ProductMissingReject,
    CatalogWrongStatus,
    NotApplicable,
    HarnessGap,
    Limitation,
    EffectNotApplied,
    ClientDesync,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReportConflict {
    pub kind: ConflictKind,
    pub context: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuiteKind {
    Smoke,
    MutatingSmoke,
    Negative,
    Ui,
    Full,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ReportCounts {
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub unasserted: usize,
}

impl From<RunCounts> for ReportCounts {
    fn from(counts: RunCounts) -> Self {
        Self {
            passed: counts.passed,
            failed: counts.failed,
            skipped: counts.skipped,
            unasserted: counts.unasserted,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum JourneyReportResult {
    Pass,
    Fail,
    Skip,
}

impl From<JourneyResult> for JourneyReportResult {
    fn from(result: JourneyResult) -> Self {
        match result {
            JourneyResult::Pass | JourneyResult::Partial => Self::Pass,
            JourneyResult::Fail => Self::Fail,
            JourneyResult::Skip => Self::Skip,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReportAvailability {
    pub intro_commit: &'static str,
    pub first_tag: Option<&'static str>,
    pub tag_count: usize,
    pub present_on_master: bool,
    pub present_on_1_4_dev: bool,
    /// Full tag list (includes backports). May be large for early features.
    pub present_in_tags: &'static [&'static str],
}

impl From<&FeatureAvailability> for ReportAvailability {
    fn from(availability: &FeatureAvailability) -> Self {
        Self {
            intro_commit: availability.intro_commit,
            first_tag: availability.first_tag(),
            tag_count: availability.present_in_tags.len(),
            present_on_master: availability.present_on_master,
            present_on_1_4_dev: availability.present_on_1_4_dev,
            present_in_tags: availability.present_in_tags,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReportTrace {
    pub landing_prs: Vec<u64>,
    pub backport_prs: Vec<u64>,
    pub follow_up_prs: Vec<u64>,
    pub squash_merge: bool,
    pub merge_method: String,
    pub intro_sha_in_pr_commits: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_commit_sha: Option<String>,
    pub issue_numbers: Vec<u64>,
}

impl ReportTrace {
    pub fn for_journey(journey_id: &str) -> Option<Self> {
        let cluster = cluster_for_journey(journey_id)?;
        let discovery = discovery_for_journey(journey_id);
        Some(Self {
            landing_prs: cluster.landing_prs.clone(),
            backport_prs: discovery.map_or_else(Vec::new, |d| d.backport_prs.clone()),
            follow_up_prs: discovery.map_or_else(Vec::new, |d| d.follow_up_prs.clone()),
            squash_merge: cluster.squash_merge,
            merge_method: cluster.merge_method.clone(),
            intro_sha_in_pr_commits: cluster.intro_sha_in_pr_commits,
            merge_commit_sha: cluster.merge_commit_sha.clone(),
            issue_numbers: discovery
                .map_or_else(Vec::new, |d| d.issues.iter().map(|i| i.number).collect()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JourneyReportEntry {
    pub id: &'static str,
    pub result: JourneyReportResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<String>,
    pub availability: ReportAvailability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace: Option<ReportTrace>,
    pub steps_passed: usize,
    pub steps_failed: usize,
    pub steps_skipped: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<ReportConflict>,
}

impl JourneyReportEntry {
    pub fn from_run(
        journey_id: JourneyId,
        availability: &FeatureAvailability,
        outcome: JourneyResult,
        step_results: &[StepResult],
    ) -> Self {
        let (steps_passed, steps_failed, steps_skipped) = count_journey_steps(step_results);
        Self {
            id: journey_id.as_str(),
            result: outcome.into(),
            skip_reason: None,
            availability: availability.into(),
            trace: ReportTrace::for_journey(journey_id.as_str()),
            steps_passed,
            steps_failed,
            steps_skipped,
            conflicts: Vec::new(),
        }
    }

    pub fn from_negative_probe(
        probe_id: &'static str,
        journey_id: JourneyId,
        availability: &FeatureAvailability,
        result: &StepResult,
    ) -> Self {
        let outcome = match result {
            StepResult::Fail(_) => JourneyResult::Fail,
            StepResult::Skip(_) | StepResult::Ignored => JourneyResult::Skip,
            _ => JourneyResult::Pass,
        };
        let (steps_passed, steps_failed, steps_skipped) =
            count_journey_steps(std::slice::from_ref(result));
        Self {
            id: probe_id,
            result: outcome.into(),
            skip_reason: None,
            availability: availability.into(),
            trace: ReportTrace::for_journey(journey_id.as_str()),
            steps_passed,
            steps_failed,
            steps_skipped,
            conflicts: Vec::new(),
        }
    }

    pub fn skipped(
        journey_id: JourneyId,
        availability: &FeatureAvailability,
        reason: impl Into<String>,
        steps_skipped: usize,
    ) -> Self {
        Self {
            id: journey_id.as_str(),
            result: JourneyReportResult::Skip,
            skip_reason: Some(reason.into()),
            availability: availability.into(),
            trace: ReportTrace::for_journey(journey_id.as_str()),
            steps_passed: 0,
            steps_failed: 0,
            steps_skipped,
            conflicts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct JourneyHttpReport {
    pub schema_version: u32,
    pub suite: SuiteKind,
    pub base: String,
    pub dut: Option<ReportDut>,
    pub started_at: String,
    pub finished_at: String,
    pub counts: ReportCounts,
    pub journeys: Vec<JourneyReportEntry>,
}

pub fn count_journey_steps(step_results: &[StepResult]) -> (usize, usize, usize) {
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    for result in step_results {
        match result {
            StepResult::Pass | StepResult::Unasserted => passed += 1,
            StepResult::Fail(_) => failed += 1,
            StepResult::Skip(_) | StepResult::Ignored => skipped += 1,
        }
    }
    (passed, failed, skipped)
}

pub fn utc_rfc3339_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before epoch")
        .as_secs();
    format_unix_utc_rfc3339(secs)
}

pub fn format_unix_utc_rfc3339(secs: u64) -> String {
    const SECS_PER_DAY: u64 = 86_400;
    let days = (secs / SECS_PER_DAY) as i64;
    let day_secs = secs % SECS_PER_DAY;
    let (year, month, day) = days_to_ymd(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        day_secs / 3600,
        (day_secs % 3600) / 60,
        day_secs % 60
    )
}

fn days_to_ymd(mut z: i64) -> (i32, u32, u32) {
    z += 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe as i32 + (era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mut month = (5 * doy + 2) / 153;
    let day = doy - (153 * month + 2) / 5 + 1;
    if month < 10 {
        month += 3;
    } else {
        month -= 9;
    }
    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}

pub fn write_journey_http_report(path: &str, report: &JourneyHttpReport) -> Result<(), String> {
    let json = serde_json::to_string_pretty(report).map_err(|err| err.to_string())?;
    std::fs::write(path, json).map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        format_unix_utc_rfc3339, JourneyHttpReport, JourneyReportEntry, ReportCounts, SuiteKind,
        SCHEMA_VERSION,
    };
    use crate::id::JourneyId;
    use crate::version::FeatureAvailability;

    const SAMPLE: FeatureAvailability = FeatureAvailability {
        intro_commit: "abc123",
        present_in_tags: &["1.5.0-beta.2"],
        present_on_master: true,
        present_on_1_4_dev: false,
    };

    #[test]
    fn minimal_report_json_contains_schema_version() {
        let report = JourneyHttpReport {
            schema_version: SCHEMA_VERSION,
            suite: SuiteKind::Smoke,
            base: "http://test".into(),
            dut: None,
            started_at: "2026-01-01T00:00:00Z".into(),
            finished_at: "2026-01-01T00:00:01Z".into(),
            counts: ReportCounts {
                passed: 1,
                failed: 0,
                skipped: 0,
                unasserted: 0,
            },
            journeys: vec![JourneyReportEntry::skipped(
                JourneyId::MonitorInternetConnectivity,
                &SAMPLE,
                "offline",
                1,
            )],
        };
        let json = serde_json::to_string(&report).expect("serialize report");
        assert!(json.contains("\"schema_version\":1"));
        assert!(json.contains("\"intro_commit\":\"abc123\""));
        assert!(json.contains("\"present_on_master\":true"));
        // Real journeys get a compact GitHub provenance summary when traces are loaded.
        assert!(json.contains("\"landing_prs\"") || !json.contains("\"trace\""));
    }

    #[test]
    fn utc_rfc3339_epoch() {
        assert_eq!(format_unix_utc_rfc3339(0), "1970-01-01T00:00:00Z");
    }
}
