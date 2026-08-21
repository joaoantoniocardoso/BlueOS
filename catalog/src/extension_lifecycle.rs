//! Live Kraken extension install/upgrade/downgrade/uninstall with effect reads.
//!
//! The 1.4 regression returned HTTP 200 with an in-band error fragment and no pull
//! progress, so status-only checks stay green. Run against a DUT with
//! `journey_http --base http://<pi> --extension-lifecycle --allow-mutating`.

use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;

use crate::journey::HttpMethod;
use crate::report::{utc_rfc3339_now, SCHEMA_VERSION};
use crate::runner::{execute_curl, join_url, streamed_fragment_error};

const IDENTIFIER: &str = "williangalvani.example1";
const TAG_V100: &str = "v1.0.0";
const TAG_V101: &str = "v1.0.1";
const DOCKER: &str = "williangalvani/blueos-example1";
const EXTENSIONS_PATH: &str = "/kraken/v2.0/extension/";
const CONTAINERS_PATH: &str = "/kraken/v2.0/container/";
const POLL_TIMEOUT: Duration = Duration::from_secs(60);
const POLL_SLEEP: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Check {
    pub name: String,
    pub phase: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LifecycleReport {
    schema_version: u32,
    base: String,
    started_at: String,
    finished_at: String,
    counts: LifecycleCounts,
    checks: Vec<Check>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct LifecycleCounts {
    passed: usize,
    failed: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InstalledExtension {
    identifier: String,
    tag: String,
    enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunningContainer {
    name: String,
    image: String,
}

impl Check {
    fn new(name: &str, phase: &str, ok: bool, detail: impl Into<String>) -> Self {
        Self {
            name: name.to_string(),
            phase: phase.to_string(),
            ok,
            detail: detail.into(),
        }
    }
}

pub fn run_extension_lifecycle(base: &str, allow_mutating: bool) -> Vec<Check> {
    let mut checks = Vec::new();
    let (original_tag, proceed) = preflight(base, allow_mutating, &mut checks);
    if proceed
        && install_phase(base, allow_mutating, TAG_V100, "install", &mut checks)
        && install_phase(base, allow_mutating, TAG_V101, "upgrade", &mut checks)
        && reinstall_phase(base, allow_mutating, &mut checks)
        && install_phase(base, allow_mutating, TAG_V100, "downgrade", &mut checks)
    {
        uninstall_phase(base, allow_mutating, TAG_V100, &mut checks);
    }

    restore(base, allow_mutating, original_tag.as_deref(), &mut checks);
    checks
}

pub fn write_extension_lifecycle_report(
    path: &str,
    base: &str,
    started_at: &str,
    checks: &[Check],
) -> Result<(), String> {
    let passed = checks.iter().filter(|check| check.ok).count();
    let failed = checks.len() - passed;
    let report = LifecycleReport {
        schema_version: SCHEMA_VERSION,
        base: base.to_string(),
        started_at: started_at.to_string(),
        finished_at: utc_rfc3339_now(),
        counts: LifecycleCounts { passed, failed },
        checks: checks.to_vec(),
    };
    let json = serde_json::to_string_pretty(&report).map_err(|err| err.to_string())?;
    std::fs::write(path, json).map_err(|err| err.to_string())
}

fn push(checks: &mut Vec<Check>, name: &str, phase: &str, ok: bool, detail: impl Into<String>) {
    checks.push(Check::new(name, phase, ok, detail));
}

fn preflight(base: &str, allow_mutating: bool, checks: &mut Vec<Check>) -> (Option<String>, bool) {
    let extensions = match fetch_extensions(base) {
        Ok(extensions) => extensions,
        Err(err) => {
            push(checks, "preflight snapshot", "preflight", false, err);
            return (None, false);
        }
    };
    let original_tag = restore_tag(&extensions, IDENTIFIER);
    let summary = format_identifier(&extensions, IDENTIFIER);
    push(
        checks,
        "preflight snapshot",
        "preflight",
        true,
        format!("example1={summary}; others left untouched"),
    );

    if !identifier_absent(&extensions, IDENTIFIER) {
        let tags: Vec<&str> = entries_for(&extensions, IDENTIFIER)
            .iter()
            .map(|ext| ext.tag.as_str())
            .collect();
        for tag in &tags {
            match uninstall_extension(base, tag, allow_mutating) {
                Ok((200 | 202 | 204, _)) => {}
                Ok((status, body)) => {
                    push(
                        checks,
                        "preflight uninstall",
                        "preflight",
                        false,
                        format!("DELETE {IDENTIFIER}/{tag} HTTP {status}: {body}"),
                    );
                    return (original_tag, false);
                }
                Err(err) => {
                    push(checks, "preflight uninstall", "preflight", false, err);
                    return (original_tag, false);
                }
            }
        }
        match wait_listed(base, |exts| identifier_absent(exts, IDENTIFIER)) {
            Ok(_) => push(
                checks,
                "preflight clean slate",
                "preflight",
                true,
                format!("uninstalled tags [{}]", tags.join(", ")),
            ),
            Err(err) => {
                push(
                    checks,
                    "preflight clean slate",
                    "preflight",
                    false,
                    format!("still present after uninstall: {err}"),
                );
                return (original_tag, false);
            }
        }
    } else {
        push(
            checks,
            "preflight clean slate",
            "preflight",
            true,
            "already absent",
        );
    }
    (original_tag, true)
}

fn install_phase(
    base: &str,
    allow_mutating: bool,
    tag: &str,
    phase: &str,
    checks: &mut Vec<Check>,
) -> bool {
    let other = if tag == TAG_V100 { TAG_V101 } else { TAG_V100 };
    let stream_name = format!("{phase} {tag} stream");
    let listed_name = format!("{phase} {tag} listed");
    let container_name = format!("{phase} {tag} container");

    if !run_install(base, tag, allow_mutating, phase, &stream_name, checks) {
        return false;
    }
    if !assert_listed_at_tag(base, tag, other, phase, &listed_name, checks) {
        return false;
    }
    assert_container(base, tag, true, phase, &container_name, checks)
}

fn reinstall_phase(base: &str, allow_mutating: bool, checks: &mut Vec<Check>) -> bool {
    let phase = "reinstall";
    if !run_install(
        base,
        TAG_V101,
        allow_mutating,
        phase,
        "reinstall v1.0.1 stream",
        checks,
    ) {
        return false;
    }
    match wait_listed(base, |exts| {
        exactly_one_at_tag_enabled(exts, IDENTIFIER, TAG_V101)
    }) {
        Ok(_) => push(
            checks,
            "reinstall v1.0.1 listed",
            phase,
            true,
            "exactly one entry at v1.0.1 enabled (idempotent)",
        ),
        Err(err) => {
            push(
                checks,
                "reinstall v1.0.1 listed",
                phase,
                false,
                format!("want exactly one entry at v1.0.1 enabled; got {err}"),
            );
            return false;
        }
    }
    assert_container(
        base,
        TAG_V101,
        true,
        phase,
        "reinstall v1.0.1 container",
        checks,
    )
}

fn uninstall_phase(base: &str, allow_mutating: bool, tag: &str, checks: &mut Vec<Check>) {
    let phase = "uninstall";
    match uninstall_extension(base, tag, allow_mutating) {
        Ok((status @ (200 | 202 | 204), _)) => {
            push(
                checks,
                "uninstall http",
                phase,
                true,
                format!("HTTP {status}"),
            );
        }
        Ok((status, body)) => {
            push(
                checks,
                "uninstall http",
                phase,
                false,
                format!("HTTP {status}: {body}"),
            );
            return;
        }
        Err(err) => {
            push(checks, "uninstall http", phase, false, err);
            return;
        }
    }
    match wait_listed(base, |exts| identifier_absent(exts, IDENTIFIER)) {
        Ok(_) => push(
            checks,
            "uninstall listed",
            phase,
            true,
            format!("{IDENTIFIER} absent"),
        ),
        Err(err) => {
            push(checks, "uninstall listed", phase, false, err);
            return;
        }
    }
    assert_container(base, tag, false, phase, "uninstall container", checks);
}

fn restore(base: &str, allow_mutating: bool, original_tag: Option<&str>, checks: &mut Vec<Check>) {
    let phase = "restore";
    let current = match fetch_extensions(base) {
        Ok(exts) => exts,
        Err(err) => {
            push(
                checks,
                "restore",
                phase,
                false,
                format!("list failed: {err}"),
            );
            return;
        }
    };

    match original_tag {
        Some(tag) => {
            if installed_at_tag_enabled(&current, IDENTIFIER, tag)
                && exactly_one_entry(&current, IDENTIFIER)
            {
                push(checks, "restore", phase, true, format!("already at {tag}"));
                return;
            }
            match install_extension(base, tag, allow_mutating) {
                Ok((status, body)) => {
                    if let Err(detail) = evaluate_install_stream(status, &body, false) {
                        push(checks, "restore", phase, false, detail);
                        return;
                    }
                }
                Err(err) => {
                    push(checks, "restore", phase, false, err);
                    return;
                }
            }
            match wait_listed(base, |exts| installed_at_tag_enabled(exts, IDENTIFIER, tag)) {
                Ok(_) => push(checks, "restore", phase, true, format!("reinstalled {tag}")),
                Err(err) => push(checks, "restore", phase, false, err),
            }
        }
        None => {
            if identifier_absent(&current, IDENTIFIER) {
                push(checks, "restore", phase, true, "was absent; left absent");
                return;
            }
            let tags: Vec<String> = entries_for(&current, IDENTIFIER)
                .iter()
                .map(|ext| ext.tag.clone())
                .collect();
            for tag in &tags {
                if let Err(err) = uninstall_extension(base, tag, allow_mutating) {
                    push(
                        checks,
                        "restore",
                        phase,
                        false,
                        format!("cleanup {tag}: {err}"),
                    );
                    return;
                }
            }
            match wait_listed(base, |exts| identifier_absent(exts, IDENTIFIER)) {
                Ok(_) => push(
                    checks,
                    "restore",
                    phase,
                    true,
                    "removed leftover install; left absent",
                ),
                Err(err) => push(checks, "restore", phase, false, err),
            }
        }
    }
}

fn run_install(
    base: &str,
    tag: &str,
    allow_mutating: bool,
    phase: &str,
    name: &str,
    checks: &mut Vec<Check>,
) -> bool {
    match install_extension(base, tag, allow_mutating) {
        Ok((status, body)) => match evaluate_install_stream(status, &body, true) {
            Ok(detail) => {
                push(checks, name, phase, true, detail);
                true
            }
            Err(detail) => {
                push(checks, name, phase, false, detail);
                false
            }
        },
        Err(err) => {
            push(checks, name, phase, false, err);
            false
        }
    }
}

fn assert_listed_at_tag(
    base: &str,
    tag: &str,
    other: &str,
    phase: &str,
    name: &str,
    checks: &mut Vec<Check>,
) -> bool {
    match wait_listed(base, |exts| {
        installed_at_tag_enabled(exts, IDENTIFIER, tag) && !has_tag(exts, IDENTIFIER, other)
    }) {
        Ok(exts) => {
            push(
                checks,
                name,
                phase,
                true,
                listed_pass_detail(phase, tag, other, &exts),
            );
            true
        }
        Err(err) => {
            push(
                checks,
                name,
                phase,
                false,
                listed_fail_detail(phase, tag, other, &err),
            );
            false
        }
    }
}

fn assert_container(
    base: &str,
    tag: &str,
    want_present: bool,
    phase: &str,
    name: &str,
    checks: &mut Vec<Check>,
) -> bool {
    match wait_container(base, tag, want_present) {
        Ok(detail) => {
            push(checks, name, phase, true, detail);
            true
        }
        Err(err) => {
            push(checks, name, phase, false, err);
            false
        }
    }
}

fn fetch_extensions(base: &str) -> Result<Vec<InstalledExtension>, String> {
    let (status, body) = execute_curl(
        &HttpMethod::Get,
        &join_url(base, EXTENSIONS_PATH),
        false,
        None,
        None,
    )?;
    if status != 200 {
        return Err(format!("GET {EXTENSIONS_PATH} HTTP {status}: {body}"));
    }
    parse_installed_extensions(&body)
}

fn fetch_containers(base: &str) -> Result<Vec<RunningContainer>, String> {
    let (status, body) = execute_curl(
        &HttpMethod::Get,
        &join_url(base, CONTAINERS_PATH),
        false,
        None,
        None,
    )?;
    if status != 200 {
        return Err(format!("GET {CONTAINERS_PATH} HTTP {status}: {body}"));
    }
    parse_running_containers(&body)
}

fn install_extension(base: &str, tag: &str, allow_mutating: bool) -> Result<(u16, String), String> {
    let path = format!("/kraken/v2.0/extension/{IDENTIFIER}/{tag}/install");
    execute_curl(
        &HttpMethod::Post,
        &join_url(base, &path),
        allow_mutating,
        None,
        None,
    )
}

fn uninstall_extension(
    base: &str,
    tag: &str,
    allow_mutating: bool,
) -> Result<(u16, String), String> {
    let path = format!("/kraken/v2.0/extension/{IDENTIFIER}/{tag}");
    execute_curl(
        &HttpMethod::Delete,
        &join_url(base, &path),
        allow_mutating,
        None,
        None,
    )
}

fn wait_listed(
    base: &str,
    pred: impl Fn(&[InstalledExtension]) -> bool,
) -> Result<Vec<InstalledExtension>, String> {
    let deadline = Instant::now() + POLL_TIMEOUT;
    loop {
        let last = match fetch_extensions(base) {
            Ok(exts) if pred(&exts) => return Ok(exts),
            Ok(exts) => format_identifier(&exts, IDENTIFIER),
            Err(err) => err,
        };
        if Instant::now() >= deadline {
            return Err(last);
        }
        thread::sleep(POLL_SLEEP);
    }
}

fn wait_container(base: &str, tag: &str, want_present: bool) -> Result<String, String> {
    let deadline = Instant::now() + POLL_TIMEOUT;
    loop {
        let last = match fetch_containers(base) {
            Ok(containers) => {
                let found = containers
                    .iter()
                    .find(|container| container_matches(container, DOCKER, tag));
                match (want_present, found) {
                    (true, Some(container)) => {
                        return Ok(format!("{} image={}", container.name, container.image));
                    }
                    (false, None) if !any_example1_container(&containers) => {
                        return Ok(format!("{DOCKER} container gone"));
                    }
                    (false, None) => "another example1 tag still listed".into(),
                    (true, None) => format!("no {DOCKER}:{tag} container"),
                    (false, Some(container)) => {
                        format!("still present: {} {}", container.name, container.image)
                    }
                }
            }
            Err(err) => err,
        };
        if Instant::now() >= deadline {
            return Err(last);
        }
        thread::sleep(POLL_SLEEP);
    }
}

fn parse_installed_extensions(body: &str) -> Result<Vec<InstalledExtension>, String> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|err| format!("extension list is not JSON: {err}"))?;
    let array = value
        .as_array()
        .ok_or_else(|| "extension list is not a JSON array".to_string())?;
    let mut out = Vec::with_capacity(array.len());
    for entry in array {
        let identifier = entry
            .get("identifier")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| "extension entry missing identifier".to_string())?;
        let tag = entry
            .get("tag")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("extension {identifier} missing tag"))?;
        let enabled = entry
            .get("enabled")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);
        out.push(InstalledExtension {
            identifier: identifier.to_string(),
            tag: tag.to_string(),
            enabled,
        });
    }
    Ok(out)
}

fn parse_running_containers(body: &str) -> Result<Vec<RunningContainer>, String> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|err| format!("container list is not JSON: {err}"))?;
    let array = value
        .as_array()
        .ok_or_else(|| "container list is not a JSON array".to_string())?;
    let mut out = Vec::with_capacity(array.len());
    for entry in array {
        let name = entry
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let image = entry
            .get("image")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        out.push(RunningContainer {
            name: name.to_string(),
            image: image.to_string(),
        });
    }
    Ok(out)
}

fn entries_for<'a>(
    exts: &'a [InstalledExtension],
    identifier: &str,
) -> Vec<&'a InstalledExtension> {
    exts.iter()
        .filter(|ext| ext.identifier == identifier)
        .collect()
}

fn identifier_absent(exts: &[InstalledExtension], identifier: &str) -> bool {
    entries_for(exts, identifier).is_empty()
}

fn installed_at_tag_enabled(exts: &[InstalledExtension], identifier: &str, tag: &str) -> bool {
    entries_for(exts, identifier)
        .iter()
        .any(|ext| ext.tag == tag && ext.enabled)
}

fn has_tag(exts: &[InstalledExtension], identifier: &str, tag: &str) -> bool {
    entries_for(exts, identifier)
        .iter()
        .any(|ext| ext.tag == tag)
}

fn exactly_one_entry(exts: &[InstalledExtension], identifier: &str) -> bool {
    entries_for(exts, identifier).len() == 1
}

fn exactly_one_at_tag_enabled(exts: &[InstalledExtension], identifier: &str, tag: &str) -> bool {
    let entries = entries_for(exts, identifier);
    entries.len() == 1 && entries[0].tag == tag && entries[0].enabled
}

fn restore_tag(exts: &[InstalledExtension], identifier: &str) -> Option<String> {
    let entries = entries_for(exts, identifier);
    entries
        .iter()
        .find(|ext| ext.enabled)
        .or(entries.first())
        .map(|ext| ext.tag.clone())
}

fn format_identifier(exts: &[InstalledExtension], identifier: &str) -> String {
    let entries = entries_for(exts, identifier);
    if entries.is_empty() {
        return "absent".into();
    }
    entries
        .iter()
        .map(|ext| format!("{} enabled={}", ext.tag, ext.enabled))
        .collect::<Vec<_>>()
        .join("; ")
}

fn listed_pass_detail(phase: &str, tag: &str, other: &str, exts: &[InstalledExtension]) -> String {
    match phase {
        "upgrade" | "downgrade" => {
            format!("exactly one entry at {tag} enabled; {other} purged")
        }
        _ => format_identifier(exts, IDENTIFIER),
    }
}

fn listed_fail_detail(phase: &str, tag: &str, other: &str, observed: &str) -> String {
    match phase {
        "upgrade" | "downgrade" => {
            format!("want exactly one entry at {tag} enabled, {other} purged; got {observed}")
        }
        _ => observed.to_string(),
    }
}

fn evaluate_install_stream(
    status: u16,
    body: &str,
    require_progress: bool,
) -> Result<String, String> {
    if !matches!(status, 200 | 201) {
        return Err(format!("HTTP {status}, expected 200/201"));
    }
    if let Some(err) = streamed_fragment_error(body) {
        return Err(err);
    }
    let progress = progress_fragment_count(body);
    if require_progress && !stream_has_progress(body) {
        return Err("no pull-progress fragments (data was null/absent)".into());
    }
    Ok(format!("HTTP {status}, {progress} progress fragment(s)"))
}

fn stream_has_progress(body: &str) -> bool {
    progress_fragment_count(body) > 0
}

fn progress_fragment_count(body: &str) -> usize {
    body.split("|\n\n|")
        .map(str::trim)
        .filter(|chunk| !chunk.is_empty())
        .filter(|chunk| {
            let Ok(value) = serde_json::from_str::<serde_json::Value>(chunk) else {
                return false;
            };
            match value.get("data") {
                Some(serde_json::Value::Null) | None => false,
                Some(serde_json::Value::String(text)) => !text.is_empty(),
                Some(_) => true,
            }
        })
        .count()
}

fn sanitized_container_suffix(docker: &str, tag: &str) -> String {
    format!("{docker}{tag}")
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}

fn container_matches(container: &RunningContainer, docker: &str, tag: &str) -> bool {
    let expected = format!("extension-{}", sanitized_container_suffix(docker, tag));
    let name = container.name.trim_start_matches('/');
    if name == expected || name.contains(&expected) {
        return true;
    }
    let image_prefix = format!("{docker}:{tag}");
    container.image == image_prefix || container.image.starts_with(&format!("{image_prefix}@"))
}

fn any_example1_container(containers: &[RunningContainer]) -> bool {
    containers.iter().any(|container| {
        container.image.starts_with(&format!("{DOCKER}:"))
            || container
                .name
                .trim_start_matches('/')
                .contains("williangalvaniblueosexample1")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIVE_LIST: &str = r#"[
  {
    "identifier": "bluerobotics.cockpit",
    "tag": "v1.19.0-beta.8",
    "name": "Cockpit Lite",
    "docker": "bluerobotics/cockpit",
    "enabled": true,
    "permissions": "{\"ExposedPorts\": {\"8000/tcp\": {}}}",
    "user_permissions": "",
    "auth": null
  },
  {
    "identifier": "blueos.major_tom",
    "tag": "2026-02-11",
    "name": "major_tom",
    "docker": "public.ecr.aws/blueos/bcloud-agent",
    "enabled": true,
    "permissions": "{}",
    "user_permissions": "",
    "auth": null
  }
]"#;

    const WITH_EXAMPLE1: &str = r#"[
  {"identifier": "bluerobotics.cockpit", "tag": "v1.19.0-beta.8", "enabled": true},
  {"identifier": "williangalvani.example1", "tag": "v1.0.0", "enabled": true},
  {"identifier": "blueos.major_tom", "tag": "2026-02-11", "enabled": true}
]"#;

    const TWO_TAGS: &str = r#"[
  {"identifier": "williangalvani.example1", "tag": "v1.0.0", "enabled": false},
  {"identifier": "williangalvani.example1", "tag": "v1.0.1", "enabled": true}
]"#;

    const REGRESSION_BODY: &str = concat!(
        r#"{"fragment": 0, "status": 500, "data": null, "error": "Extension williangalvani.example1 not found"}"#,
        "|\n\n|",
    );

    const SUCCESS_BODY: &str = concat!(
        r#"{"fragment": 0, "status": 200, "data": "cHVsbGluZyBmcw==", "error": null}"#,
        "|\n\n|",
        r#"{"fragment": 1, "status": 200, "data": "ZG9uZQ==", "error": null}"#,
        "|\n\n|",
    );

    #[test]
    fn parse_live_extension_list_into_triples() {
        let exts = parse_installed_extensions(LIVE_LIST).expect("parse");
        assert_eq!(
            exts.iter()
                .map(|ext| (ext.identifier.as_str(), ext.tag.as_str(), ext.enabled))
                .collect::<Vec<_>>(),
            vec![
                ("bluerobotics.cockpit", "v1.19.0-beta.8", true),
                ("blueos.major_tom", "2026-02-11", true),
            ]
        );
        assert!(identifier_absent(&exts, IDENTIFIER));
        assert!(!installed_at_tag_enabled(&exts, IDENTIFIER, TAG_V100));
    }

    #[test]
    fn installed_absent_and_exactly_one_decisions() {
        let present = parse_installed_extensions(WITH_EXAMPLE1).expect("parse");
        assert!(installed_at_tag_enabled(&present, IDENTIFIER, TAG_V100));
        assert!(!installed_at_tag_enabled(&present, IDENTIFIER, TAG_V101));
        assert!(!identifier_absent(&present, IDENTIFIER));
        assert!(exactly_one_entry(&present, IDENTIFIER));
        assert!(exactly_one_at_tag_enabled(&present, IDENTIFIER, TAG_V100));
        assert!(!exactly_one_at_tag_enabled(&present, IDENTIFIER, TAG_V101));

        let two = parse_installed_extensions(TWO_TAGS).expect("parse");
        assert!(!exactly_one_entry(&two, IDENTIFIER));
        assert!(installed_at_tag_enabled(&two, IDENTIFIER, TAG_V101));
        assert!(!installed_at_tag_enabled(&two, IDENTIFIER, TAG_V100));
        assert!(has_tag(&two, IDENTIFIER, TAG_V100));
        assert_eq!(restore_tag(&two, IDENTIFIER).as_deref(), Some(TAG_V101));
    }

    #[test]
    fn regression_stream_has_no_progress_and_is_an_error() {
        assert!(!stream_has_progress(REGRESSION_BODY));
        assert_eq!(progress_fragment_count(REGRESSION_BODY), 0);
        let err = streamed_fragment_error(REGRESSION_BODY).expect("in-band error");
        assert!(err.contains("williangalvani.example1"));
        let detail = evaluate_install_stream(200, REGRESSION_BODY, true).unwrap_err();
        assert!(detail.contains("williangalvani.example1"));
    }

    #[test]
    fn success_stream_has_progress_fragments() {
        assert!(stream_has_progress(SUCCESS_BODY));
        assert_eq!(progress_fragment_count(SUCCESS_BODY), 2);
        assert!(streamed_fragment_error(SUCCESS_BODY).is_none());
        let detail = evaluate_install_stream(200, SUCCESS_BODY, true).expect("ok");
        assert!(detail.contains("2 progress"));
    }

    #[test]
    fn empty_or_error_only_stream_fails_progress_requirement() {
        assert!(!stream_has_progress(""));
        let detail = evaluate_install_stream(200, "", true).unwrap_err();
        assert!(detail.contains("no pull-progress"));
        assert!(evaluate_install_stream(500, SUCCESS_BODY, true).is_err());
    }

    #[test]
    fn container_name_matches_kraken_sanitization() {
        let container = RunningContainer {
            name: "/extension-williangalvaniblueosexample1v100".into(),
            image: "williangalvani/blueos-example1:v1.0.0".into(),
        };
        assert!(container_matches(&container, DOCKER, TAG_V100));
        assert!(!container_matches(&container, DOCKER, TAG_V101));
        assert!(any_example1_container(std::slice::from_ref(&container)));
        let cockpit = RunningContainer {
            name: "/extension-blueroboticscockpitv1190beta8".into(),
            image: "bluerobotics/cockpit:v1.19.0-beta.8".into(),
        };
        assert!(!any_example1_container(&[cockpit]));
    }
}
