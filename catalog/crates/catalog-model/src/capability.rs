use catalog_kernel::{
    aggregate::Aggregate,
    id::{capability::CapabilityId, page::PageId, service::ServiceId},
};

pub struct CapabilityDef {
    pub id: CapabilityId,
    pub aggregate: Aggregate,
    pub owner: ServiceId,
}

pub struct FrontendCapabilityDef {
    pub id: CapabilityId,
    pub aggregate: Aggregate,
    pub owner: PageId,
}
