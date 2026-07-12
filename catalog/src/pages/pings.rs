use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::Pings,
        route: Observed::known(
            "/vehicle/pings",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 27,
            },
        ),
        name: Observed::known(
            "Pings",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 28,
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/Pings.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 29,
            },
        ),
        menu_title: Observed::known(
            "Ping Sonar Devices",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 91,
            },
        ),
        advanced_only: Observed::known(
            false,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 94,
            },
        ),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "ping",
                Evidence {
                    file: "core/frontend/src/views/Pings.vue",
                    line: 37,
                },
            ),
            Evidenced::new(
                "system",
                Evidence {
                    file: "core/frontend/src/views/Pings.vue",
                    line: 38,
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Ping),
                    endpoint: "GET /ping/v1.0/sensors",
                    purpose: "poll detected Ping1D and Ping360 devices while page listener is registered",
                },
                Evidence {
                    file: "core/frontend/src/store/ping.ts",
                    line: 61,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Ping),
                    endpoint: "POST /ping/v1.0/sensors",
                    purpose: "toggle MAVLink DISTANCE_SENSOR driver for Ping1D devices",
                },
                Evidence {
                    file: "core/frontend/src/components/ping/ping1d.vue",
                    line: 98,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/serial?udev=true",
                    purpose: "refresh serial port list for DevicePathHelper context on ping cards",
                },
                Evidence {
                    file: "core/frontend/src/store/system-information.ts",
                    line: 276,
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "ping card expand state",
                    store: "ping1d.vue / ping360.vue component data expand",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "chevron toggles optional firmware/device detail tables",
                },
                "per-card expand/collapse is ephemeral UI state",
            ),
            Rationaled::new(
                ClientState {
                    name: "mavlink driver desired state",
                    store: "ping1d.vue user_desired_mavlink_driver_state",
                    ownership: StateOwnership::Shared,
                    notes: "optimistic toggle until driver_status.mavlink_driver_enabled matches",
                },
                "Ping1D MAVLink distance toggle holds desired state client-side during POST round-trip",
            ),
            Rationaled::new(
                ClientState {
                    name: "detected ping devices",
                    store: "store/ping available_ping_devices",
                    ownership: StateOwnership::BackendOwned,
                    notes: "PingDevice[] mirrored from GET /ping/v1.0/sensors",
                },
                "device list originates from ping service and is displayed without client mutation",
            ),
            Rationaled::new(
                ClientState {
                    name: "ping listener registration",
                    store: "store/ping ping_listeners_number",
                    ownership: StateOwnership::Shared,
                    notes: "registerObject increments listener count to keep 5s poll active while page mounted",
                },
                "page lifecycle registers a weak-ref listener that enables background sensor polling",
            ),
        ]),
    };
