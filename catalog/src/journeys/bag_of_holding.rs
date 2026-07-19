// bag_of_holding is infrastructure: most frontend writes (wizard state, settings, vehicle
// images, Major Tom tokens) are on behalf of other features and belong in those journeys.
// The operator docs describe one first-class workflow — the advanced Bag Editor page.
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, HttpMethod, JourneyStep, Precondition, RouteRef, SoftwareRequirement, StepOutcome,
    UserJourney, Visibility,
};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const BAG_MAIN: &str = "core/services/bag_of_holding/main.py";
const BAG_STORE: &str = "core/frontend/src/store/bag.ts";
const BAG_VIEW: &str = "core/frontend/src/views/BagEditorView.vue";
const BAG_MENUS: &str = "core/frontend/src/menus.ts";

pub const JOURNEYS: &[UserJourney] = &[MODIFY_BAG_DATABASE];

const MODIFY_BAG_DATABASE: UserJourney = UserJourney {
    id: JourneyId::ModifyBagDatabase,
    summary: Grounded::known(
        "Modify the JSON database used to persist frontend interface state",
        Provenance::doc(ADV, 382),
    ),
    visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 376)),
    services: BAG_OF_HOLDING_SERVICES,
    capability_refs: GroundedSet::known(&[cap(
        CapabilityId::EditBagJsonStore,
        "Bag Editor loads the full document tree and overwrites it on save",
    )]),
    preconditions: GroundedSet::known(&[GroundedItem::new(
        Precondition::Software(SoftwareRequirement::PirateMode),
        Provenance::doc(ADV, 376),
    )]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Bag Editor page from the sidebar",
            None,
            Provenance::source(BAG_MENUS, 25),
            None,
        ),
        operator_step(
            "Load the full bag database into the JSON editor",
            Some(sourced_route(HttpMethod::Get, "/get/*", Some("v1.0"), 64)),
            Provenance::source(BAG_VIEW, 26),
            None,
        ),
        operator_step(
            "Save edited JSON to overwrite the bag database",
            Some(sourced_route(
                HttpMethod::Post,
                "/overwrite",
                Some("v1.0"),
                24,
            )),
            Provenance::source(BAG_VIEW, 30),
            Some(source_outcome(200, 73)),
        ),
    ]),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const BAG_OF_HOLDING_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::BagOfHolding,
    Provenance::doc(ADV, 379),
)]);

const fn route(method: HttpMethod, path: &'static str, version: Option<&'static str>) -> RouteRef {
    RouteRef {
        service: ServiceId::BagOfHolding,
        method,
        path,
        version,
    }
}

const fn source_outcome(status: u16, line: u32) -> Grounded<StepOutcome> {
    Grounded::known(
        StepOutcome {
            expected_status: Some(status),
            body_predicate: None,
            transition: None,
        },
        Provenance::source(BAG_MAIN, line),
    )
}

const fn sourced_route(
    method: HttpMethod,
    path: &'static str,
    version: Option<&'static str>,
    line: u32,
) -> Grounded<RouteRef> {
    Grounded::known(
        route(method, path, version),
        Provenance::source(BAG_STORE, line),
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
