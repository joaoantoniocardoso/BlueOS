use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::journey::{Actor, BlastRadius, JourneyStep, StepOutcome, UserJourney, Visibility};
use crate::journey_presence::PRESENCE_APPLY_PARAMETER_FILE;
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
        Provenance::doc(ADV, 353, "- Allows loading parameters from a file, and saving the curr"),
    ),
    visibility: Grounded::known(Visibility::Default, Provenance::source(MENUS, 11, "title: 'Autopilot Parameters',")),
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
            Provenance::source(MENUS, 11, "title: 'Autopilot Parameters',"),
        ),
        operator_step(
            "Choose a parameter file to load into the editor",
            Provenance::source(EDITOR, 128, "@change=\"setParameterFile\""),
        ),
        frontend_step(
            "setParameterFile parses the three ArduPilot parameter file formats into a draft dictionary in the browser",
            Provenance::source(EDITOR, 310, "async setParameterFile(file: (File | null)): Promise<void> {"),
            None,
        ),
        frontend_step(
            "Operator selects which parameters to apply; ParameterLoader batch-writes them via mavlink2rest PARAM_SET",
            Provenance::source(LOADER, 276, "mavlink2rest.setParam(name, value, autopilot_data.system_id)"),
            Some(mutation_outcome()),
        ),
        frontend_step(
            "If a changed parameter needs a reboot, the dialog requests POST /ardupilot-manager/v1.0/restart",
            Provenance::source(DIALOG, 119, "await AutopilotManager.restart()"),
            None,
        ),
    ]),
    availability: PRESENCE_APPLY_PARAMETER_FILE,
    blast_radius: Grounded::known(
        BlastRadius::Disruptive,
        Provenance::asserted(
            "Batch PARAM_SET writes autopilot EEPROM and may request ardupilot-manager restart for reboot-required parameters",
        ),
    ),
    chains_from: None,
};

const fn cap(id: CapabilityId, rationale: &'static str) -> GroundedItem<CapabilityId> {
    GroundedItem::new(id, Provenance::asserted(rationale))
}

const SERVICES: GroundedSet<ServiceId> = GroundedSet::known(&[
    GroundedItem::new(
        ServiceId::Mavlink2rest,
        Provenance::source(
            M2R_LIB,
            243,
            "setParam(name: string, value: number, sysid: number, type?: ",
        ),
    ),
    GroundedItem::new(
        ServiceId::ArdupilotManager,
        Provenance::source(DIALOG, 119, "await AutopilotManager.restart()"),
    ),
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
