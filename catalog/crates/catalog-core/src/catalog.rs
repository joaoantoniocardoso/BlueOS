use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Serialize;

use catalog_data::{all_journeys, all_pages, all_services};
use catalog_kernel::id::journey::JourneyId;
use catalog_kernel::id::page::PageId;
use catalog_kernel::id::service::ServiceId;
use catalog_model::journey::UseCase;
use catalog_model::observed::ObservedFacts;
use catalog_model::page::Page;
use catalog_model::runtime::RuntimeFacts;
use catalog_model::service::{Service, ServiceJudgment};

use crate::validate::ValidationError;

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Catalog {
    services: Vec<Service>,
    journeys: Vec<UseCase>,
    pages: Vec<Page>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct CouplingMatrix {
    pub service_ids: Vec<ServiceId>,
    pub weights: Vec<Vec<f64>>,
}

impl Catalog {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
            journeys: Vec::new(),
            pages: Vec::new(),
        }
    }

    pub fn with_parts(services: Vec<Service>, journeys: Vec<UseCase>, pages: Vec<Page>) -> Self {
        Self {
            services,
            journeys,
            pages,
        }
    }

    pub fn bootstrap() -> Self {
        Self::with_parts(all_services(), all_journeys(), all_pages())
    }

    pub fn services(&self) -> &[Service] {
        &self.services
    }

    pub fn service_by_id(&self, id: &ServiceId) -> Option<&Service> {
        self.services.iter().find(|service| &service.id == id)
    }

    pub fn definition_by_id(&self, id: &ServiceId) -> Option<&ServiceJudgment> {
        self.service_by_id(id).map(|service| &service.definition)
    }

    pub fn observed_by_id(&self, id: &ServiceId) -> Option<&ObservedFacts> {
        self.service_by_id(id).map(|service| &service.observed)
    }

    pub fn runtime_by_id(&self, id: &ServiceId) -> Option<&RuntimeFacts> {
        self.service_by_id(id).map(|service| &service.runtime)
    }

    pub fn journeys(&self) -> &[UseCase] {
        &self.journeys
    }

    pub fn journey_by_id(&self, id: &JourneyId) -> Option<&UseCase> {
        self.journeys.iter().find(|journey| &journey.id == id)
    }

    pub fn pages(&self) -> &[Page] {
        &self.pages
    }

    pub fn page_by_id(&self, id: &PageId) -> Option<&Page> {
        self.pages.iter().find(|page| &page.id == id)
    }

    pub fn service_index(&self) -> HashMap<&ServiceId, &Service> {
        self.services
            .iter()
            .map(|service| (&service.id, service))
            .collect()
    }

    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        crate::validate::validate(self)
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}
