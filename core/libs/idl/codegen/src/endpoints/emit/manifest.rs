//! `manifest_endpoints` vec source.

use super::super::{Endpoint, Kind};

pub(in super::super) fn manifest_endpoints_vec_source(
    endpoints: &[Endpoint],
    service: &str,
) -> String {
    if endpoints.is_empty() {
        return "vec![]".to_owned();
    }
    let rows = endpoints
        .iter()
        .map(|endpoint| endpoint_info_row(endpoint, service))
        .collect::<Vec<_>>()
        .join("");
    format!("vec![\n{rows}    ]")
}

fn endpoint_info_row(endpoint: &Endpoint, service: &str) -> String {
    format!(
        "        blueos_idl::msg::blueos_msgs::EndpointInfo {{\n            kind: \"{kind}\".into(),\n            \
         name: \"{name}\".into(),\n            key: \"{key}\".into(),\n            interface_type: \
         \"{interface_type}\".into(),\n            schema: \
         blueos_idl::schema(\"{interface_type}\").unwrap_or_default().into(),\n        }},\n",
        kind = endpoint.kind_name(),
        name = endpoint.name,
        key = endpoint.key(service),
        interface_type = endpoint.interface.schema_name(),
    )
}

pub(in super::super) fn domain_trait(endpoints: &[Endpoint]) -> &'static str {
    if endpoints
        .iter()
        .any(|endpoint| endpoint.kind == Kind::Query)
    {
        "DomainQueries"
    } else {
        "Domain"
    }
}
