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
    commit, discovery_for_journey, feature_traces, intro_commit_for_journey, issue, journey_ref,
    pull_request, TracePullRequest,
};
use crate::id::JourneyId;
use crate::report::ReportTrace;
use crate::tools::feature_trace_enrich::journey_pickaxe_term;
use crate::version::{parse_release_tag, BlueOsChannel};

/// The 8 precision-baseline goldens (`IMPROVE_DESIGN.md` / `PRECISION_QA.md`
/// §1, plus `NEXT11_DESIGN.md` N11's 3 additions); default journey selection
/// when `--journey` is not given.
pub const GOLDEN_JOURNEY_IDS: &[&str] = &[
    JourneyId::InspectZenohNetwork.as_str(),
    JourneyId::ChangeUiThemeColor.as_str(),
    JourneyId::InspectDiskUsage.as_str(),
    JourneyId::RunInternetSpeedTest.as_str(),
    JourneyId::LevelHorizon.as_str(),
    JourneyId::AccessWebTerminal.as_str(),
    JourneyId::InspectMavlinkMessagesInBrowser.as_str(),
    JourneyId::CalibrateGyroscope.as_str(),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PrRef {
    pub number: u64,
    pub title: Option<String>,
    pub url: Option<String>,
    pub merged_at: Option<String>,
    pub closed_at: Option<String>,
    /// `Some` only for `landing_prs` (from the intro cluster's
    /// `merge_method`, e.g. `"squash"`/`"rebase"`/`"merge_commit"`); `None`
    /// for follow-up/backport PRs (no per-PR merge method is recorded) and
    /// for landing PRs whose cluster method is `"unknown"`.
    pub merge_method: Option<String>,
    /// `Some` only for `follow_up_prs` on a journey carrying an
    /// `Override::Pickaxe` term (N5); `None` otherwise — including for
    /// `landing_prs`/`backport_prs`, which this field never covers.
    pub term_hit: Option<bool>,
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
    /// N9: version tags/channels containing the journey's intro commit,
    /// sourced from `TraceJourneyRef` (already embedded in
    /// `feature_traces.json`'s `journeys`, itself sourced from
    /// `feature_presence_map.json`) — rendered as chips above the timeline.
    pub present_in_tags: Vec<String>,
    pub present_on_master: bool,
    pub present_on_1_4_dev: bool,
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
    Html,
    Both,
}

/// One row of a journey's timeline, ordered by [`timeline_rows`]. Shared by
/// the markdown and HTML renderers so both stay in sync on ordering/content.
enum TimelineEvent<'a> {
    Intro,
    Pr(&'static str, &'a PrRef),
    Issue(&'a IssueRef),
}

fn pr_ref(number: u64) -> PrRef {
    match pull_request(number) {
        Some(pr) => PrRef {
            number,
            title: pr.title.clone(),
            url: pr.url.clone(),
            merged_at: pr.merged_at.clone(),
            closed_at: pr.closed_at.clone(),
            merge_method: None,
            term_hit: None,
        },
        None => PrRef {
            number,
            title: None,
            url: None,
            merged_at: None,
            closed_at: None,
            merge_method: None,
            term_hit: None,
        },
    }
}

/// `pr_ref` plus `merge_method`, scored only for `landing_prs` — the intro
/// cluster's `merge_method` describes how the cluster's landing PR(s)
/// merged; follow-up/backport PRs have no per-PR merge method recorded (N7).
fn landing_pr_ref(number: u64, merge_method: &str) -> PrRef {
    let mut reference = pr_ref(number);
    reference.merge_method = (merge_method != "unknown").then(|| merge_method.to_string());
    reference
}

/// The PR's stored `url`, or a constructed `github.com/<repo>/pull/<n>` link
/// when absent (e.g. a landing PR number not present in `pull_requests`).
fn pr_url(pr: &PrRef) -> String {
    pr.url.clone().unwrap_or_else(|| {
        format!(
            "https://github.com/{}/pull/{}",
            feature_traces().repo,
            pr.number
        )
    })
}

/// Same fallback as [`pr_url`], for issues.
fn issue_url(entry: &IssueRef) -> String {
    entry.url.clone().unwrap_or_else(|| {
        format!(
            "https://github.com/{}/issues/{}",
            feature_traces().repo,
            entry.number
        )
    })
}

/// Whether `term` (a journey's `Override::Pickaxe` term) hits the PR's title
/// or any `files_changed` path. Case-insensitive — matches the enrich-time
/// `entry_mentions_token` check (`feature_trace_enrich.rs`) that this score
/// re-derives, so a PR that passed the pickaxe filter during enrich always
/// scores `term_hit: true` here too.
fn pr_term_hit(pr: &TracePullRequest, term: &str) -> bool {
    let needle = term.to_lowercase();
    let title_hit = pr
        .title
        .as_deref()
        .unwrap_or("")
        .to_lowercase()
        .contains(&needle);
    let file_hit = pr
        .files_changed
        .iter()
        .any(|f| f.to_lowercase().contains(&needle));
    title_hit || file_hit
}

/// `pr_ref` plus `term_hit`, scored only when `pickaxe_term` is `Some` (the
/// journey carries an `Override::Pickaxe`). Report-layer computation from
/// `feature_traces.json` + `OVERRIDES` — no re-enrich required (N5).
fn follow_up_pr_ref(number: u64, pickaxe_term: Option<&str>) -> PrRef {
    let mut reference = pr_ref(number);
    reference.term_hit = pickaxe_term.map(|term| {
        pull_request(number)
            .map(|pr| pr_term_hit(pr, term))
            .unwrap_or(false)
    });
    reference
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
    let pickaxe_term = journey_pickaxe_term(journey_id);
    let presence = journey_ref(journey_id);
    Some(JourneyTimeline {
        journey_id: journey_id.to_string(),
        present_in_tags: presence
            .map(|p| p.present_in_tags.clone())
            .unwrap_or_default(),
        present_on_master: presence.is_some_and(|p| p.present_on_master),
        present_on_1_4_dev: presence.is_some_and(|p| p.present_on_1_4_dev),
        intro_commit: intro_sha.to_string(),
        intro_subject: intro.map(|c| c.subject.clone()).unwrap_or_default(),
        intro_date: intro.and_then(|c| c.authored_at.clone()),
        landing_prs: trace
            .landing_prs
            .iter()
            .map(|&n| landing_pr_ref(n, &trace.merge_method))
            .collect(),
        follow_up_prs: trace
            .follow_up_prs
            .iter()
            .map(|&n| follow_up_pr_ref(n, pickaxe_term))
            .collect(),
        backport_prs: trace.backport_prs.iter().map(|&n| pr_ref(n)).collect(),
        issues: trace.issue_numbers.iter().map(|&n| issue_ref(n)).collect(),
        discovery_paths,
    })
}

fn short_sha(sha: &str) -> String {
    sha.chars().take(12).collect()
}

/// `(term match)`/`(squash|rebase|merge_commit)` suffixes for a PR line,
/// shared by the markdown and HTML renderers.
fn pr_suffixes(pr: &PrRef) -> Vec<String> {
    let mut suffixes = vec![];
    if let Some(method) = &pr.merge_method {
        suffixes.push(format!("({method})"));
    }
    if pr.term_hit == Some(true) {
        suffixes.push("(term match)".to_string());
    }
    suffixes
}

fn pr_line(label: &str, pr: &PrRef) -> String {
    let suffixes = pr_suffixes(pr);
    let suffix = if suffixes.is_empty() {
        String::new()
    } else {
        format!(" {}", suffixes.join(" "))
    };
    format!(
        "- **{label}** [#{}]({}) — {}{suffix}",
        pr.number,
        pr_url(pr),
        pr.title.as_deref().unwrap_or("(no title)")
    )
}

fn issue_line(entry: &IssueRef) -> String {
    format!(
        "- **Issue** [#{}]({}) — {}",
        entry.number,
        issue_url(entry),
        entry.title.as_deref().unwrap_or("(no title)")
    )
}

fn pr_date(pr: &PrRef) -> Option<String> {
    pr.merged_at.clone().or_else(|| pr.closed_at.clone())
}

/// Ordered `(date, event)` rows for one journey's timeline. Kept separate
/// from formatting so the sort is a pure, testable step, and shared by the
/// markdown and HTML renderers so both stay in sync on ordering.
fn timeline_rows(timeline: &JourneyTimeline) -> Vec<(Option<String>, TimelineEvent<'_>)> {
    let mut rows = vec![(timeline.intro_date.clone(), TimelineEvent::Intro)];
    rows.extend(
        timeline
            .landing_prs
            .iter()
            .map(|pr| (pr_date(pr), TimelineEvent::Pr("Landing PR", pr))),
    );
    rows.extend(
        timeline
            .follow_up_prs
            .iter()
            .map(|pr| (pr_date(pr), TimelineEvent::Pr("Follow-up PR", pr))),
    );
    rows.extend(
        timeline
            .backport_prs
            .iter()
            .map(|pr| (pr_date(pr), TimelineEvent::Pr("Backport PR", pr))),
    );
    rows.extend(
        timeline
            .issues
            .iter()
            .map(|entry| (entry.created_at.clone(), TimelineEvent::Issue(entry))),
    );
    // Chronological, undated events last; stable sort keeps each kind's
    // relative insertion order (intro → landing → follow-up → backport → issue)
    // as the tie-break among events sharing a date (or lacking one).
    rows.sort_by(|(a, _), (b, _)| match (a, b) {
        (Some(x), Some(y)) => x.cmp(y),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
    rows
}

/// "Pickaxe hits: K/N follow-ups" summary line, or `None` when the journey
/// carries no `Override::Pickaxe` term (all `follow_up_prs[].term_hit` are
/// `None` in that case, so there's nothing to score).
fn pickaxe_hit_summary(timeline: &JourneyTimeline) -> Option<String> {
    let total = timeline.follow_up_prs.len();
    let scored = timeline
        .follow_up_prs
        .iter()
        .filter(|pr| pr.term_hit.is_some())
        .count();
    if scored == 0 {
        return None;
    }
    let hits = timeline
        .follow_up_prs
        .iter()
        .filter(|pr| pr.term_hit == Some(true))
        .count();
    Some(format!("Pickaxe hits: {hits}/{total} follow-ups"))
}

/// Human-readable markdown timeline for one journey. Pure formatting helper —
/// no I/O, safe to unit test without `feature_traces.json` data.
pub fn format_timeline_markdown(timeline: &JourneyTimeline) -> String {
    let mut out = format!("## {}\n\n", timeline.journey_id);
    for (date, event) in timeline_rows(timeline) {
        let line = match event {
            TimelineEvent::Intro => format!(
                "- **Intro commit** `{}` — {}",
                short_sha(&timeline.intro_commit),
                timeline.intro_subject
            ),
            TimelineEvent::Pr(label, pr) => pr_line(label, pr),
            TimelineEvent::Issue(entry) => issue_line(entry),
        };
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
    if let Some(summary) = pickaxe_hit_summary(timeline) {
        out.push_str(&format!("{summary}\n"));
    }
    out
}

/// Escapes `&`, `<`, `>`, `"`, `'` for safe inline placement in HTML text
/// nodes and `href`/attribute values (N7). `&` must be replaced first.
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Inline-styled chip `<span>` — no CSS framework/JS (N9's constraint).
fn html_chip(text: &str, active: bool) -> String {
    let (bg, fg) = if active {
        ("#1b4332", "#d8f3dc")
    } else {
        ("#2b2b2b", "#8a8a8a")
    };
    format!(
        "<span style=\"display:inline-block;padding:2px 8px;margin:2px 4px 2px 0;\
border-radius:10px;font-size:12px;background:{bg};color:{fg};\">{}</span>",
        html_escape(text)
    )
}

/// GitHub release URL for `tag`, or `None` when it doesn't parse as a real
/// numbered release tag (`master`/`1.4-dev`/unparseable — P12). No network
/// call: derived purely from `parse_release_tag`'s existing shape-matching.
fn tag_release_url(tag: &str) -> Option<String> {
    match parse_release_tag(tag) {
        BlueOsChannel::Numbered { .. } => Some(format!(
            "https://github.com/{}/releases/tag/{tag}",
            feature_traces().repo
        )),
        BlueOsChannel::Master | BlueOsChannel::Dev { .. } | BlueOsChannel::Other(_) => None,
    }
}

/// One `present_in_tags` chip, wrapped in a link to its GitHub release page
/// when `tag` is a real release tag (P12); plain chip otherwise.
fn tag_chip_html(tag: &str) -> String {
    let chip = html_chip(tag, true);
    match tag_release_url(tag) {
        Some(url) => format!("<a href=\"{}\">{chip}</a>", html_escape(&url)),
        None => chip,
    }
}

/// `master`/`1.4-dev` channel chips plus one chip per `present_in_tags`
/// entry, rendered above the journey's timeline table (N9).
fn presence_chips_html(timeline: &JourneyTimeline) -> String {
    let mut chips = html_chip("master", timeline.present_on_master);
    chips.push_str(&html_chip("1.4-dev", timeline.present_on_1_4_dev));
    for tag in &timeline.present_in_tags {
        chips.push_str(&tag_chip_html(tag));
    }
    chips
}

/// Self-contained (no CSS framework/JS) HTML `<section>` for one journey's
/// timeline: presence chips (N9) + a table with clickable GitHub links (N7).
fn format_timeline_html(timeline: &JourneyTimeline) -> String {
    let mut rows_html = String::new();
    for (date, event) in timeline_rows(timeline) {
        let date_cell = html_escape(date.as_deref().unwrap_or("—"));
        let (kind, link_cell, details_cell) = match event {
            TimelineEvent::Intro => (
                "Intro commit".to_string(),
                format!(
                    "<code>{}</code>",
                    html_escape(&short_sha(&timeline.intro_commit))
                ),
                html_escape(&timeline.intro_subject),
            ),
            TimelineEvent::Pr(label, pr) => {
                let suffixes = pr_suffixes(pr);
                let suffix = if suffixes.is_empty() {
                    String::new()
                } else {
                    format!(" {}", suffixes.join(" "))
                };
                (
                    label.to_string(),
                    format!(
                        "<a href=\"{}\">#{}</a>",
                        html_escape(&pr_url(pr)),
                        pr.number
                    ),
                    html_escape(&format!(
                        "{}{suffix}",
                        pr.title.as_deref().unwrap_or("(no title)")
                    )),
                )
            }
            TimelineEvent::Issue(entry) => (
                "Issue".to_string(),
                format!(
                    "<a href=\"{}\">#{}</a>",
                    html_escape(&issue_url(entry)),
                    entry.number
                ),
                html_escape(entry.title.as_deref().unwrap_or("(no title)")),
            ),
        };
        rows_html.push_str(&format!(
            "<tr><td>{date_cell}</td><td>{kind}</td><td>{link_cell}</td><td>{details_cell}</td></tr>\n"
        ));
    }
    let discovery = if timeline.discovery_paths.is_empty() {
        "(none)".to_string()
    } else {
        html_escape(&timeline.discovery_paths.join(", "))
    };
    let pickaxe = pickaxe_hit_summary(timeline)
        .map(|s| format!("<p>{}</p>\n", html_escape(&s)))
        .unwrap_or_default();
    let journey_id = html_escape(&timeline.journey_id);
    format!(
        "<section id=\"{journey_id}\">\n\
<h2>{journey_id}</h2>\n\
<div class=\"chips\">{}</div>\n\
<table>\n<thead><tr><th>Date</th><th>Kind</th><th>Ref</th><th>Details</th></tr></thead>\n\
<tbody>\n{rows_html}</tbody>\n</table>\n\
<p>Discovery paths: {discovery}</p>\n\
{pickaxe}</section>",
        presence_chips_html(timeline)
    )
}

/// Self-contained HTML page (N7/N8): one journey's timeline, or a
/// multi-journey index when `timelines.len() > 1`. No CSS framework/JS.
pub fn format_report_html(timelines: &[JourneyTimeline]) -> String {
    let title = match timelines {
        [single] => single.journey_id.clone(),
        _ => "Feature Trace Report".to_string(),
    };
    let nav = if timelines.len() > 1 {
        let links = timelines
            .iter()
            .map(|t| {
                let id = html_escape(&t.journey_id);
                format!("<a href=\"#{id}\">{id}</a>")
            })
            .collect::<Vec<_>>()
            .join(" · ");
        format!("<nav>{links}</nav>\n")
    } else {
        String::new()
    };
    let sections = timelines
        .iter()
        .map(format_timeline_html)
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
<title>{}</title>\n<style>\n\
body {{ font-family: -apple-system, sans-serif; background:#1e1e1e; color:#eaeaea; margin:2rem; }}\n\
table {{ border-collapse: collapse; width:100%; margin-bottom:1rem; }}\n\
th, td {{ border:1px solid #444; padding:6px 10px; text-align:left; font-size:14px; }}\n\
th {{ background:#2a2a2a; }}\n\
a {{ color:#7fb3ff; }}\n\
nav {{ margin-bottom:1.5rem; }}\n\
section {{ margin-bottom:2.5rem; }}\n\
</style>\n</head>\n<body>\n{nav}{sections}\n</body>\n</html>\n",
        html_escape(&title)
    )
}

/// Precision-regression gate (`IMPROVE_DESIGN.md` "Precision regression
/// invariants" / `PRECISION_QA.md` §1, plus `NEXT11_DESIGN.md` N11): fails
/// loudly if any of the 8 golden journeys' timeline drifts from the
/// locked-in precision baseline. Always checks all 8, independent of
/// `--journey`.
pub fn check_goldens() -> Result<(), String> {
    let mut violations: Vec<String> = vec![];

    match build_timeline(JourneyId::InspectZenohNetwork.as_str()) {
        None => violations.push("inspect_zenoh_network: journey not found".to_string()),
        Some(t) => {
            let landing: Vec<u64> = t.landing_prs.iter().map(|p| p.number).collect();
            if landing != [3300] {
                violations.push(format!(
                    "inspect_zenoh_network: expected landing_prs == [3300], got {landing:?}"
                ));
            }
            let follow: Vec<u64> = t.follow_up_prs.iter().map(|p| p.number).collect();
            for pr in [3313, 3953] {
                if !follow.contains(&pr) {
                    violations.push(format!(
                        "inspect_zenoh_network: follow_up_prs missing #{pr}"
                    ));
                }
            }
        }
    }

    match build_timeline(JourneyId::ChangeUiThemeColor.as_str()) {
        None => violations.push("change_ui_theme_color: journey not found".to_string()),
        Some(t) => {
            if !t.follow_up_prs.is_empty() || !t.backport_prs.is_empty() {
                violations.push(
                    "change_ui_theme_color: expected empty follow_up_prs and backport_prs"
                        .to_string(),
                );
            }
        }
    }

    match build_timeline(JourneyId::InspectDiskUsage.as_str()) {
        None => violations.push("inspect_disk_usage: journey not found".to_string()),
        Some(t) => {
            let mut follow: Vec<u64> = t.follow_up_prs.iter().map(|p| p.number).collect();
            follow.sort_unstable();
            if follow != [3681, 3691, 3743] {
                violations.push(format!(
                    "inspect_disk_usage: expected follow_up_prs == [3681, 3691, 3743], got {follow:?}"
                ));
            }
        }
    }

    match build_timeline(JourneyId::RunInternetSpeedTest.as_str()) {
        None => violations.push("run_internet_speed_test: journey not found".to_string()),
        Some(t) => {
            let landing: Vec<u64> = t.landing_prs.iter().map(|p| p.number).collect();
            let follow: Vec<u64> = t.follow_up_prs.iter().map(|p| p.number).collect();
            let backport: Vec<u64> = t.backport_prs.iter().map(|p| p.number).collect();
            let issues: Vec<u64> = t.issues.iter().map(|p| p.number).collect();
            if !landing.contains(&3602) {
                violations.push("run_internet_speed_test: landing PR #3602 missing".to_string());
            }
            if follow.contains(&3686) || backport.contains(&3686) {
                violations.push(
                    "run_internet_speed_test: PR #3686 must be absent from follow_up/backport"
                        .to_string(),
                );
            }
            if !issues.contains(&2146) {
                violations.push("run_internet_speed_test: issue #2146 missing".to_string());
            }
        }
    }

    match build_timeline(JourneyId::LevelHorizon.as_str()) {
        None => violations.push("level_horizon: journey not found".to_string()),
        Some(t) => {
            let follow: Vec<u64> = t.follow_up_prs.iter().map(|p| p.number).collect();
            let backport: Vec<u64> = t.backport_prs.iter().map(|p| p.number).collect();
            if !backport.contains(&3867) {
                violations.push("level_horizon: backport PR #3867 missing".to_string());
            }
            if follow.contains(&3930) || backport.contains(&3930) {
                violations.push(
                    "level_horizon: PR #3930 must be absent from follow_up/backport".to_string(),
                );
            }
        }
    }

    match build_timeline(JourneyId::AccessWebTerminal.as_str()) {
        None => violations.push("access_web_terminal: journey not found".to_string()),
        Some(t) => {
            let mut follow: Vec<u64> = t.follow_up_prs.iter().map(|p| p.number).collect();
            follow.sort_unstable();
            if follow != [659, 2279] {
                violations.push(format!(
                    "access_web_terminal: expected follow_up_prs == [659, 2279], got {follow:?}"
                ));
            }
        }
    }

    match build_timeline(JourneyId::InspectMavlinkMessagesInBrowser.as_str()) {
        None => {
            violations.push("inspect_mavlink_messages_in_browser: journey not found".to_string())
        }
        Some(t) => {
            let follow: Vec<u64> = t.follow_up_prs.iter().map(|p| p.number).collect();
            if follow != [3310] {
                violations.push(format!(
                    "inspect_mavlink_messages_in_browser: expected follow_up_prs == [3310], got {follow:?}"
                ));
            }
        }
    }

    match build_timeline(JourneyId::CalibrateGyroscope.as_str()) {
        None => violations.push("calibrate_gyroscope: journey not found".to_string()),
        Some(t) => {
            let follow: Vec<u64> = t.follow_up_prs.iter().map(|p| p.number).collect();
            let backport: Vec<u64> = t.backport_prs.iter().map(|p| p.number).collect();
            if follow != [3443] {
                violations.push(format!(
                    "calibrate_gyroscope: expected follow_up_prs == [3443], got {follow:?}"
                ));
            }
            if !backport.contains(&3867) {
                violations.push(
                    "calibrate_gyroscope: backport PR #3867 missing (shared calibration-family backport)"
                        .to_string(),
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
                    "html" => Format::Html,
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
    let html_text = format_report_html(&timelines);

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
            if matches!(format, Format::Html) {
                let path = Path::new(dir).join("feature_trace_report.html");
                fs::write(&path, &html_text).map_err(|e| e.to_string())?;
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
            if matches!(format, Format::Html) {
                println!("{html_text}");
            }
        }
    }

    if check_goldens_flag {
        check_goldens().map_err(|e| format!("--check-goldens: {e}"))?;
        println!("--check-goldens: all 8 golden journeys match PRECISION_QA.md §1 + NEXT11_DESIGN.md N11");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_timeline() -> JourneyTimeline {
        JourneyTimeline {
            journey_id: "InspectDiskUsage".to_string(),
            present_in_tags: vec!["v1.4.0".to_string()],
            present_on_master: true,
            present_on_1_4_dev: false,
            intro_commit: "abcdef0123456789".to_string(),
            intro_subject: "core: services: disk_usage: Add service".to_string(),
            intro_date: Some("2023-01-01T00:00:00Z".to_string()),
            landing_prs: vec![PrRef {
                number: 3300,
                title: Some("Land disk usage".to_string()),
                url: None,
                merged_at: Some("2023-01-02T00:00:00Z".to_string()),
                closed_at: None,
                merge_method: Some("rebase".to_string()),
                term_hit: None,
            }],
            follow_up_prs: vec![PrRef {
                number: 3691,
                title: Some("Fix disk usage bug".to_string()),
                url: None,
                merged_at: Some("2023-01-03T00:00:00Z".to_string()),
                closed_at: None,
                merge_method: None,
                term_hit: None,
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

    fn fixture_pr(title: Option<&str>, files_changed: &[&str]) -> TracePullRequest {
        TracePullRequest {
            number: 9999,
            title: title.map(str::to_string),
            url: None,
            state: None,
            author: None,
            merged_at: None,
            closed_at: None,
            base_ref: None,
            head_ref: None,
            labels: vec![],
            body: String::new(),
            body_truncated: false,
            commit_shas: vec![],
            commit_headlines: vec![],
            files_changed: files_changed.iter().map(|f| f.to_string()).collect(),
            files_changed_truncated: false,
            merge_commit_sha: None,
        }
    }

    #[test]
    fn pr_term_hit_true_when_term_in_title() {
        let pr = fixture_pr(Some("Fix disktest timing race"), &[]);
        assert!(pr_term_hit(&pr, "disktest"));
    }

    #[test]
    fn pr_term_hit_true_when_term_in_files_changed_case_insensitive() {
        let pr = fixture_pr(Some("Unrelated title"), &["core/services/DiskTest/main.py"]);
        assert!(pr_term_hit(&pr, "disktest"));
    }

    #[test]
    fn pr_term_hit_false_when_term_absent() {
        let pr = fixture_pr(Some("Unrelated fix"), &["core/frontend/src/App.vue"]);
        assert!(!pr_term_hit(&pr, "disktest"));
    }

    #[test]
    fn markdown_shows_term_match_suffix_and_pickaxe_summary() {
        let mut timeline = sample_timeline();
        timeline.follow_up_prs[0].term_hit = Some(true);
        let md = format_timeline_markdown(&timeline);
        assert!(md.contains("Fix disk usage bug (term match)"));
        assert!(md.contains("Pickaxe hits: 1/1 follow-ups"));
    }

    #[test]
    fn markdown_omits_pickaxe_summary_when_term_hit_unscored() {
        let md = format_timeline_markdown(&sample_timeline());
        assert!(!md.contains("Pickaxe hits"));
    }

    #[test]
    fn disk_usage_follow_ups_have_no_pickaxe_term_hit() {
        // InspectDiskUsage carries only `Override::Path` rows (no Pickaxe),
        // so term_hit must stay `None` — see feature_presence.rs OVERRIDES.
        let timeline =
            build_timeline(JourneyId::InspectDiskUsage.as_str()).expect("disk usage timeline");
        assert!(!timeline.follow_up_prs.is_empty());
        assert!(timeline
            .follow_up_prs
            .iter()
            .all(|pr| pr.term_hit.is_none()));
    }

    #[test]
    fn markdown_renders_links_and_merge_method_suffix() {
        let mut timeline = sample_timeline();
        timeline.landing_prs[0].url =
            Some("https://github.com/bluerobotics/BlueOS/pull/3300".to_string());
        let md = format_timeline_markdown(&timeline);
        assert!(md.contains("[#3300](https://github.com/bluerobotics/BlueOS/pull/3300)"));
        assert!(md.contains("Land disk usage (rebase)"));
    }

    #[test]
    fn pr_url_constructs_github_link_when_stored_url_missing() {
        let pr = PrRef {
            number: 4242,
            title: None,
            url: None,
            merged_at: None,
            closed_at: None,
            merge_method: None,
            term_hit: None,
        };
        assert_eq!(
            pr_url(&pr),
            "https://github.com/bluerobotics/BlueOS/pull/4242"
        );
    }

    #[test]
    fn issue_url_constructs_github_link_when_stored_url_missing() {
        let entry = IssueRef {
            number: 555,
            title: None,
            url: None,
            created_at: None,
        };
        assert_eq!(
            issue_url(&entry),
            "https://github.com/bluerobotics/BlueOS/issues/555"
        );
    }

    #[test]
    fn html_escape_escapes_special_characters() {
        let escaped = html_escape(r#"<script>alert("x & 'y'")</script>"#);
        assert_eq!(
            escaped,
            "&lt;script&gt;alert(&quot;x &amp; &#39;y&#39;&quot;)&lt;/script&gt;"
        );
        assert!(!escaped.contains('<'));
        assert!(!escaped.contains('>'));
    }

    #[test]
    fn html_render_escapes_hostile_title_and_links_pr_number() {
        let mut timeline = sample_timeline();
        timeline.landing_prs[0].title = Some("<img src=x onerror=alert(1)>".to_string());
        timeline.landing_prs[0].url =
            Some("https://github.com/bluerobotics/BlueOS/pull/3300".to_string());
        let html = format_timeline_html(&timeline);
        assert!(!html.contains("<img src=x"));
        assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
        assert!(
            html.contains("<a href=\"https://github.com/bluerobotics/BlueOS/pull/3300\">#3300</a>")
        );
    }

    #[test]
    fn html_render_links_pr_with_constructed_url_when_missing() {
        let timeline = sample_timeline();
        let html = format_timeline_html(&timeline);
        assert!(
            html.contains("<a href=\"https://github.com/bluerobotics/BlueOS/pull/3691\">#3691</a>")
        );
    }

    #[test]
    fn html_render_shows_presence_chips() {
        let html = format_timeline_html(&sample_timeline());
        assert!(html.contains("master"));
        assert!(html.contains("1.4-dev"));
        assert!(html.contains("v1.4.0"));
    }

    #[test]
    fn format_report_html_wraps_single_journey_without_nav() {
        let html = format_report_html(&[sample_timeline()]);
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<title>InspectDiskUsage</title>"));
        assert!(!html.contains("<nav>"));
    }

    #[test]
    fn format_report_html_wraps_multiple_journeys_with_index_nav() {
        let mut other = sample_timeline();
        other.journey_id = "LevelHorizon".to_string();
        let html = format_report_html(&[sample_timeline(), other]);
        assert!(html.contains("<title>Feature Trace Report</title>"));
        assert!(html.contains("<nav>"));
        assert!(html.contains("href=\"#InspectDiskUsage\""));
        assert!(html.contains("href=\"#LevelHorizon\""));
    }

    #[test]
    fn tag_release_url_links_numbered_release_tag() {
        assert_eq!(
            tag_release_url("1.4.0-beta.12"),
            Some("https://github.com/bluerobotics/BlueOS/releases/tag/1.4.0-beta.12".to_string())
        );
        assert_eq!(
            tag_release_url("1.0.0"),
            Some("https://github.com/bluerobotics/BlueOS/releases/tag/1.0.0".to_string())
        );
    }

    #[test]
    fn tag_release_url_none_for_master_and_dev_channels() {
        assert_eq!(tag_release_url("master"), None);
        assert_eq!(tag_release_url("1.4-dev"), None);
        assert_eq!(tag_release_url("not-a-tag"), None);
    }

    #[test]
    fn html_render_links_release_tag_chip_to_github_releases() {
        let html = format_timeline_html(&sample_timeline());
        assert!(html
            .contains("<a href=\"https://github.com/bluerobotics/BlueOS/releases/tag/v1.4.0\">"));
    }

    #[test]
    fn calibrate_gyroscope_landing_pr_has_merge_method_from_cluster() {
        let timeline = build_timeline(JourneyId::CalibrateGyroscope.as_str())
            .expect("calibrate gyroscope timeline");
        assert!(timeline
            .landing_prs
            .iter()
            .all(|pr| pr.merge_method.as_deref() == Some("rebase")));
    }
}
