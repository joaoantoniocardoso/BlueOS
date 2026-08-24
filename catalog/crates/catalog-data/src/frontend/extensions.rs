use catalog_kernel::id::page::PageId;
use catalog_kernel::provenance::{
    AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled,
};
use catalog_model::page::{ClientState, ConsumeTarget, Page, PageServiceCall, StateOwnership};

pub const PAGE: Page =
    Page {
        id: PageId::Extensions,
        route: Observed::known(
            "/extensions/:port",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 112,
                anchor: "path: '/extensions/:port',",
            },
        ),
        name: Observed::known(
            "Extensions",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 113,
                anchor: "name: 'Extensions',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/ExtensionView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 114,
                anchor: "component: ExtensionView,",
            },
        ),
        menu_title: Observed::unknown("not in menu"),
        advanced_only: Observed::unknown("not in menu"),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "helper",
                Evidence {
                    file: "core/frontend/src/views/ExtensionView.vue",
                    line: 16,
                    anchor: "import helper from '@/store/helper'",
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::External,
                    endpoint: "GET {protocol}//{hostname}:{port}/{remaining_path}",
                    purpose: "BrIframe loads legacy extension HTTP server on discovered port with cache-busting query",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionView.vue",
                    line: 57,
                    anchor: "return `${window.location.protocol}//${window.location.hostn",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::External,
                    endpoint: "GET /extensionv2/{name}",
                    purpose: "BrIframe loads v2 relative-path extension UI when works_in_relative_paths is true",
                },
                Evidence {
                    file: "core/frontend/src/views/ExtensionView.vue",
                    line: 55,
                    anchor: "return `/extensionv2/${this.$route.params.name}`",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "resolved extension port",
                    store: "ExtensionView.vue data detected_port",
                    ownership: StateOwnership::Shared,
                    notes: "route.params.port for /extensions/:port; helper.services match on sanitized route.params.name for named alias routes",
                },
                "detected_port from route param port or helper service lookup by sanitized name; four router aliases share ExtensionView",
            ),
            Rationaled::new(
                ClientState {
                    name: "iframe source path",
                    store: "ExtensionView.vue computed service_path",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "chooses /extensionv2/{name} vs absolute host:port URL; appends cache_busting_time",
                },
                "client builds iframe URL and remaining wildcard path from route params",
            ),
            Rationaled::new(
                ClientState {
                    name: "document title override",
                    store: "ExtensionView.vue watch service_name",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "sets document.title from extension metadata name",
                },
                "browser tab title is updated client-side from helper service metadata",
            ),
            Rationaled::new(
                ClientState {
                    name: "full-page layout flag",
                    store: "ExtensionView.vue computed fullpage",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "true when route query full_page=true",
                },
                "query flag toggles iframe CSS to cover the viewport",
            ),
            Rationaled::new(
                ClientState {
                    name: "helper web services registry",
                    store: "store/helper services",
                    ownership: StateOwnership::BackendOwned,
                    notes: "read-only list polled globally; ExtensionView filters by sanitized_name",
                },
                "extension metadata and ports mirror helper /web_services; page does not trigger the fetch itself",
            ),
            Rationaled::new(
                ClientState {
                    name: "alias route params",
                    store: "ExtensionView.vue route.params name / pathMatch",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "also serves /extension/:name, /extension/:name/*, /extensionv2/:name routes",
                },
                "single component backs four router aliases; named and v2 paths supply params.name",
            ),
        ]),
    };
