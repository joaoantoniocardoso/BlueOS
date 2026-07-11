use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{Actor, JourneyStep, Precondition, StepOutcome, UserJourney, Visibility};
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const OVERVIEW: &str = "content/usage/overview/index.md";
const TERMINAL_MENUS: &str = "core/frontend/src/menus.ts";
const TERMINAL_VIEW: &str = "core/frontend/src/views/TerminalView.vue";
const NGINX: &str = "core/tools/nginx/nginx.conf";

pub const JOURNEYS: &[UserJourney] = &[ACCESS_WEB_TERMINAL];

const ACCESS_WEB_TERMINAL: UserJourney =
    UserJourney {
        id: JourneyId::AccessWebTerminal,
        summary: Grounded::known(
            "Access a web-based terminal with tmux session and direct access into the core BlueOS docker container"
                ,
            Provenance::doc(ADV, 623),
        ),
        visibility: Grounded::known(Visibility::Advanced, Provenance::source(TERMINAL_MENUS, 117)),
        services: TTYD_SERVICES,
        capability_refs: GroundedSet::known(&[cap(CapabilityId::AccessWebTerminal,
            "Terminal page embeds ttyd web terminal over WebSocket at /terminal/ attached to user_terminal tmux",
        )]),
        preconditions: GroundedSet::known(&[GroundedItem::new(
            Precondition::Other("Advanced mode enabled to access the Terminal page"),
            Provenance::source(TERMINAL_MENUS, 117),
        )]),
        steps: GroundedSet::known(&[
            operator_step(
                "Open the Terminal page from the sidebar",
                None,
                Provenance::source(TERMINAL_MENUS, 114),
                None,
            ),
            operator_step(
                "Load the embedded web terminal connecting to /terminal/",
                None,
                Provenance::source(TERMINAL_VIEW, 19),
                None,
            ),
            operator_step(
                "Connect over WebSocket to /terminal/ proxied to ttyd on port 8088",
                None,
                Provenance::source(NGINX, 216),
                None,
            ),
            operator_step(
                "Use the interactive Linux shell attached to the persistent user_terminal tmux session",
                None,
                Provenance::doc(OVERVIEW, 125),
                Some(live_outcome(
                    "interactive terminal session cannot be captured by a simple HTTP probe",
                )),
            ),
        ]),
        chains_from: None,
    };

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const TTYD_SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[GroundedItem::new(
    ServiceId::Ttyd,
    Provenance::doc(ADV, 620),
)]);

const fn live_outcome(reason: &'static str) -> Grounded<StepOutcome> {
    Grounded::unknown(reason)
}

const fn operator_step(
    description: &'static str,
    route: Option<Grounded<crate::journey::RouteRef>>,
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
