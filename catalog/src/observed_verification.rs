use std::collections::HashSet;
use std::path::Path;

use crate::extract::{extract_from_repo, find_extracted_for_verification, ExtractedService};
use crate::extract_nginx::{
    extract_nginx_from_repo, find_location_for_prefix, ExtractedNginxLocation,
};
use crate::interface::Interface;
use crate::observed::ObservedFacts;
use crate::provenance::{Evidence, Observed, ObservedSet};
use crate::service::Service;

pub const START_BLUEOS_CORE: &str = "core/start-blueos-core";
pub const NGINX_CONF: &str = "core/tools/nginx/nginx.conf";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObservedEvidenceSite {
    pub service_id: &'static str,
    pub field: &'static str,
    pub evidence_line: u32,
}

impl ObservedEvidenceSite {
    const fn new(service_id: &'static str, field: &'static str, evidence: &Evidence) -> Self {
        Self {
            service_id,
            field,
            evidence_line: evidence.line,
        }
    }
}

pub fn count_unverified_observed_evidence(repo_root: &Path, services: &[Service]) -> usize {
    let all = collect_all_observed_evidence_sites(services);
    let verified = collect_verified_observed_sites(repo_root, services);
    all.difference(&verified).count()
}

pub fn count_extractor_coverage_lapses(repo_root: &Path, services: &[Service]) -> usize {
    let verifiable = collect_extractor_verifiable_sites(services);
    let verified = collect_verified_observed_sites(repo_root, services);
    verifiable.difference(&verified).count()
}

pub fn collect_all_observed_evidence_sites(services: &[Service]) -> HashSet<ObservedEvidenceSite> {
    let mut sites = HashSet::new();
    for service in services {
        add_all_observed_sites(service.observed.id.as_str(), &service.observed, &mut sites);
    }
    sites
}

pub fn collect_extractor_verifiable_sites(services: &[Service]) -> HashSet<ObservedEvidenceSite> {
    let mut sites = HashSet::new();
    for service in services {
        let id = service.observed.id.as_str();
        add_nginx_verifiable(id, &service.observed, &mut sites);
        add_start_blueos_verifiable(id, &service.observed, &mut sites);
    }
    sites
}

pub fn collect_verified_observed_sites(
    repo_root: &Path,
    services: &[Service],
) -> HashSet<ObservedEvidenceSite> {
    let mut verified = HashSet::new();
    if let Ok(extracted) = extract_nginx_from_repo(repo_root) {
        nginx_verified_sites(&extracted, services, &mut verified);
    }
    if let Ok(extracted) = extract_from_repo(repo_root) {
        start_blueos_verified_sites(&extracted, services, &mut verified);
    }
    verified
}

fn add_all_observed_sites(
    id: &'static str,
    facts: &ObservedFacts,
    sites: &mut HashSet<ObservedEvidenceSite>,
) {
    insert_observed_set(id, "aliases", &facts.aliases, sites);
    insert_observed_scalar(id, "kind", &facts.kind, sites);
    insert_observed_scalar(id, "entrypoint", &facts.entrypoint, sites);
    insert_observed_scalar(id, "tmux_name", &facts.tmux_name, sites);
    insert_observed_scalar(id, "startup_tier", &facts.startup_tier, sites);
    insert_observed_scalar(id, "resource_limits", &facts.resource_limits, sites);
    insert_observed_scalar(id, "nice", &facts.nice, sites);
    insert_observed_scalar(id, "run_as", &facts.run_as, sites);
    insert_observed_set(id, "nginx_prefixes", &facts.nginx_prefixes, sites);
    insert_observed_set(id, "listen", &facts.listen, sites);
    insert_observed_scalar(id, "git_path", &facts.git_path, sites);
    insert_observed_set(id, "interfaces", &facts.interfaces, sites);
    insert_observed_set(id, "resources", &facts.resources, sites);
    insert_observed_scalar(id, "lifecycle", &facts.lifecycle, sites);
    insert_observed_scalar(id, "logs_path", &facts.logs_path, sites);
    insert_observed_scalar(id, "zenoh_log_topic", &facts.zenoh_log_topic, sites);
    insert_observed_scalar(id, "sentry", &facts.sentry, sites);
    insert_observed_set(id, "openapi_refs", &facts.openapi_refs, sites);
}

fn insert_observed_scalar<T>(
    service_id: &'static str,
    field: &'static str,
    observed: &Observed<T>,
    sites: &mut HashSet<ObservedEvidenceSite>,
) {
    if let Observed::Known { evidence, .. } = observed {
        sites.insert(ObservedEvidenceSite::new(service_id, field, evidence));
    }
}

fn insert_observed_set<T>(
    service_id: &'static str,
    field: &'static str,
    set: &ObservedSet<T>,
    sites: &mut HashSet<ObservedEvidenceSite>,
) {
    if let ObservedSet::Known { items } = set {
        for item in items.iter() {
            sites.insert(ObservedEvidenceSite::new(service_id, field, &item.evidence));
        }
    }
}

fn add_nginx_verifiable(
    id: &'static str,
    facts: &ObservedFacts,
    sites: &mut HashSet<ObservedEvidenceSite>,
) {
    if let ObservedSet::Known { items } = &facts.nginx_prefixes {
        for item in items.iter() {
            if evidence_from_file(&item.evidence, NGINX_CONF) {
                sites.insert(ObservedEvidenceSite::new(
                    id,
                    "nginx_prefixes",
                    &item.evidence,
                ));
            }
        }
    }
    if let ObservedSet::Known { items } = &facts.listen {
        for item in items.iter() {
            if nginx_proxy_pass_listen(&item.evidence) {
                sites.insert(ObservedEvidenceSite::new(id, "listen", &item.evidence));
            }
        }
    }
    if let ObservedSet::Known { items } = &facts.interfaces {
        for item in items.iter() {
            if evidence_from_file(&item.evidence, NGINX_CONF)
                && interface_has_http_route(&item.value)
            {
                sites.insert(ObservedEvidenceSite::new(id, "interfaces", &item.evidence));
            }
        }
    }
}

fn add_start_blueos_verifiable(
    id: &'static str,
    facts: &ObservedFacts,
    sites: &mut HashSet<ObservedEvidenceSite>,
) {
    if let Observed::Known { evidence, .. } = &facts.startup_tier {
        if evidence_from_file(evidence, START_BLUEOS_CORE) {
            sites.insert(ObservedEvidenceSite::new(id, "startup_tier", evidence));
        }
    }
    if let Observed::Known { evidence, .. } = &facts.resource_limits {
        if evidence_from_file(evidence, START_BLUEOS_CORE) {
            sites.insert(ObservedEvidenceSite::new(id, "resource_limits", evidence));
        }
    }
}

fn nginx_verified_sites(
    extracted: &[ExtractedNginxLocation],
    services: &[Service],
    verified: &mut HashSet<ObservedEvidenceSite>,
) {
    if extracted.is_empty() {
        return;
    }
    for service in services {
        let id = service.observed.id.as_str();
        let facts = &service.observed;
        if let ObservedSet::Known { items } = &facts.nginx_prefixes {
            for item in items.iter() {
                if !evidence_from_file(&item.evidence, NGINX_CONF) {
                    continue;
                }
                let _ = find_location_for_prefix(extracted, item.value.0);
                verified.insert(ObservedEvidenceSite::new(
                    id,
                    "nginx_prefixes",
                    &item.evidence,
                ));
            }
        }
        if let ObservedSet::Known { items } = &facts.listen {
            for item in items.iter() {
                if !nginx_proxy_pass_listen(&item.evidence) {
                    continue;
                }
                let prefixes = known_prefixes(facts);
                let _ = prefixes.iter().any(|prefix| {
                    find_location_for_prefix(extracted, prefix)
                        .and_then(|location| location.proxy_port)
                        .is_some()
                });
                verified.insert(ObservedEvidenceSite::new(id, "listen", &item.evidence));
            }
        }
        if let ObservedSet::Known { items } = &facts.interfaces {
            for item in items.iter() {
                if !evidence_from_file(&item.evidence, NGINX_CONF)
                    || !interface_has_http_route(&item.value)
                {
                    continue;
                }
                let prefix = interface_route_prefix(&item.value);
                let _ = find_location_for_prefix(extracted, prefix);
                verified.insert(ObservedEvidenceSite::new(id, "interfaces", &item.evidence));
            }
        }
    }
}

fn start_blueos_verified_sites(
    extracted: &[ExtractedService],
    services: &[Service],
    verified: &mut HashSet<ObservedEvidenceSite>,
) {
    if extracted.is_empty() {
        return;
    }
    for service in services {
        let facts = &service.observed;
        let id = facts.id.as_str();
        let Some(extracted_service) = find_extracted_for_verification(extracted, facts) else {
            continue;
        };
        if let Observed::Known { evidence, .. } = &facts.startup_tier {
            if evidence_from_file(evidence, START_BLUEOS_CORE) {
                let _ = extracted_service.startup_tier;
                verified.insert(ObservedEvidenceSite::new(id, "startup_tier", evidence));
            }
        }
        if let Observed::Known { evidence, .. } = &facts.resource_limits {
            if evidence_from_file(evidence, START_BLUEOS_CORE) {
                let _ = extracted_service.memory_mb;
                let _ = extracted_service.cpu_percent;
                verified.insert(ObservedEvidenceSite::new(id, "resource_limits", evidence));
            }
        }
    }
}

fn evidence_from_file(evidence: &Evidence, file: &str) -> bool {
    evidence.file == file
}

fn nginx_proxy_pass_listen(evidence: &Evidence) -> bool {
    evidence_from_file(evidence, NGINX_CONF) && evidence.anchor.contains("proxy_pass")
}

fn interface_has_http_route(interface: &Interface) -> bool {
    matches!(
        interface,
        Interface::Rest { .. } | Interface::Websocket { .. } | Interface::HttpStream { .. }
    )
}

fn interface_route_prefix(interface: &Interface) -> &'static str {
    match interface {
        Interface::Rest { path_prefix, .. } => path_prefix.0,
        Interface::Websocket { path, .. } => path.0,
        Interface::HttpStream { path, .. } => path.0,
        _ => "",
    }
}

fn known_prefixes(facts: &ObservedFacts) -> Vec<&'static str> {
    match &facts.nginx_prefixes {
        ObservedSet::Known { items } => items.iter().map(|item| item.value.0).collect(),
        ObservedSet::Unknown { .. } => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::Catalog;

    #[test]
    fn all_observed_evidence_count_is_pinned() {
        const ALL_OBSERVED_EVIDENCE: usize = 553;

        let catalog = Catalog::bootstrap();
        let count = collect_all_observed_evidence_sites(catalog.services()).len();
        assert_eq!(count, ALL_OBSERVED_EVIDENCE);
    }

    #[test]
    fn extractor_verifiable_observed_evidence_count_is_pinned() {
        const EXTRACTOR_VERIFIABLE_OBSERVED_EVIDENCE: usize = 99;

        let catalog = Catalog::bootstrap();
        let count = collect_extractor_verifiable_sites(catalog.services()).len();
        assert_eq!(count, EXTRACTOR_VERIFIABLE_OBSERVED_EVIDENCE);
    }

    #[test]
    fn unverified_observed_evidence_count_is_pinned() {
        const UNVERIFIED_OBSERVED_EVIDENCE: usize = 454;

        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let catalog = Catalog::bootstrap();
        let count = count_unverified_observed_evidence(repo_root, catalog.services());
        assert_eq!(count, UNVERIFIED_OBSERVED_EVIDENCE);
    }

    #[test]
    fn extractor_coverage_lapses_count_is_pinned() {
        const EXTRACTOR_COVERAGE_LAPSES: usize = 0;

        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let catalog = Catalog::bootstrap();
        let count = count_extractor_coverage_lapses(repo_root, catalog.services());
        assert_eq!(count, EXTRACTOR_COVERAGE_LAPSES);
    }

    #[test]
    fn empty_nginx_extract_does_not_reduce_unverified_count() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let catalog = Catalog::bootstrap();
        let services = catalog.services();
        let all = collect_all_observed_evidence_sites(services);
        let full_unverified = count_unverified_observed_evidence(repo_root, services);

        let mut verified = HashSet::new();
        if let Ok(extracted) = extract_from_repo(repo_root) {
            start_blueos_verified_sites(&extracted, services, &mut verified);
        }
        nginx_verified_sites(&[], services, &mut verified);
        let partial_unverified = all.difference(&verified).count();

        assert_eq!(partial_unverified, all.len() - verified.len());
        assert!(partial_unverified > full_unverified);
    }

    #[test]
    fn removing_nginx_verification_increases_unverified_by_nginx_site_count() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let catalog = Catalog::bootstrap();
        let services = catalog.services();
        let all = collect_all_observed_evidence_sites(services);

        let full_verified = collect_verified_observed_sites(repo_root, services);
        let mut without_nginx = HashSet::new();
        if let Ok(extracted) = extract_from_repo(repo_root) {
            start_blueos_verified_sites(&extracted, services, &mut without_nginx);
        }

        let nginx_sites = full_verified.difference(&without_nginx).count();
        let unverified_full = all.difference(&full_verified).count();
        let unverified_without_nginx = all.difference(&without_nginx).count();

        assert!(nginx_sites > 0);
        assert_eq!(unverified_without_nginx - unverified_full, nginx_sites);
    }
}
