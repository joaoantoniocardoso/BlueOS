//! Pure string fragments for one endpoint in `Conversions` source.

use convert_case::{Case, Casing};

use crate::endpoints::{Endpoint, Kind};

pub(super) struct ApiEndpointChunk {
    pub(super) methods: String,
    pub(super) note: String,
    pub(super) package: Option<String>,
}

struct JobGoalHeaderParts {
    error: String,
    job_id: &'static str,
    goal: String,
}

fn chunk_methods(methods: String) -> String {
    format!("{methods}\n\n")
}

pub(super) fn api_endpoint_chunk(
    endpoint: &Endpoint,
    key: &str,
    title: &str,
) -> Option<ApiEndpointChunk> {
    let interface = &endpoint.interface;
    let function = endpoint.function.as_str();
    let builder = conversion_chunk_builder(endpoint.kind)?;
    Some(builder(endpoint, function, key, title, interface))
}

type ConversionChunkBuilder =
    fn(&Endpoint, &str, &str, &str, &crate::endpoints::InterfaceType) -> ApiEndpointChunk;

fn conversion_chunk_builder(kind: Kind) -> Option<ConversionChunkBuilder> {
    if kind == Kind::Job {
        Some(job_chunk)
    } else if kind == Kind::Query {
        Some(query_chunk)
    } else if kind == Kind::State {
        Some(state_chunk)
    } else if kind == Kind::Event {
        Some(event_chunk)
    } else {
        None
    }
}

fn job_chunk(
    endpoint: &Endpoint,
    function: &str,
    key: &str,
    title: &str,
    interface: &crate::endpoints::InterfaceType,
) -> ApiEndpointChunk {
    let (goal_header, goal_mapping) =
        job_goal_header_pair(endpoint, function, key, title, interface);
    let tail = job_feedback_and_result(function, title, interface);
    let note =
        format!("{goal_mapping}`{function}_feedback` and `{function}_result` for the {title}");
    ApiEndpointChunk {
        methods: chunk_methods(format!("{goal_header}{tail}")),
        note,
        package: Some(interface.package.clone()),
    }
}

fn empty_goal_header_if_custom(custom: bool) -> Option<(String, String)> {
    custom.then(|| (String::new(), String::new()))
}

fn job_goal_header_pair(
    endpoint: &Endpoint,
    function: &str,
    key: &str,
    title: &str,
    interface: &crate::endpoints::InterfaceType,
) -> (String, String) {
    match empty_goal_header_if_custom(endpoint.custom) {
        Some(pair) => pair,
        None => mapped_job_goal_header(endpoint, function, key, title, interface),
    }
}

fn mapped_job_goal_header(
    endpoint: &Endpoint,
    function: &str,
    key: &str,
    title: &str,
    interface: &crate::endpoints::InterfaceType,
) -> (String, String) {
    let parts = job_goal_header_parts(endpoint, interface);
    let header = job_goal_header_text(function, key, title, &parts);
    let goal_mapping = job_goal_mapping_note(function, &parts.error);
    (header, goal_mapping)
}

fn job_goal_mapping_note(function: &str, error: &str) -> String {
    format!("`{error}`, `{function}`, ")
}

fn job_goal_header_parts(
    endpoint: &Endpoint,
    interface: &crate::endpoints::InterfaceType,
) -> JobGoalHeaderParts {
    JobGoalHeaderParts {
        error: format!("{}Error", endpoint.name.to_case(Case::UpperCamel)),
        job_id: endpoint.job_id_parameter(),
        goal: interface.path("Goal"),
    }
}

fn job_goal_header_text(
    function: &str,
    key: &str,
    title: &str,
    parts: &JobGoalHeaderParts,
) -> String {
    format!(
        "/// Why a Goal of the {title} is rejected.\ntype {error}: core::error::Error + Send + Sync + \
         'static;\n\n/// The {title}, at `{key}`.\n///\n/// The Domain's Request for the Goal, or why it is \
         rejected.\nfn {function}({job_id}goal: {goal}) -> Result<Self::Request, Self::{error}>;\n\n",
        error = parts.error,
        job_id = parts.job_id,
        goal = parts.goal,
    )
}

fn job_feedback_and_result(
    function: &str,
    title: &str,
    interface: &crate::endpoints::InterfaceType,
) -> String {
    format!(
        "/// The Feedback of a Job of the {title}.\n///\n/// That of the Job `job_id` in `snapshot`, or \
         `None` while it has none.\nfn {function}_feedback(snapshot: &Self::Snapshot, job_id: JobId) -> \
         Option<{feedback}>;\n\n/// The Job result of a Job of the {title}.\n///\n/// That of the Job \
         `job_id`, in the `snapshot` of the step that ended it.\nfn {function}_result(snapshot: \
         &Self::Snapshot, job_id: JobId) -> {result};\n",
        feedback = interface.path("Feedback"),
        result = interface.path("Result"),
    )
}

fn query_chunk(
    endpoint: &Endpoint,
    function: &str,
    key: &str,
    title: &str,
    interface: &crate::endpoints::InterfaceType,
) -> ApiEndpointChunk {
    let _ = endpoint;
    let methods = format!(
        "/// The {title}, at `{key}`.\n///\n/// The Domain's Query for the request.\nfn {function}(request: \
         {request}) -> Self::Query;\n\n/// The reply of the {title}.\n///\n/// The response for `response`, or \
         `None` when `response` answers another Query.\nfn {function}_response(response: Self::Response) -> \
         Option<{response}>;\n",
        request = interface.path("Request"),
        response = interface.path("Response"),
    );
    let note = format!("`{function}` and `{function}_response` for the {title}");
    ApiEndpointChunk {
        methods: chunk_methods(methods),
        note,
        package: Some(interface.package.clone()),
    }
}

fn state_chunk(
    endpoint: &Endpoint,
    function: &str,
    key: &str,
    title: &str,
    interface: &crate::endpoints::InterfaceType,
) -> ApiEndpointChunk {
    let _ = endpoint;
    let methods = format!(
        "/// The {title}, at `{key}`.\n///\n/// Its value for `snapshot`.\nfn {function}(snapshot: \
         &Self::Snapshot) -> {message};\n",
        message = interface.path(""),
    );
    ApiEndpointChunk {
        methods: chunk_methods(methods),
        note: format!("`{function}` for the {title}"),
        package: Some(interface.package.clone()),
    }
}

fn event_chunk(
    endpoint: &Endpoint,
    function: &str,
    key: &str,
    title: &str,
    interface: &crate::endpoints::InterfaceType,
) -> ApiEndpointChunk {
    let _ = endpoint;
    let methods = format!(
        "/// The {title}, at `{key}`.\n///\n/// What it publishes for `event`, or `None` when `event` is \
         another one.\nfn {function}(event: &Self::Event) -> Option<{message}>;\n",
        message = interface.path(""),
    );
    ApiEndpointChunk {
        methods: chunk_methods(methods),
        note: format!("`{function}` for the {title}"),
        package: Some(interface.package.clone()),
    }
}

#[cfg(test)]
mod tests {
    use crate::endpoints::{Endpoint, InterfaceType, Kind};

    use super::*;

    fn example_interface(form: &'static str, name: &str) -> InterfaceType {
        InterfaceType {
            package: "blueos_example_msgs".into(),
            form,
            name: name.into(),
        }
    }

    fn example_endpoint(kind: Kind, form: &'static str, name: &str, function: &str) -> Endpoint {
        Endpoint {
            kind,
            name: "Set".into(),
            function: function.into(),
            interface: example_interface(form, name),
            custom: false,
            nature: None,
        }
    }

    #[test]
    fn api_endpoint_chunk_covers_every_conversion_kind() {
        let key = "blueos/v1/example/command/Set";
        let title = "Job `Set`";
        let job = example_endpoint(Kind::Job, "action", "SetLevel", "set_level");
        let job_chunk = api_endpoint_chunk(&job, key, title).expect("job chunk");
        assert!(job_chunk.methods.contains("fn set_level("));

        let mut custom_job = job;
        custom_job.custom = true;
        let custom_job_chunk =
            api_endpoint_chunk(&custom_job, key, title).expect("custom job chunk");
        assert!(!custom_job_chunk.methods.contains("fn set_level("));

        let query = example_endpoint(Kind::Query, "srv", "Level", "level");
        let query_chunk = api_endpoint_chunk(&query, key, title).expect("query chunk");
        assert!(query_chunk.methods.contains("fn level("));

        let state = example_endpoint(Kind::State, "msg", "PumpState", "pump");
        let state_chunk = api_endpoint_chunk(&state, key, title).expect("state chunk");
        assert!(state_chunk.methods.contains("fn pump("));

        let event = example_endpoint(Kind::Event, "msg", "PumpState", "pump");
        let event_chunk = api_endpoint_chunk(&event, key, title).expect("event chunk");
        assert!(event_chunk.methods.contains("fn pump("));

        let io_query = example_endpoint(Kind::IoQuery, "srv", "Level", "probe");
        assert!(api_endpoint_chunk(&io_query, key, title).is_none());
    }
}
