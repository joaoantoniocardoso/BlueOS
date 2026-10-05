//! `Conversions` trait source for a Service.

mod endpoint;

use super::super::{Endpoint, GENERATED_NOTICE, Kind};
use super::manifest::domain_trait;
use alloc::collections::BTreeSet;

pub(in super::super) fn api_source(service: &str, endpoints: &[Endpoint]) -> String {
    let header = format!(
        "//! The conversions between the Messages and the Domain of the `{service}` Service, one function per \
         endpoint.\n//!\n{GENERATED_NOTICE}"
    );
    let assembled = assemble_conversion_methods(endpoints, service);
    if assembled.methods.is_empty() {
        return header;
    }
    format!(
        "{header}{}",
        conversions_trait_body(service, endpoints, &assembled)
    )
}

fn conversions_trait_body(
    service: &str,
    endpoints: &[Endpoint],
    assembled: &AssembledConversionMethods,
) -> String {
    let domain = domain_trait(endpoints);
    let list = assembled.notes.join(", ");
    let jobs = if endpoints.iter().any(|endpoint| endpoint.kind == Kind::Job) {
        "use blueos_jobs::JobId;\n"
    } else {
        ""
    };
    format!(
        "\nuse blueos_domain::{domain};\nuse blueos_idl::msg::{{{packages}}};\n{jobs}\n/// Converts between the Domain and \
         the Messages of every endpoint of the `{service}` Service but its IO\n/// queries. Implement it for the \
         Domain in this crate: `register` in the app crate calls it.\n#[diagnostic::on_unimplemented(\nmessage = \
         \"`{{Self}}` does not convert the endpoints of the `{service}` Service\",\nlabel = \"no `impl \
         Conversions for {{Self}}` in the `{service}` logic/api crate\",\nnote = \"implement {list}\"\n)]\npub \
         trait Conversions: {domain} {{\n{methods}}}\n",
        packages = assembled
            .packages
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(", "),
        methods = assembled.methods,
    )
}

struct AssembledConversionMethods {
    methods: String,
    notes: Vec<String>,
    packages: BTreeSet<String>,
}

fn assemble_conversion_methods(
    endpoints: &[Endpoint],
    service: &str,
) -> AssembledConversionMethods {
    let chunks = conversion_chunks(endpoints, service);
    AssembledConversionMethods {
        methods: chunks
            .iter()
            .map(|chunk| chunk.methods.as_str())
            .collect::<String>(),
        notes: chunks.iter().map(|chunk| chunk.note.clone()).collect(),
        packages: chunks
            .iter()
            .filter_map(|chunk| chunk.package.clone())
            .collect(),
    }
}

fn conversion_chunks(endpoints: &[Endpoint], service: &str) -> Vec<endpoint::ApiEndpointChunk> {
    endpoints
        .iter()
        .filter_map(|endpoint| {
            let key = endpoint.key(service);
            let title = endpoint.title();
            endpoint::api_endpoint_chunk(endpoint, &key, &title)
        })
        .collect()
}
