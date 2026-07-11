use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub fn page() -> Page {
    Page {
        id: PageId::Disk,
        route: Observed::known(
            "/tools/disk".to_string(),
            Evidence {
                file: "core/frontend/src/router/index.ts".to_string(),
                line: 47,
            },
        ),
        name: Observed::known(
            "Disk".to_string(),
            Evidence {
                file: "core/frontend/src/router/index.ts".to_string(),
                line: 48,
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/Disk.vue".to_string(),
            Evidence {
                file: "core/frontend/src/router/index.ts".to_string(),
                line: 49,
            },
        ),
        menu_title: Observed::known(
            "Disk".to_string(),
            Evidence {
                file: "core/frontend/src/menus.ts".to_string(),
                line: 47,
            },
        ),
        advanced_only: Observed::known(
            true,
            Evidence {
                file: "core/frontend/src/menus.ts".to_string(),
                line: 50,
            },
        ),
        stores: ObservedSet::known(vec![
            Evidenced::new(
                "disk".to_string(),
                Evidence {
                    file: "core/frontend/src/views/Disk.vue".to_string(),
                    line: 240,
                },
            ),
            Evidenced::new(
                "settings".to_string(),
                Evidence {
                    file: "core/frontend/src/components/disk/DiskSpeedGraph.vue".to_string(),
                    line: 13,
                },
            ),
        ]),
        consumes: ObservedSet::known(vec![
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::DiskUsage),
                    endpoint: "GET /disk-usage/v1.0/disk/usage".to_string(),
                    purpose: "fetch du-backed usage tree for current path (depth, include_files, min_size_bytes query params)".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/store/disk.ts".to_string(),
                    line: 87,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::DiskUsage),
                    endpoint: "DELETE /disk-usage/v1.0/disk/paths/{target_path}".to_string(),
                    purpose: "delete selected files or folders after browser confirm dialog".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/store/disk.ts".to_string(),
                    line: 115,
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::DiskUsage),
                    endpoint: "GET /disk-usage/v1.0/disk/speed/stream".to_string(),
                    purpose: "stream multi-size disktest benchmark points for Speed Test tab".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/store/disk.ts".to_string(),
                    line: 192,
                },
            ),
        ]),
        frontend_features: AssertedSet::established(vec![]),
        client_state: AssertedSet::established(vec![
            Rationaled::new(
                ClientState {
                    name: "active tab".to_string(),
                    store: "Disk.vue component data".to_string(),
                    ownership: StateOwnership::FrontendOwned,
                    notes: "activeTab selects Disk Usage vs Speed Test tab".to_string(),
                },
                "tab selection is ephemeral UI state with no backend persistence",
            ),
            Rationaled::new(
                ClientState {
                    name: "directory navigation path".to_string(),
                    store: "Disk.vue component data".to_string(),
                    ownership: StateOwnership::Shared,
                    notes: "path updated by navigate/goUp; drives GET /disk/usage query".to_string(),
                },
                "current directory path is held client-side and sent to disk_usage on each drill-down",
            ),
            Rationaled::new(
                ClientState {
                    name: "usage query defaults".to_string(),
                    store: "Disk.vue component data".to_string(),
                    ownership: StateOwnership::FrontendOwned,
                    notes: "depth=2, includeFiles=true, minSizeKb=0 passed to fetchUsage".to_string(),
                },
                "fixed fetch parameters are component defaults not exposed in the UI",
            ),
            Rationaled::new(
                ClientState {
                    name: "delete selection".to_string(),
                    store: "Disk.vue component data".to_string(),
                    ownership: StateOwnership::FrontendOwned,
                    notes: "selectedPaths, allSelected/someSelected, toggleSelectAll".to_string(),
                },
                "checkbox selection for bulk delete is ephemeral until confirmDeleteSelected runs",
            ),
            Rationaled::new(
                ClientState {
                    name: "disk usage tree".to_string(),
                    store: "store/disk usage field".to_string(),
                    ownership: StateOwnership::BackendOwned,
                    notes: "DiskUsageResponse mirrored from GET /disk/usage".to_string(),
                },
                "usage tree is fetched from disk_usage and displayed without client mutation",
            ),
            Rationaled::new(
                ClientState {
                    name: "fetch and delete status flags".to_string(),
                    store: "store/disk loading, deleting, error".to_string(),
                    ownership: StateOwnership::Shared,
                    notes: "loading/deleting booleans and error strings around axios lifecycle".to_string(),
                },
                "request status flags are client-managed wrappers around backend calls",
            ),
            Rationaled::new(
                ClientState {
                    name: "speed test results".to_string(),
                    store: "store/disk speedResults".to_string(),
                    ownership: StateOwnership::BackendOwned,
                    notes: "DiskSpeedTestPoint[] accumulated from streaming NDJSON fragments".to_string(),
                },
                "benchmark points originate from disk_usage /speed/stream and are appended client-side as they arrive",
            ),
            Rationaled::new(
                ClientState {
                    name: "speed test progress".to_string(),
                    store: "store/disk speedTesting, speedTestProgress, speedError".to_string(),
                    ownership: StateOwnership::Shared,
                    notes: "in-flight flag, per-size progress text, and error message during stream parse".to_string(),
                },
                "speed test UI state tracks streaming download progress locally",
            ),
            Rationaled::new(
                ClientState {
                    name: "sorted usage children".to_string(),
                    store: "Disk.vue computed sortedChildren".to_string(),
                    ownership: StateOwnership::Shared,
                    notes: "usage.root.children sorted descending by size_bytes for table display".to_string(),
                },
                "table ordering is client-derived from the cached usage tree",
            ),
            Rationaled::new(
                ClientState {
                    name: "usage bar percentages".to_string(),
                    store: "Disk.vue method getPercentage".to_string(),
                    ownership: StateOwnership::Shared,
                    notes: "child size_bytes as percentage of parent root size_bytes".to_string(),
                },
                "progress bar widths are computed client-side from backend-reported sizes",
            ),
            Rationaled::new(
                ClientState {
                    name: "average speed summaries".to_string(),
                    store: "Disk.vue computed avgWriteSpeed / avgReadSpeed".to_string(),
                    ownership: StateOwnership::Shared,
                    notes: "mean of non-null write_speed/read_speed across speedResults".to_string(),
                },
                "headline MiB/s averages are client-computed from streamed benchmark points",
            ),
        ]),
    }
}
