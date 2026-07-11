use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const DEV_CORE: &str = "content/development/core/index.md";
const GETTING: &str = "content/usage/getting-started/index.md";
const NGINX_CONF: &str = "core/tools/nginx/nginx.conf";
const API_TS: &str = "core/frontend/src/utils/api.ts";
const RUNTIME_CAPTURE: &str = "runtime-captures/nginx__pi4_navigator_master.json";
const RUNTIME_ENV: &str = "BlueOS master (bluerobotics/blueos-core:master @ sha256:cdccc74464076e7fa8b5dc8a85c83db0ec95c27cb77130cb1e180d481320674e; RepoDigest sha256:0406983a568a66df2a56f682b52161858f30ac87ab273f15309e52f5ab87e22a), Raspberry Pi 4, Navigator";

pub fn journeys() -> Vec<UserJourney> {
    vec![access_blueos_web_interface()]
}

fn access_blueos_web_interface() -> UserJourney {
    UserJourney {
        id: JourneyId::AccessBlueosWebInterface,
        summary: Grounded::known(
            "Open the BlueOS web interface in a browser to access and configure vehicle services"
                .into(),
            Provenance::doc(GETTING, 22),
        ),
        visibility: Grounded::known(Visibility::Default, Provenance::doc(GETTING, 26)),
        services: nginx_services(),
        capability_refs: GroundedSet::known(vec![cap(
            CapabilityId::AccessBlueosWebInterface,
            "nginx listens on port 80 and serves the frontend SPA at / for browser access",
        )]),
        preconditions: GroundedSet::known(vec![]),
        steps: GroundedSet::known(vec![
            operator_step(
                "Open the BlueOS web interface in a browser (e.g. http://blueos.local)",
                None,
                Provenance::doc(GETTING, 29),
                None,
            ),
            service_step(
                "Serve the frontend SPA from /",
                Some(sourced_route(HttpMethod::Get, "/", None, 268)),
                Provenance::source(NGINX_CONF, 269),
                Some(runtime_outcome(
                    200,
                    Some("<!DOCTYPE html>".into()),
                    "#running_baseline",
                )),
            ),
            service_step(
                "Verify the backend is online via GET /status (expects HTTP 204)",
                Some(sourced_route(HttpMethod::Get, "/status", None, 62)),
                Provenance::source(API_TS, 23),
                Some(runtime_outcome(204, None, "#running_baseline")),
            ),
        ]),
        chains_from: None,
    }
}

fn cap(id: CapabilityId, rationale: &str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

fn nginx_services() -> GroundedSet<ServiceId> {
    GroundedSet::known(vec![GroundedItem::new(
        ServiceId::Nginx,
        Provenance::doc(DEV_CORE, 71),
    )])
}

fn route(method: HttpMethod, path: &str, version: Option<&str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Nginx,
        method,
        path: path.into(),
        version: version.map(str::to_string),
    }
}

fn sourced_route(
    method: HttpMethod,
    path: &str,
    version: Option<&str>,
    line: u32,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(NGINX_CONF, line),
    )
}

fn runtime_outcome(status: u16, body: Option<String>, key: &str) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            transition: None,
        },
        Provenance::runtime(format!("{RUNTIME_CAPTURE}{key}"), RUNTIME_ENV),
    )
}

fn operator_step(
    description: &str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description: description.into(),
            route,
            outcome,
        },
        provenance,
    )
}

fn service_step(
    description: &str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Service(ServiceId::Nginx),
            description: description.into(),
            route,
            outcome,
        },
        provenance,
    )
}
