#[cfg(test)]
mod tests {
    use catalog_analysis::journey_matrix::PAGE_LOAD_UI;
    use catalog_data::capability_registry::frontend_capability_def;
    use catalog_harness::oracle::{derive_oracle_class, journey_has_frontend_actor};
    use catalog_harness::ui::ui_plan;
    use catalog_kernel::id::journey::JourneyId;
    use catalog_kernel::id::page::PageId;
    use catalog_kernel::provenance::{Grounded, GroundedItem, GroundedSet, Provenance};
    use catalog_kernel::version::Availability;
    use catalog_model::journey::{
        Actor, JourneyStep, OracleClass, Precondition, UseCase, Visibility, BLAST_RADIUS_UNKNOWN,
    };

    use crate::catalog::Catalog;

    const DOC: Provenance = Provenance::doc("test.md", 1, "");

    const TEST_PRESENCE: Availability = Availability {
        intro_commit: "0000000000000000000000000000000000000001",
        present_in_tags: &["1.0.0"],
        present_on_master: true,
        present_on_1_4_dev: true,
    };

    const fn empty_journey(
        preconditions: GroundedSet<Precondition>,
        steps: GroundedSet<JourneyStep>,
    ) -> UseCase {
        UseCase {
            id: JourneyId::ConnectToWifiNetwork,
            summary: Grounded::known("test", DOC),
            visibility: Grounded::known(Visibility::Default, DOC),
            services: GroundedSet::unknown("test"),
            capability_refs: GroundedSet::unknown("test"),
            preconditions,
            steps,
            availability: TEST_PRESENCE,
            blast_radius: BLAST_RADIUS_UNKNOWN,
            chains_from: None,
        }
    }

    #[test]
    fn derive_oracle_class_frontend_actor_is_client_orchestrated() {
        static STEPS: &[GroundedItem<JourneyStep>] = &[GroundedItem::new(
            JourneyStep {
                actor: Actor::Frontend(PageId::VehicleSetup),
                description: "run wizard",
                route: None,
                outcome: None,
            },
            DOC,
        )];
        let journey = empty_journey(GroundedSet::known(&[]), GroundedSet::known(STEPS));
        let catalog = Catalog::bootstrap();
        assert_eq!(
            derive_oracle_class(&catalog, &journey),
            OracleClass::ClientOrchestrated
        );
    }

    #[test]
    fn frontend_capability_refs_subset_of_frontend_actor_journeys() {
        let catalog = Catalog::bootstrap();
        for journey in catalog.journeys() {
            let has_frontend_cap = match &journey.capability_refs {
                GroundedSet::Known { items } => items
                    .iter()
                    .any(|item| frontend_capability_def(item.value).is_some()),
                GroundedSet::Unknown { .. } => false,
            };
            if has_frontend_cap {
                assert!(
                    journey_has_frontend_actor(journey),
                    "{:?} has frontend capability_refs but no Actor::Frontend step -- \
                     classify via page frontend_features, not capability_refs",
                    journey.id
                );
            }
        }
    }

    #[test]
    fn derive_oracle_class_page_frontend_features_without_actor() {
        let catalog = Catalog::bootstrap();
        let journey = catalog
            .journeys()
            .iter()
            .find(|j| {
                !journey_has_frontend_actor(j)
                    && derive_oracle_class(&catalog, j) == OracleClass::ClientOrchestrated
            })
            .expect("journey orchestrated via page frontend_features");
        assert!(!journey_has_frontend_actor(journey));
    }

    #[test]
    fn derive_oracle_class_client_composed_from_shared_state() {
        let catalog = Catalog::bootstrap();
        let journey = catalog
            .journeys()
            .iter()
            .find(|j| derive_oracle_class(&catalog, j) == OracleClass::ClientComposed)
            .expect("ClientComposed journey");
        assert_ne!(
            derive_oracle_class(&catalog, journey),
            OracleClass::ClientOrchestrated
        );
    }

    #[test]
    fn derive_oracle_class_http_passthrough_backend_owned() {
        let catalog = Catalog::bootstrap();
        let journey = catalog
            .journeys()
            .iter()
            .find(|j| derive_oracle_class(&catalog, j) == OracleClass::HttpPassthrough)
            .expect("HttpPassthrough journey");
        assert!(!journey_has_frontend_actor(journey));
    }

    #[test]
    fn oracle_class_inventory_counts() {
        let catalog = Catalog::bootstrap();
        let mut orchestrated = 0;
        let mut composed = 0;
        let mut passthrough = 0;
        let mut missing_ui_plan = Vec::new();
        for journey in catalog.journeys() {
            match derive_oracle_class(&catalog, journey) {
                OracleClass::ClientOrchestrated => {
                    orchestrated += 1;
                    let has_plan =
                        ui_plan(journey.id).is_some() || PAGE_LOAD_UI.contains(&journey.id);
                    if !has_plan {
                        missing_ui_plan.push(journey.id);
                    }
                }
                OracleClass::ClientComposed => composed += 1,
                OracleClass::HttpPassthrough => passthrough += 1,
            }
        }
        println!(
            "orchestrated={orchestrated} composed={composed} passthrough={passthrough} \
             missing_ui_plan={missing_ui_plan:?}"
        );
        assert!(orchestrated > 0);
        assert!(composed > 0);
        assert!(passthrough > 0);
    }
}
