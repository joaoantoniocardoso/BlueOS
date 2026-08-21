use crate::page::{ClientState, Page, PageId, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Observed, ObservedSet, Rationaled};

pub const PAGE: Page = Page {
    id: PageId::Terminal,
    route: Observed::known(
        "/tools/web-terminal",
        Evidence {
            file: "core/frontend/src/router/index.ts",
            line: 52,
            anchor: "path: '/tools/disk',",
        },
    ),
    name: Observed::known(
        "Terminal",
        Evidence {
            file: "core/frontend/src/router/index.ts",
            line: 53,
            anchor: "name: 'Disk',",
        },
    ),
    component: Observed::known(
        "core/frontend/src/views/TerminalView.vue",
        Evidence {
            file: "core/frontend/src/router/index.ts",
            line: 54,
            anchor: "component: defineAsyncComponent(() => import('../views/Disk.",
        },
    ),
    menu_title: Observed::known(
        "Terminal",
        Evidence {
            file: "core/frontend/src/menus.ts",
            line: 114,
            anchor: "title: 'System Information',",
        },
    ),
    advanced_only: Observed::known(
        true,
        Evidence {
            file: "core/frontend/src/menus.ts",
            line: 117,
            anchor: "advanced: false,",
        },
    ),
    stores: ObservedSet::known(&[]),
    consumes: ObservedSet::known(&[]),
    frontend_features: AssertedSet::established(&[]),
    client_state: AssertedSet::established(&[
        Rationaled::new(
            ClientState {
                name: "ttyd iframe source",
                store: "TerminalView.vue component data",
                ownership: StateOwnership::FrontendOwned,
                notes: "service_path='/terminal/' passed to BrIframe as fixed src",
            },
            "page only embeds the ttyd web terminal via nginx proxy; no REST client calls",
        ),
        Rationaled::new(
            ClientState {
                name: "iframe loading overlay",
                store: "BrIframe.vue component data",
                ownership: StateOwnership::FrontendOwned,
                notes:
                    "iframe_loaded, gl_compatible, WebGL bubble animation until iframe load event",
            },
            "loading chrome is ephemeral client UI around the embedded ttyd session",
        ),
    ]),
};
