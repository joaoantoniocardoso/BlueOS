use crate::id::{CapabilityId, ServiceId};
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::ParameterEditor,
        route: Observed::known(
            "/vehicle/parameters",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 132,
            },
        ),
        name: Observed::known(
            "Parameter Editor",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 133,
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/ParameterEditorView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 134,
            },
        ),
        menu_title: Observed::known(
            "Autopilot Parameters",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 11,
            },
        ),
        advanced_only: Observed::unknown("menus.ts entry has no advanced field"),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "autopilot_data",
                Evidence {
                    file: "core/frontend/src/components/parameter-editor/ParameterEditor.vue",
                    line: 162,
                },
            ),
            Evidenced::new(
                "autopilot",
                Evidence {
                    file: "core/frontend/src/components/parameter-editor/ParameterEditor.vue",
                    line: 163,
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                    endpoint: "mavlink2rest PARAM_SET",
                    purpose: "write single parameter edits, restore defaults, and batch file loads",
                },
                Evidence {
                    file: "core/frontend/src/libs/MAVLink2Rest/index.ts",
                    line: 243,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "POST /ardupilot-manager/v1.0/restart",
                    purpose: "reboot autopilot when ParameterEditorDialog save requests reboot after PARAM_SET",
                },
                Evidence {
                    file: "core/frontend/src/components/parameter-editor/ParameterEditorDialog.vue",
                    line: 119,
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[
            Rationaled::new(
                CapabilityId::EditAutopilotParameters,
                "ParameterEditor.vue provides searchable table, inline edit dialog, and per-param default restore via mavlink2rest.setParam",
            ),
            Rationaled::new(
                CapabilityId::ApplyParameterSet,
                "ParameterLoader.vue and setParameterFile parse .params/.parm multi-format files client-side and batch PARAM_SET writes with checkbox selection",
            ),
        ]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "search query and fuse index",
                    store: "ParameterEditor.vue search field and fuse computed",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "Fuse.js fuzzy search over name and description keys",
                },
                "parameter search is ephemeral client-side filtering over cached parameters",
            ),
            Rationaled::new(
                ClientState {
                    name: "edit and load dialogs",
                    store: "ParameterEditor.vue edit_dialog, load_param_dialog, edited_param",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "dialog visibility and currently edited Parameter object",
                },
                "editor dialogs hold transient UI state for single-param edit and file load flows",
            ),
            Rationaled::new(
                ClientState {
                    name: "parsed parameter file draft",
                    store: "ParameterEditor.vue loaded_parameter Dictionary",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "populated by setParameterFile regex parsers for three ArduPilot file formats",
                },
                "file load flow parses parameter files in-browser before ParameterLoader applies selections",
            ),
            Rationaled::new(
                ClientState {
                    name: "batch load selection checkboxes",
                    store: "ParameterLoader.vue param_checkboxes, select_all, user_selected_params",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "filters readonly/changed/selected params before writeSelectedParams",
                },
                "parameter file loader tracks per-param inclusion and retry interval locally",
            ),
            Rationaled::new(
                ClientState {
                    name: "sorted parameter table rows",
                    store: "ParameterEditor.vue computed params_no_input",
                    ownership: StateOwnership::Shared,
                    notes: "alphabetical sort wrapper over autopilot_data.parameters",
                },
                "table ordering is client-derived from the cached parameter list",
            ),
            Rationaled::new(
                ClientState {
                    name: "load progress percentage",
                    store: "ParameterEditor.vue computed params_percentage",
                    ownership: StateOwnership::Shared,
                    notes: "parameters.length / parameters_total progress bar",
                },
                "fetch progress is computed from global autopilot_data counters the page displays",
            ),
            Rationaled::new(
                ClientState {
                    name: "autopilot parameter cache",
                    store: "store/autopilot_data parameters, reboot_required",
                    ownership: StateOwnership::Shared,
                    notes: "globally fetched via PARAM_VALUE; mutated by page PARAM_SET writes",
                },
                "parameter cache is synced from vehicle but edited through this page's mavlink2rest calls",
            ),
            Rationaled::new(
                ClientState {
                    name: "save file header metadata",
                    store: "ParameterEditor.vue saveParametersToFile",
                    ownership: StateOwnership::Shared,
                    notes: "embeds vehicle/platform/version from autopilot store into exported .params header",
                },
                "export filename and comment header combine client timestamp with backend-reported firmware metadata",
            ),
        ]),
    };
