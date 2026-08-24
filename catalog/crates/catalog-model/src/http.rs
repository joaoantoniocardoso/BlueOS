use catalog_kernel::provenance::ObservedSet;

use crate::journey::{HttpMethod, RouteRef};
use crate::observed::ObservedFacts;

pub fn http_method_label(method: &HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
        HttpMethod::Patch => "PATCH",
    }
}

pub fn resolve_http_path(observed: &ObservedFacts, route: &RouteRef) -> Option<String> {
    let path = route.path;
    if path.contains('{') {
        return None;
    }
    if path_starts_with_service_prefix(observed, path) {
        return Some(ensure_leading_slash(path));
    }

    let prefix = first_nginx_prefix(observed)?;

    let mut full = prefix.trim_end_matches('/').to_string();
    if let Some(version) = route.version {
        full.push('/');
        full.push_str(version);
    }
    if path.starts_with('/') {
        full.push_str(path);
    } else {
        full.push('/');
        full.push_str(path);
    }
    Some(full)
}

fn path_starts_with_service_prefix(observed: &ObservedFacts, path: &str) -> bool {
    match &observed.nginx_prefixes {
        ObservedSet::Known { items } => items.iter().any(|prefix| {
            let prefix = prefix.value.0;
            if prefix == "/" {
                return false;
            }
            path.starts_with(prefix) || path.starts_with(prefix.trim_end_matches('/'))
        }),
        ObservedSet::Unknown { .. } => false,
    }
}

fn first_nginx_prefix(observed: &ObservedFacts) -> Option<&'static str> {
    match &observed.nginx_prefixes {
        ObservedSet::Known { items } => items.first().map(|prefix| prefix.value.0),
        ObservedSet::Unknown { .. } => None,
    }
}

fn ensure_leading_slash(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}
