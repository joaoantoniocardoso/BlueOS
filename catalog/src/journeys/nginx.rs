use crate::capture_env::RUNTIME_CAPTURE_ENV_PI4_NAVIGATOR_REPODIGEST;
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, BlastRadius, BodyKind, HttpMethod, JourneyStep, RouteRef, StepOutcome, UserJourney,
    Visibility,
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
        Provenance::doc(GETTING, 22, "BlueOS is designed as a modular collection of services, whic"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::doc(GETTING, 26, "### Interface Access")),
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
            Provenance::doc(GETTING, 29, "- When BlueOS is connected via a wired connection, it is als"),
            None,
        ),
        service_step(
            "Serve the frontend SPA from /",
            Some(sourced_route(HttpMethod::Get, "/", None, 268, "location / {")),
            Provenance::source(NGINX_CONF, 269, "root /home/pi/frontend;"),
            Some(runtime_outcome(
                200,
                Some("<!DOCTYPE html>"),
                "runtime-captures/nginx__pi4_navigator_master.json#running_baseline",
            )),
        ),
        service_step(
            "Verify the backend is online via GET /status (expects HTTP 204)",
            Some(sourced_route(HttpMethod::Get, "/status", None, 62, "location = /status {")),
            Provenance::source(API_TS, 23, "// Backend status verification through /status endpoint shou"),
            Some(runtime_outcome(
                204,
                None,
                "runtime-captures/nginx__pi4_navigator_master.json#running_baseline",
            )),
        ),
    ]),
    availability: PRESENCE_ACCESS_BLUEOS_WEB_INTERFACE,
    blast_radius: Grounded::known(
        BlastRadius::Safe,
        Provenance::asserted(
            "loads the frontend SPA and polls GET /status without mutating vehicle configuration or services",
        ),
    ),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const NGINX_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Nginx,
    Provenance::doc(
        DEV_CORE,
        71,
        "| BlueOS | The main BlueOS interface | - [Dashboard](../../u",
    ),
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
    anchor: &'static str,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(NGINX_CONF, line, anchor),
    )
}

const fn runtime_body_kind(status: u16, body: Option<&'static str>) -> BodyKind {
    match body {
        Some(_) => BodyKind::Payload,
        None if status == 204 => BodyKind::Empty,
        None => BodyKind::Unknown,
    }
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
            body_kind: runtime_body_kind(status, body),
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
