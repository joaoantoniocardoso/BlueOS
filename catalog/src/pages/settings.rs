use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::Settings,
        route: Observed::known(
            "/settings",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 142,
                anchor: "path: '/tools/zenoh-inspector',",
            },
        ),
        name: Observed::known(
            "Settings",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 143,
                anchor: "name: 'Zenoh Inspector',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/SettingsView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 144,
                anchor: "component: defineAsyncComponent(() => import('../views/Zenoh",
            },
        ),
        menu_title: Observed::unknown("not in menu"),
        advanced_only: Observed::unknown("not in menu"),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "settings",
                Evidence {
                    file: "core/frontend/src/views/SettingsView.vue",
                    line: 503,
                    anchor: "import settings from '@/libs/settings'",
                },
            ),
            Evidenced::new(
                "customization",
                Evidence {
                    file: "core/frontend/src/components/customization/ThemeCustomization.vue",
                    line: 245,
                    anchor: "import customization_store from '@/store/customization'",
                },
            ),
            Evidenced::new(
                "commander",
                Evidence {
                    file: "core/frontend/src/views/SettingsView.vue",
                    line: 506,
                    anchor: "import commander from '@/store/commander'",
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::BagOfHolding),
                    endpoint: "POST /bag/v1.0/set/settings",
                    purpose: "persist theme, pirate mode, and dev mode toggles via SettingsStore.save",
                },
                Evidence {
                    file: "core/frontend/src/store/settings.ts",
                    line: 106,
                    anchor: "await bag.setData('settings', SettingsStore.state)",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::BagOfHolding),
                    endpoint: "POST /bag/v1.0/set/wizard",
                    purpose: "re-enable configuration wizard with version 0 payload",
                },
                Evidence {
                    file: "core/frontend/src/views/SettingsView.vue",
                    line: 773,
                    anchor: "await bag.setData('wizard', payload)",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Commander),
                    endpoint: "GET /commander/v1.0/services/check_log_folder_size",
                    purpose: "poll BlueOS service log folder size every 30s",
                },
                Evidence {
                    file: "core/frontend/src/views/SettingsView.vue",
                    line: 614,
                    anchor: "url: `${API_URL}/services/check_log_folder_size`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Commander),
                    endpoint: "GET /commander/v1.0/services/check_mavlink_log_folder_size",
                    purpose: "poll MAVLink log folder size every 30s",
                },
                Evidence {
                    file: "core/frontend/src/views/SettingsView.vue",
                    line: 632,
                    anchor: "url: `${API_URL}/services/check_mavlink_log_folder_size`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Commander),
                    endpoint: "POST /commander/v1.0/services/remove_log_stream",
                    purpose: "stream-delete BlueOS service logs with per-file progress",
                },
                Evidence {
                    file: "core/frontend/src/views/SettingsView.vue",
                    line: 701,
                    anchor: "url: `${API_URL}/services/remove_log_stream`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Commander),
                    endpoint: "POST /commander/v1.0/services/remove_mavlink_log",
                    purpose: "clear all MAVLink flight logs after confirm dialog",
                },
                Evidence {
                    file: "core/frontend/src/views/SettingsView.vue",
                    line: 749,
                    anchor: "url: `${API_URL}/services/remove_mavlink_log`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Commander),
                    endpoint: "POST /commander/v1.0/settings/reset",
                    purpose: "factory-reset all BlueOS service settings",
                },
                Evidence {
                    file: "core/frontend/src/views/SettingsView.vue",
                    line: 669,
                    anchor: "url: `${API_URL}/settings/reset`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Commander),
                    endpoint: "POST /commander/v1.0/shutdown",
                    purpose: "reboot onboard computer after settings reset",
                },
                Evidence {
                    file: "core/frontend/src/views/SettingsView.vue",
                    line: 688,
                    anchor: "commander.shutdown(ShutdownType.Reboot)",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Filebrowser),
                    endpoint: "GET /file-browser/api/resources/system_logs",
                    purpose: "download zipped BlueOS service logs",
                },
                Evidence {
                    file: "core/frontend/src/views/SettingsView.vue",
                    line: 600,
                    anchor: "const folder = await filebrowser.fetchFolder('system_logs')",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Filebrowser),
                    endpoint: "GET /file-browser/api/resources/ardupilot_logs/logs",
                    purpose: "download zipped MAVLink flight logs",
                },
                Evidence {
                    file: "core/frontend/src/views/SettingsView.vue",
                    line: 608,
                    anchor: "const folder = await filebrowser.fetchFolder('ardupilot_logs",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Customization),
                    endpoint: "GET /customization/v1.0/theme",
                    purpose: "load current theme palette on ThemeCustomization mount",
                },
                Evidence {
                    file: "core/frontend/src/store/customization.ts",
                    line: 124,
                    anchor: "const response = await back_axios({ method: 'get', url: `${A",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Customization),
                    endpoint: "PUT /customization/v1.0/theme",
                    purpose: "save primary color from color picker Apply button",
                },
                Evidence {
                    file: "core/frontend/src/store/customization.ts",
                    line: 137,
                    anchor: "url: `${API_URL}/theme`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Customization),
                    endpoint: "DELETE /customization/v1.0/theme",
                    purpose: "reset theme to BlueOS defaults",
                },
                Evidence {
                    file: "core/frontend/src/store/customization.ts",
                    line: 154,
                    anchor: "await back_axios({ method: 'delete', url: `${API_URL}/theme`",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Customization),
                    endpoint: "GET /customization/v1.0/models",
                    purpose: "list uploaded 3D model overrides",
                },
                Evidence {
                    file: "core/frontend/src/store/customization.ts",
                    line: 167,
                    anchor: "const response = await back_axios({ method: 'get', url: `${A",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Customization),
                    endpoint: "POST /customization/v1.0/models",
                    purpose: "upload .glb model override file",
                },
                Evidence {
                    file: "core/frontend/src/store/customization.ts",
                    line: 182,
                    anchor: "url: `${API_URL}/models`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Customization),
                    endpoint: "DELETE /customization/v1.0/models/{name}",
                    purpose: "delete model override from list",
                },
                Evidence {
                    file: "core/frontend/src/store/customization.ts",
                    line: 203,
                    anchor: "url: `${API_URL}/models/${encoded_name}`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Customization),
                    endpoint: "GET /customization/v1.0/branding/logo",
                    purpose: "fetch custom project logo metadata and URL",
                },
                Evidence {
                    file: "core/frontend/src/store/customization.ts",
                    line: 215,
                    anchor: "const response = await back_axios({ method: 'get', url: `${A",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Customization),
                    endpoint: "POST /customization/v1.0/branding/logo",
                    purpose: "upload custom project logo image",
                },
                Evidence {
                    file: "core/frontend/src/store/customization.ts",
                    line: 230,
                    anchor: "url: `${API_URL}/branding/logo`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Customization),
                    endpoint: "DELETE /customization/v1.0/branding/logo",
                    purpose: "remove custom project logo",
                },
                Evidence {
                    file: "core/frontend/src/store/customization.ts",
                    line: 246,
                    anchor: "await back_axios({ method: 'delete', url: `${API_URL}/brandi",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Customization),
                    endpoint: "GET /customization/v1.0/branding/vehicle-image",
                    purpose: "fetch custom vehicle image metadata and URL",
                },
                Evidence {
                    file: "core/frontend/src/store/customization.ts",
                    line: 258,
                    anchor: "url: `${API_URL}/branding/vehicle-image`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Customization),
                    endpoint: "POST /customization/v1.0/branding/vehicle-image",
                    purpose: "upload custom vehicle image",
                },
                Evidence {
                    file: "core/frontend/src/store/customization.ts",
                    line: 275,
                    anchor: "url: `${API_URL}/branding/vehicle-image`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Customization),
                    endpoint: "DELETE /customization/v1.0/branding/vehicle-image",
                    purpose: "remove custom vehicle image",
                },
                Evidence {
                    file: "core/frontend/src/store/customization.ts",
                    line: 293,
                    anchor: "url: `${API_URL}/branding/vehicle-image`,",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "appearance and mode preferences",
                    store: "store/settings is_dark_theme / is_pirate_mode / is_dev_mode",
                    ownership: StateOwnership::Shared,
                    notes: "mutated via SettingsView switches and theme cards; persisted to bag",
                },
                "operator-facing theme and advanced-mode toggles are client-held and synced to bag_of_holding",
            ),
            Rationaled::new(
                ClientState {
                    name: "theme customization panel",
                    store: "ThemeCustomization.vue data expanded / theme_primary / model dialog",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "collapsible customization section; color picker before Apply",
                },
                "customization sub-panel keeps ephemeral editor state separate from saved theme",
            ),
            Rationaled::new(
                ClientState {
                    name: "customization assets cache",
                    store: "store/customization themeStatus / models / logo / vehicleImage",
                    ownership: StateOwnership::BackendOwned,
                    notes: "mirrored from customization service on refreshAll",
                },
                "branding and model lists originate from customization REST and are displayed with upload/remove actions",
            ),
            Rationaled::new(
                ClientState {
                    name: "log folder sizes and clear guards",
                    store: "SettingsView.vue log_size_bytes / disable_remove / log_folder_size chips",
                    ownership: StateOwnership::Shared,
                    notes: "100MB threshold enables Clear buttons and warning chip color",
                },
                "log size display and delete eligibility are client-derived from commander size endpoints",
            ),
            Rationaled::new(
                ClientState {
                    name: "log deletion streaming progress",
                    store: "SettingsView.vue deletion_in_progress / current_deletion_path / current_deletion_total_size",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "parses NDJSON fragments from remove_log_stream onDownloadProgress",
                },
                "service log deletion UI tracks per-file progress client-side during streamed response",
            ),
            Rationaled::new(
                ClientState {
                    name: "operation overlay and dialogs",
                    store: "SettingsView.vue operation_in_progress / show_reset_warning / show_log_clear_confirm",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "confirm dialogs gate destructive commander actions",
                },
                "modal and overlay state orchestrates reset and log-clear workflows",
            ),
            Rationaled::new(
                ClientState {
                    name: "pending log clear type",
                    store: "SettingsView.vue pending_log_clear_type",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "service vs mavlink branch for shared confirm dialog",
                },
                "single confirm dialog re-used for two log types via ephemeral enum",
            ),
        ]),
    };
