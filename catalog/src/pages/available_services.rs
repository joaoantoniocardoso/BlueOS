use crate::page::{ClientState, Page, PageId, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::AvailableServices,
        route: Observed::known(
            "/tools/available-services",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 82,
                anchor: "path: '/tools/nmea-injector',",
            },
        ),
        name: Observed::known(
            "Available Services",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 83,
                anchor: "name: 'NMEA Injector',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/AvailableServicesView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 84,
                anchor: "component: defineAsyncComponent(() => import('../views/NMEAI",
            },
        ),
        menu_title: Observed::known(
            "Available Services",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 17,
                anchor: "title: 'Available Services',",
            },
        ),
        advanced_only: Observed::known(
            true,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 20,
                anchor: "advanced: true,",
            },
        ),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "helper",
                Evidence {
                    file: "core/frontend/src/components/scanner/availableServicesTable.vue",
                    line: 78,
                    anchor: "import helper from '@/store/helper'",
                },
            ),
        ]),
        consumes: ObservedSet::known(&[]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "sorted service list",
                    store: "availableServicesTable.vue computed availableServices",
                    ownership: StateOwnership::Shared,
                    notes: "helper.services sorted alphabetically by title for table rows",
                },
                "display order is client-derived from helper scan cache",
            ),
            Rationaled::new(
                ClientState {
                    name: "service scan cache",
                    store: "store/helper services",
                    ownership: StateOwnership::BackendOwned,
                    notes: "Service[] from GET /helper/latest/web_services; helper polls every 5s globally",
                },
                "page only reads helper.services populated by global background scan, not page-triggered",
            ),
            Rationaled::new(
                ClientState {
                    name: "external link URLs",
                    store: "availableServicesTable.vue method createWebpageUrl",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "builds http://hostname:port/path links opening service UI and swagger in new tab",
                },
                "link targets are composed client-side from scanned port/path metadata",
            ),
        ]),
    };
