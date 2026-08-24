use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use regex::Regex;

use catalog_paths::{catalog_dir, docs_root, repo_root};

pub const ANCHOR_WINDOW: i32 = 50;
pub const ANCHOR_MIN_LEN: usize = 12;
pub const ANCHOR_MAX_LEN: usize = 60;
pub const LINE_MAX_LEN: usize = 120;

pub fn min_anchor_len(full: &str) -> usize {
    full.len().clamp(1, ANCHOR_MIN_LEN)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchorMatchStatus {
    MatchesCitedLine,
    Relocated { new_line: u32 },
    AmbiguousAnchor,
    AnchorMovedFar { found_line: u32 },
    AnchorLost,
}

pub fn extract_anchor_text(line: &str) -> String {
    extract_anchor_text_with_max(line, ANCHOR_MAX_LEN)
}

pub fn extract_anchor_text_with_max(line: &str, max_len: usize) -> String {
    let ascii: String = line.chars().filter(|c| c.is_ascii()).collect();
    let collapsed: String = ascii.split_whitespace().collect::<Vec<_>>().join(" ");
    if !collapsed
        .chars()
        .any(|c| c.is_ascii() && !c.is_whitespace())
    {
        return String::new();
    }
    let cap = max_len.max(1);
    if collapsed.len() > cap {
        collapsed[..cap].to_string()
    } else {
        collapsed
    }
}

fn anchor_unique_in_window(lines: &[&str], cited_line: u32, anchor: &str) -> bool {
    let start = cited_line.saturating_sub(ANCHOR_WINDOW as u32).max(1);
    let end = (cited_line + ANCHOR_WINDOW as u32).min(lines.len() as u32);
    let mut matches = 0usize;
    for line_no in start..=end {
        if extract_anchor_text_with_max(lines[line_no as usize - 1], anchor.len()) == anchor {
            matches += 1;
        }
    }
    matches == 1
}

pub fn shortest_unique_anchor(path: &Path, cited_line: u32, max_len: usize) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    let lines: Vec<&str> = content.lines().collect();
    if cited_line == 0 || cited_line as usize > lines.len() {
        return None;
    }
    let full = extract_anchor_text_with_max(lines[cited_line as usize - 1], 10_000);
    if full.is_empty() {
        return None;
    }
    let min_len = min_anchor_len(&full);
    let cap = max_len.min(full.len()).max(min_len);
    for len in (min_len..=cap).rev() {
        let candidate = &full[..len];
        if anchor_unique_in_window(&lines, cited_line, candidate) {
            return Some(candidate.to_string());
        }
    }
    None
}

pub fn window_match_count(path: &Path, cited_line: u32, anchor: &str) -> usize {
    if anchor.is_empty() || cited_line == 0 {
        return 0;
    }
    let Ok(content) = std::fs::read_to_string(path) else {
        return 0;
    };
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return 0;
    }
    let start = cited_line.saturating_sub(ANCHOR_WINDOW as u32).max(1);
    let end = (cited_line + ANCHOR_WINDOW as u32).min(lines.len() as u32);
    let mut matches = 0usize;
    for line_no in start..=end {
        if extract_anchor_text_with_max(lines[line_no as usize - 1], anchor.len()) == anchor {
            matches += 1;
        }
    }
    matches
}

fn max_anchor_bytes_for_evidence_field(field_indent: &str) -> usize {
    LINE_MAX_LEN
        .saturating_sub(field_indent.len() + r#"anchor: """#.len() + 1)
        .clamp(ANCHOR_MIN_LEN, ANCHOR_MAX_LEN)
}

pub fn escape_rust_string_literal(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            _ => out.push(ch),
        }
    }
    out
}

pub fn unescape_rust_string_literal(escaped: &str) -> String {
    let mut out = String::with_capacity(escaped.len());
    let mut chars = escaped.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(ch);
        }
    }
    out
}

pub fn read_file_line(path: &Path, line: u32) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    content.lines().nth(line as usize - 1).map(str::to_string)
}

pub fn verify_anchor(path: &Path, cited_line: u32, anchor: &str) -> AnchorMatchStatus {
    if anchor.is_empty() {
        return AnchorMatchStatus::MatchesCitedLine;
    }

    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(_) => return AnchorMatchStatus::AnchorLost,
    };
    let lines: Vec<&str> = content.lines().collect();
    if cited_line as usize > lines.len() || cited_line == 0 {
        return AnchorMatchStatus::AnchorLost;
    }

    let compare_len = anchor.len().max(1);
    let cited = extract_anchor_text_with_max(lines[cited_line as usize - 1], compare_len);
    if cited == anchor {
        return AnchorMatchStatus::MatchesCitedLine;
    }

    let start = cited_line.saturating_sub(ANCHOR_WINDOW as u32).max(1);
    let end = (cited_line + ANCHOR_WINDOW as u32).min(lines.len() as u32);
    let mut window_matches = Vec::new();
    for line_no in start..=end {
        if line_no == cited_line {
            continue;
        }
        if extract_anchor_text_with_max(lines[line_no as usize - 1], compare_len) == anchor {
            window_matches.push(line_no);
        }
    }
    if window_matches.len() == 1 {
        return AnchorMatchStatus::Relocated {
            new_line: window_matches[0],
        };
    }
    if window_matches.len() > 1 {
        return AnchorMatchStatus::AmbiguousAnchor;
    }

    for (index, line) in lines.iter().enumerate() {
        let line_no = index as u32 + 1;
        if line_no == cited_line {
            continue;
        }
        if extract_anchor_text_with_max(line, compare_len) == anchor {
            return AnchorMatchStatus::AnchorMovedFar {
                found_line: line_no,
            };
        }
    }

    AnchorMatchStatus::AnchorLost
}

pub fn fit_overlong_citation_lines() -> std::io::Result<usize> {
    let catalog_src = catalog_dir().join("src");
    let mut fixes = 0;
    for entry in walk_rs_files(&catalog_src)? {
        if entry.components().any(|part| part.as_os_str() == "bin") {
            continue;
        }
        let original = std::fs::read_to_string(&entry)?;
        let aliases = collect_const_aliases(&original);
        let mut content = original.clone();
        let mut file_fixes = 0;
        loop {
            let (updated, pass_fixes) = fit_file_overlong_lines(&content, &aliases);
            content = updated;
            if pass_fixes == 0 {
                break;
            }
            file_fixes += pass_fixes;
        }
        if content != original {
            std::fs::write(&entry, content)?;
            fixes += file_fixes;
        }
    }
    Ok(fixes)
}

fn fit_file_overlong_lines(content: &str, aliases: &HashMap<String, String>) -> (String, usize) {
    let route_file =
        sourced_route_default_file(content, aliases).or_else(|| doc_route_default_file(content));
    let mut lines: Vec<String> = content.lines().map(String::from).collect();
    let mut fixes = 0;
    for index in 0..lines.len() {
        if !line_has_citation_anchor(&lines[index]) {
            continue;
        }
        if let Some(new_line) = try_fit_evidence_anchor_line(&lines, index, content, aliases) {
            lines[index] = new_line;
            fixes += 1;
            continue;
        }
        if let Some(new_line) = try_fit_embedded_provenance(&lines[index], content, aliases) {
            lines[index] = new_line;
            fixes += 1;
            continue;
        }
        if let Some(new_line) =
            try_fit_route_call_line(&lines[index], route_file.as_deref(), aliases, content)
        {
            lines[index] = new_line;
            fixes += 1;
        }
    }
    let mut out = lines.join("\n");
    if content.ends_with('\n') {
        out.push('\n');
    }
    (out, fixes)
}

fn line_has_citation_anchor(line: &str) -> bool {
    line.contains("anchor:")
        || line.contains("Provenance::source")
        || line.contains("Provenance::doc")
        || line.contains("sourced_route")
        || line.contains("doc_route")
        || line.contains("source_outcome")
}

pub(crate) fn source_line_supports_asserted_value(source_line: &str, asserted: &str) -> bool {
    if asserted.is_empty() {
        return true;
    }
    let line = extract_anchor_text_with_max(source_line, 10_000);
    if line.contains(asserted) {
        return true;
    }
    if let Some(name) = asserted.rsplit('/').next() {
        if name.contains('.') && line.contains(name) {
            return true;
        }
    }
    false
}

fn nearby_asserted_value(lines: &[String], anchor_index: usize) -> Option<String> {
    let evidence_idx = lines[..=anchor_index]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, line)| line.contains("Evidence {"))
        .map(|(idx, _)| idx)?;
    let mut prefix = lines[..evidence_idx].join("\n");
    if !prefix.is_empty() {
        prefix.push('\n');
    }
    prefix.push_str(&lines[evidence_idx]);
    let cut = prefix.rfind("Evidence {")?;
    first_arg_immediately_before(&prefix[..cut])
}

fn first_arg_immediately_before(before: &str) -> Option<String> {
    let before = before.trim_end().strip_suffix(',')?.trim_end();
    if let Some(value) = trailing_bool_token(before) {
        return Some(value.to_string());
    }
    let re = Regex::new(r#""((?:\\.|[^"\\])*)"\s*$"#).ok()?;
    let caps = re.captures(before)?;
    Some(unescape_rust_string_literal(&caps[1]))
}

fn trailing_bool_token(input: &str) -> Option<&'static str> {
    for token in ["true", "false"] {
        if let Some(rest) = input.strip_suffix(token) {
            let prev_is_ident = rest
                .chars()
                .last()
                .map(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                .unwrap_or(false);
            if !prev_is_ident {
                return Some(token);
            }
        }
    }
    None
}

fn citation_line_field_matches(text: &str, line_number: u32) -> bool {
    let needle = format!("line: {line_number}");
    let bytes = text.as_bytes();
    let mut start = 0;
    while let Some(rel) = text[start..].find(&needle) {
        let idx = start + rel;
        let before_ok = idx == 0 || {
            let prev = bytes[idx - 1];
            !prev.is_ascii_alphanumeric() && prev != b'_'
        };
        let after = idx + needle.len();
        let after_ok = bytes.get(after).is_none_or(|b| !b.is_ascii_digit());
        if before_ok && after_ok {
            return true;
        }
        start = idx + 1;
    }
    false
}

fn asserted_value_near_citation(content: &str, old_line: u32, anchor: &str) -> Option<String> {
    let lines: Vec<String> = content.lines().map(String::from).collect();
    for (index, line) in lines.iter().enumerate() {
        if !citation_line_field_matches(line, old_line) {
            continue;
        }
        let window_end = (index + 3).min(lines.len().saturating_sub(1));
        let Some(anchor_idx) = lines[index..=window_end]
            .iter()
            .position(|row| row.contains("anchor:") && row.contains(anchor))
            .map(|offset| index + offset)
        else {
            continue;
        };
        if let Some(value) = nearby_asserted_value(&lines, anchor_idx) {
            return Some(value);
        }
    }
    None
}

fn relocation_preserves_identity(
    rel: &LineRelocation,
    catalog_content: &str,
    target_root: &Path,
) -> bool {
    let Some(asserted) = asserted_value_near_citation(catalog_content, rel.old_line, &rel.anchor)
    else {
        return true;
    };
    let Some(raw) = read_file_line(&target_root.join(&rel.file), rel.new_line) else {
        return true;
    };
    source_line_supports_asserted_value(&raw, &asserted)
}

fn refit_anchor(
    target: &Path,
    cited_line: u32,
    current_escaped: &str,
    line_len: usize,
    asserted: Option<&str>,
) -> Option<String> {
    let current = unescape_rust_string_literal(current_escaped);
    if current.is_empty() {
        return None;
    }
    let raw = read_file_line(target, cited_line)?;
    if let Some(value) = asserted {
        if !source_line_supports_asserted_value(&raw, value) {
            return None;
        }
    }
    let full = extract_anchor_text_with_max(&raw, 10_000);
    if full.is_empty() {
        return None;
    }
    let min_len = min_anchor_len(&full);
    let overlong = line_len > LINE_MAX_LEN;
    let too_short = current.len() < min_len;
    if !matches!(
        verify_anchor(target, cited_line, &current),
        AnchorMatchStatus::MatchesCitedLine
    ) {
        return None;
    }
    if !too_short && !(overlong && current.len() > min_len) {
        return None;
    }
    let room = LINE_MAX_LEN.saturating_sub(line_len.saturating_sub(current_escaped.len()));
    let max_len = room.min(ANCHOR_MAX_LEN).min(full.len()).max(min_len);
    let anchor = shortest_unique_anchor(target, cited_line, max_len)
        .unwrap_or_else(|| extract_anchor_text_with_max(&raw, max_len));
    if anchor.is_empty() {
        return None;
    }
    let escaped = escape_rust_string_literal(&anchor);
    if escaped == current_escaped {
        return None;
    }
    Some(escaped)
}

fn try_fit_evidence_anchor_line(
    lines: &[String],
    index: usize,
    content: &str,
    aliases: &HashMap<String, String>,
) -> Option<String> {
    let line = &lines[index];
    let anchor_re = Regex::new(r#"^(\s*)anchor:\s*"((?:\\.|[^"\\])*)",?\s*$"#).ok()?;
    let caps = anchor_re.captures(line)?;
    let field_indent = caps.get(1)?.as_str();
    let old_escaped = caps.get(2)?.as_str();
    let (file_expr, cited_line) = nearby_evidence_fields(lines, index)?;
    let (resolved, is_doc) = resolve_file_expr(&file_expr, aliases, content);
    let resolved = resolved?;
    let target = if is_doc {
        docs_root().join(&resolved)
    } else {
        repo_root().join(&resolved)
    };
    let asserted = nearby_asserted_value(lines, index);
    let escaped = refit_anchor(
        &target,
        cited_line,
        old_escaped,
        line.len(),
        asserted.as_deref(),
    )?;
    Some(format!("{field_indent}anchor: \"{escaped}\","))
}

fn try_fit_embedded_provenance(
    line: &str,
    content: &str,
    aliases: &HashMap<String, String>,
) -> Option<String> {
    let re = Regex::new(
        r#"Provenance::(source|doc)\(\s*([^,]+)\s*,\s*(\d+)\s*,\s*"((?:\\.|[^"\\])*)"\s*\)"#,
    )
    .ok()?;
    let hits: Vec<(usize, usize, String, String, u32, String)> = re
        .captures_iter(line)
        .filter_map(|caps| {
            Some((
                caps.get(0)?.start(),
                caps.get(0)?.end(),
                caps.get(1)?.as_str().to_string(),
                caps.get(2)?.as_str().trim().to_string(),
                caps.get(3)?.as_str().parse().ok()?,
                caps.get(4)?.as_str().to_string(),
            ))
        })
        .collect();
    if hits.is_empty() {
        return None;
    }
    let mut updated = line.to_string();
    let mut changed = false;
    for (start, end, kind, file_expr, cited_line, old_escaped) in hits.into_iter().rev() {
        let (resolved, is_doc) = resolve_file_expr(&file_expr, aliases, content);
        let Some(resolved) = resolved else {
            continue;
        };
        let target = if is_doc {
            docs_root().join(&resolved)
        } else {
            repo_root().join(&resolved)
        };
        let Some(escaped) = refit_anchor(&target, cited_line, &old_escaped, updated.len(), None)
        else {
            continue;
        };
        let replacement = format!("Provenance::{kind}({file_expr}, {cited_line}, \"{escaped}\")");
        updated.replace_range(start..end, &replacement);
        changed = true;
    }
    if changed {
        Some(updated)
    } else {
        None
    }
}

fn try_fit_route_call_line(
    line: &str,
    default_file: Option<&str>,
    aliases: &HashMap<String, String>,
    content: &str,
) -> Option<String> {
    let patterns = [
        Regex::new(concat!(
            r#"(?P<prefix>.*doc_route\([^,]+,[^,]+,[^,]+,\s*)"#,
            r#"(?P<file>[^,]+),\s*(?P<line>\d+)\s*,\s*""#,
            r#"(?P<anchor>(?:\\.|[^"\\])*)"\s*\)"#,
        ))
        .ok()?,
        Regex::new(concat!(
            r#"(?P<prefix>.*sourced_route\([^,]+,[^,]+,[^,]+,\s*)"#,
            r#"(?P<file>[^,]+),\s*(?P<line>\d+)\s*,\s*""#,
            r#"(?P<anchor>(?:\\.|[^"\\])*)"\s*\)"#,
        ))
        .ok()?,
        Regex::new(concat!(
            r#"(?P<prefix>.*sourced_route\([^,]+,[^,]+,[^,]+,\s*)"#,
            r#"(?P<line>\d+)\s*,\s*""#,
            r#"(?P<anchor>(?:\\.|[^"\\])*)"\s*\)"#,
        ))
        .ok()?,
        Regex::new(concat!(
            r#"(?P<prefix>.*source_outcome\(\d+,\s*)"#,
            r#"(?P<line>\d+)\s*,\s*""#,
            r#"(?P<anchor>(?:\\.|[^"\\])*)"\s*\)"#,
        ))
        .ok()?,
        Regex::new(concat!(
            r#"(?P<prefix>.*source_outcome\(\d+,\s*)"#,
            r#"(?P<file>[^,]+),\s*(?P<line>\d+)\s*,\s*""#,
            r#"(?P<anchor>(?:\\.|[^"\\])*)"\s*\)"#,
        ))
        .ok()?,
    ];
    let mut hit = None;
    for re in &patterns {
        if let Some(caps) = re.captures(line) {
            let start = caps.get(0)?.start();
            let end = caps.get(0)?.end();
            let cited_line: u32 = caps.name("line")?.as_str().parse().ok()?;
            let old_escaped = caps.name("anchor")?.as_str().to_string();
            let prefix = caps.name("prefix")?.as_str().to_string();
            let file_part = caps.name("file").map(|m| m.as_str().to_string());
            let file_expr = file_part
                .as_deref()
                .map(str::trim)
                .or(default_file)?
                .to_string();
            hit = Some((
                start,
                end,
                cited_line,
                old_escaped,
                prefix,
                file_part,
                file_expr,
            ));
            break;
        }
    }
    let (start, end, cited_line, old_escaped, prefix, file_part, file_expr) = hit?;
    let (resolved, is_doc) = resolve_file_expr(&file_expr, aliases, content);
    let resolved = resolved?;
    let target = if is_doc {
        docs_root().join(&resolved)
    } else {
        repo_root().join(&resolved)
    };
    let asserted = route_call_asserted_path(line);
    let escaped = refit_anchor(
        &target,
        cited_line,
        &old_escaped,
        line.len(),
        asserted.as_deref(),
    )?;
    let replacement = if let Some(file_part) = file_part {
        format!("{prefix}{file_part}, {cited_line}, \"{escaped}\")")
    } else {
        format!("{prefix}{cited_line}, \"{escaped}\")")
    };
    let mut updated = line.to_string();
    updated.replace_range(start..end, &replacement);
    Some(updated)
}

fn route_call_asserted_path(line: &str) -> Option<String> {
    let re = Regex::new(r#"(?:sourced_route|doc_route)\([^,]+,\s*"((?:\\.|[^"\\])*)""#).ok()?;
    Some(unescape_rust_string_literal(&re.captures(line)?[1]))
}

fn nearby_evidence_fields(lines: &[String], anchor_index: usize) -> Option<(String, u32)> {
    let file_re = Regex::new(r#"file:\s*([^,\n]+)"#).ok()?;
    let line_re = Regex::new(r"line:\s*(\d+)").ok()?;
    let start = anchor_index.saturating_sub(12);
    let mut file = None;
    let mut line = None;
    for row in &lines[start..=anchor_index] {
        if let Some(caps) = file_re.captures(row) {
            file = Some(caps[1].trim().to_string());
        }
        if let Some(caps) = line_re.captures(row) {
            line = caps[1].parse().ok();
        }
    }
    Some((file?, line?))
}

fn sourced_route_default_file(content: &str, _aliases: &HashMap<String, String>) -> Option<String> {
    let re = Regex::new(r"Provenance::source\(([A-Z_][A-Z0-9_]*),\s*line,\s*anchor\)").ok()?;
    Some(re.captures(content)?.get(1)?.as_str().to_string())
}

fn doc_route_default_file(content: &str) -> Option<String> {
    let re = Regex::new(r"Provenance::doc\(([A-Z_][A-Z0-9_]*),\s*line,\s*anchor\)").ok()?;
    Some(re.captures(content)?.get(1)?.as_str().to_string())
}

#[derive(Debug, Clone)]
pub struct UnanchoredReason {
    pub file: String,
    pub line: u32,
    pub reason: &'static str,
}

#[derive(Debug, Default)]
pub struct MigrateReport {
    pub sites_touched: usize,
    pub unanchored: Vec<UnanchoredReason>,
}

pub fn migrate_catalog_sources() -> std::io::Result<MigrateReport> {
    let catalog_src = catalog_dir().join("src");
    let mut report = MigrateReport::default();
    for entry in walk_rs_files(&catalog_src)? {
        if entry.components().any(|part| part.as_os_str() == "bin") {
            continue;
        }
        let touched = migrate_file(&entry, &mut report.unanchored)?;
        report.sites_touched += touched;
    }
    Ok(report)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineRelocation {
    pub site: PathBuf,
    pub file: String,
    pub old_line: u32,
    pub new_line: u32,
    pub anchor: String,
}

pub fn apply_relocated_line_fixes(
    relocations: &[LineRelocation],
) -> std::io::Result<Vec<LineRelocation>> {
    apply_relocated_line_fixes_in(&catalog_dir().join("src"), relocations)
}

pub fn apply_relocated_line_fixes_in(
    catalog_src: &Path,
    relocations: &[LineRelocation],
) -> std::io::Result<Vec<LineRelocation>> {
    apply_relocated_line_fixes_with_root(catalog_src, relocations, &repo_root())
}

pub(crate) fn apply_relocated_line_fixes_with_root(
    catalog_src: &Path,
    relocations: &[LineRelocation],
    target_root: &Path,
) -> std::io::Result<Vec<LineRelocation>> {
    let mut by_site: BTreeMap<PathBuf, Vec<&LineRelocation>> = BTreeMap::new();
    for rel in relocations {
        let path = if rel.site.is_absolute() {
            rel.site.clone()
        } else {
            catalog_src.join(&rel.site)
        };
        by_site.entry(path).or_default().push(rel);
    }
    let mut applied = Vec::new();
    for (path, rels) in by_site {
        if !path.is_file() {
            continue;
        }
        let mut content = std::fs::read_to_string(&path)?;
        let aliases = collect_const_aliases(&content);
        let mut changed = false;
        for rel in rels {
            if !relocation_preserves_identity(rel, &content, target_root) {
                continue;
            }
            if apply_line_fix(
                &mut content,
                &aliases,
                &rel.file,
                rel.old_line,
                rel.new_line,
                &rel.anchor,
            ) {
                changed = true;
                applied.push((*rel).clone());
            }
        }
        if changed {
            std::fs::write(&path, content)?;
        }
    }
    Ok(applied)
}

fn walk_rs_files(root: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    walk_rs_files_inner(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn walk_rs_files_inner(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            walk_rs_files_inner(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    Ok(())
}

fn migrate_file(path: &Path, unanchored: &mut Vec<UnanchoredReason>) -> std::io::Result<usize> {
    let original = std::fs::read_to_string(path)?;
    let aliases = collect_const_aliases(&original);
    let (content, evidence_touched) = migrate_evidence_blocks(&original, &aliases, unanchored);
    let (content, provenance_touched) = migrate_provenance_calls(&content, &aliases, unanchored);
    let touched = evidence_touched + provenance_touched;
    if content != original && !content.is_empty() {
        std::fs::write(path, content)?;
    }
    Ok(touched)
}

fn migrate_evidence_blocks(
    content: &str,
    aliases: &HashMap<String, String>,
    unanchored: &mut Vec<UnanchoredReason>,
) -> (String, usize) {
    let mut out = String::with_capacity(content.len());
    let mut touched = 0;
    let evidence_start = Regex::new(r"Evidence \{\n\s*file:").expect("evidence literal regex");
    let mut starts: Vec<usize> = evidence_start
        .find_iter(content)
        .map(|mat| mat.start())
        .collect();
    starts.sort_unstable();
    let file_re = Regex::new(r"file:\s*([^,\n]+)").expect("file field regex");
    let line_re = Regex::new(r"line:\s*(\d+)").expect("line field regex");
    let mut index = 0;
    for start in starts {
        if start < index {
            continue;
        }
        out.push_str(&content[index..start]);
        let Some((block_len, block)) = extract_brace_block(&content[start..]) else {
            out.push_str(&content[start..]);
            break;
        };
        if block.contains("anchor:") {
            out.push_str(block);
        } else {
            let Some(file_expr) = file_re
                .captures(block)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().trim())
            else {
                out.push_str(block);
                index = start + block_len;
                continue;
            };
            let Some(line) = line_re
                .captures(block)
                .and_then(|caps| caps.get(1))
                .and_then(|m| m.as_str().parse::<u32>().ok())
            else {
                out.push_str(block);
                index = start + block_len;
                continue;
            };
            let field_indent = block
                .lines()
                .find_map(|row| row.find("file:").map(|idx| &row[..idx]))
                .unwrap_or("    ");
            let max_len = max_anchor_bytes_for_evidence_field(field_indent);
            let anchor =
                resolve_anchor_text(file_expr, line, aliases, content, unanchored, max_len);
            out.push_str(&insert_evidence_anchor(block, &anchor));
            touched += 1;
        }
        index = start + block_len;
    }
    out.push_str(&content[index..]);
    (out, touched)
}

fn migrate_provenance_calls(
    content: &str,
    aliases: &HashMap<String, String>,
    unanchored: &mut Vec<UnanchoredReason>,
) -> (String, usize) {
    let re = Regex::new(r"Provenance::(source|doc)\(\s*([^,)]+)\s*,\s*(\d+)\s*\)")
        .expect("provenance migration regex");
    let mut touched = 0;
    let updated = re
        .replace_all(content, |caps: &regex::Captures| {
            let kind = caps.get(1).expect("kind").as_str();
            let file_expr = caps.get(2).expect("file").as_str().trim();
            let line: u32 = caps
                .get(3)
                .expect("line")
                .as_str()
                .parse()
                .expect("line number");
            let line_indent = line_indent_at(content, caps.get(0).expect("match").start());
            let max_len = provenance_single_line_budget(&line_indent, file_expr, line);
            let anchor =
                resolve_anchor_text(file_expr, line, aliases, content, unanchored, max_len);
            touched += 1;
            format_provenance_call(kind, &line_indent, file_expr, line, &anchor)
        })
        .into_owned();
    (updated, touched)
}

fn provenance_single_line_budget(indent: &str, file_expr: &str, line: u32) -> usize {
    let before = format!("{indent}Provenance::source({file_expr}, {line}, \"");
    let after = "\")";
    LINE_MAX_LEN
        .saturating_sub(before.len() + after.len())
        .clamp(ANCHOR_MIN_LEN, ANCHOR_MAX_LEN)
}

fn resolve_anchor_text(
    file_expr: &str,
    line: u32,
    aliases: &HashMap<String, String>,
    content: &str,
    unanchored: &mut Vec<UnanchoredReason>,
    max_len: usize,
) -> String {
    let (resolved, is_doc) = resolve_file_expr(file_expr, aliases, content);
    let Some(resolved_path) = resolved else {
        unanchored.push(UnanchoredReason {
            file: file_expr.to_string(),
            line,
            reason: "could not resolve file expression",
        });
        return String::new();
    };
    let target = if is_doc {
        docs_root().join(&resolved_path)
    } else {
        repo_root().join(&resolved_path)
    };
    let line_text = read_file_line(&target, line);
    let anchor = line_text
        .as_ref()
        .and_then(|raw| {
            shortest_unique_anchor(&target, line, max_len).or_else(|| {
                let extracted = extract_anchor_text_with_max(raw, max_len);
                if extracted.is_empty() {
                    None
                } else {
                    Some(extracted)
                }
            })
        })
        .unwrap_or_default();
    if anchor.is_empty() {
        let reason = match line_text {
            None => "cited line unreadable or out of range",
            Some(raw) if raw.trim().is_empty() => "cited line is blank",
            Some(raw) if !raw.chars().any(|c| c.is_ascii() && !c.is_whitespace()) => {
                "cited line has no usable ASCII content"
            }
            _ => "anchor normalization produced empty string",
        };
        unanchored.push(UnanchoredReason {
            file: resolved_path,
            line,
            reason,
        });
    }
    escape_rust_string_literal(&anchor)
}

fn insert_evidence_anchor(block: &str, anchor: &str) -> String {
    if !block.contains('\n') {
        return expand_single_line_evidence(block, anchor);
    }
    let lines: Vec<&str> = block.lines().collect();
    let close_idx = lines
        .iter()
        .rposition(|line| {
            let trimmed = line.trim();
            trimmed == "}" || trimmed == "},"
        })
        .unwrap_or(lines.len().saturating_sub(1));
    let field_indent = lines
        .iter()
        .find_map(|line| line.find("file:").map(|idx| &line[..idx]))
        .unwrap_or("    ");
    let mut updated = String::new();
    for line in &lines[..close_idx] {
        updated.push_str(line);
        updated.push('\n');
    }
    updated.push_str(&format!("{field_indent}anchor: \"{anchor}\",\n"));
    for line in &lines[close_idx..] {
        updated.push_str(line);
        updated.push('\n');
    }
    updated.trim_end_matches('\n').to_string()
}

fn expand_single_line_evidence(block: &str, anchor: &str) -> String {
    let file_re = Regex::new(r"file:\s*([^,]+)").expect("file field regex");
    let line_re = Regex::new(r"line:\s*(\d+)").expect("line field regex");
    let file_expr = file_re
        .captures(block)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim())
        .unwrap_or("\"\"");
    let line = line_re
        .captures(block)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .unwrap_or("0");
    let indent = block
        .find("Evidence")
        .map(|idx| &block[..idx])
        .unwrap_or("");
    let inner = format!("{indent}    ");
    format!(
        "{indent}Evidence {{\n{inner}file: {file_expr},\n{inner}line: {line},\n\
{inner}anchor: \"{anchor}\",\n{indent}}},"
    )
}

fn format_provenance_call(
    kind: &str,
    indent: &str,
    file_expr: &str,
    line: u32,
    anchor: &str,
) -> String {
    let single = format!("Provenance::{kind}({file_expr}, {line}, \"{anchor}\")");
    if single.len() <= LINE_MAX_LEN {
        return single;
    }
    let inner = format!("{indent}    ");
    format!(
        "{indent}Provenance::{kind}(\n{inner}{file_expr},\n{inner}{line},\n{inner}\"{anchor}\",\n{indent})"
    )
}

fn extract_brace_block(content: &str) -> Option<(usize, &str)> {
    let open = content.find('{')?;
    let bytes = content.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escape = false;
    let mut quote = b'"';
    for (offset, &byte) in bytes[open..].iter().enumerate() {
        let ch = byte as char;
        if in_string {
            if escape {
                escape = false;
            } else if ch == '\\' {
                escape = true;
            } else if byte == quote {
                in_string = false;
            }
            continue;
        }
        if ch == '"' || ch == '\'' {
            in_string = true;
            quote = byte;
            continue;
        }
        match ch {
            '{' => depth += 1,
            '}' => {
                if depth == 0 {
                    return None;
                }
                depth -= 1;
                if depth == 0 {
                    let end = open + offset + 1;
                    let mut block_end = end;
                    if content.get(block_end..=block_end) == Some(",") {
                        block_end += 1;
                    }
                    return Some((block_end, &content[..block_end]));
                }
            }
            _ => {}
        }
    }
    None
}

fn line_indent_at(content: &str, pos: usize) -> String {
    let line_start = content[..pos].rfind('\n').map(|idx| idx + 1).unwrap_or(0);
    content[line_start..pos]
        .chars()
        .take_while(|ch| matches!(ch, ' ' | '\t'))
        .collect()
}

pub(crate) fn provenance_source_fix_pattern(
    file_pattern: &str,
    old_line: u32,
    escaped_anchor: &str,
) -> String {
    format!(
        r#"Provenance::source\(\s*(?P<file>{file_pattern})\s*,\s*{old_line}\s*,\s*"(?P<anchor>{escaped_anchor})"\s*,?\s*\)"#
    )
}

pub(crate) fn provenance_doc_fix_pattern(
    file_pattern: &str,
    old_line: u32,
    escaped_anchor: &str,
) -> String {
    format!(
        r#"Provenance::doc\(\s*(?P<file>{file_pattern})\s*,\s*{old_line}\s*,\s*"(?P<anchor>{escaped_anchor})"\s*,?\s*\)"#
    )
}

pub(crate) fn evidence_fix_pattern(
    file_pattern: &str,
    old_line: u32,
    escaped_anchor: &str,
) -> String {
    format!(
        concat!(
            r#"(?P<prefix>Evidence\s*\{{\s*file:\s*{file_pattern}\s*,\s*)"#,
            r#"line:\s*{old_line}(?P<suffix>\s*,\s*anchor:\s*"{escaped_anchor}")"#,
        ),
        file_pattern = file_pattern,
        old_line = old_line,
        escaped_anchor = escaped_anchor,
    )
}

pub(crate) fn helper_call_fix_pattern(
    helper: &str,
    file_pattern: &str,
    old_line: u32,
    escaped_anchor: &str,
) -> String {
    format!(
        r#"(?P<pre>{helper}\((?:[^()]|\([^()]*\))*?,\s*{file_pattern}\s*,\s*){old_line}(?P<suf>\s*,\s*"{escaped_anchor}"\s*,?\s*\))"#
    )
}

pub(crate) fn helper_call_implicit_fix_pattern(
    helper: &str,
    old_line: u32,
    escaped_anchor: &str,
) -> String {
    format!(
        r#"(?P<pre>{helper}\((?:[^()]|\([^()]*\))*?(?:Some\("[^"]*"\)|None|\d+)\s*,\s*){old_line}(?P<suf>\s*,\s*"{escaped_anchor}"\s*,?\s*\))"#
    )
}

fn apply_line_fix(
    content: &mut String,
    aliases: &HashMap<String, String>,
    file: &str,
    old_line: u32,
    new_line: u32,
    anchor: &str,
) -> bool {
    let escaped_anchor = regex::escape(&escape_rust_string_literal(anchor));
    let file_pattern = file_expr_pattern(file, aliases, content);
    let patterns = [
        (
            "Provenance::source",
            provenance_source_fix_pattern(&file_pattern, old_line, &escaped_anchor),
        ),
        (
            "Provenance::doc",
            provenance_doc_fix_pattern(&file_pattern, old_line, &escaped_anchor),
        ),
        (
            "Evidence",
            evidence_fix_pattern(&file_pattern, old_line, &escaped_anchor),
        ),
        (
            "sourced_route",
            helper_call_fix_pattern("sourced_route", &file_pattern, old_line, &escaped_anchor),
        ),
        (
            "source_outcome",
            helper_call_fix_pattern("source_outcome", &file_pattern, old_line, &escaped_anchor),
        ),
        (
            "doc_route",
            helper_call_fix_pattern("doc_route", &file_pattern, old_line, &escaped_anchor),
        ),
        (
            "sourced_route",
            helper_call_implicit_fix_pattern("sourced_route", old_line, &escaped_anchor),
        ),
        (
            "source_outcome",
            helper_call_implicit_fix_pattern("source_outcome", old_line, &escaped_anchor),
        ),
        (
            "doc_route",
            helper_call_implicit_fix_pattern("doc_route", old_line, &escaped_anchor),
        ),
    ];
    let mut changed = false;
    for (kind, pattern) in patterns {
        let re = Regex::new(&pattern).expect("valid fix regex");
        if !re.is_match(content) {
            continue;
        }
        let updated = match kind {
            "Provenance::source" => re
                .replace_all(content, |caps: &regex::Captures| {
                    format!(
                        "Provenance::source({}, {new_line}, \"{}\")",
                        &caps["file"], &caps["anchor"]
                    )
                })
                .into_owned(),
            "Provenance::doc" => re
                .replace_all(content, |caps: &regex::Captures| {
                    format!(
                        "Provenance::doc({}, {new_line}, \"{}\")",
                        &caps["file"], &caps["anchor"]
                    )
                })
                .into_owned(),
            "Evidence" => re
                .replace_all(content, |caps: &regex::Captures| {
                    format!("{}line: {new_line}{}", &caps["prefix"], &caps["suffix"])
                })
                .into_owned(),
            "sourced_route" | "source_outcome" | "doc_route" => re
                .replace_all(content, |caps: &regex::Captures| {
                    format!("{}{new_line}{}", &caps["pre"], &caps["suf"])
                })
                .into_owned(),
            _ => unreachable!(),
        };
        *content = updated;
        changed = true;
    }
    changed
}

fn file_expr_pattern(file: &str, aliases: &HashMap<String, String>, _content: &str) -> String {
    let escaped = regex::escape(file);
    let mut names: Vec<&str> = aliases
        .iter()
        .filter(|(_, path)| path.as_str() == file)
        .map(|(name, _)| name.as_str())
        .collect();
    names.sort_unstable();
    if names.is_empty() {
        return format!(r#""{escaped}""#);
    }
    let alts = names
        .into_iter()
        .map(regex::escape)
        .collect::<Vec<_>>()
        .join("|");
    format!(r#"(?:{alts}|"{escaped}")"#)
}

fn collect_const_aliases(content: &str) -> HashMap<String, String> {
    let re = Regex::new(r#"(?:pub\s+)?const\s+(\w+)\s*:\s*&str\s*=\s*"([^"]+)";"#)
        .expect("valid const regex");
    let mut aliases = HashMap::new();
    for caps in re.captures_iter(content) {
        aliases.insert(
            caps.get(1).expect("name").as_str().to_string(),
            caps.get(2).expect("value").as_str().to_string(),
        );
    }
    aliases
}

fn resolve_file_expr(
    expr: &str,
    aliases: &HashMap<String, String>,
    content: &str,
) -> (Option<String>, bool) {
    if let Some(stripped) = expr.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return (Some(stripped.to_string()), is_doc_path(stripped));
    }
    if let Some(path) = aliases.get(expr) {
        return (Some(path.clone()), is_doc_path(path));
    }
    let re = Regex::new(r#""([^"]+)""#).expect("valid string literal regex");
    if let Some(found) = re.find(expr) {
        let path = found.as_str().trim_matches('"').to_string();
        return (Some(path.clone()), is_doc_path(&path));
    }
    if content.contains(&format!("const {expr}:"))
        || content.contains(&format!("pub const {expr}:"))
    {
        return (None, false);
    }
    (None, false)
}

fn is_doc_path(path: &str) -> bool {
    path.starts_with("content/") || path.ends_with(".md")
}

pub fn group_unanchored_reasons(unanchored: &[UnanchoredReason]) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for item in unanchored {
        *counts.entry(item.reason).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_and_replace(
        pattern: &str,
        haystack: &str,
        replacer: impl Fn(&regex::Captures) -> String,
    ) -> String {
        let re = Regex::new(pattern).expect("repair pattern must compile");
        re.replace_all(haystack, |caps: &regex::Captures| replacer(caps))
            .into_owned()
    }

    #[test]
    fn provenance_source_fix_pattern_compiles_and_substitutes() {
        let pattern =
            provenance_source_fix_pattern(r#"(?:VC|"core/foo.py")"#, 10, r#"async delete\(\) \{"#);
        let src = concat!(
            "Provenance::source(\n",
            "                VC,\n",
            "                10,\n",
            "                \"async delete() {\",\n",
            "            )",
        );
        let out = compile_and_replace(&pattern, src, |caps| {
            format!(
                "Provenance::source({}, 20, \"{}\")",
                &caps["file"], &caps["anchor"]
            )
        });
        assert_eq!(out, r#"Provenance::source(VC, 20, "async delete() {")"#);
    }

    #[test]
    fn provenance_doc_fix_pattern_compiles_and_substitutes() {
        let pattern = provenance_doc_fix_pattern(r#""content/usage.md""#, 4, "operator intent");
        let src = r#"Provenance::doc("content/usage.md", 4, "operator intent")"#;
        let out = compile_and_replace(&pattern, src, |caps| {
            format!(
                "Provenance::doc({}, 14, \"{}\")",
                &caps["file"], &caps["anchor"]
            )
        });
        assert_eq!(
            out,
            r#"Provenance::doc("content/usage.md", 14, "operator intent")"#
        );
    }

    #[test]
    fn evidence_fix_pattern_compiles_and_substitutes() {
        let pattern = evidence_fix_pattern(r#""core/foo.py""#, 10, "unique_anchor_text");
        let src = concat!(
            "Evidence {\n",
            "            file: \"core/foo.py\",\n",
            "            line: 10,\n",
            "            anchor: \"unique_anchor_text\",\n",
            "        }",
        );
        let out = compile_and_replace(&pattern, src, |caps| {
            format!("{}line: 20{}", &caps["prefix"], &caps["suffix"])
        });
        assert!(out.contains("line: 20"), "{out}");
        assert!(out.contains("anchor: \"unique_anchor_text\""), "{out}");
        assert!(!out.contains("line: 10"), "{out}");
    }

    #[test]
    fn helper_call_fix_pattern_compiles_and_substitutes() {
        let file_pattern = r#"(?:FILE_A|"core/a.py")"#;
        let pattern =
            helper_call_fix_pattern("sourced_route", file_pattern, 10, "unique_anchor_text");
        let src = concat!(
            r#"Some(sourced_route(HttpMethod::Post, "/a", Some("v1.0"), FILE_A, 10, "unique_anchor_text")),"#,
            "\n",
            r#"Some(sourced_route(HttpMethod::Post, "/b", Some("v1.0"), FILE_B, 10, "unique_anchor_text")),"#,
            "\n",
            r#"Some(sourced_route(HttpMethod::Post, "/c", Some("v1.0"), 20, "other_anchor_text")),"#,
        );
        let out = compile_and_replace(&pattern, src, |caps| {
            format!("{}15{}", &caps["pre"], &caps["suf"])
        });
        assert!(out.contains("FILE_A, 15, \"unique_anchor_text\")"), "{out}");
        assert!(out.contains("FILE_B, 10, \"unique_anchor_text\")"), "{out}");
        assert!(out.contains(", 20, \"other_anchor_text\")"), "{out}");
        for helper in ["source_outcome", "doc_route"] {
            let pattern = helper_call_fix_pattern(helper, file_pattern, 10, "unique_anchor_text");
            Regex::new(&pattern).unwrap_or_else(|err| panic!("{helper}: {err}"));
        }
        let outcome = helper_call_fix_pattern(
            "source_outcome",
            r#"(?:FILE|"core/foo.py")"#,
            10,
            "unique_anchor_text",
        );
        let src = r#"Some(source_outcome(200, FILE, 10, "unique_anchor_text"))"#;
        let out = compile_and_replace(&outcome, src, |caps| {
            format!("{}15{}", &caps["pre"], &caps["suf"])
        });
        assert_eq!(
            out,
            r#"Some(source_outcome(200, FILE, 15, "unique_anchor_text"))"#
        );
        let implicit = helper_call_implicit_fix_pattern("source_outcome", 10, "unique_anchor_text");
        let src = r#"Some(source_outcome(200, 10, "unique_anchor_text"))"#;
        let out = compile_and_replace(&implicit, src, |caps| {
            format!("{}15{}", &caps["pre"], &caps["suf"])
        });
        assert_eq!(
            out,
            r#"Some(source_outcome(200, 15, "unique_anchor_text"))"#
        );
        let implicit_route =
            helper_call_implicit_fix_pattern("sourced_route", 10, "unique_anchor_text");
        Regex::new(&implicit_route).expect("implicit sourced_route compiles");
        let none_src = r#"Some(sourced_route(HttpMethod::Post, "/post_file", None, 10, "unique_anchor_text")),"#;
        let out = compile_and_replace(&implicit_route, none_src, |caps| {
            format!("{}15{}", &caps["pre"], &caps["suf"])
        });
        assert_eq!(
            out,
            r#"Some(sourced_route(HttpMethod::Post, "/post_file", None, 15, "unique_anchor_text")),"#
        );
    }

    #[test]
    fn file_expr_pattern_escapes_metacharacters_and_is_deterministic() {
        let path = "catalog/extras/BlueOS Release Testing Tracker - BlueOS 1.x.x [TEMPLATE].csv";
        let mut aliases = HashMap::new();
        aliases.insert("ZETA".to_string(), path.to_string());
        aliases.insert("ALPHA".to_string(), path.to_string());
        let pattern = file_expr_pattern(path, &aliases, "");
        let re = Regex::new(&pattern).expect("escaped path must compile");
        assert!(re.is_match("ALPHA"), "{pattern}");
        assert!(re.is_match(&format!("\"{path}\"")), "{pattern}");
        assert!(
            pattern.starts_with("(?:ALPHA|ZETA|"),
            "aliases must be sorted: {pattern}"
        );
        assert!(
            pattern.contains(r"\[TEMPLATE\]"),
            "brackets escaped: {pattern}"
        );
    }

    fn test_scratch_dir(name: &str) -> PathBuf {
        let base = std::env::var_os("CARGO_TARGET_DIR")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("TMPDIR").map(PathBuf::from))
            .unwrap_or_else(std::env::temp_dir);
        base.join(format!("blueos-catalog-{name}-{}", std::process::id()))
    }

    #[test]
    fn apply_relocated_source_call_rewrites_line_number() {
        let dir = test_scratch_dir("fix-source");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let site = dir.join("module_a.rs");
        std::fs::write(
            &site,
            r#"let p = Provenance::source("core/foo.py", 10, "unique_anchor_text");"#,
        )
        .expect("write");
        let fixes = apply_relocated_line_fixes_in(
            &dir,
            &[LineRelocation {
                site: site.clone(),
                file: "core/foo.py".to_string(),
                old_line: 10,
                new_line: 20,
                anchor: "unique_anchor_text".to_string(),
            }],
        )
        .expect("apply");
        assert_eq!(fixes.len(), 1);
        let body = std::fs::read_to_string(&site).expect("read");
        assert!(
            body.contains(r#"Provenance::source("core/foo.py", 20, "unique_anchor_text")"#),
            "{body}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn apply_relocated_line_fix_is_scoped_to_citation_site() {
        let dir = test_scratch_dir("fix-scope");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let drifted = dir.join("module_a.rs");
        let untouched = dir.join("module_b.rs");
        let body = r#"
        Evidence {
            file: "core/foo.py",
            line: 10,
            anchor: "unique_anchor_text",
        }
"#;
        std::fs::write(&drifted, body).expect("write a");
        std::fs::write(&untouched, body).expect("write b");
        let fixes = apply_relocated_line_fixes_in(
            &dir,
            &[LineRelocation {
                site: drifted.clone(),
                file: "core/foo.py".to_string(),
                old_line: 10,
                new_line: 20,
                anchor: "unique_anchor_text".to_string(),
            }],
        )
        .expect("apply");
        assert_eq!(fixes.len(), 1);
        let a = std::fs::read_to_string(&drifted).expect("read a");
        let b = std::fs::read_to_string(&untouched).expect("read b");
        assert!(a.contains("line: 20"), "{a}");
        assert!(b.contains("line: 10"), "{b}");
        assert!(!b.contains("line: 20"), "{b}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn unique_target(dir: &Path) -> PathBuf {
        let path = dir.join("target.py");
        let mut body = String::new();
        for i in 1..=5 {
            body.push_str(&format!("filler_line_{i}_padding_text\n"));
        }
        body.push_str("unique_verify_anchor_token_alpha\n");
        for i in 7..=12 {
            body.push_str(&format!("filler_line_{i}_padding_text\n"));
        }
        std::fs::write(&path, body).expect("write target");
        path
    }

    #[test]
    fn verify_anchor_matches_cited_line() {
        let dir = test_scratch_dir("verify-match");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = unique_target(&dir);
        let status = verify_anchor(&path, 6, "unique_verify_anchor_token_alpha");
        assert_eq!(status, AnchorMatchStatus::MatchesCitedLine);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_anchor_relocated_within_window() {
        let dir = test_scratch_dir("verify-reloc");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = unique_target(&dir);
        let mut lines: Vec<String> = std::fs::read_to_string(&path)
            .expect("read")
            .lines()
            .map(str::to_string)
            .collect();
        for i in 0..3 {
            lines.insert(0, format!("inserted_pad_{i}"));
        }
        std::fs::write(&path, lines.join("\n") + "\n").expect("write");
        let status = verify_anchor(&path, 6, "unique_verify_anchor_token_alpha");
        assert_eq!(status, AnchorMatchStatus::Relocated { new_line: 9 });
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_anchor_lost_when_deleted() {
        let dir = test_scratch_dir("verify-lost");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = unique_target(&dir);
        let mut lines: Vec<String> = std::fs::read_to_string(&path)
            .expect("read")
            .lines()
            .map(str::to_string)
            .collect();
        lines.remove(5);
        std::fs::write(&path, lines.join("\n") + "\n").expect("write");
        let status = verify_anchor(&path, 6, "unique_verify_anchor_token_alpha");
        assert_eq!(status, AnchorMatchStatus::AnchorLost);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_anchor_ambiguous_in_window() {
        let dir = test_scratch_dir("verify-ambig");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = unique_target(&dir);
        let mut lines: Vec<String> = std::fs::read_to_string(&path)
            .expect("read")
            .lines()
            .map(str::to_string)
            .collect();
        let token = lines[5].clone();
        lines[5] = "dummy_replaced_cited_line".to_string();
        lines.insert(7, token.clone());
        lines.insert(9, token);
        std::fs::write(&path, lines.join("\n") + "\n").expect("write");
        let status = verify_anchor(&path, 6, "unique_verify_anchor_token_alpha");
        assert_eq!(status, AnchorMatchStatus::AmbiguousAnchor);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_anchor_moved_beyond_window() {
        let dir = test_scratch_dir("verify-far");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let path = unique_target(&dir);
        let mut lines: Vec<String> = std::fs::read_to_string(&path)
            .expect("read")
            .lines()
            .map(str::to_string)
            .collect();
        let token = lines[5].clone();
        lines[5] = "dummy_replaced_cited_line".to_string();
        for _ in 0..60 {
            lines.push("far_pad".to_string());
        }
        lines.push(token);
        std::fs::write(&path, lines.join("\n") + "\n").expect("write");
        let status = verify_anchor(&path, 6, "unique_verify_anchor_token_alpha");
        assert_eq!(status, AnchorMatchStatus::AnchorMovedFar { found_line: 73 });
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn source_line_supports_asserted_route_and_component() {
        assert!(source_line_supports_asserted_value(
            "    path: '/tools/file-browser/:path*',",
            "/tools/file-browser/:path*",
        ));
        assert!(!source_line_supports_asserted_value(
            "    path: '/tools/feature-provenance',",
            "/tools/file-browser/:path*",
        ));
        assert!(source_line_supports_asserted_value(
            "    component: defineAsyncComponent(() => import('../views/FileBrowserView.vue')),",
            "core/frontend/src/views/FileBrowserView.vue",
        ));
        assert!(!source_line_supports_asserted_value(
            "    component: defineAsyncComponent(() => import('../views/FeatureProvenanceView.vue')),",
            "core/frontend/src/views/FileBrowserView.vue",
        ));
    }

    #[test]
    fn inserted_router_route_refit_refuses_neighbor_anchor() {
        let dir = test_scratch_dir("refit-identity");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("mkdir");
        let repo = catalog_paths::repo_root();
        let original = std::fs::read_to_string(repo.join("core/frontend/src/router/index.ts"))
            .expect("read router");
        let src_lines: Vec<&str> = original.lines().collect();
        let mut without = Vec::new();
        let mut idx = 0usize;
        while idx < src_lines.len() {
            if src_lines[idx].contains("path: '/tools/feature-provenance'") {
                if without
                    .last()
                    .map(|row: &String| row.trim() == "{")
                    .unwrap_or(false)
                {
                    without.pop();
                }
                while idx < src_lines.len() && !src_lines[idx].trim().starts_with("},") {
                    idx += 1;
                }
                idx += 1;
                continue;
            }
            without.push(src_lines[idx].to_string());
            idx += 1;
        }
        let file_browser_idx = without
            .iter()
            .position(|line| line.contains("path: '/tools/file-browser/:path*'"))
            .expect("file-browser path");
        without.insert(file_browser_idx, "  },".to_string());
        without.insert(
            file_browser_idx,
            "    component: defineAsyncComponent(() => import('../views/InsertedView.vue')),"
                .to_string(),
        );
        without.insert(file_browser_idx, "    name: 'Inserted',".to_string());
        without.insert(
            file_browser_idx,
            "    path: '/tools/inserted-neighbor',".to_string(),
        );
        without.insert(file_browser_idx, "  {".to_string());
        let router = dir.join("router.ts");
        std::fs::write(&router, without.join("\n") + "\n").expect("write router");
        let cited = without
            .iter()
            .position(|line| line.contains("path: '/tools/inserted-neighbor'"))
            .expect("inserted path")
            + 1;
        let short = "path: '/too";
        let stolen = refit_anchor(&router, cited as u32, short, 40, None)
            .expect("unguarded refit unique-ifies the occupant");
        println!("reconstructed unguarded refit: {stolen}");
        assert!(
            stolen.contains("inserted-neighbor"),
            "unguarded refit should steal neighbor, got {stolen}"
        );
        let refused = refit_anchor(
            &router,
            cited as u32,
            short,
            40,
            Some("/tools/file-browser/:path*"),
        );
        println!("reconstructed guarded refit: {refused:?}");
        assert!(
            refused.is_none(),
            "guard must refuse neighbor refit, got {refused:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn relocated_line_fix_refuses_when_new_line_is_a_different_entity() {
        let dir = test_scratch_dir("reloc-identity");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("core/frontend/src/router")).expect("mkdir");
        let router = dir.join("core/frontend/src/router/index.ts");
        std::fs::write(
            &router,
            concat!(
                "const routes = [\n",
                "  {\n",
                "    path: '/tools/feature-provenance',\n",
                "    name: 'Feature Provenance',\n",
                "  },\n",
                "  {\n",
                "    path: '/tools/file-browser/:path*',\n",
                "    name: 'File Browser',\n",
                "  },\n",
                "]\n",
            ),
        )
        .expect("write router");
        let site = dir.join("page.rs");
        std::fs::write(
            &site,
            concat!(
                "route: Observed::known(\n",
                "    \"/tools/file-browser/:path*\",\n",
                "    Evidence {\n",
                "        file: \"core/frontend/src/router/index.ts\",\n",
                "        line: 10,\n",
                "        anchor: \"path: '/tools/feature-provenance',\",\n",
                "    },\n",
                "),\n",
            ),
        )
        .expect("write site");
        let applied = apply_relocated_line_fixes_with_root(
            &dir,
            &[LineRelocation {
                site: site.clone(),
                file: "core/frontend/src/router/index.ts".to_string(),
                old_line: 10,
                new_line: 3,
                anchor: "path: '/tools/feature-provenance',".to_string(),
            }],
            &dir,
        )
        .expect("apply");
        assert!(
            applied.is_empty(),
            "identity guard must refuse, applied={applied:?}"
        );
        let body = std::fs::read_to_string(&site).expect("read site");
        assert!(body.contains("line: 10"), "{body}");
        assert!(body.contains("/tools/file-browser/:path*"), "{body}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn nearby_asserted_value_uses_enclosing_known_not_preceding_string() {
        let lines: Vec<String> = [
            r#"        menu_title: Observed::known("#,
            r#"            "Serial Bridges","#,
            r#"            Evidence {"#,
            r#"                file: "core/frontend/src/menus.ts","#,
            r#"                line: 106,"#,
            r#"                anchor: "title: 'Serial Bridges',","#,
            r#"            },"#,
            r#"        ),"#,
            r#"        advanced_only: Observed::known("#,
            r#"            true,"#,
            r#"            Evidence {"#,
            r#"                file: "core/frontend/src/menus.ts","#,
            r#"                line: 109,"#,
            r#"                anchor: "advanced: true,","#,
            r#"            },"#,
            r#"        ),"#,
        ]
        .into_iter()
        .map(str::to_string)
        .collect();
        assert_eq!(
            nearby_asserted_value(&lines, 5).as_deref(),
            Some("Serial Bridges")
        );
        assert_eq!(nearby_asserted_value(&lines, 13).as_deref(), Some("true"));
    }

    #[test]
    fn asserted_value_near_citation_respects_line_number_field_boundary() {
        let content = concat!(
            "        menu_title: Observed::known(\n",
            "            \"Serial Bridges\",\n",
            "            Evidence {\n",
            "                file: \"core/frontend/src/menus.ts\",\n",
            "                line: 109,\n",
            "                anchor: \"title: 'Serial Bridges',\",\n",
            "            },\n",
            "        ),\n",
        );
        assert!(
            asserted_value_near_citation(content, 10, "title: 'Serial Bridges',").is_none(),
            "line: 10 must not match line: 109"
        );
        assert_eq!(
            asserted_value_near_citation(content, 109, "title: 'Serial Bridges',").as_deref(),
            Some("Serial Bridges")
        );
    }

    #[test]
    fn nearby_asserted_value_uses_evidenced_new_not_preceding_bool() {
        let lines: Vec<String> = [
            r#"        advanced_only: Observed::known("#,
            r#"            false,"#,
            r#"            Evidence {"#,
            r#"                file: "core/frontend/src/menus.ts","#,
            r#"                line: 6,"#,
            r#"                anchor: "advanced: false,","#,
            r#"            },"#,
            r#"        ),"#,
            r#"        stores: ObservedSet::known(&["#,
            r#"            Evidenced::new("#,
            r#"                "autopilot_data","#,
            r#"                Evidence {"#,
            r#"                    file: "core/frontend/src/views/Autopilot.vue","#,
            r#"                    line: 158,"#,
            r#"                    anchor: "import autopilot_data from '@/store/autopilot'","#,
            r#"                },"#,
            r#"            ),"#,
        ]
        .into_iter()
        .map(str::to_string)
        .collect();
        assert_eq!(nearby_asserted_value(&lines, 5).as_deref(), Some("false"));
        assert_eq!(
            nearby_asserted_value(&lines, 14).as_deref(),
            Some("autopilot_data")
        );
    }

    #[test]
    fn relocated_advanced_only_repairs_when_new_line_has_bool() {
        let dir = test_scratch_dir("reloc-bool");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("core/frontend/src")).expect("mkdir");
        let menus = dir.join("core/frontend/src/menus.ts");
        std::fs::write(
            &menus,
            concat!(
                "const menus = [\n",
                "  {\n",
                "    title: 'Inserted',\n",
                "    route: '/tools/inserted',\n",
                "    advanced: false,\n",
                "    text: 'inserted description padding.',\n",
                "  },\n",
                "  {\n",
                "    title: 'Serial Bridges',\n",
                "    route: '/tools/bridges',\n",
                "    advanced: true,\n",
                "    text: 'Allows creating unique UDP/TCP to Serial bridges here.',\n",
                "  },\n",
                "]\n",
            ),
        )
        .expect("write menus");
        let site = dir.join("page.rs");
        std::fs::write(
            &site,
            concat!(
                "menu_title: Observed::known(\n",
                "    \"Serial Bridges\",\n",
                "    Evidence {\n",
                "        file: \"core/frontend/src/menus.ts\",\n",
                "        line: 9,\n",
                "        anchor: \"title: 'Serial Bridges',\",\n",
                "    },\n",
                "),\n",
                "advanced_only: Observed::known(\n",
                "    true,\n",
                "    Evidence {\n",
                "        file: \"core/frontend/src/menus.ts\",\n",
                "        line: 4,\n",
                "        anchor: \"advanced: true,\",\n",
                "    },\n",
                "),\n",
            ),
        )
        .expect("write site");
        let applied = apply_relocated_line_fixes_with_root(
            &dir,
            &[LineRelocation {
                site: site.clone(),
                file: "core/frontend/src/menus.ts".to_string(),
                old_line: 4,
                new_line: 11,
                anchor: "advanced: true,".to_string(),
            }],
            &dir,
        )
        .expect("apply");
        assert_eq!(
            applied.len(),
            1,
            "bool identity must allow, applied={applied:?}"
        );
        let body = std::fs::read_to_string(&site).expect("read site");
        assert!(body.contains("line: 11"), "{body}");
        let refused = apply_relocated_line_fixes_with_root(
            &dir,
            &[LineRelocation {
                site: site.clone(),
                file: "core/frontend/src/menus.ts".to_string(),
                old_line: 11,
                new_line: 3,
                anchor: "advanced: true,".to_string(),
            }],
            &dir,
        )
        .expect("apply refuse");
        assert!(
            refused.is_empty(),
            "must refuse neighbour title, applied={refused:?}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
