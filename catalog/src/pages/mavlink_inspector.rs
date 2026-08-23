use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::MavlinkInspector,
        route: Observed::known(
            "/tools/mavlink-inspector",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 97,
                anchor: "path: '/tools/mavlink-inspector',",
            },
        ),
        name: Observed::known(
            "Mavlink Inspector",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 98,
                anchor: "name: 'Mavlink Inspector',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/MavlinkInspectorView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 99,
                anchor: "component: defineAsyncComponent(() => import('../views/Mavli",
            },
        ),
        menu_title: Observed::known(
            "MAVLink Inspector",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 77,
                anchor: "title: 'MAVLink Inspector',",
            },
        ),
        advanced_only: Observed::known(
            true,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 80,
                anchor: "advanced: true,",
            },
        ),
        stores: ObservedSet::known(&[]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Helper),
                    endpoint: "GET http://{hostname}:8080/",
                    purpose: "embed helper standalone MAVLink inspector UI via BrIframe (bypasses nginx /helper/ prefix)",
                },
                Evidence {
                    file: "core/frontend/src/views/MavlinkInspectorView.vue",
                    line: 19,
                    anchor: "service_path: `${window.location.protocol}//${window.locatio",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "iframe source URL",
                    store: "MavlinkInspectorView.vue component data service_path",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "protocol//hostname:8080 passed to BrIframe",
                },
                "only client state is the helper iframe URL; inspection UX runs inside embedded helper app",
            ),
            Rationaled::new(
                ClientState {
                    name: "iframe load overlay",
                    store: "BrIframe.vue iframe_loaded/gl_compatible",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "WebGL bubble loading animation until iframe load event",
                },
                "loading chrome is ephemeral BrIframe UI state around the embedded app",
            ),
        ]),
    };
