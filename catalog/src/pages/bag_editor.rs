use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::BagEditor,
        route: Observed::known(
            "/tools/bag-editor",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 107,
                anchor: "path: '/tools/bag-editor',",
            },
        ),
        name: Observed::known(
            "Bag editor",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 108,
                anchor: "name: 'Bag editor',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/BagEditorView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 109,
                anchor: "component: defineAsyncComponent(() => import('../views/BagEd",
            },
        ),
        menu_title: Observed::known(
            "Bag Editor",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 25,
                anchor: "title: 'Bag Editor',",
            },
        ),
        advanced_only: Observed::known(
            true,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 28,
                anchor: "advanced: true,",
            },
        ),
        stores: ObservedSet::known(&[Evidenced::new(
            "bag",
            Evidence {
                file: "core/frontend/src/views/BagEditorView.vue",
                line: 13,
                anchor: "import bag from '@/store/bag'",
            },
        )]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::BagOfHolding),
                    endpoint: "GET /bag/v1.0/get/{path}",
                    purpose: "load entire bag database on mount (path='*')",
                },
                Evidence {
                    file: "core/frontend/src/store/bag.ts",
                    line: 64,
                    anchor: "url: `${this.API_URL}/get/${path}`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::BagOfHolding),
                    endpoint: "POST /bag/v1.0/overwrite",
                    purpose: "persist edited JSON tree on JsonEditor save",
                },
                Evidence {
                    file: "core/frontend/src/store/bag.ts",
                    line: 24,
                    anchor: "url: `${this.API_URL}/overwrite`,",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "bag JSON document",
                    store: "BagEditorView.vue component data",
                    ownership: StateOwnership::Shared,
                    notes: "json loaded via bag.getData('*') on mounted; updated locally until save calls overwrite",
                },
                "document is cached from bag_of_holding then edited client-side before POST /overwrite",
            ),
            Rationaled::new(
                ClientState {
                    name: "jsoneditor mode and dirty tracking",
                    store: "JsonEditor.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "code_mode toggle, edited_code, is_different save icon state",
                },
                "tree/code editor presentation and unsaved-change detection are ephemeral client UI",
            ),
        ]),
    };
