use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::Records,
        route: Observed::known(
            "/tools/records",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 67,
                anchor: "path: '/vehicle/video-manager',",
            },
        ),
        name: Observed::known(
            "Records",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 68,
                anchor: "name: 'Video Manager',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/RecordsView.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 69,
                anchor: "component: defineAsyncComponent(() => import('../views/Video",
            },
        ),
        menu_title: Observed::known(
            "Records",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 135,
                anchor: "title: 'Video Streams',",
            },
        ),
        advanced_only: Observed::known(
            false,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 138,
                anchor: "advanced: false,",
            },
        ),
        stores: ObservedSet::known(&[Evidenced::new(
            "records",
            Evidence {
                file: "core/frontend/src/views/RecordsView.vue",
                line: 207,
                anchor: "import records_store from '@/store/records'",
            },
        )]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::RecorderExtractor),
                    endpoint: "GET /recorder-extractor/v1.0/recorder/files",
                    purpose: "list available MP4 recordings with thumbnail, stream, and download URLs",
                },
                Evidence {
                    file: "core/frontend/src/store/records.ts",
                    line: 47,
                    anchor: "url: `${this.API_URL}/files`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::RecorderExtractor),
                    endpoint: "DELETE /recorder-extractor/v1.0/recorder/files/{target_path}",
                    purpose: "delete a recording after user clicks delete on card",
                },
                Evidence {
                    file: "core/frontend/src/store/records.ts",
                    line: 69,
                    anchor: "url: `${this.API_URL}/files/${file.path}`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::RecorderExtractor),
                    endpoint: "GET /recorder-extractor/v1.0/recorder/status",
                    purpose: "poll in-flight video extraction jobs (5s interval via OneMoreTime)",
                },
                Evidence {
                    file: "core/frontend/src/store/records.ts",
                    line: 84,
                    anchor: "url: `${this.API_URL}/status`,",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "recordings list",
                    store: "store/records recordings",
                    ownership: StateOwnership::BackendOwned,
                    notes: "RecordingFile[] mirrored from GET /recorder/files",
                },
                "recording metadata originates from recorder_extractor and is displayed without client mutation",
            ),
            Rationaled::new(
                ClientState {
                    name: "processing files list",
                    store: "store/records processing_files",
                    ownership: StateOwnership::BackendOwned,
                    notes: "ProcessingFile[] from GET /recorder/status processing field",
                },
                "in-flight extraction jobs are mirrored from recorder_extractor status polling",
            ),
            Rationaled::new(
                ClientState {
                    name: "fetch status flags",
                    store: "store/records loading, error",
                    ownership: StateOwnership::Shared,
                    notes: "loading boolean and error string around axios lifecycle",
                },
                "request status flags are client-managed wrappers around backend calls",
            ),
            Rationaled::new(
                ClientState {
                    name: "video player dialog",
                    store: "RecordsView.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "playerOpen, activeRecord; openPlayer/closePlayer with HTMLVideoElement pause/reset",
                },
                "inline MP4 preview dialog is ephemeral UI state with no backend persistence",
            ),
            Rationaled::new(
                ClientState {
                    name: "thumbnail load tracking",
                    store: "RecordsView.vue component data",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "brokenThumbnails, loadingThumbnails; onThumbnailError/onThumbnailLoad/isThumbnailLoading",
                },
                "per-card thumbnail loading and broken-image fallback are client-side presentation state",
            ),
            Rationaled::new(
                ClientState {
                    name: "processing status poller",
                    store: "RecordsView.vue statusPoller OneMoreTime",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "5s interval re-fetches status and recordings when processing_files non-empty",
                },
                "polling cadence and conditional refresh logic live only in the browser",
            ),
            Rationaled::new(
                ClientState {
                    name: "formatted size and date",
                    store: "RecordsView.vue methods formatSize/formatDate",
                    ownership: StateOwnership::Shared,
                    notes: "prettifySize on size_bytes; toLocaleString on modified timestamp",
                },
                "display formatting is client-derived from backend-reported file metadata",
            ),
        ]),
    };
