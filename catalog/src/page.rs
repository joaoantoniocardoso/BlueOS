use std::fmt;

use schemars::JsonSchema;
use serde::Serialize;

use crate::id::{CapabilityId, ServiceId};
use crate::provenance::{AssertedSet, Observed, ObservedSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, JsonSchema)]
pub enum PageId {
    #[serde(rename = "disk")]
    Disk,
    #[serde(rename = "vehicle_setup")]
    VehicleSetup,
    #[serde(rename = "video_manager")]
    VideoManager,
}

impl PageId {
    pub const ALL: [PageId; 3] = [PageId::Disk, PageId::VehicleSetup, PageId::VideoManager];

    pub const fn as_str(&self) -> &'static str {
        match self {
            PageId::Disk => "disk",
            PageId::VehicleSetup => "vehicle_setup",
            PageId::VideoManager => "video_manager",
        }
    }

    pub fn from_str_id(s: &str) -> Option<PageId> {
        PageId::ALL.into_iter().find(|v| v.as_str() == s)
    }
}

impl fmt::Display for PageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl crate::id::Entity for PageId {
    const ALL: &'static [PageId] = &PageId::ALL;
    fn as_str(&self) -> &'static str {
        PageId::as_str(self)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct Page {
    pub id: PageId,
    pub route: Observed<&'static str>,
    pub name: Observed<&'static str>,
    pub component: Observed<&'static str>,
    pub menu_title: Observed<&'static str>,
    pub advanced_only: Observed<bool>,
    pub stores: ObservedSet<&'static str>,
    pub consumes: ObservedSet<PageServiceCall>,
    pub frontend_features: AssertedSet<CapabilityId>,
    pub client_state: AssertedSet<ClientState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct PageServiceCall {
    pub service: ConsumeTarget,
    pub endpoint: &'static str,
    pub purpose: &'static str,
}

/// What a page consumes: a cataloged service, or the public internet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConsumeTarget {
    Service(ServiceId),
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct ClientState {
    pub name: &'static str,
    pub store: &'static str,
    pub ownership: StateOwnership,
    pub notes: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StateOwnership {
    BackendOwned,
    FrontendOwned,
    Shared,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provenance::{Evidence, Evidenced, Rationaled};

    fn sample_page() -> Page {
        Page {
            id: PageId::VehicleSetup,
            route: Observed::known(
                "/vehicle/setup/:tab?/:subtab?",
                Evidence {
                    file: "core/frontend/src/router/index.ts",
                    line: 42,
                },
            ),
            name: Observed::known(
                "Vehicle Setup",
                Evidence {
                    file: "core/frontend/src/router/index.ts",
                    line: 43,
                },
            ),
            component: Observed::known(
                "core/frontend/src/views/VehicleSetupView.vue",
                Evidence {
                    file: "core/frontend/src/router/index.ts",
                    line: 44,
                },
            ),
            menu_title: Observed::known(
                "Vehicle Setup",
                Evidence {
                    file: "core/frontend/src/menus.ts",
                    line: 10,
                },
            ),
            advanced_only: Observed::known(
                false,
                Evidence {
                    file: "core/frontend/src/menus.ts",
                    line: 11,
                },
            ),
            stores: ObservedSet::known(
                const {
                    &[Evidenced::new(
                        "calibration",
                        Evidence {
                            file: "core/frontend/src/views/VehicleSetupView.vue",
                            line: 5,
                        },
                    )]
                },
            ),
            consumes: ObservedSet::known(
                const {
                    &[Evidenced::new(
                        PageServiceCall {
                            service: ConsumeTarget::Service(ServiceId::Mavlink2rest),
                            endpoint: "mavlink2rest MAV_CMD_PREFLIGHT_CALIBRATION",
                            purpose: "calibrate accelerometer",
                        },
                        Evidence {
                            file: "core/frontend/src/views/VehicleSetupView.vue",
                            line: 80,
                        },
                    )]
                },
            ),
            frontend_features: AssertedSet::established(
                const {
                    &[Rationaled::new(
                        CapabilityId::CalibrateAccelerometer,
                        "client-side calibration wizard with no dedicated backend capability",
                    )]
                },
            ),
            client_state: AssertedSet::established(
                const {
                    &[Rationaled::new(
                        ClientState {
                            name: "calibration progress",
                            store: "calibration.ts Calibrator singleton",
                            ownership: StateOwnership::FrontendOwned,
                            notes: "1.x anti-pattern; 2.0 should re-home",
                        },
                        "wizard tracks step progress locally",
                    )]
                },
            ),
        }
    }

    #[test]
    fn page_round_trips_through_serde_json() {
        let page = sample_page();
        let json = serde_json::to_string(&page).expect("serialize page");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert!(value.is_object());
    }
}
