//! Pure string fragments for app endpoint registration source.

use crate::endpoints::{Endpoint, Kind};

pub(super) fn handler_method_block(endpoint: &Endpoint, key: &str, title: &str) -> String {
    let function = endpoint.function.as_str();
    let interface = &endpoint.interface;
    let (description, parameter, output) = match endpoint.kind {
        Kind::Job => (
            "The Domain's Request for the Goal",
            format!(
                "{}goal: {}",
                endpoint.job_id_parameter(),
                interface.path("Goal")
            ),
            "Result<D::Request, Refusal>".to_owned(),
        ),
        Kind::Query | Kind::IoQuery | Kind::State | Kind::Event => (
            "Its response, read outside the Inbox",
            format!("request: {}", interface.path("Request")),
            format!(
                "impl Future<Output = Result<{}, Refusal>> + Send",
                interface.path("Response")
            ),
        ),
    };
    format!(
        "/// The {title}, at `{key}`.\n///\n/// {description}, or why it is refused.\nfn {function}(&self, \
         {parameter}) -> {output};\n"
    )
}

pub(super) fn registration_line(endpoint: &Endpoint) -> String {
    let name = endpoint.name.as_str();
    let function = endpoint.function.as_str();
    let interface = &endpoint.interface;
    match endpoint.kind {
        Kind::Job => job_registration(name, function, endpoint, interface),
        Kind::Query => format!(
            ".query(\"{name}\", |request: {request}| Ok(<D as Conversions>::{function}(request)), <D as \
             Conversions>::{function}_response)",
            request = interface.path("Request"),
        ),
        Kind::IoQuery => format!(
            ".io_query(\"{name}\", {{\nlet handlers = Arc::clone(&handlers);\nmove |request: {request}| \
             {{\nlet handlers = Arc::clone(&handlers);\nBox::pin(async move {{ H::{function}(&handlers, \
             request).await }})\n}}\n}})",
            request = interface.path("Request"),
        ),
        Kind::State => format!(".state(\"{name}\", <D as Conversions>::{function})"),
        Kind::Event => format!(".event(\"{name}\", <D as Conversions>::{function})"),
    }
}

fn job_registration(
    name: &str,
    function: &str,
    endpoint: &Endpoint,
    interface: &crate::endpoints::InterfaceType,
) -> String {
    let goal = interface.path("Goal");
    let job_id = if endpoint.nature.is_some() {
        "job_id, "
    } else {
        ""
    };
    let into_request = if endpoint.custom {
        format!(
            "{{\nlet handlers = Arc::clone(&handlers);\nmove |{job_id}goal: {goal}| H::{function}(&handlers, \
             {job_id}goal)\n}}"
        )
    } else {
        format!(
            "|{job_id}goal: {goal}| <D as Conversions>::{function}({job_id}goal).map_err(Refusal::from)"
        )
    };
    let job_type = match &endpoint.nature {
        Some(nature) => format!(
            ".job(\"{name}\", JobNature {{ lasting: {}, cancellable: {}, pausable: {}, needs_permission: {} }}, \
             {into_request})",
            nature.lasting, nature.cancellable, nature.pausable, nature.needs_permission
        ),
        None => format!(".command(\"{name}\", {into_request})"),
    };
    format!(
        "{job_type}\n.job_feedback(\"{name}\", <D as Conversions>::{function}_feedback)\n.job_result(\"{name}\", \
         <D as Conversions>::{function}_result)"
    )
}

pub(super) fn message_import_packages(
    endpoints: &[Endpoint],
) -> alloc::collections::BTreeSet<String> {
    endpoints
        .iter()
        .filter(|endpoint| matches!(endpoint.kind, Kind::Job | Kind::Query | Kind::IoQuery))
        .map(|endpoint| endpoint.interface.package.clone())
        .collect()
}
