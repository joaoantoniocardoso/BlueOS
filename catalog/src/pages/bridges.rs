use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::Bridges,
        route: Observed::known(
            "/tools/bridges",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 72,
                anchor: "path: '/tools/records',",
            },
        ),
        name: Observed::known(
            "Bridges",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 73,
                anchor: "name: 'Records',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/BridgesView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 74,
                anchor: "component: defineAsyncComponent(() => import('../views/Recor",
            },
        ),
        menu_title: Observed::known(
            "Serial Bridges",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 99,
                anchor: "icon: 'mdi-radar',",
            },
        ),
        advanced_only: Observed::known(
            true,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 102,
                anchor: "text: 'Manage detected Ping family sonar devices, connected ",
            },
        ),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "bridget",
                Evidence {
                    file: "core/frontend/src/components/bridges/Bridget.vue",
                    line: 68,
                    anchor: "import bridget from '@/store/bridget'",
                },
            ),
            Evidenced::new(
                "system",
                Evidence {
                    file: "core/frontend/src/components/bridges/Bridget.vue",
                    line: 69,
                    anchor: "import system_information from '@/store/system-information'",
                },
            ),
            Evidenced::new(
                "autopilot",
                Evidence {
                    file: "core/frontend/src/components/bridges/BridgeCreationDialog.vue",
                    line: 165,
                    anchor: "import autopilot from '@/store/autopilot_manager'",
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Bridget),
                    endpoint: "GET /bridget/v1.0/bridges",
                    purpose: "poll active serial-to-UDP/TCP bridges (5s interval after page registers listener)",
                },
                Evidence {
                    file: "core/frontend/src/store/bridget.ts",
                    line: 88,
                    anchor: "url: `${this.API_URL}/bridges`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Bridget),
                    endpoint: "GET /bridget/v1.0/serial_ports",
                    purpose: "poll serial ports available for bridging (5s module task; used by creation dialog)",
                },
                Evidence {
                    file: "core/frontend/src/store/bridget.ts",
                    line: 112,
                    anchor: "url: `${this.API_URL}/serial_ports`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Bridget),
                    endpoint: "POST /bridget/v1.0/bridges",
                    purpose: "create new serial bridge from BridgeCreationDialog form",
                },
                Evidence {
                    file: "core/frontend/src/components/bridges/BridgeCreationDialog.vue",
                    line: 312,
                    anchor: "url: `${bridget.API_URL}/bridges`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Bridget),
                    endpoint: "DELETE /bridget/v1.0/bridges",
                    purpose: "remove bridge from BridgeCard delete button",
                },
                Evidence {
                    file: "core/frontend/src/components/bridges/BridgeCard.vue",
                    line: 100,
                    anchor: "method: 'delete',",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/serial?udev=true",
                    purpose: "poll serial port metadata for display names (5s interval from Bridget mounted)",
                },
                Evidence {
                    file: "core/frontend/src/store/system-information.ts",
                    line: 278,
                    anchor: "url: `${this.API_URL}/${type}`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::ArdupilotManager),
                    endpoint: "GET /ardupilot-manager/v1.0/serials",
                    purpose: "fetch autopilot serial endpoint ownership for creation dialog port selector",
                },
                Evidence {
                    file: "core/frontend/src/components/autopilot/AutopilotManagerUpdater.ts",
                    line: 17,
                    anchor: "url: `${autopilot.API_URL}/serials`,",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "creation dialog visibility",
                    store: "Bridget.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "show_creation_dialog toggled by FAB + button",
                },
                "dialog open/close is ephemeral UI state",
            ),
            Rationaled::new(
                ClientState {
                    name: "bridge list with serial info",
                    store: "Bridget.vue computed available_bridges",
                    ownership: StateOwnership::Shared,
                    notes: "joins bridget.available_bridges with system.serial.ports for display names",
                },
                "bridge records from bridget are enriched client-side with linux2rest serial metadata",
            ),
            Rationaled::new(
                ClientState {
                    name: "available bridges cache",
                    store: "store/bridget available_bridges",
                    ownership: StateOwnership::BackendOwned,
                    notes: "Bridge[] mirrored from GET /bridget/bridges",
                },
                "active bridge list is fetched from bridget and displayed without client mutation",
            ),
            Rationaled::new(
                ClientState {
                    name: "available serial ports cache",
                    store: "store/bridget available_serial_ports",
                    ownership: StateOwnership::BackendOwned,
                    notes: "string[] mirrored from GET /bridget/serial_ports",
                },
                "bridge-eligible serial port names originate from bridget",
            ),
            Rationaled::new(
                ClientState {
                    name: "bridge fetch status flags",
                    store: "store/bridget updating_bridges, updating_serial_ports",
                    ownership: StateOwnership::Shared,
                    notes: "loading booleans around bridge and serial port axios lifecycle",
                },
                "request status flags are client-managed wrappers around backend calls",
            ),
            Rationaled::new(
                ClientState {
                    name: "bridge creation form",
                    store: "BridgeCreationDialog.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "tab server/client, bridge serial_path/baud/ip/udp ports; form validation rules",
                },
                "creation wizard fields are held client-side until POST /bridges",
            ),
            Rationaled::new(
                ClientState {
                    name: "bridge mode label",
                    store: "BridgeCreationDialog.vue computed bridge_mode",
                    ownership: StateOwnership::Shared,
                    notes: "derives Server/Client mode label from bridge.ip address",
                },
                "mode description is client-computed from form IP selection",
            ),
            Rationaled::new(
                ClientState {
                    name: "serial port selector enrichment",
                    store: "BridgeCreationDialog.vue computed available_serial_ports",
                    ownership: StateOwnership::Shared,
                    notes: "filters system serial ports by bridget list; marks ports used by autopilot",
                },
                "port dropdown merges bridget, linux2rest, and ardupilot_manager serial data client-side",
            ),
        ]),
    };
