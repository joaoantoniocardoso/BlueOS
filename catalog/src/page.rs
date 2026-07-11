use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::id::{CapabilityId, ServiceId};
use crate::provenance::{AssertedSet, Observed, ObservedSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct PageId(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Page {
    pub id: PageId,
    pub route: Observed<String>,
    pub name: Observed<String>,
    pub component: Observed<String>,
    pub menu_title: Observed<String>,
    pub advanced_only: Observed<bool>,
    pub stores: ObservedSet<String>,
    pub consumes: ObservedSet<PageServiceCall>,
    pub frontend_features: AssertedSet<CapabilityId>,
    pub client_state: AssertedSet<ClientState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PageServiceCall {
    pub service: ServiceId,
    pub endpoint: String,
    pub purpose: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ClientState {
    pub name: String,
    pub store: String,
    pub ownership: StateOwnership,
    pub notes: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
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
            id: PageId("vehicle_setup".to_string()),
            route: Observed::known(
                "/vehicle/setup/:tab?/:subtab?".to_string(),
                Evidence {
                    file: "core/frontend/src/router/index.ts".to_string(),
                    line: 42,
                },
            ),
            name: Observed::known(
                "Vehicle Setup".to_string(),
                Evidence {
                    file: "core/frontend/src/router/index.ts".to_string(),
                    line: 43,
                },
            ),
            component: Observed::known(
                "core/frontend/src/views/VehicleSetupView.vue".to_string(),
                Evidence {
                    file: "core/frontend/src/router/index.ts".to_string(),
                    line: 44,
                },
            ),
            menu_title: Observed::known(
                "Vehicle Setup".to_string(),
                Evidence {
                    file: "core/frontend/src/menus.ts".to_string(),
                    line: 10,
                },
            ),
            advanced_only: Observed::known(
                false,
                Evidence {
                    file: "core/frontend/src/menus.ts".to_string(),
                    line: 11,
                },
            ),
            stores: ObservedSet::known(vec![Evidenced::new(
                "calibration".to_string(),
                Evidence {
                    file: "core/frontend/src/views/VehicleSetupView.vue".to_string(),
                    line: 5,
                },
            )]),
            consumes: ObservedSet::known(vec![Evidenced::new(
                PageServiceCall {
                    service: ServiceId("mavlink2rest".to_string()),
                    endpoint: "mavlink2rest MAV_CMD_PREFLIGHT_CALIBRATION".to_string(),
                    purpose: "calibrate accelerometer".to_string(),
                },
                Evidence {
                    file: "core/frontend/src/views/VehicleSetupView.vue".to_string(),
                    line: 80,
                },
            )]),
            frontend_features: AssertedSet::established(vec![Rationaled::new(
                CapabilityId("calibrate_accelerometer".to_string()),
                "client-side calibration wizard with no dedicated backend capability",
            )]),
            client_state: AssertedSet::established(vec![Rationaled::new(
                ClientState {
                    name: "calibration progress".to_string(),
                    store: "calibration.ts Calibrator singleton".to_string(),
                    ownership: StateOwnership::FrontendOwned,
                    notes: "1.x anti-pattern; 2.0 should re-home".to_string(),
                },
                "wizard tracks step progress locally",
            )]),
        }
    }

    #[test]
    fn page_round_trips_through_serde_json() {
        let page = sample_page();
        let json = serde_json::to_string(&page).expect("serialize page");
        let restored: Page = serde_json::from_str(&json).expect("deserialize page");
        assert_eq!(page, restored);
    }
}
