use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_REPODIGEST;
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, RouteRef, StepOutcome, UserJourney, Visibility,
};
use crate::journey_presence::PRESENCE_ACCESS_BLUEOS_WEB_INTERFACE;
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const DEV_CORE: &str = "content/development/core/index.md";
const GETTING: &str = "content/usage/getting-started/index.md";
const NGINX_CONF: &str = "core/tools/nginx/nginx.conf";
const API_TS: &str = "core/frontend/src/utils/api.ts";
const RUNTIME_ENV: &str = RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_REPODIGEST;

pub const JOURNEYS: &[UserJourney] = &[ACCESS_BLUEOS_WEB_INTERFACE];

const ACCESS_BLUEOS_WEB_INTERFACE: UserJourney = UserJourney {
    id: JourneyId::AccessBlueosWebInterface,
    summary: Grounded::known(
        "Open the BlueOS web interface in a browser to access and configure vehicle services",
        Provenance::doc(GETTING, 22),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(GETTING, 26)),
    services: NGINX_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::AccessBlueosWebInterface,
        "nginx listens on port 80 and serves the frontend SPA at / for browser access",
    )]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
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
                Some("<!DOCTYPE html>"),
                "runtime-captures/nginx__pi4_navigator_master.json#running_baseline",
            )),
        ),
        service_step(
            "Verify the backend is online via GET /status (expects HTTP 204)",
            Some(sourced_route(HttpMethod::Get, "/status", None, 62)),
            Provenance::source(API_TS, 23),
            Some(runtime_outcome(
                204,
                None,
                "runtime-captures/nginx__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    availability: PRESENCE_ACCESS_BLUEOS_WEB_INTERFACE,
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const NGINX_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Nginx,
    Provenance::doc(DEV_CORE, 71),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::Nginx,
        method,
        path,
        version,
    }
}

const fn sourced_route(
    method: HttpMethod,
    path: &'static str,
    version: Option<&'static str>,
    line: u32,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(NGINX_CONF, line),
    )
}

const fn runtime_outcome(
    status: u16,
    body: Option<&'static str>,
    key: &'static str,
) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: body,
            transition: None,
        },
        Provenance::runtime(key, RUNTIME_ENV),
    )
}

const fn operator_step(
    description: &'static str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description,
            route,
            outcome,
        },
        provenance,
    )
}

const fn service_step(
    description: &'static str,
    route: Option<Grounded<RouteRef>>,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Service(ServiceId::Nginx),
            description,
            route,
            outcome,
        },
        provenance,
    )
}
