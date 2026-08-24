use std::collections::HashSet;

use catalog_core::catalog::Catalog;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::AssertedSet;
use catalog_kernel::provenance::GroundedSet;
use catalog_kernel::provenance::ObservedSet;
use catalog_model::journey::{Actor, OracleClass, UseCase};
use catalog_model::page::{ConsumeTarget, Page, StateOwnership};

pub fn derive_oracle_class(catalog: &Catalog, journey: &UseCase) -> OracleClass {
    if journey_has_frontend_actor(journey) {
        return OracleClass::ClientOrchestrated;
    }
    let pages = pages_for_journey(catalog, journey);
    if pages.iter().any(|page| page_has_frontend_features(page)) {
        return OracleClass::ClientOrchestrated;
    }
    let total_consumes = pages
        .iter()
        .map(|page| page_consume_count(page))
        .sum::<usize>();
    if pages
        .iter()
        .any(|page| page_has_shared_or_frontend_owned_state(page))
        || total_consumes > 1
    {
        return OracleClass::ClientComposed;
    }
    OracleClass::HttpPassthrough
}

pub fn journey_has_frontend_actor(journey: &UseCase) -> bool {
    match &journey.steps {
        GroundedSet::Known { items } => items
            .iter()
            .any(|step| matches!(step.value.actor, Actor::Frontend(_))),
        GroundedSet::Unknown { .. } => false,
    }
}

fn pages_for_journey<'a>(catalog: &'a Catalog, journey: &UseCase) -> Vec<&'a Page> {
    let service_ids: HashSet<ServiceId> = match &journey.services {
        GroundedSet::Known { items } => items.iter().map(|item| item.value).collect(),
        GroundedSet::Unknown { .. } => return Vec::new(),
    };
    if service_ids.is_empty() {
        return Vec::new();
    }
    catalog
        .pages()
        .iter()
        .filter(|page| page_consumes_any_service(page, &service_ids))
        .collect()
}

fn page_consumes_any_service(page: &Page, service_ids: &HashSet<ServiceId>) -> bool {
    match &page.consumes {
        ObservedSet::Known { items } => items.iter().any(|item| {
            matches!(
                item.value.service,
                ConsumeTarget::Service(id) if service_ids.contains(&id)
            )
        }),
        ObservedSet::Unknown { .. } => false,
    }
}

fn page_consume_count(page: &Page) -> usize {
    match &page.consumes {
        ObservedSet::Known { items } => items.len(),
        ObservedSet::Unknown { .. } => 0,
    }
}

fn page_has_frontend_features(page: &Page) -> bool {
    matches!(
        &page.frontend_features,
        AssertedSet::Established { items } if !items.is_empty()
    )
}

fn page_has_shared_or_frontend_owned_state(page: &Page) -> bool {
    match &page.client_state {
        AssertedSet::Established { items } => items.iter().any(|item| {
            matches!(
                item.value.ownership,
                StateOwnership::Shared | StateOwnership::FrontendOwned
            )
        }),
        AssertedSet::Unknown { .. } => false,
    }
}
