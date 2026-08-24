use catalog_kernel::id::page::PageId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{
    AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled,
};
use catalog_model::page::{ClientState, ConsumeTarget, Page, PageServiceCall, StateOwnership};

pub const PAGE: Page =
    Page {
        id: PageId::NmeaInjector,
        route: Observed::known(
            "/tools/nmea-injector",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 82,
                anchor: "path: '/tools/nmea-injector',",
            },
        ),
        name: Observed::known(
            "NMEA Injector",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 83,
                anchor: "name: 'NMEA Injector',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/NMEAInjectorView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 84,
                anchor: "component: defineAsyncComponent(() => import('../views/NMEAI",
            },
        ),
        menu_title: Observed::known(
            "NMEA Injector",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 84,
                anchor: "title: 'NMEA Injector',",
            },
        ),
        advanced_only: Observed::known(
            true,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 87,
                anchor: "advanced: true,",
            },
        ),
        stores: ObservedSet::known(&[Evidenced::new(
            "nmea_injector",
            Evidence {
                file: "core/frontend/src/components/nmea-injector/NMEAInjector.vue",
                line: 102,
                anchor: "import nmea_injector from '@/store/nmea-injector'",
            },
        )]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::NmeaInjector),
                    endpoint: "GET /nmea-injector/v1.0/socks",
                    purpose: "poll configured NMEA UDP/TCP listener sockets (5s interval via OneMoreTime)",
                },
                Evidence {
                    file: "core/frontend/src/store/nmea-injector.ts",
                    line: 42,
                    anchor: "url: `${this.API_URL}/socks`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::NmeaInjector),
                    endpoint: "POST /nmea-injector/v1.0/socks",
                    purpose: "create new NMEA socket from NMEASocketCreationDialog form",
                },
                Evidence {
                    file: "core/frontend/src/store/nmea-injector.ts",
                    line: 80,
                    anchor: "url: `${this.API_URL}/socks`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::NmeaInjector),
                    endpoint: "DELETE /nmea-injector/v1.0/socks",
                    purpose: "remove socket from NMEASocketCard delete button",
                },
                Evidence {
                    file: "core/frontend/src/store/nmea-injector.ts",
                    line: 63,
                    anchor: "url: `${this.API_URL}/socks`,",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "creation dialog visibility",
                    store: "NMEAInjector.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "show_creation_dialog toggled by FAB + button",
                },
                "dialog open/close is ephemeral UI state",
            ),
            Rationaled::new(
                ClientState {
                    name: "loading debounce flag",
                    store: "NMEAInjector.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "updating_nmea_sockets_debounced gates spinner for 300ms after fetch",
                },
                "debounced loading indicator prevents flicker between poll cycles",
            ),
            Rationaled::new(
                ClientState {
                    name: "NMEA socket creation form",
                    store: "NMEASocketCreationDialog.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "nmea_socket kind/port/component_id defaults; form validation rules",
                },
                "creation form fields are held client-side until POST /socks",
            ),
            Rationaled::new(
                ClientState {
                    name: "available NMEA sockets cache",
                    store: "store/nmea_injector available_nmea_sockets",
                    ownership: StateOwnership::BackendOwned,
                    notes: "NMEASocket[] mirrored from GET /nmea-injector/socks",
                },
                "configured socket list is fetched from nmea_injector and displayed without client mutation",
            ),
            Rationaled::new(
                ClientState {
                    name: "socket fetch status flag",
                    store: "store/nmea_injector updating_nmea_sockets",
                    ownership: StateOwnership::Shared,
                    notes: "loading boolean around axios lifecycle for list/create/delete",
                },
                "request status flag is a client-managed wrapper around backend calls",
            ),
        ]),
    };
