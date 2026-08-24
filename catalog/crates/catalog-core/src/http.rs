use catalog_kernel::id::service::ServiceId;
use catalog_model::journey::RouteRef;
use catalog_model::observed::ObservedFacts;

use crate::catalog::Catalog;

pub fn resolve_http_path(catalog: &Catalog, route: &RouteRef) -> Option<String> {
    let observed = catalog.observed_by_id(&route.service)?;
    catalog_model::http::resolve_http_path(observed, route)
}

pub fn resolve_http_path_for_service(observed: &ObservedFacts, route: &RouteRef) -> Option<String> {
    catalog_model::http::resolve_http_path(observed, route)
}

pub fn path_starts_with_service_prefix(catalog: &Catalog, service: ServiceId, path: &str) -> bool {
    let Some(observed) = catalog.observed_by_id(&service) else {
        return false;
    };
    match &observed.nginx_prefixes {
        catalog_kernel::provenance::ObservedSet::Known { items } => items.iter().any(|prefix| {
            let prefix = prefix.value.0;
            if prefix == "/" {
                return false;
            }
            path.starts_with(prefix) || path.starts_with(prefix.trim_end_matches('/'))
        }),
        catalog_kernel::provenance::ObservedSet::Unknown { .. } => false,
    }
}
