use schemars::JsonSchema;
use serde::Serialize;

use catalog_kernel::{
    id::{capability::CapabilityId, page::PageId, service::ServiceId},
    provenance::{AssertedSet, Observed, ObservedSet},
};

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Page {
    pub id: PageId,
    pub route: Observed<&'static str>,
    pub name: Observed<&'static str>,
    pub component: Observed<&'static str>,
    pub menu_title: Observed<&'static str>,
    pub advanced_only: Observed<bool>,
    pub stores: ObservedSet<&'static str>,
    pub consumes: ObservedSet<PageServiceCall>,
    pub frontend_features: AssertedSet<CapabilityId>,
    pub client_state: AssertedSet<ClientState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct PageServiceCall {
    pub service: ConsumeTarget,
    pub endpoint: &'static str,
    pub purpose: &'static str,
}

/// What a page consumes: a cataloged service, or the public internet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConsumeTarget {
    Service(ServiceId),
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ClientState {
    pub name: &'static str,
    pub store: &'static str,
    pub ownership: StateOwnership,
    pub notes: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StateOwnership {
    BackendOwned,
    FrontendOwned,
    Shared,
}
