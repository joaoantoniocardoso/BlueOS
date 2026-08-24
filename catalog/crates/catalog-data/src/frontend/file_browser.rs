use catalog_kernel::id::page::PageId;
use catalog_kernel::provenance::{AssertedSet, Evidence, Observed, ObservedSet, Rationaled};
use catalog_model::page::{ClientState, Page, StateOwnership};

pub const PAGE: Page =
    Page {
        id: PageId::FileBrowser,
        route: Observed::known(
            "/tools/file-browser/:path*",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 47,
                anchor: "path: '/tools/file-browser/:path*',",
            },
        ),
        name: Observed::known(
            "File Browser",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 48,
                anchor: "name: 'File Browser',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/FileBrowserView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 49,
                anchor: "component: defineAsyncComponent(() => import('../views/FileB",
            },
        ),
        menu_title: Observed::known(
            "File Browser",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 46,
                anchor: "title: 'File Browser',",
            },
        ),
        advanced_only: Observed::known(
            true,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 49,
                anchor: "advanced: true,",
            },
        ),
        stores: ObservedSet::known(&[]),
        consumes: ObservedSet::known(&[]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "filebrowser iframe base path",
                    store: "FileBrowserView.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "service_path='/file-browser/' prepended to route params for BrIframe src",
                },
                "fixed nginx proxy prefix for the embedded filebrowser application",
            ),
            Rationaled::new(
                ClientState {
                    name: "iframe source path",
                    store: "FileBrowserView.vue computed full_path",
                    ownership: StateOwnership::Shared,
                    notes: "full_path joins service_path with $route.params.path for deep-linking into folders",
                },
                "browser-held route param is forwarded into the iframe URL on navigation",
            ),
            Rationaled::new(
                ClientState {
                    name: "iframe loading overlay",
                    store: "BrIframe.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "iframe_loaded, gl_compatible, WebGL bubble animation until iframe load event",
                },
                "loading chrome is ephemeral client UI around the embedded filebrowser session",
            ),
        ]),
    };
