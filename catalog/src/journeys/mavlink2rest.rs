use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{
    Actor, BlastRadius, JourneyStep, Precondition, RouteRef, SoftwareAssumption, StepOutcome,
    UseCase, Visibility,
};
use crate::journey_presence::PRESENCE_INSPECT_MAVLINK_MESSAGES_IN_BROWSER;
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const OVERVIEW: &str = "content/usage/overview/index.md";
const MAVLINK_INSPECTOR_MENUS: &str = "core/frontend/src/menus.ts";
const MAVLINK_INSPECTOR_VIEW: &str = "core/frontend/src/views/MavlinkInspectorView.vue";

pub const JOURNEYS: &[UseCase] = &[INSPECT_MAVLINK_MESSAGES_IN_BROWSER];

const INSPECT_MAVLINK_MESSAGES_IN_BROWSER: UseCase =
    UseCase {
        id: JourneyId::InspectMavlinkMessagesInBrowser,
        summary: Grounded::known(
            "See and inspect MAVLink messages in real time from the browser",
            Provenance::doc(OVERVIEW, 122, "| [**MAVLink inspector**](../advanced/#mavlink-inspector) | "),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::doc(ADV, 496, "{% pirate() %}")),
        services: MAVLINK2REST_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::InspectLiveMavlinkMessages,
            "MAVLink Inspector filters, lists, and expands live MAVLink messages from the vehicle stream",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Software(SoftwareAssumption::AdvancedMode),
            Provenance::source(MAVLINK_INSPECTOR_MENUS, 73, "text: 'Manage MAVLink endpoints for internal/external "),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the MAVLink Inspector page from the sidebar",
                None,
                Provenance::source(MAVLINK_INSPECTOR_MENUS, 70, "icon: 'mdi-arrow-decision',"),
                None,
            ),
            operator_step(
                "Load the MAVLink Inspector interface",
                None,
                Provenance::source(MAVLINK_INSPECTOR_VIEW, 3, ":source=\"service_path\""),
                None,
            ),
            operator_step(
                "Filter for particular MAVLink messages",
                None,
                Provenance::doc(ADV, 504, "- filter for particular messages"),
                None,
            ),
            operator_step(
                "View past and current MAVLink messages",
                None,
                Provenance::doc(ADV, 505, "- view past and current messages"),
                None,
            ),
            operator_step(
                "Click on a message to see its full details",
                None,
                Provenance::doc(ADV, 506, "- click on messages to see their full details"),
                None,
            ),
        ]),
        availability: PRESENCE_INSPECT_MAVLINK_MESSAGES_IN_BROWSER,
        blast_radius: Grounded::known(
            BlastRadius::Safe,
            Provenance::asserted(
                "MAVLink Inspector only subscribes to and displays the live mavlink2rest message stream",
            ),
        ),
        chains_from: None,
    };

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const MAVLINK2REST_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Mavlink2rest,
    Provenance::doc(
        ADV,
        499,
        "{{ service(service=\"MAVLink2Rest\", port=6040, link=\"https://",
    ),
)]);

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
