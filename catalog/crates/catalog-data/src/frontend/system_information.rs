use catalog_kernel::id::page::PageId;
use catalog_kernel::id::service::ServiceId;
use catalog_kernel::provenance::{
    AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled,
};
use catalog_model::page::{ClientState, ConsumeTarget, Page, PageServiceCall, StateOwnership};

pub const PAGE: Page =
    Page {
        id: PageId::SystemInformation,
        route: Observed::known(
            "/tools/system-information",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 92,
                anchor: "path: '/tools/system-information',",
            },
        ),
        name: Observed::known(
            "System Information",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 93,
                anchor: "name: 'System Information',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/SystemInformationView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 94,
                anchor: "component: defineAsyncComponent(() => import('../views/Syste",
            },
        ),
        menu_title: Observed::known(
            "System Information",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 114,
                anchor: "title: 'System Information',",
            },
        ),
        advanced_only: Observed::known(
            false,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 117,
                anchor: "advanced: false,",
            },
        ),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "system",
                Evidence {
                    file: "core/frontend/src/components/system-information/Processes.vue",
                    line: 58,
                    anchor: "import system_information, { FetchType } from '@/store/syste",
                },
            ),
            Evidenced::new(
                "commander",
                Evidence {
                    file: "core/frontend/src/components/system-information/Firmware.vue",
                    line: 88,
                    anchor: "import commander from '@/store/commander'",
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/system/process",
                    purpose: "poll process list every 5s from Processes tab",
                },
                Evidence {
                    file: "core/frontend/src/components/system-information/Processes.vue",
                    line: 102,
                    anchor: "this.timer = setInterval(() => system_information.fetchSyste",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/system/cpu",
                    purpose: "poll CPU usage every 2s from System Monitor tab",
                },
                Evidence {
                    file: "core/frontend/src/components/system-information/SystemCondition.vue",
                    line: 135,
                    anchor: "system_information.fetchSystemInformation(FetchType.SystemCp",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/system/disk",
                    purpose: "poll disk usage every 2s from System Monitor tab",
                },
                Evidence {
                    file: "core/frontend/src/components/system-information/SystemCondition.vue",
                    line: 136,
                    anchor: "system_information.fetchSystemInformation(FetchType.SystemDi",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/system/memory",
                    purpose: "poll memory usage every 2s from System Monitor tab",
                },
                Evidence {
                    file: "core/frontend/src/components/system-information/SystemCondition.vue",
                    line: 137,
                    anchor: "system_information.fetchSystemInformation(FetchType.SystemMe",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/system/unix_time_seconds",
                    purpose: "poll clock every 1s for About tab time display",
                },
                Evidence {
                    file: "core/frontend/src/components/system-information/AboutThisSystem.vue",
                    line: 86,
                    anchor: "system_information.fetchSystemInformation(FetchType.SystemUn",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/model",
                    purpose: "poll board model every 1s for About tab",
                },
                Evidence {
                    file: "core/frontend/src/components/system-information/AboutThisSystem.vue",
                    line: 88,
                    anchor: "this.timer_model = setInterval(() => system_information.fetc",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/journal",
                    purpose: "initial journal snapshot when Journal tab mounts (live tail via global ws)",
                },
                Evidence {
                    file: "core/frontend/src/components/system-information/Journal.vue",
                    line: 52,
                    anchor: "system_information.fetchSystemInformation(FetchType.JournalT",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Linux2rest),
                    endpoint: "GET /system-information/usb",
                    purpose: "poll USB device tree every 5s from USB tab",
                },
                Evidence {
                    file: "core/frontend/src/components/system-information/Usb.vue",
                    line: 177,
                    anchor: "url: '/system-information/usb',",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Commander),
                    endpoint: "GET /commander/v1.0/raspi/vcgencmd",
                    purpose: "read Raspberry Pi firmware and bootloader strings on Firmware tab mount",
                },
                Evidence {
                    file: "core/frontend/src/store/commander.ts",
                    line: 157,
                    anchor: "url: `${this.API_URL}/raspi/vcgencmd`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Commander),
                    endpoint: "GET /commander/v1.0/raspi/eeprom_update",
                    purpose: "compare current vs latest EEPROM/bootloader on Firmware tab mount",
                },
                Evidence {
                    file: "core/frontend/src/store/commander.ts",
                    line: 202,
                    anchor: "url: `${this.API_URL}/raspi/eeprom_update`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Commander),
                    endpoint: "POST /commander/v1.0/raspi/eeprom_update",
                    purpose: "apply Raspberry Pi EEPROM update from Firmware tab Update button",
                },
                Evidence {
                    file: "core/frontend/src/store/commander.ts",
                    line: 223,
                    anchor: "url: `${this.API_URL}/raspi/eeprom_update`,",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "active tab",
                    store: "SystemInformationView.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "page_selected v-tabs index; items filtered by pirate mode",
                },
                "tab selection and pirate-gated tab list are ephemeral UI state",
            ),
            Rationaled::new(
                ClientState {
                    name: "process table filters",
                    store: "Processes.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "search text and filterHeader checkbox selection for visible columns",
                },
                "process search and column visibility are local UI state",
            ),
            Rationaled::new(
                ClientState {
                    name: "usb fetch status",
                    store: "Usb.vue component data",
                    ownership: StateOwnership::Shared,
                    notes: "loading, error, devices[], 5s poll timer; buses/treeItems computed client-side",
                },
                "USB tab tracks fetch lifecycle locally and builds bus tree from flat device list",
            ),
            Rationaled::new(
                ClientState {
                    name: "firmware panel state",
                    store: "Firmware.vue component data",
                    ownership: StateOwnership::Shared,
                    notes: "vcgencmd, eeprom_update, do_eeprom_update, waiting_for_update; parsed CURRENT/LATEST dates",
                },
                "firmware tab caches commander command output and derives update availability client-side",
            ),
            Rationaled::new(
                ClientState {
                    name: "system monitor aggregates",
                    store: "SystemCondition.vue computed cpu/memory/disk/temperature",
                    ownership: StateOwnership::Shared,
                    notes: "mean CPU %, RAM/SWAP percentages, root disk %, main temperature from store/system fields",
                },
                "monitor cards derive headline percentages and HTML detail text from polled system snapshots",
            ),
            Rationaled::new(
                ClientState {
                    name: "about panel display",
                    store: "AboutThisSystem.vue computed info/avatar",
                    ownership: StateOwnership::Shared,
                    notes: "OS icon map, model string, live clock from unix_time_seconds",
                },
                "about tab formats polled model and clock fields for display",
            ),
            Rationaled::new(
                ClientState {
                    name: "journal presentation",
                    store: "Journal.vue methods priorityClass/priorityLabel",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "priority-to-color mapping and timestamp formatting for virtual scroll rows",
                },
                "journal entry styling is client-side presentation over streamed entries",
            ),
            Rationaled::new(
                ClientState {
                    name: "kernel log presentation",
                    store: "Kernel.vue method getClass",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "syslog level to Vuetify color class mapping",
                },
                "kernel message colors are client-side presentation over websocket buffer",
            ),
            Rationaled::new(
                ClientState {
                    name: "network card ordering",
                    store: "Network.vue computed networks",
                    ownership: StateOwnership::Shared,
                    notes: "system.network sorted by interface name for NetworkCard grid",
                },
                "network tab sorts interfaces client-side; data comes from global background network poll",
            ),
            Rationaled::new(
                ClientState {
                    name: "derived interface and disk rates",
                    store: "store/system updateSystemNetwork/updateSystemDisk mutations",
                    ownership: StateOwnership::Shared,
                    notes: "upload_speed, download_speed, write_rate_Bps computed from successive samples",
                },
                "store derives throughput rates between polls for widgets that read the global system cache",
            ),
            Rationaled::new(
                ClientState {
                    name: "system information cache",
                    store: "store/system system, journal_entries, kernel_message, model",
                    ownership: StateOwnership::BackendOwned,
                    notes: "linux2rest snapshots mirrored in Vuex; network/temperature/platform also polled globally in App bootstrap",
                },
                "core system fields are backend-owned mirrors; page reads global store populated outside tab mounts",
            ),
        ]),
    };
