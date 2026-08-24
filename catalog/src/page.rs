#[cfg(test)]
mod tests {
    use catalog_kernel::id::capability::CapabilityId;
    use catalog_kernel::id::page::PageId;
    use catalog_kernel::id::service::ServiceId;
    use catalog_kernel::provenance::{
        AssertedSet, Evidence, Evidenced, Observed, ObservedSet, Rationaled,
    };
    use catalog_model::page::{ClientState, ConsumeTarget, Page, PageServiceCall, StateOwnership};

    fn sample_page() -> Page {
        Page {
            id: PageId::VehicleSetup,
            route: Observed::known(
                "/vehicle/setup/:tab?/:subtab?",
                Evidence {
                    file: "core/frontend/src/router/index.ts",
                    line: 42,
                    anchor: "path: '/tools/feature-provenance',",
                },
            ),
            name: Observed::known(
                "Vehicle Setup",
                Evidence {
                    file: "core/frontend/src/router/index.ts",
                    line: 43,
                    anchor: "name: 'Feature Provenance',",
                },
            ),
            component: Observed::known(
                "core/frontend/src/views/VehicleSetupView.vue",
                Evidence {
                    file: "core/frontend/src/router/index.ts",
                    line: 44,
                    anchor: "component: defineAsyncComponent(() => import('../views/Featu",
                },
            ),
            menu_title: Observed::known(
                "Vehicle Setup",
                Evidence {
                    file: "core/frontend/src/menus.ts",
                    line: 10,
                    anchor: "{",
                },
            ),
            advanced_only: Observed::known(
                false,
                Evidence {
                    file: "core/frontend/src/menus.ts",
                    line: 11,
                    anchor: "title: 'Autopilot Parameters',",
                },
            ),
            stores: ObservedSet::known(
                const {
                    &[Evidenced::new(
                        "calibration",
                        Evidence {
                            file: "core/frontend/src/views/VehicleSetupView.vue",
                            line: 5,
                            anchor: "centered",
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
                            anchor: "mounted() {",
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
