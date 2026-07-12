use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{Actor, JourneyStep, StepOutcome, UserJourney, Visibility};
use crate::page::PageId;
use crate::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};

const ADV: &str = "content/usage/advanced/index.md";
const MENUS: &str = "core/frontend/src/menus.ts";
const EDITOR: &str = "core/frontend/src/components/parameter-editor/ParameterEditor.vue";
const LOADER: &str = "core/frontend/src/components/parameter-editor/ParameterLoader.vue";
const DIALOG: &str = "core/frontend/src/components/parameter-editor/ParameterEditorDialog.vue";
const M2R_LIB: &str = "core/frontend/src/libs/MAVLink2Rest/index.ts";

pub const JOURNEYS: &[UserJourney] = &[APPLY_PARAMETER_FILE];

const APPLY_PARAMETER_FILE: UserJourney = UserJourney {
    id: JourneyId::ApplyParameterFile,
    summary: Grounded::known(
        "Operator loads a parameter file on the Autopilot Parameters page; the browser parses it and batch-writes the selected parameters to the autopilot",
        Provenance::doc(ADV, 353),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::source(MENUS, 11)),
    services: SERVICES,
    capability_refs: GroundedSet::known(&[
        cap(
            CapabilityId::EditAutopilotParameters,
            "ParameterEditor.vue provides the searchable table and per-parameter edit/restore via mavlink2rest.setParam",
        ),
        cap(
            CapabilityId::ApplyParameterSet,
            "ParameterLoader.vue parses .params/.parm files client-side and batches PARAM_SET writes with checkbox selection",
        ),
    ]),
    preconditions: GroundedSet::known(&[]),
    steps: GroundedSet::known(&[
        operator_step(
            "Open the Autopilot Parameters page from the sidebar",
            Provenance::source(MENUS, 11),
        ),
        operator_step(
            "Choose a parameter file to load into the editor",
            Provenance::source(EDITOR, 128),
        ),
        frontend_step(
            "setParameterFile parses the three ArduPilot parameter file formats into a draft dictionary in the browser",
            Provenance::source(EDITOR, 310),
            None,
        ),
        frontend_step(
            "Operator selects which parameters to apply; ParameterLoader batch-writes them via mavlink2rest PARAM_SET",
            Provenance::source(LOADER, 276),
            Some(mutation_outcome()),
        ),
        frontend_step(
            "If a changed parameter needs a reboot, the dialog requests POST /ardupilot-manager/v1.0/restart",
            Provenance::source(DIALOG, 119),
            None,
        ),
    ]),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[
    GroundedItem::new(ServiceId::Mavlink2rest, Provenance::source(M2R_LIB, 243)),
    GroundedItem::new(ServiceId::ArdupilotManager, Provenance::source(DIALOG, 119)),
]);

const fn operator_step(
    description: &'static str,
    provenance: Provenance,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Operator,
            description,
            route: None,
            outcome: None,
        },
        provenance,
    )
}

const fn frontend_step(
    description: &'static str,
    provenance: Provenance,
    outcome: Option<Grounded<StepOutcome>>,
) -> GroundedItem<JourneyStep> {
    GroundedItem::new(
        JourneyStep {
            actor: Actor::Frontend(PageId::ParameterEditor),
            description,
            route: None,
            outcome,
        },
        provenance,
    )
}

const fn mutation_outcome() -> Grounded<StepOutcome> {
    Grounded::unknown(
        "batch PARAM_SET result requires runtime capture during a live parameter apply on the Pi (mutating; not performed)",
    )
}
