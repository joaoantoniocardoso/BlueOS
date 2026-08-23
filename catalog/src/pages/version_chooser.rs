use crate::id::ServiceId;
use crate::page::{ClientState, ConsumeTarget, Page, PageId, PageServiceCall, StateOwnership};
use crate::provenance::{AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled};

pub const PAGE: Page =
    Page {
        id: PageId::VersionChooser,
        route: Observed::known(
            "/tools/version-chooser",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 62,
                anchor: "path: '/tools/version-chooser',",
            },
        ),
        name: Observed::known(
            "Version Chooser",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 63,
                anchor: "name: 'Version Chooser',",
            },
        ),
        component: Observed::known(
            "core/frontend/src/views/VersionChooser.vue",
            Evidence {
                file: "core/frontend/src/router/index.ts",
                line: 64,
                anchor: "component: defineAsyncComponent(() => import('../views/Versi",
            },
        ),
        menu_title: Observed::known(
            "BlueOS Version",
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 32,
                anchor: "title: 'BlueOS Version',",
            },
        ),
        advanced_only: Observed::known(
            false,
            Evidence {
                file: "core/frontend/src/menus.ts",
                line: 35,
                anchor: "advanced: false,",
            },
        ),
        stores: ObservedSet::known(&[
            Evidenced::new(
                "helper",
                Evidence {
                    file: "core/frontend/src/components/version-chooser/VersionChooser.vue",
                    line: 251,
                    anchor: "import helper from '@/store/helper'",
                },
            ),
            Evidenced::new(
                "settings",
                Evidence {
                    file: "core/frontend/src/components/version-chooser/VersionChooser.vue",
                    line: 250,
                    anchor: "import settings from '@/libs/settings'",
                },
            ),
        ]),
        consumes: ObservedSet::known(&[
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Versionchooser),
                    endpoint: "GET /version-chooser/v1.0/version/current/",
                    purpose: "load active BlueOS image on mount and during backend restart polling",
                },
                Evidence {
                    file: "core/frontend/src/utils/version_chooser.ts",
                    line: 157,
                    anchor: "url: `${API_URL}/version/current/`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Versionchooser),
                    endpoint: "GET /version-chooser/v1.0/bootstrap/current/",
                    purpose: "display bootstrap version alongside local images",
                },
                Evidence {
                    file: "core/frontend/src/utils/version_chooser.ts",
                    line: 164,
                    anchor: "url: `${API_URL}/bootstrap/current/`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Versionchooser),
                    endpoint: "GET /version-chooser/v1.0/version/available/local",
                    purpose: "list locally stored BlueOS docker images",
                },
                Evidence {
                    file: "core/frontend/src/utils/version_chooser.ts",
                    line: 135,
                    anchor: "url: `${API_URL}/version/available/local`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Versionchooser),
                    endpoint: "GET /version-chooser/v1.0/version/available/{repository}",
                    purpose: "list remote registry tags when internet is available",
                },
                Evidence {
                    file: "core/frontend/src/utils/version_chooser.ts",
                    line: 147,
                    anchor: "url: `${API_URL}/version/available/${remote_image_name}`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Versionchooser),
                    endpoint: "POST /version-chooser/v1.0/version/load",
                    purpose: "upload local .tar docker image to onboard storage",
                },
                Evidence {
                    file: "core/frontend/src/components/version-chooser/VersionChooser.vue",
                    line: 521,
                    anchor: "url: '/version-chooser/v1.0/version/load',",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Versionchooser),
                    endpoint: "POST /version-chooser/v1.0/version/pull/",
                    purpose: "stream docker pull output when fetching a remote image",
                },
                Evidence {
                    file: "core/frontend/src/components/version-chooser/VersionChooser.vue",
                    line: 573,
                    anchor: "url: '/version-chooser/v1.0/version/pull/',",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Versionchooser),
                    endpoint: "POST /version-chooser/v1.0/version/current",
                    purpose: "activate a selected local image (triggers stack restart)",
                },
                Evidence {
                    file: "core/frontend/src/components/version-chooser/VersionChooser.vue",
                    line: 644,
                    anchor: "url: '/version-chooser/v1.0/version/current',",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Versionchooser),
                    endpoint: "DELETE /version-chooser/v1.0/version/delete",
                    purpose: "remove a local docker image tag",
                },
                Evidence {
                    file: "core/frontend/src/components/version-chooser/VersionChooser.vue",
                    line: 660,
                    anchor: "url: '/version-chooser/v1.0/version/delete',",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Versionchooser),
                    endpoint: "POST /version-chooser/v1.0/bootstrap/current",
                    purpose: "update bootstrap image tag after pull",
                },
                Evidence {
                    file: "core/frontend/src/components/version-chooser/VersionChooser.vue",
                    line: 617,
                    anchor: "url: '/version-chooser/v1.0/bootstrap/current',",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Versionchooser),
                    endpoint: "POST /version-chooser/v1.0/docker/login/",
                    purpose: "register docker registry credentials from DockerLogin dialog",
                },
                Evidence {
                    file: "core/frontend/src/utils/version_chooser.ts",
                    line: 180,
                    anchor: "url: `${API_URL}/docker/login/`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Versionchooser),
                    endpoint: "POST /version-chooser/v1.0/docker/logout/",
                    purpose: "remove docker registry credentials from DockerLogin dialog",
                },
                Evidence {
                    file: "core/frontend/src/utils/version_chooser.ts",
                    line: 188,
                    anchor: "url: `${API_URL}/docker/logout/`,",
                },
            ),
            Evidenced::new(
                PageServiceCall {
                    service: ConsumeTarget::Service(ServiceId::Versionchooser),
                    endpoint: "GET /version-chooser/v1.0/docker/accounts/",
                    purpose: "list saved docker registry accounts in DockerLogin dialog",
                },
                Evidence {
                    file: "core/frontend/src/utils/version_chooser.ts",
                    line: 196,
                    anchor: "url: `${API_URL}/docker/accounts/`,",
                },
            ),
        ]),
        frontend_features: AssertedSet::established(&[]),
        client_state: AssertedSet::established(&[
            Rationaled::new(
                ClientState {
                    name: "local and remote version catalogs",
                    store: "VersionChooser.vue data local_versions / available_versions",
                    ownership: StateOwnership::Shared,
                    notes: "sorted/filtered lists from versionchooser with client-side semver ordering",
                },
                "version lists are cached from backend then sorted and paginated in the browser",
            ),
            Rationaled::new(
                ClientState {
                    name: "current and bootstrap version selection",
                    store: "VersionChooser.vue data current_version / bootstrap_version / selected_image",
                    ownership: StateOwnership::Shared,
                    notes: "active image and repository picker drive available-version queries",
                },
                "selected repository and current tag are held client-side to filter remote catalog fetches",
            ),
            Rationaled::new(
                ClientState {
                    name: "pull and upload progress",
                    store: "VersionChooser.vue data pull_output / download_percentage / upload_percentage",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "PullTracker digests streamed docker pull/load progress text",
                },
                "install progress UI is ephemeral client state around versionchooser streaming responses",
            ),
            Rationaled::new(
                ClientState {
                    name: "latest stable and beta hints",
                    store: "VersionChooser.vue computed newStableAvailable / newBetaAvailable",
                    ownership: StateOwnership::Shared,
                    notes: "derived from remote catalog via VCU.getLatestStable/getLatestBeta",
                },
                "update availability chips are client-computed semver comparisons per docs release-type rules",
            ),
            Rationaled::new(
                ClientState {
                    name: "remote versions pagination",
                    store: "VersionChooser.vue computed paginatedComponents / page",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "10 remote images per page",
                },
                "pagination is pure UI state over the cached remote image list",
            ),
            Rationaled::new(
                ClientState {
                    name: "backend restart wait flag",
                    store: "VersionChooser.vue data waiting",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "set during waitForBackendToRestart after version switch",
                },
                "restart polling state blocks UI until versionchooser answers /version/current again",
            ),
            Rationaled::new(
                ClientState {
                    name: "docker login dialog state",
                    store: "DockerLogin.vue data log_in_info / accounts / op_loading",
                    ownership: StateOwnership::FrontendOwned,
                    notes: "credentials form and connected-account list",
                },
                "registry login UX state is held locally until posted to versionchooser",
            ),
        ]),
    };
