use catalog_core::frontend_routes::{
    check_frontend_route_refs, expected_resolved_path, FRONTEND_API_ENDPOINTS,
};
use catalog_core::http::resolve_http_path;
use catalog_kernel::provenance::{Grounded, GroundedSet};
use catalog_model::http::http_method_label;

use blueos_catalog::catalog::Catalog;

fn main() {
    let catalog = Catalog::bootstrap();

    println!(
        "FRONTEND_API_ENDPOINTS ({} entries):",
        FRONTEND_API_ENDPOINTS.len()
    );
    for endpoint in FRONTEND_API_ENDPOINTS {
        let expected = expected_resolved_path(endpoint);
        println!(
            "  {}:{} {} + {} → {}",
            endpoint.call_file,
            endpoint.call_line,
            endpoint.base.api_url,
            endpoint.relative_path,
            expected
        );
    }

    let errors = check_frontend_route_refs(&catalog);
    if errors.is_empty() {
        println!("\nAll frontend-backed journey routes match FRONTEND_API_ENDPOINTS.");
        return;
    }

    println!("\nFrontend route mismatches ({}):", errors.len());
    for error in &errors {
        println!("  {error}");
    }

    for journey in catalog.journeys() {
        let GroundedSet::Known { items: steps } = &journey.steps else {
            continue;
        };
        for (step_index, step) in steps.iter().enumerate() {
            let Some(Grounded::Known { value: route, .. }) = &step.value.route else {
                continue;
            };
            let Some(resolved) = resolve_http_path(&catalog, route) else {
                continue;
            };
            println!(
                "  {} step {step_index}: {} {} → {resolved}",
                journey.id,
                http_method_label(&route.method),
                route.path
            );
        }
    }
}
