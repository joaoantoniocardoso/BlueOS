use std::fs;
use std::path::Path;

use schemars::JsonSchema;
use serde::Serialize;

use crate::drift::{DriftFinding, DriftReport};
use catalog_kernel::provenance::Observed;
use catalog_model::observed::{ObservedFacts, StartupTier};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ExtractedService {
    pub tmux_name: String,
    pub service_dir: Option<String>,
    pub startup_tier: StartupTier,
    pub memory_mb: Option<u32>,
    pub cpu_percent: Option<u32>,
    pub command: String,
}

#[derive(Debug)]
pub enum ExtractError {
    Io(String),
    Parse(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArrayContext {
    Priority,
    Normal,
}

pub fn extract_from_repo(repo_root: &Path) -> Result<Vec<ExtractedService>, ExtractError> {
    let path = repo_root.join("core/start-blueos-core");
    let contents = fs::read_to_string(&path)
        .map_err(|err| ExtractError::Io(format!("read {}: {err}", path.display())))?;

    let mut context: Option<ArrayContext> = None;
    let mut services = Vec::new();

    for (line_number, line) in contents.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with('#') {
            continue;
        }

        if trimmed == "PRIORITY_SERVICES=(" {
            context = Some(ArrayContext::Priority);
            continue;
        }
        if trimmed == "SERVICES=(" {
            context = Some(ArrayContext::Normal);
            continue;
        }
        if trimmed == ")" {
            context = None;
            continue;
        }

        let Some(tier) = context else {
            continue;
        };

        let first_non_space = line.chars().find(|ch| !ch.is_whitespace());
        if first_non_space != Some('\'') {
            continue;
        }

        let startup_tier = match tier {
            ArrayContext::Priority => StartupTier::Priority,
            ArrayContext::Normal => StartupTier::Normal,
        };
        let service = parse_tuple_line(line, startup_tier).map_err(|message| {
            ExtractError::Parse(format!("line {}: {message}", line_number + 1))
        })?;
        services.push(service);
    }

    Ok(services)
}

fn parse_tuple_line(line: &str, tier: StartupTier) -> Result<ExtractedService, String> {
    let trimmed = line.trim();
    let name = parse_quoted_name(trimmed)?;
    let after_name = trimmed[name.len() + 2..].trim_start();
    if !after_name.starts_with(',') {
        return Err(format!("expected comma after name in `{trimmed}`"));
    }

    let fields_and_command = after_name[1..].trim_start();
    let (memory_mb, cpu_percent, _io_read, _io_write, command) =
        parse_fields_and_command(fields_and_command)?;

    Ok(ExtractedService {
        tmux_name: name,
        service_dir: service_dir_from_command(&command),
        startup_tier: tier,
        memory_mb: Some(memory_mb),
        cpu_percent: Some(cpu_percent),
        command,
    })
}

fn parse_quoted_name(line: &str) -> Result<String, String> {
    if !line.starts_with('\'') {
        return Err(format!("tuple line must start with quoted name: `{line}`"));
    }
    let end = line[1..]
        .find('\'')
        .ok_or_else(|| format!("unterminated service name in `{line}`"))?;
    Ok(line[1..=end].to_string())
}

fn parse_fields_and_command(input: &str) -> Result<(u32, u32, u32, u32, String), String> {
    let mut rest = input;
    let memory_mb = parse_next_u32_field(&mut rest)?;
    let cpu_percent = parse_next_u32_field(&mut rest)?;
    let io_read = parse_next_u32_field(&mut rest)?;
    let io_write = parse_next_u32_field(&mut rest)?;
    let command = parse_quoted_command(rest.trim())?;
    Ok((memory_mb, cpu_percent, io_read, io_write, command))
}

fn parse_next_u32_field(rest: &mut &str) -> Result<u32, String> {
    let trimmed = rest.trim_start();
    let comma = trimmed
        .find(',')
        .ok_or_else(|| format!("expected comma-separated integer field in `{trimmed}`"))?;
    let field = trimmed[..comma].trim();
    let value = field
        .parse::<u32>()
        .map_err(|_| format!("invalid integer field `{field}`"))?;
    *rest = &trimmed[comma + 1..];
    Ok(value)
}

fn parse_quoted_command(input: &str) -> Result<String, String> {
    let Some(quote) = input.chars().next().filter(|ch| *ch == '"' || *ch == '\'') else {
        return Err(format!("expected quoted command in `{input}`"));
    };
    let inner = &input[1..];
    let end = inner
        .rfind(quote)
        .ok_or_else(|| format!("unterminated command quote in `{input}`"))?;
    Ok(inner[..end].to_string())
}

fn service_dir_from_command(command: &str) -> Option<String> {
    const PREFIX: &str = "$SERVICES_PATH/";
    let start = command.find(PREFIX)? + PREFIX.len();
    let remainder = &command[start..];
    let slash = remainder.find('/')?;
    let dir = &remainder[..slash];
    if remainder[slash..].starts_with("/main.py") {
        Some(dir.to_string())
    } else {
        None
    }
}

pub fn check_against_observed(
    extracted: &[ExtractedService],
    services: &[catalog_model::service::Service],
) -> DriftReport {
    let mut findings = Vec::new();

    for service in services {
        let facts = &service.observed;
        let Some(service) = find_extracted(extracted, facts) else {
            continue;
        };

        if let Observed::Known { value, .. } = &facts.startup_tier {
            if *value != service.startup_tier {
                findings.push(DriftFinding {
                    field: format!("{}.startup_tier", facts.id.as_str()),
                    message: format!("observed {value:?}, extracted {:?}", service.startup_tier),
                });
            }
        }

        if let Observed::Known { value: limits, .. } = &facts.resource_limits {
            if limits.memory_mb != service.memory_mb {
                findings.push(DriftFinding {
                    field: format!("{}.memory_mb", facts.id.as_str()),
                    message: format!(
                        "observed {:?}, extracted {:?}",
                        limits.memory_mb, service.memory_mb
                    ),
                });
            }
            if limits.cpu_percent != service.cpu_percent {
                findings.push(DriftFinding {
                    field: format!("{}.cpu_percent", facts.id.as_str()),
                    message: format!(
                        "observed {:?}, extracted {:?}",
                        limits.cpu_percent, service.cpu_percent
                    ),
                });
            }
        }

        // nice is excluded: shell `nice --19` vs `nice -19` sign semantics are ambiguous in start-blueos-core.
    }

    DriftReport { findings }
}

pub fn find_extracted_for_verification<'a>(
    extracted: &'a [ExtractedService],
    facts: &ObservedFacts,
) -> Option<&'a ExtractedService> {
    find_extracted(extracted, facts)
}

fn find_extracted<'a>(
    extracted: &'a [ExtractedService],
    facts: &ObservedFacts,
) -> Option<&'a ExtractedService> {
    let id = facts.id.as_str();
    if let Some(service) = extracted
        .iter()
        .find(|service| service.service_dir.as_deref() == Some(id))
    {
        return Some(service);
    }
    if let Some(service) = extracted.iter().find(|service| service.tmux_name == id) {
        return Some(service);
    }
    // Binaries whose catalog id differs from the tmux name (e.g. id `mavlink-camera-manager`
    // for tmux `video`) still cross-check against source via the observed tmux_name.
    if let Observed::Known { value: tmux, .. } = &facts.tmux_name {
        return extracted.iter().find(|service| service.tmux_name == *tmux);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use catalog_core::catalog::Catalog;

    #[test]
    fn extract_finds_known_services() {
        let repo_root = catalog_paths::repo_root();
        let extracted = extract_from_repo(&repo_root).expect("extract from repo");

        let kraken = extracted
            .iter()
            .find(|service| service.service_dir == Some("kraken".to_string()))
            .expect("kraken entry");
        assert_eq!(kraken.startup_tier, StartupTier::Normal);
        assert_eq!(kraken.memory_mb, Some(0));

        let ardupilot = extracted
            .iter()
            .find(|service| service.service_dir == Some("ardupilot_manager".to_string()))
            .expect("ardupilot_manager entry");
        assert_eq!(ardupilot.startup_tier, StartupTier::Priority);
    }

    #[test]
    fn bootstrap_observed_matches_source() {
        let repo_root = catalog_paths::repo_root();
        let extracted = extract_from_repo(&repo_root).expect("extract from repo");
        let report = check_against_observed(&extracted, Catalog::bootstrap().services());
        assert!(!report.has_drift(), "drift findings: {:?}", report.findings);
    }
}
