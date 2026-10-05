//! App `Handlers` and `register` source for a Service.

mod import_lines;
mod import_plan;
mod imports;
mod parts;

use core::fmt::Write as _;

use super::super::{Endpoint, GENERATED_NOTICE, Kind};
use super::manifest::{domain_trait, manifest_endpoints_vec_source};

pub(in super::super) fn app_source(
    service: &str,
    api_crate: &str,
    endpoints: &[Endpoint],
) -> String {
    let conversions = endpoints
        .iter()
        .any(|endpoint| endpoint.kind != Kind::IoQuery);
    let handled: Vec<&Endpoint> = endpoints
        .iter()
        .filter(|endpoint| endpoint.is_handled())
        .collect();
    let methods = handler_methods(service, &handled);
    let registrations = registration_block(endpoints);
    let packages = parts::message_import_packages(endpoints);
    let domain = domain_trait(endpoints);
    let import_block = imports::import_block(imports::ImportBlockInput {
        api_crate,
        domain,
        endpoints,
        handled: &handled,
        conversions,
        packages: &packages,
    });
    let manifest_endpoints = manifest_endpoints_vec_source(endpoints, service);
    let bound = register_type_bound(conversions, endpoints, domain);
    let manifest_tail = format!("\n.manifest_endpoints({manifest_endpoints})");
    let mut source = format!(
        "//! The endpoints of the `{service}` Service, registered as `endpoints.toml` lists them.\n//!\n\
         {GENERATED_NOTICE}\n{import_block}\n/// The name of the `{service}` Service, in each of its keys: \
         `blueos/v1/{service}/...`.\npub const NAME: &str = \"{service}\";\n"
    );
    if handled.is_empty() {
        _ = write!(
            source,
            "\n/// Registers every endpoint of `endpoints.toml` on `builder`.\npub fn register<D: {bound}, Context>(builder: \
             ServiceBuilder<D, Context>) -> ServiceBuilder<D, Context> {{\nbuilder\n{registrations}{manifest_tail}\n}}\n"
        );
        return source;
    }
    let list = handled
        .iter()
        .map(|endpoint| format!("`{}` for the {}", endpoint.function, endpoint.title()))
        .collect::<Vec<_>>()
        .join(", ");
    _ = write!(
        source,
        "\n/// Handles the endpoints of the `{service}` Service whose mapping is not a plain conversion: the \
         `custom`\n/// Job types and the IO queries. Implement it in `handlers.rs` and pass it to \
         [`register`].\n#[diagnostic::on_unimplemented(\nmessage = \"`{{Self}}` does not handle the custom \
         endpoints of the `{service}` Service\",\nlabel = \"no `impl Handlers<D> for {{Self}}` in the \
         `{service}` app crate\",\nnote = \"implement {list}\"\n)]\npub trait Handlers<D: {domain}>: Send + Sync + \
         'static {{\n{methods}}}\n\n/// Registers every endpoint of `endpoints.toml` on `builder`, with `handlers` \
         for the custom ones.\npub fn register<D: {bound}, H: Handlers<D>, Context>(builder: ServiceBuilder<D, \
         Context>, handlers: H) -> ServiceBuilder<D, Context> {{\nlet handlers = Arc::new(handlers);\nbuilder\n{registrations}{manifest_tail}\n}}\n"
    );
    source
}

fn handler_methods(service: &str, handled: &[&Endpoint]) -> String {
    handled
        .iter()
        .map(|endpoint| {
            let key = endpoint.key(service);
            let title = endpoint.title();
            parts::handler_method_block(endpoint, &key, &title)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn registration_block(endpoints: &[Endpoint]) -> String {
    endpoints
        .iter()
        .map(parts::registration_line)
        .map(|line| format!("{line}\n"))
        .collect()
}

fn register_type_bound(conversions: bool, endpoints: &[Endpoint], domain: &str) -> String {
    let jobs = endpoints.iter().any(|endpoint| endpoint.nature.is_some());
    match (conversions, jobs) {
        (true, true) => "Conversions + DomainJobs".to_owned(),
        (true, false) => "Conversions".to_owned(),
        (false, true) => format!("{domain} + DomainJobs"),
        (false, false) => domain.to_owned(),
    }
}
