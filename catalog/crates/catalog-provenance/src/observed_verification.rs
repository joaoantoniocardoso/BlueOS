use std::collections::HashSet;
use std::path::Path;

use catalog_core::catalog::Catalog;
use catalog_extract::extract::{
    extract_from_repo, find_extracted_for_verification, ExtractedService,
};
use catalog_extract::extract_fastapi::{
    evidence_from_fastapi_source, extract_fastapi_from_repo, extracted_fastapi_matches_observed,
    find_fastapi_route_for_observed, ExtractedFastApiRoute,
};
use catalog_extract::extract_frontend_router::{
    evidence_from_menus, evidence_from_router, extract_frontend_router_from_repo,
    find_menu_entry_at_line, find_router_route_at_line, menu_matches_page_title,
    ExtractedFrontendMenu, ExtractedFrontendRoute,
};
use catalog_extract::extract_nginx::{
    extract_nginx_from_repo, find_location_for_prefix, ExtractedNginxLocation,
};
use catalog_kernel::id::refs::PortRef;
use catalog_kernel::provenance::{
    Evidence, Grounded, GroundedSet, Observed, ObservedSet, Provenance,
};
use catalog_model::interface::PortKind;
use catalog_model::journey::UseCase;
use catalog_model::observed::ObservedFacts;
use catalog_model::page::Page;
use catalog_model::service::Service;

pub const START_BLUEOS_CORE: &str = "core/start-blueos-core";
pub const NGINX_CONF: &str = "core/tools/nginx/nginx.conf";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObservedEvidenceSite {
    pub scope: &'static str,
    pub entity_id: &'static str,
    pub field: &'static str,
    pub evidence_line: u32,
}

impl ObservedEvidenceSite {
    const fn new(
        scope: &'static str,
        entity_id: &'static str,
        field: &'static str,
        evidence: &Evidence,
    ) -> Self {
        Self {
            scope,
            entity_id,
            field,
            evidence_line: evidence.line,
        }
    }
}

pub fn count_unverified_observed_evidence(repo_root: &Path, catalog: &Catalog) -> usize {
    let all = collect_all_observed_evidence_sites(catalog);
    let verified = collect_verified_observed_sites(repo_root, catalog);
    all.difference(&verified).count()
}

pub fn count_extractor_coverage_lapses(repo_root: &Path, catalog: &Catalog) -> usize {
    let verifiable = collect_extractor_verifiable_sites(catalog);
    let verified = collect_verified_observed_sites(repo_root, catalog);
    verifiable.difference(&verified).count()
}

pub fn collect_all_observed_evidence_sites(catalog: &Catalog) -> HashSet<ObservedEvidenceSite> {
    let mut sites = HashSet::new();
    for service in catalog.services() {
        add_all_observed_sites(service.observed.id.as_str(), &service.observed, &mut sites);
    }
    for page in catalog.pages() {
        add_all_page_observed_sites(page, &mut sites);
    }
    for journey in catalog.journeys() {
        add_all_journey_source_sites(journey, &mut sites);
    }
    sites
}

pub fn collect_extractor_verifiable_sites(catalog: &Catalog) -> HashSet<ObservedEvidenceSite> {
    let mut sites = HashSet::new();
    for service in catalog.services() {
        let id = service.observed.id.as_str();
        add_nginx_verifiable(id, &service.observed, &mut sites);
        add_start_blueos_verifiable(id, &service.observed, &mut sites);
    }
    for journey in catalog.journeys() {
        add_journey_fastapi_verifiable(journey, &mut sites);
    }
    for page in catalog.pages() {
        add_page_frontend_verifiable(page, &mut sites);
    }
    sites
}

pub fn collect_verified_observed_sites(
    repo_root: &Path,
    catalog: &Catalog,
) -> HashSet<ObservedEvidenceSite> {
    let mut verified = HashSet::new();
    let services = catalog.services();
    if let Ok(extracted) = extract_nginx_from_repo(repo_root) {
        nginx_verified_sites(&extracted, services, &mut verified);
    }
    if let Ok(extracted) = extract_from_repo(repo_root) {
        start_blueos_verified_sites(&extracted, services, &mut verified);
    }
    if let Ok(extracted) = extract_fastapi_from_repo(repo_root) {
        fastapi_verified_sites(&extracted, catalog.journeys(), &mut verified);
    }
    if let Ok((routes, menus)) = extract_frontend_router_from_repo(repo_root) {
        frontend_verified_sites(&routes, &menus, catalog.pages(), &mut verified);
    }
    verified
}

fn add_journey_fastapi_verifiable(journey: &UseCase, sites: &mut HashSet<ObservedEvidenceSite>) {
    let Some(steps) = journey_steps(journey) else {
        return;
    };
    for step in steps {
        let Some(Grounded::Known {
            provenance: Provenance::Source(evidence),
            ..
        }) = &step.value.route
        else {
            continue;
        };
        if evidence_from_fastapi_source(evidence) {
            sites.insert(ObservedEvidenceSite::new(
                "journey",
                journey.id.as_str(),
                "step.route",
                evidence,
            ));
        }
    }
}

fn add_page_frontend_verifiable(page: &Page, sites: &mut HashSet<ObservedEvidenceSite>) {
    if let Observed::Known { evidence, .. } = &page.route {
        if evidence_from_router(evidence) {
            sites.insert(ObservedEvidenceSite::new(
                "page",
                page.id.as_str(),
                "route",
                evidence,
            ));
        }
    }
    if let Observed::Known { evidence, .. } = &page.name {
        if evidence_from_router(evidence) {
            sites.insert(ObservedEvidenceSite::new(
                "page",
                page.id.as_str(),
                "name",
                evidence,
            ));
        }
    }
    if let Observed::Known { evidence, .. } = &page.component {
        if evidence_from_router(evidence) {
            sites.insert(ObservedEvidenceSite::new(
                "page",
                page.id.as_str(),
                "component",
                evidence,
            ));
        }
    }
    if let Observed::Known { evidence, .. } = &page.menu_title {
        if evidence_from_menus(evidence) {
            sites.insert(ObservedEvidenceSite::new(
                "page",
                page.id.as_str(),
                "menu_title",
                evidence,
            ));
        }
    }
    if let Observed::Known { evidence, .. } = &page.advanced_only {
        if evidence_from_menus(evidence) {
            sites.insert(ObservedEvidenceSite::new(
                "page",
                page.id.as_str(),
                "advanced_only",
                evidence,
            ));
        }
    }
}

fn add_all_observed_sites(
    id: &'static str,
    facts: &ObservedFacts,
    sites: &mut HashSet<ObservedEvidenceSite>,
) {
    insert_observed_set("service", id, "aliases", &facts.aliases, sites);
    insert_observed_scalar("service", id, "kind", &facts.kind, sites);
    insert_observed_scalar("service", id, "entrypoint", &facts.entrypoint, sites);
    insert_observed_scalar("service", id, "tmux_name", &facts.tmux_name, sites);
    insert_observed_scalar("service", id, "startup_tier", &facts.startup_tier, sites);
    insert_observed_scalar(
        "service",
        id,
        "resource_limits",
        &facts.resource_limits,
        sites,
    );
    insert_observed_scalar("service", id, "nice", &facts.nice, sites);
    insert_observed_scalar("service", id, "run_as", &facts.run_as, sites);
    insert_observed_set(
        "service",
        id,
        "nginx_prefixes",
        &facts.nginx_prefixes,
        sites,
    );
    insert_observed_set("service", id, "listen", &facts.listen, sites);
    insert_observed_scalar("service", id, "git_path", &facts.git_path, sites);
    insert_observed_set("service", id, "interfaces", &facts.interfaces, sites);
    insert_observed_set("service", id, "resources", &facts.resources, sites);
    insert_observed_scalar("service", id, "lifecycle", &facts.lifecycle, sites);
    insert_observed_scalar("service", id, "logs_path", &facts.logs_path, sites);
    insert_observed_scalar(
        "service",
        id,
        "zenoh_log_topic",
        &facts.zenoh_log_topic,
        sites,
    );
    insert_observed_scalar("service", id, "sentry", &facts.sentry, sites);
    insert_observed_set("service", id, "openapi_refs", &facts.openapi_refs, sites);
}

fn add_all_page_observed_sites(page: &Page, sites: &mut HashSet<ObservedEvidenceSite>) {
    let id = page.id.as_str();
    insert_observed_scalar("page", id, "route", &page.route, sites);
    insert_observed_scalar("page", id, "name", &page.name, sites);
    insert_observed_scalar("page", id, "component", &page.component, sites);
    insert_observed_scalar("page", id, "menu_title", &page.menu_title, sites);
    insert_observed_scalar("page", id, "advanced_only", &page.advanced_only, sites);
    insert_observed_set("page", id, "stores", &page.stores, sites);
    insert_observed_set("page", id, "consumes", &page.consumes, sites);
}

fn add_all_journey_source_sites(journey: &UseCase, sites: &mut HashSet<ObservedEvidenceSite>) {
    let id = journey.id.as_str();
    insert_grounded_source("journey", id, "summary", &journey.summary, sites);
    insert_grounded_source("journey", id, "visibility", &journey.visibility, sites);
    insert_grounded_set_source("journey", id, "services", &journey.services, sites);
    insert_grounded_set_source(
        "journey",
        id,
        "capability_refs",
        &journey.capability_refs,
        sites,
    );
    insert_grounded_set_source(
        "journey",
        id,
        "preconditions",
        &journey.preconditions,
        sites,
    );
    insert_grounded_source("journey", id, "blast_radius", &journey.blast_radius, sites);
    let Some(steps) = journey_steps(journey) else {
        return;
    };
    for step in steps {
        insert_source_evidence("journey", id, "step", &step.provenance, sites);
        if let Some(route) = &step.value.route {
            insert_grounded_source("journey", id, "step.route", route, sites);
        }
        if let Some(outcome) = &step.value.outcome {
            insert_grounded_source("journey", id, "step.outcome", outcome, sites);
        }
    }
}

fn insert_observed_scalar<T>(
    scope: &'static str,
    entity_id: &'static str,
    field: &'static str,
    observed: &Observed<T>,
    sites: &mut HashSet<ObservedEvidenceSite>,
) {
    if let Observed::Known { evidence, .. } = observed {
        sites.insert(ObservedEvidenceSite::new(scope, entity_id, field, evidence));
    }
}

fn insert_observed_set<T>(
    scope: &'static str,
    entity_id: &'static str,
    field: &'static str,
    set: &ObservedSet<T>,
    sites: &mut HashSet<ObservedEvidenceSite>,
) {
    if let ObservedSet::Known { items } = set {
        for item in items.iter() {
            sites.insert(ObservedEvidenceSite::new(
                scope,
                entity_id,
                field,
                &item.evidence,
            ));
        }
    }
}

fn insert_grounded_source<T>(
    scope: &'static str,
    entity_id: &'static str,
    field: &'static str,
    grounded: &Grounded<T>,
    sites: &mut HashSet<ObservedEvidenceSite>,
) {
    if let Grounded::Known {
        provenance: Provenance::Source(evidence),
        ..
    } = grounded
    {
        sites.insert(ObservedEvidenceSite::new(scope, entity_id, field, evidence));
    }
}

fn insert_grounded_set_source<T>(
    scope: &'static str,
    entity_id: &'static str,
    field: &'static str,
    set: &GroundedSet<T>,
    sites: &mut HashSet<ObservedEvidenceSite>,
) {
    if let GroundedSet::Known { items } = set {
        for item in items.iter() {
            insert_source_evidence(scope, entity_id, field, &item.provenance, sites);
        }
    }
}

fn insert_source_evidence(
    scope: &'static str,
    entity_id: &'static str,
    field: &'static str,
    provenance: &Provenance,
    sites: &mut HashSet<ObservedEvidenceSite>,
) {
    if let Provenance::Source(evidence) = provenance {
        sites.insert(ObservedEvidenceSite::new(scope, entity_id, field, evidence));
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
                    "service",
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
                sites.insert(ObservedEvidenceSite::new(
                    "service",
                    id,
                    "listen",
                    &item.evidence,
                ));
            }
        }
    }
    if let ObservedSet::Known { items } = &facts.interfaces {
        for item in items.iter() {
            if evidence_from_file(&item.evidence, NGINX_CONF)
                && interface_has_http_route(&item.value)
            {
                sites.insert(ObservedEvidenceSite::new(
                    "service",
                    id,
                    "interfaces",
                    &item.evidence,
                ));
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
            sites.insert(ObservedEvidenceSite::new(
                "service",
                id,
                "startup_tier",
                evidence,
            ));
        }
    }
    if let Observed::Known { evidence, .. } = &facts.resource_limits {
        if evidence_from_file(evidence, START_BLUEOS_CORE) {
            sites.insert(ObservedEvidenceSite::new(
                "service",
                id,
                "resource_limits",
                evidence,
            ));
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
                if find_location_for_prefix(extracted, item.value.0).is_some() {
                    verified.insert(ObservedEvidenceSite::new(
                        "service",
                        id,
                        "nginx_prefixes",
                        &item.evidence,
                    ));
                }
            }
        }
        if let ObservedSet::Known { items } = &facts.listen {
            for item in items.iter() {
                if !nginx_proxy_pass_listen(&item.evidence) {
                    continue;
                }
                let PortRef::Literal(port) = item.value else {
                    continue;
                };
                let prefixes = known_prefixes(facts);
                if prefixes.iter().any(|prefix| {
                    find_location_for_prefix(extracted, prefix)
                        .and_then(|location| location.proxy_port)
                        == Some(port)
                }) {
                    verified.insert(ObservedEvidenceSite::new(
                        "service",
                        id,
                        "listen",
                        &item.evidence,
                    ));
                }
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
                let expected_port = match &item.value {
                    PortKind::Rest {
                        port: PortRef::Literal(port),
                        ..
                    }
                    | PortKind::Websocket {
                        port: PortRef::Literal(port),
                        ..
                    }
                    | PortKind::HttpStream {
                        port: PortRef::Literal(port),
                        ..
                    } => *port,
                    _ => continue,
                };
                if find_location_for_prefix(extracted, prefix)
                    .and_then(|location| location.proxy_port)
                    == Some(expected_port)
                {
                    verified.insert(ObservedEvidenceSite::new(
                        "service",
                        id,
                        "interfaces",
                        &item.evidence,
                    ));
                }
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
        if let Observed::Known { value, evidence } = &facts.startup_tier {
            if evidence_from_file(evidence, START_BLUEOS_CORE)
                && extracted_service.startup_tier == *value
            {
                verified.insert(ObservedEvidenceSite::new(
                    "service",
                    id,
                    "startup_tier",
                    evidence,
                ));
            }
        }
        if let Observed::Known {
            value: limits,
            evidence,
        } = &facts.resource_limits
        {
            if evidence_from_file(evidence, START_BLUEOS_CORE)
                && limits.memory_mb == extracted_service.memory_mb
                && limits.cpu_percent == extracted_service.cpu_percent
            {
                verified.insert(ObservedEvidenceSite::new(
                    "service",
                    id,
                    "resource_limits",
                    evidence,
                ));
            }
        }
    }
}

fn fastapi_verified_sites(
    extracted: &[ExtractedFastApiRoute],
    journeys: &[UseCase],
    verified: &mut HashSet<ObservedEvidenceSite>,
) {
    if extracted.is_empty() {
        return;
    }
    for journey in journeys {
        let Some(steps) = journey_steps(journey) else {
            continue;
        };
        for step in steps {
            let Some(Grounded::Known {
                value: route,
                provenance: Provenance::Source(evidence),
            }) = &step.value.route
            else {
                continue;
            };
            if !evidence_from_fastapi_source(evidence) {
                continue;
            }
            let Some(extracted_route) = find_fastapi_route_for_observed(extracted, evidence, route)
            else {
                continue;
            };
            if extracted_fastapi_matches_observed(extracted_route, route) {
                verified.insert(ObservedEvidenceSite::new(
                    "journey",
                    journey.id.as_str(),
                    "step.route",
                    evidence,
                ));
            }
        }
    }
}

fn frontend_verified_sites(
    routes: &[ExtractedFrontendRoute],
    menus: &[ExtractedFrontendMenu],
    pages: &[Page],
    verified: &mut HashSet<ObservedEvidenceSite>,
) {
    if routes.is_empty() && menus.is_empty() {
        return;
    }
    for page in pages {
        if let Observed::Known { value, evidence } = &page.route {
            if evidence_from_router(evidence)
                && find_router_route_at_line(routes, evidence.line)
                    .is_some_and(|extracted| extracted.path == *value)
            {
                verified.insert(ObservedEvidenceSite::new(
                    "page",
                    page.id.as_str(),
                    "route",
                    evidence,
                ));
            }
        }
        if let Observed::Known { value, evidence } = &page.name {
            if evidence_from_router(evidence)
                && find_router_route_at_line(routes, evidence.line)
                    .is_some_and(|extracted| extracted.name == *value)
            {
                verified.insert(ObservedEvidenceSite::new(
                    "page",
                    page.id.as_str(),
                    "name",
                    evidence,
                ));
            }
        }
        if let Observed::Known { value, evidence } = &page.component {
            if evidence_from_router(evidence)
                && find_router_route_at_line(routes, evidence.line)
                    .is_some_and(|extracted| extracted.component == *value)
            {
                verified.insert(ObservedEvidenceSite::new(
                    "page",
                    page.id.as_str(),
                    "component",
                    evidence,
                ));
            }
        }
        if let Observed::Known { value, evidence } = &page.menu_title {
            if evidence_from_menus(evidence)
                && find_menu_entry_at_line(menus, evidence.line)
                    .is_some_and(|extracted| extracted.title == *value)
            {
                verified.insert(ObservedEvidenceSite::new(
                    "page",
                    page.id.as_str(),
                    "menu_title",
                    evidence,
                ));
            }
        }
        if let Observed::Known { value, evidence } = &page.advanced_only {
            if evidence_from_menus(evidence)
                && find_menu_entry_at_line(menus, evidence.line).is_some_and(|extracted| {
                    menu_matches_page_title(page, extracted)
                        && extracted.advanced_only == Some(*value)
                })
            {
                verified.insert(ObservedEvidenceSite::new(
                    "page",
                    page.id.as_str(),
                    "advanced_only",
                    evidence,
                ));
            }
        }
    }
}

fn journey_steps(
    journey: &UseCase,
) -> Option<&[catalog_kernel::provenance::GroundedItem<catalog_model::journey::JourneyStep>]> {
    match &journey.steps {
        GroundedSet::Known { items } => Some(items),
        GroundedSet::Unknown { .. } => None,
    }
}

fn evidence_from_file(evidence: &Evidence, file: &str) -> bool {
    evidence.file == file
}

fn nginx_proxy_pass_listen(evidence: &Evidence) -> bool {
    evidence_from_file(evidence, NGINX_CONF) && evidence.anchor.contains("proxy_pass")
}

fn interface_has_http_route(interface: &PortKind) -> bool {
    matches!(
        interface,
        PortKind::Rest { .. } | PortKind::Websocket { .. } | PortKind::HttpStream { .. }
    )
}

fn interface_route_prefix(interface: &PortKind) -> &'static str {
    match interface {
        PortKind::Rest { path_prefix, .. } => path_prefix.0,
        PortKind::Websocket { path, .. } => path.0,
        PortKind::HttpStream { path, .. } => path.0,
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
    use catalog_core::catalog::Catalog;
    use catalog_model::journey::HttpMethod;
    use catalog_model::observed::StartupTier;

    #[test]
    fn all_observed_evidence_count_is_pinned() {
        const ALL_OBSERVED_EVIDENCE: usize = 1398;

        let catalog = Catalog::bootstrap();
        let count = collect_all_observed_evidence_sites(&catalog).len();
        assert_eq!(count, ALL_OBSERVED_EVIDENCE);
    }

    #[test]
    fn extractor_verifiable_observed_evidence_count_is_pinned() {
        const EXTRACTOR_VERIFIABLE_OBSERVED_EVIDENCE: usize = 287;

        let catalog = Catalog::bootstrap();
        let count = collect_extractor_verifiable_sites(&catalog).len();
        assert_eq!(count, EXTRACTOR_VERIFIABLE_OBSERVED_EVIDENCE);
    }

    #[test]
    fn unverified_observed_evidence_count_is_pinned() {
        const UNVERIFIED_OBSERVED_EVIDENCE: usize = 1113;

        let repo_root = catalog_paths::repo_root();
        let catalog = Catalog::bootstrap();
        let count = count_unverified_observed_evidence(&repo_root, &catalog);
        assert_eq!(count, UNVERIFIED_OBSERVED_EVIDENCE);
    }

    #[test]
    fn extractor_coverage_lapses_count_is_pinned() {
        const EXTRACTOR_COVERAGE_LAPSES: usize = 2;

        let repo_root = catalog_paths::repo_root();
        let catalog = Catalog::bootstrap();
        let count = count_extractor_coverage_lapses(&repo_root, &catalog);
        assert_eq!(count, EXTRACTOR_COVERAGE_LAPSES);
    }

    #[test]
    fn empty_nginx_extract_does_not_mark_sites_verified() {
        let repo_root = catalog_paths::repo_root();
        let catalog = Catalog::bootstrap();
        let mut verified = HashSet::new();
        nginx_verified_sites(&[], catalog.services(), &mut verified);
        assert!(
            verified.is_empty(),
            "empty nginx extract must not mark sites verified"
        );
        let full_verified = collect_verified_observed_sites(&repo_root, &catalog);
        assert!(!full_verified.is_empty());
    }

    fn verified_sites_from_extracts(
        catalog: &Catalog,
        nginx: &[ExtractedNginxLocation],
        start_blueos: &[ExtractedService],
        fastapi: &[ExtractedFastApiRoute],
        routes: &[ExtractedFrontendRoute],
        menus: &[ExtractedFrontendMenu],
    ) -> HashSet<ObservedEvidenceSite> {
        let mut verified = HashSet::new();
        nginx_verified_sites(nginx, catalog.services(), &mut verified);
        start_blueos_verified_sites(start_blueos, catalog.services(), &mut verified);
        fastapi_verified_sites(fastapi, catalog.journeys(), &mut verified);
        frontend_verified_sites(routes, menus, catalog.pages(), &mut verified);
        verified
    }

    struct Extracts {
        nginx: Vec<ExtractedNginxLocation>,
        start_blueos: Vec<ExtractedService>,
        fastapi: Vec<ExtractedFastApiRoute>,
        routes: Vec<ExtractedFrontendRoute>,
        menus: Vec<ExtractedFrontendMenu>,
    }

    impl Extracts {
        fn load(repo_root: &Path) -> Self {
            let nginx = extract_nginx_from_repo(repo_root).expect("nginx");
            let start_blueos = extract_from_repo(repo_root).expect("start-blueos");
            let fastapi = extract_fastapi_from_repo(repo_root).expect("fastapi");
            let (routes, menus) = extract_frontend_router_from_repo(repo_root).expect("frontend");
            Self {
                nginx,
                start_blueos,
                fastapi,
                routes,
                menus,
            }
        }

        fn verified(&self, catalog: &Catalog) -> HashSet<ObservedEvidenceSite> {
            verified_sites_from_extracts(
                catalog,
                &self.nginx,
                &self.start_blueos,
                &self.fastapi,
                &self.routes,
                &self.menus,
            )
        }
    }

    fn assert_scope_field_uncredited(
        scope: &'static str,
        field: &'static str,
        verifiable: &HashSet<ObservedEvidenceSite>,
        verified: &HashSet<ObservedEvidenceSite>,
    ) -> usize {
        let sites: Vec<_> = verifiable
            .iter()
            .filter(|site| site.scope == scope && site.field == field)
            .cloned()
            .collect();
        assert!(
            !sites.is_empty(),
            "no extractor-verifiable {scope}.{field} sites"
        );
        for site in &sites {
            assert!(
                !verified.contains(site),
                "{scope}.{field} still credited after extracted value mutation: {site:?}"
            );
        }
        sites.len()
    }

    fn assert_page_field_uncredited(
        field: &'static str,
        verifiable: &HashSet<ObservedEvidenceSite>,
        verified: &HashSet<ObservedEvidenceSite>,
    ) -> usize {
        assert_scope_field_uncredited("page", field, verifiable, verified)
    }

    #[test]
    fn mutating_extracted_service_values_increases_unverified_and_lapses() {
        let repo_root = catalog_paths::repo_root();
        let catalog = Catalog::bootstrap();
        let all = collect_all_observed_evidence_sites(&catalog);
        let verifiable = collect_extractor_verifiable_sites(&catalog);
        let extracts = Extracts::load(&repo_root);
        let baseline = extracts.verified(&catalog);
        let unverified_full = all.difference(&baseline).count();
        let lapses_full = verifiable.difference(&baseline).count();
        let Extracts {
            nginx,
            start_blueos,
            fastapi,
            routes,
            menus,
        } = extracts;

        let mut nginx_mut = nginx.clone();
        let mut bag_mutated = false;
        for location in nginx_mut.iter_mut() {
            if let catalog_extract::extract_nginx::LocationMatch::Prefix(prefix) =
                &mut location.match_kind
            {
                if prefix == "/bag/" {
                    prefix.push_str("ZZ");
                    bag_mutated = true;
                }
            }
        }
        assert!(bag_mutated, "expected extracted /bag/ location");
        let verified_nginx = verified_sites_from_extracts(
            &catalog,
            &nginx_mut,
            &start_blueos,
            &fastapi,
            &routes,
            &menus,
        );
        let unverified_nginx = all.difference(&verified_nginx).count();
        let lapses_nginx = verifiable.difference(&verified_nginx).count();
        assert!(
            unverified_nginx > unverified_full,
            "nginx prefix drift must raise unverified, {unverified_nginx} vs {unverified_full}"
        );
        assert!(
            lapses_nginx > lapses_full,
            "nginx prefix miss must raise lapses, {lapses_nginx} vs {lapses_full}"
        );

        let mut start_mut = start_blueos.clone();
        let mut memory_mutated = false;
        for service in start_mut.iter_mut() {
            if service.tmux_name == "linux2rest" {
                service.memory_mb = Some(999);
                memory_mutated = true;
            }
        }
        assert!(memory_mutated, "expected extracted linux2rest");
        let verified_mem =
            verified_sites_from_extracts(&catalog, &nginx, &start_mut, &fastapi, &routes, &menus);
        let unverified_mem = all.difference(&verified_mem).count();
        let lapses_mem = verifiable.difference(&verified_mem).count();
        assert_eq!(unverified_mem, unverified_full + 1);
        assert_eq!(lapses_mem, lapses_full + 1);
    }

    #[test]
    fn frontend_extracted_value_mismatch_uncredits_each_field() {
        let repo_root = catalog_paths::repo_root();
        let catalog = Catalog::bootstrap();
        let verifiable = collect_extractor_verifiable_sites(&catalog);
        let extracts = Extracts::load(&repo_root);
        let baseline = extracts.verified(&catalog);
        let lapses_full = verifiable.difference(&baseline).count();
        let Extracts {
            nginx,
            start_blueos,
            fastapi,
            routes,
            menus,
        } = extracts;

        let mut routes_path = routes.clone();
        for route in routes_path.iter_mut() {
            route.path.push_str("ZZ");
        }
        let verified_path = verified_sites_from_extracts(
            &catalog,
            &nginx,
            &start_blueos,
            &fastapi,
            &routes_path,
            &menus,
        );
        let path_count = assert_page_field_uncredited("route", &verifiable, &verified_path);
        assert_eq!(
            verifiable.difference(&verified_path).count(),
            lapses_full + path_count
        );

        let mut routes_name = routes.clone();
        for route in routes_name.iter_mut() {
            route.name.push_str("ZZ");
        }
        let verified_name = verified_sites_from_extracts(
            &catalog,
            &nginx,
            &start_blueos,
            &fastapi,
            &routes_name,
            &menus,
        );
        let name_count = assert_page_field_uncredited("name", &verifiable, &verified_name);
        assert_eq!(
            verifiable.difference(&verified_name).count(),
            lapses_full + name_count
        );

        let mut routes_component = routes.clone();
        for route in routes_component.iter_mut() {
            route.component.push_str("ZZ");
        }
        let verified_component = verified_sites_from_extracts(
            &catalog,
            &nginx,
            &start_blueos,
            &fastapi,
            &routes_component,
            &menus,
        );
        let component_count =
            assert_page_field_uncredited("component", &verifiable, &verified_component);
        assert_eq!(
            verifiable.difference(&verified_component).count(),
            lapses_full + component_count
        );

        let mut menus_title = menus.clone();
        for menu in menus_title.iter_mut() {
            menu.title.push_str("ZZ");
        }
        let verified_title = verified_sites_from_extracts(
            &catalog,
            &nginx,
            &start_blueos,
            &fastapi,
            &routes,
            &menus_title,
        );
        assert_page_field_uncredited("menu_title", &verifiable, &verified_title);
        assert_page_field_uncredited("advanced_only", &verifiable, &verified_title);

        let mut menus_advanced = menus.clone();
        for menu in menus_advanced.iter_mut() {
            menu.advanced_only = Some(!menu.advanced_only.unwrap_or(false));
        }
        let verified_advanced = verified_sites_from_extracts(
            &catalog,
            &nginx,
            &start_blueos,
            &fastapi,
            &routes,
            &menus_advanced,
        );
        let advanced_count =
            assert_page_field_uncredited("advanced_only", &verifiable, &verified_advanced);
        assert_eq!(
            verifiable.difference(&verified_advanced).count(),
            lapses_full + advanced_count
        );
    }

    fn extracts_and_verifiable() -> (
        Catalog,
        Extracts,
        HashSet<ObservedEvidenceSite>,
        HashSet<ObservedEvidenceSite>,
        usize,
    ) {
        let repo_root = catalog_paths::repo_root();
        let catalog = Catalog::bootstrap();
        let verifiable = collect_extractor_verifiable_sites(&catalog);
        let extracts = Extracts::load(&repo_root);
        let baseline = extracts.verified(&catalog);
        let lapses_full = verifiable.difference(&baseline).count();
        (catalog, extracts, verifiable, baseline, lapses_full)
    }

    #[test]
    fn fastapi_extracted_value_mismatch_uncredits_step_route() {
        const STEP_ROUTE_SITES: usize = 78;
        let (catalog, extracts, verifiable, _, lapses_full) = extracts_and_verifiable();
        let Extracts {
            nginx,
            start_blueos,
            fastapi,
            routes,
            menus,
        } = extracts;

        let mut fastapi_method = fastapi.clone();
        for route in fastapi_method.iter_mut() {
            route.method = match route.method {
                HttpMethod::Get => HttpMethod::Post,
                _ => HttpMethod::Get,
            };
        }
        let verified_method = verified_sites_from_extracts(
            &catalog,
            &nginx,
            &start_blueos,
            &fastapi_method,
            &routes,
            &menus,
        );
        let method_count =
            assert_scope_field_uncredited("journey", "step.route", &verifiable, &verified_method);
        assert_eq!(method_count, STEP_ROUTE_SITES);
        assert_eq!(
            verifiable.difference(&verified_method).count(),
            lapses_full + method_count
        );

        let mut fastapi_path = fastapi.clone();
        for route in fastapi_path.iter_mut() {
            route.path.push_str("ZZ");
        }
        let verified_path = verified_sites_from_extracts(
            &catalog,
            &nginx,
            &start_blueos,
            &fastapi_path,
            &routes,
            &menus,
        );
        let path_count =
            assert_scope_field_uncredited("journey", "step.route", &verifiable, &verified_path);
        assert_eq!(path_count, STEP_ROUTE_SITES);
        assert_eq!(
            verifiable.difference(&verified_path).count(),
            lapses_full + path_count
        );

        let mut fastapi_version = fastapi.clone();
        for route in fastapi_version.iter_mut() {
            match &mut route.version {
                Some(version) => version.push_str("ZZ"),
                None => route.version = Some("ZZ".to_string()),
            }
        }
        let verified_version = verified_sites_from_extracts(
            &catalog,
            &nginx,
            &start_blueos,
            &fastapi_version,
            &routes,
            &menus,
        );
        let version_count =
            assert_scope_field_uncredited("journey", "step.route", &verifiable, &verified_version);
        assert_eq!(version_count, STEP_ROUTE_SITES);
        assert_eq!(
            verifiable.difference(&verified_version).count(),
            lapses_full + version_count
        );

        let mut fastapi_dir = fastapi.clone();
        for route in fastapi_dir.iter_mut() {
            route.service_dir.push_str("ZZ");
        }
        let verified_dir = verified_sites_from_extracts(
            &catalog,
            &nginx,
            &start_blueos,
            &fastapi_dir,
            &routes,
            &menus,
        );
        let dir_count =
            assert_scope_field_uncredited("journey", "step.route", &verifiable, &verified_dir);
        assert_eq!(dir_count, STEP_ROUTE_SITES);
        assert_eq!(
            verifiable.difference(&verified_dir).count(),
            lapses_full + dir_count
        );
    }

    #[test]
    fn startup_tier_extracted_mismatch_uncredits_sites() {
        const STARTUP_TIER_SITES: usize = 26;
        let (catalog, extracts, verifiable, _, lapses_full) = extracts_and_verifiable();
        let Extracts {
            nginx,
            start_blueos,
            fastapi,
            routes,
            menus,
        } = extracts;

        let mut start_mut = start_blueos.clone();
        for service in start_mut.iter_mut() {
            service.startup_tier = match service.startup_tier {
                StartupTier::Priority => StartupTier::Normal,
                StartupTier::Normal => StartupTier::Priority,
            };
        }
        let verified =
            verified_sites_from_extracts(&catalog, &nginx, &start_mut, &fastapi, &routes, &menus);
        let count =
            assert_scope_field_uncredited("service", "startup_tier", &verifiable, &verified);
        assert_eq!(count, STARTUP_TIER_SITES);
        assert_eq!(
            verifiable.difference(&verified).count(),
            lapses_full + count
        );
    }

    #[test]
    fn nginx_listen_proxy_port_mismatch_uncredits_sites() {
        const LISTEN_SITES: usize = 4;
        let (catalog, extracts, verifiable, _, _) = extracts_and_verifiable();
        let Extracts {
            nginx,
            start_blueos,
            fastapi,
            routes,
            menus,
        } = extracts;

        let mut nginx_mut = nginx.clone();
        for location in nginx_mut.iter_mut() {
            location.proxy_port = location.proxy_port.map(|_| 1);
        }
        let verified = verified_sites_from_extracts(
            &catalog,
            &nginx_mut,
            &start_blueos,
            &fastapi,
            &routes,
            &menus,
        );
        let count = assert_scope_field_uncredited("service", "listen", &verifiable, &verified);
        assert_eq!(count, LISTEN_SITES);
    }

    #[test]
    fn resource_limits_cpu_percent_mismatch_uncredits_sites() {
        let (catalog, extracts, verifiable, _, lapses_full) = extracts_and_verifiable();
        let Extracts {
            nginx,
            start_blueos,
            fastapi,
            routes,
            menus,
        } = extracts;

        let mut start_mut = start_blueos.clone();
        for service in start_mut.iter_mut() {
            service.cpu_percent = Some(service.cpu_percent.unwrap_or(0).wrapping_add(1));
        }
        let verified =
            verified_sites_from_extracts(&catalog, &nginx, &start_mut, &fastapi, &routes, &menus);
        let count =
            assert_scope_field_uncredited("service", "resource_limits", &verifiable, &verified);
        assert_eq!(
            verifiable.difference(&verified).count(),
            lapses_full + count
        );
    }

    #[test]
    fn nginx_interfaces_proxy_port_mismatch_uncredits_sites() {
        let (catalog, extracts, verifiable, baseline, _) = extracts_and_verifiable();
        let Extracts {
            nginx,
            start_blueos,
            fastapi,
            routes,
            menus,
        } = extracts;

        assert!(
            verifiable.iter().any(|site| {
                site.scope == "service" && site.field == "interfaces" && baseline.contains(site)
            }),
            "no credited service.interfaces sites in baseline"
        );

        let mut nginx_mut = nginx.clone();
        for location in nginx_mut.iter_mut() {
            location.proxy_port = location.proxy_port.map(|_| 1);
        }
        let verified = verified_sites_from_extracts(
            &catalog,
            &nginx_mut,
            &start_blueos,
            &fastapi,
            &routes,
            &menus,
        );
        assert_scope_field_uncredited("service", "interfaces", &verifiable, &verified);
    }
}
