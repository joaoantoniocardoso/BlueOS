use std::fs;
use std::path::Path;

use schemars::JsonSchema;
use serde::Serialize;

use crate::drift::{DriftFinding, DriftReport};
use crate::id::PortRef;
use crate::interface::Interface;
use crate::observed::ObservedFacts;
use crate::provenance::{Evidence, ObservedSet};

pub const NGINX_LOCATION_BLOCK_COUNT: usize = 38;
pub const NGINX_LOCALHOST_PROXY_COUNT: usize = 25;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub enum LocationMatch {
    Prefix(String),
    Exact(String),
    Regex(String),
    PrefixPriority(String),
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ExtractedNginxLocation {
    pub line: usize,
    pub match_kind: LocationMatch,
    pub proxy_pass_line: Option<usize>,
    pub proxy_pass_raw: Option<String>,
    pub proxy_port: Option<u16>,
}

#[derive(Debug)]
pub enum ExtractNginxError {
    Io(String),
    Parse(String),
}

pub fn extract_nginx_from_repo(
    repo_root: &Path,
) -> Result<Vec<ExtractedNginxLocation>, ExtractNginxError> {
    let path = repo_root.join("core/tools/nginx/nginx.conf");
    let contents = fs::read_to_string(&path)
        .map_err(|err| ExtractNginxError::Io(format!("read {}: {err}", path.display())))?;
    extract_nginx_from_contents(&contents)
}

pub fn extract_nginx_from_contents(
    contents: &str,
) -> Result<Vec<ExtractedNginxLocation>, ExtractNginxError> {
    let mut locations = Vec::new();
    let mut block: Option<BlockBuilder> = None;

    for (line_number, line) in contents.lines().enumerate() {
        let line_number = line_number + 1;
        if block.is_none() {
            if let Some(match_kind) = parse_location_directive(line) {
                block = Some(BlockBuilder::new(
                    line_number,
                    match_kind,
                    brace_delta(line),
                ));
            }
            continue;
        }

        let builder = block.as_mut().expect("block open");
        if builder.proxy_pass_raw.is_none() {
            if let Some(proxy_pass) = parse_proxy_pass_line(line) {
                builder.proxy_pass_line = Some(line_number);
                builder.proxy_pass_raw = Some(proxy_pass.clone());
                builder.proxy_port = parse_localhost_proxy_port(&proxy_pass);
            }
        }
        builder.depth += brace_delta(line);
        if builder.depth == 0 {
            locations.push(block.take().expect("block open").finish());
        }
    }

    if block.is_some() {
        return Err(ExtractNginxError::Parse(
            "unclosed location block at end of nginx.conf".into(),
        ));
    }

    Ok(locations)
}

struct BlockBuilder {
    line: usize,
    match_kind: LocationMatch,
    depth: i32,
    proxy_pass_line: Option<usize>,
    proxy_pass_raw: Option<String>,
    proxy_port: Option<u16>,
}

impl BlockBuilder {
    fn new(line: usize, match_kind: LocationMatch, depth: i32) -> Self {
        Self {
            line,
            match_kind,
            depth,
            proxy_pass_line: None,
            proxy_pass_raw: None,
            proxy_port: None,
        }
    }

    fn finish(self) -> ExtractedNginxLocation {
        ExtractedNginxLocation {
            line: self.line,
            match_kind: self.match_kind,
            proxy_pass_line: self.proxy_pass_line,
            proxy_pass_raw: self.proxy_pass_raw,
            proxy_port: self.proxy_port,
        }
    }
}

fn brace_delta(line: &str) -> i32 {
    let mut delta = 0i32;
    for ch in line.chars() {
        if ch == '{' {
            delta += 1;
        } else if ch == '}' {
            delta -= 1;
        }
    }
    delta
}

fn parse_location_directive(line: &str) -> Option<LocationMatch> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("location ")?.trim_start();
    if let Some(rest) = rest.strip_prefix('=') {
        let pattern = rest.trim().trim_end_matches('{').trim();
        return Some(LocationMatch::Exact(pattern.to_string()));
    }
    if let Some(rest) = rest.strip_prefix("^~") {
        let pattern = rest.trim().trim_end_matches('{').trim();
        return Some(LocationMatch::PrefixPriority(pattern.to_string()));
    }
    if let Some(rest) = rest.strip_prefix('~') {
        let pattern = rest.trim().trim_end_matches('{').trim();
        return Some(LocationMatch::Regex(pattern.to_string()));
    }
    if let Some(rest) = rest.strip_prefix('@') {
        let name = rest.trim().trim_end_matches('{').trim();
        return Some(LocationMatch::Named(name.to_string()));
    }
    let pattern = rest.trim().trim_end_matches('{').trim();
    Some(LocationMatch::Prefix(pattern.to_string()))
}

fn parse_proxy_pass_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("proxy_pass ")?;
    Some(rest.trim_end_matches(';').trim().to_string())
}

fn parse_localhost_proxy_port(proxy_pass: &str) -> Option<u16> {
    const HOST: &str = "127.0.0.1:";
    let idx = proxy_pass.find(HOST)?;
    let port_part = &proxy_pass[idx + HOST.len()..];
    let end = port_part
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(port_part.len());
    port_part[..end].parse().ok()
}

pub fn check_nginx_against_observed(
    extracted: &[ExtractedNginxLocation],
    services: &[crate::service::Service],
) -> DriftReport {
    let mut findings = Vec::new();

    for service in services {
        let facts = &service.observed;
        check_service_nginx_prefixes(extracted, facts, &mut findings);
        check_service_listen_from_nginx(extracted, facts, &mut findings);
        check_service_nginx_interfaces(extracted, facts, &mut findings);
    }

    DriftReport { findings }
}

fn check_service_nginx_prefixes(
    extracted: &[ExtractedNginxLocation],
    facts: &ObservedFacts,
    findings: &mut Vec<DriftFinding>,
) {
    let ObservedSet::Known { items } = &facts.nginx_prefixes else {
        return;
    };
    for prefix in items.iter() {
        let observed = prefix.value.0;
        let Some(location) = find_location_for_prefix(extracted, observed) else {
            findings.push(DriftFinding {
                field: format!("{}.nginx_prefixes", facts.id.as_str()),
                message: format!(
                    "observed prefix {observed:?}, no matching nginx.conf location (cited line {})",
                    prefix.evidence.line
                ),
            });
            continue;
        };
        if let Some(proxy_port) = location.proxy_port {
            if let Some(listen_ports) = literal_listen_ports(facts) {
                if !listen_ports.is_empty() && !listen_ports.contains(&proxy_port) {
                    findings.push(DriftFinding {
                        field: format!("{}.listen", facts.id.as_str()),
                        message: format!(
                            "nginx.conf:{} proxy_pass port {proxy_port}, observed listen ports {listen_ports:?}",
                            location.proxy_pass_line.unwrap_or(location.line)
                        ),
                    });
                }
            }
        }
    }
}

fn check_service_listen_from_nginx(
    extracted: &[ExtractedNginxLocation],
    facts: &ObservedFacts,
    findings: &mut Vec<DriftFinding>,
) {
    let ObservedSet::Known { items } = &facts.listen else {
        return;
    };
    for item in items.iter() {
        if !evidence_from_nginx(&item.evidence) || !item.evidence.anchor.contains("proxy_pass") {
            continue;
        }
        let PortRef::Literal(port) = item.value else {
            continue;
        };
        let prefixes = known_prefixes(facts);
        if prefixes.is_empty() {
            continue;
        }
        let matches = prefixes.iter().any(|prefix| {
            find_location_for_prefix(extracted, prefix).and_then(|location| location.proxy_port)
                == Some(port)
        });
        if !matches {
            findings.push(DriftFinding {
                field: format!("{}.listen", facts.id.as_str()),
                message: format!(
                    "observed port {port} (nginx.conf:{}), no extracted proxy_pass for prefixes {prefixes:?}",
                    item.evidence.line
                ),
            });
        }
    }
}

fn check_service_nginx_interfaces(
    extracted: &[ExtractedNginxLocation],
    facts: &ObservedFacts,
    findings: &mut Vec<DriftFinding>,
) {
    let ObservedSet::Known { items } = &facts.interfaces else {
        return;
    };
    for item in items.iter() {
        if !evidence_from_nginx(&item.evidence) {
            continue;
        }
        let (prefix, port) = match &item.value {
            Interface::Rest {
                path_prefix, port, ..
            } => (path_prefix.0, port),
            Interface::Websocket { path, port } => (path.0, port),
            Interface::HttpStream { path, port } => (path.0, port),
            _ => continue,
        };
        let PortRef::Literal(expected_port) = *port else {
            continue;
        };
        let Some(location) = find_location_for_prefix(extracted, prefix) else {
            findings.push(DriftFinding {
                field: format!("{}.interfaces", facts.id.as_str()),
                message: format!(
                    "observed interface prefix {prefix:?} port {expected_port} (nginx.conf:{}), no location",
                    item.evidence.line
                ),
            });
            continue;
        };
        let Some(proxy_port) = location.proxy_port else {
            continue;
        };
        if proxy_port != expected_port {
            findings.push(DriftFinding {
                field: format!("{}.interfaces", facts.id.as_str()),
                message: format!(
                    "observed port {expected_port} (nginx.conf:{}), extracted proxy_pass port {proxy_port} at nginx.conf:{}",
                    item.evidence.line,
                    location.proxy_pass_line.unwrap_or(location.line)
                ),
            });
        }
    }
}

fn evidence_from_nginx(evidence: &Evidence) -> bool {
    evidence.file == "core/tools/nginx/nginx.conf"
}

fn known_prefixes(facts: &ObservedFacts) -> Vec<&'static str> {
    match &facts.nginx_prefixes {
        ObservedSet::Known { items } => items.iter().map(|item| item.value.0).collect(),
        ObservedSet::Unknown { .. } => Vec::new(),
    }
}

fn literal_listen_ports(facts: &ObservedFacts) -> Option<Vec<u16>> {
    match &facts.listen {
        ObservedSet::Known { items } => Some(
            items
                .iter()
                .filter_map(|item| match item.value {
                    PortRef::Literal(port) => Some(port),
                    PortRef::Env(_) => None,
                })
                .collect(),
        ),
        ObservedSet::Unknown { .. } => None,
    }
}

pub(crate) fn find_location_for_prefix<'a>(
    extracted: &'a [ExtractedNginxLocation],
    observed: &'static str,
) -> Option<&'a ExtractedNginxLocation> {
    extracted
        .iter()
        .find(|location| observed_prefix_matches(location, observed))
}

pub fn location_match_kind_histogram(extracted: &[ExtractedNginxLocation]) -> [usize; 5] {
    let mut counts = [0usize; 5];
    for location in extracted {
        let idx = match &location.match_kind {
            LocationMatch::Prefix(_) => 0,
            LocationMatch::Exact(_) => 1,
            LocationMatch::Regex(_) => 2,
            LocationMatch::PrefixPriority(_) => 3,
            LocationMatch::Named(_) => 4,
        };
        counts[idx] += 1;
    }
    counts
}

pub const NGINX_LOCATION_MATCH_HISTOGRAM: [usize; 5] = [31, 1, 2, 2, 2];

fn observed_prefix_matches(location: &ExtractedNginxLocation, observed: &str) -> bool {
    match &location.match_kind {
        LocationMatch::Prefix(prefix) | LocationMatch::PrefixPriority(prefix) => prefix == observed,
        LocationMatch::Exact(prefix) => observed == *prefix,
        LocationMatch::Regex(pattern) => regex_covers_observed_prefix(pattern, observed),
        LocationMatch::Named(_) => false,
    }
}

fn regex_covers_observed_prefix(pattern: &str, observed: &str) -> bool {
    let Some(literal) = literal_prefix_from_nginx_regex(pattern) else {
        return false;
    };
    observed == literal.as_str() || observed.starts_with(literal.as_str())
}

fn literal_prefix_from_nginx_regex(pattern: &str) -> Option<String> {
    let pattern = pattern.trim();
    if !pattern.starts_with('^') {
        return None;
    }
    let rest = &pattern[1..];
    const REGEX_METACHARS: &[char] = &['(', '[', '|', '$', '*', '+', '?', '.'];
    let literal_end = rest
        .char_indices()
        .find_map(|(idx, ch)| REGEX_METACHARS.contains(&ch).then_some(idx))
        .unwrap_or(rest.len());
    let literal = rest[..literal_end].trim();
    if literal.is_empty() || !literal.starts_with('/') {
        return None;
    }
    Some(if literal.ends_with('/') {
        literal.to_string()
    } else {
        format!("{literal}/")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Catalog;

    #[test]
    fn extract_pins_location_block_count() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let extracted = extract_nginx_from_repo(repo_root).expect("extract nginx");
        assert_eq!(extracted.len(), NGINX_LOCATION_BLOCK_COUNT);
        assert_eq!(
            extracted
                .iter()
                .filter(|location| location.proxy_port.is_some())
                .count(),
            NGINX_LOCALHOST_PROXY_COUNT
        );
    }

    #[test]
    fn extract_pins_location_match_kind_histogram() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let extracted = extract_nginx_from_repo(repo_root).expect("extract nginx");
        assert_eq!(
            location_match_kind_histogram(&extracted),
            NGINX_LOCATION_MATCH_HISTOGRAM
        );
    }

    #[test]
    fn extract_bag_prefix_maps_to_port_9101() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let extracted = extract_nginx_from_repo(repo_root).expect("extract nginx");
        let bag = extracted
            .iter()
            .find(|location| {
                matches!(
                    &location.match_kind,
                    LocationMatch::Prefix(prefix) if prefix == "/bag/"
                )
            })
            .expect("/bag/ location");
        assert_eq!(bag.line, 86);
        assert_eq!(bag.proxy_port, Some(9101));
        assert_eq!(
            bag.proxy_pass_raw.as_deref(),
            Some("http://127.0.0.1:9101/")
        );
    }

    #[test]
    fn bootstrap_nginx_observed_matches_source() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let extracted = extract_nginx_from_repo(repo_root).expect("extract nginx");
        let report = check_nginx_against_observed(&extracted, Catalog::bootstrap().services());
        assert!(!report.has_drift(), "drift findings: {:?}", report.findings);
    }

    #[test]
    fn extract_requires_location_directive_prefix() {
        let contents =
            "http {\n    bogus /bag/ {\n        proxy_pass http://127.0.0.1:9101/;\n    }\n}\n";
        let extracted = extract_nginx_from_contents(contents).expect("parse synthetic nginx");
        assert_eq!(extracted.len(), 0);
    }

    #[test]
    fn perturbed_proxy_port_reports_drift() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut extracted = extract_nginx_from_repo(repo_root).expect("extract nginx");
        let bag = extracted
            .iter_mut()
            .find(|location| {
                matches!(
                    &location.match_kind,
                    LocationMatch::Prefix(prefix) if prefix == "/bag/"
                )
            })
            .expect("/bag/ location");
        bag.proxy_port = Some(9191);
        let report = check_nginx_against_observed(&extracted, Catalog::bootstrap().services());
        assert!(
            report.has_drift(),
            "expected drift, got {:?}",
            report.findings
        );
        assert!(report.findings.iter().any(|finding| {
            finding.field.contains("bag_of_holding") && finding.message.contains("9191")
        }));
    }
}
