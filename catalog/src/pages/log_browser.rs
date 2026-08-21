use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::LogBrowser,
        route: Observed::known(
            "/vehicle/logs",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 32,
                anchor: "path: '/vehicle/logs',",
            },
        ),
        name: Observed::known(
            "Log Browser",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 33,
                anchor: "name: 'Log Browser',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/LogView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 34,
                anchor: "component: defineAsyncComponent(() => import('../views/LogVi",
            },
        ),
        menu_title: Observed::known(
            "Log Browser",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 54,
                anchor: "title: 'Disk',",
            },
        ),
        advanced_only: Observed::known(
            false,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 57,
                anchor: "advanced: true,",
            },
        ),
        stores: ObservedSet::known(&[]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Filebrowser),
                    endpoint: "POST /file-browser/api/login",
                    purpose: "authenticate filebrowser singleton before folder listing",
                },
                Evidence {
                    file: "core/frontend/src/libs/filebrowser.ts",
                    line: 22,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Filebrowser),
                    endpoint: "GET /file-browser/api/resources{folder_path}",
                    purpose: "list .bin and .tlog files under /ardupilot_logs/firmware/logs/ and /ardupilot_logs/logs/",
                },
                Evidence {
                    file: "core/frontend/src/libs/filebrowser.ts",
                    line: 55,
                    anchor: "return back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Filebrowser),
                    endpoint: "DELETE /file-browser/api/resources{file_path}",
                    purpose: "delete selected telemetry or binary logs after confirm",
                },
                Evidence {
                    file: "core/frontend/src/libs/filebrowser.ts",
                    line: 111,
                    anchor: "await back_axios({",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Filebrowser),
                    endpoint: "GET /file-browser/api/raw{file_path}",
                    purpose: "download single log or zip of selected logs via window.open",
                },
                Evidence {
                    file: "core/frontend/src/libs/filebrowser.ts",
                    line: 161,
                    anchor: "url = await this.singleFileRelativeURL(files[0])",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "log table selection",
                    store: "LogManager.vue selected_logs",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "v-data-table show-select for batch download/delete",
                },
                "checkbox selection for bulk operations is ephemeral until action completes",
            ),
            Rationaled::new(
                ClientState {
                    name: "table sort preferences",
                    store: "LogManager.vue sortBy, sortDesc",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "defaults sortBy=modified sortDesc=true",
                },
                "column sort state is local UI preference",
            ),
            Rationaled::new(
                ClientState {
                    name: "download and delete status",
                    store: "LogManager.vue downloading, deleting, logs_fetched",
                    ownership: StateOwnership::Shared,
                    notes: "booleans around filebrowser downloadFiles/deleteFiles lifecycle",
                },
                "operation flags track in-flight filebrowser requests client-side",
            ),
            Rationaled::new(
                ClientState {
                    name: "filtered log file list",
                    store: "LogManager.vue available_logs",
                    ownership: StateOwnership::Shared,
                    notes: "merged folder listings filtered to .bin/.tlog with size > 100 bytes",
                },
                "log list is fetched from filebrowser then filtered client-side by extension and size",
            ),
            Rationaled::new(
                ClientState {
                    name: "formatted modified timestamps",
                    store: "LogManager.vue computed parsed_logs",
                    ownership: StateOwnership::Shared,
                    notes: "date-fns format applied to log.modified for table display",
                },
                "display timestamps are client-formatted from backend file metadata",
            ),
            Rationaled::new(
                ClientState {
                    name: "human-readable file sizes",
                    store: "LogManager.vue method printSize",
                    ownership: StateOwnership::Shared,
                    notes: "prettifySize on size_bytes / 1024",
                },
                "size column converts raw byte counts for display",
            ),
        ]),
    };
