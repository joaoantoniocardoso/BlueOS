//! Feature presence helpers (tag membership maps).
//!
//! Presence data lives in [`crate::journey_presence`] (generated from git).
//! Regenerate with:
//! `cargo run -p blueos-catalog --bin generate_feature_presence`
//!
//! GitHub provenance is first-class in [`crate::feature_trace`]:
//! `cargo run -p blueos-catalog --bin enrich_feature_traces`

use crate::journey_presence::ALL_JOURNEY_PRESENCE;
use crate::version::{feature_present_on, journeys_present_on, Availability};

/// Feature map for one DUT tag / channel tip: journey id → present?
pub fn feature_map_for_version(dut_tag: &str) -> Vec<(&'static str, bool)> {
    ALL_JOURNEY_PRESENCE
        .iter()
        .map(|(id, avail)| (*id, feature_present_on(dut_tag, avail)))
        .collect()
}

/// Journey ids present on `dut_tag` (exact tag membership or channel tip).
pub fn journeys_for_version(dut_tag: &str) -> Vec<&'static str> {
    journeys_present_on(dut_tag, ALL_JOURNEY_PRESENCE)
}

/// Look up generated presence for a journey id string (e.g. `"inspect_zenoh_network"`).
pub fn presence_for_journey(journey_id: &str) -> Option<Availability> {
    ALL_JOURNEY_PRESENCE
        .iter()
        .find(|(id, _)| *id == journey_id)
        .map(|(_, avail)| *avail)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::JourneyId;

    #[test]
    fn zenoh_inspector_present_on_backport_and_master_not_1_4_dev() {
        let avail = presence_for_journey(JourneyId::InspectZenohNetwork.as_str()).expect("seeded");
        assert!(avail.present_on_master);
        assert!(!avail.present_on_1_4_dev);
        assert!(avail.present_in_tags.contains(&"1.5.0-beta.2"));
        assert!(!avail.present_in_tags.contains(&"1.4.4-beta.16"));
        assert!(feature_present_on("1.5.0-beta.2", &avail));
        assert!(!feature_present_on("1.4.4-beta.16", &avail));
        assert!(!feature_present_on("1.4-dev", &avail));
    }

    #[test]
    fn level_horizon_present_on_1_4_dev_without_local_only_tags() {
        let avail = presence_for_journey(JourneyId::LevelHorizon.as_str()).expect("seeded");
        assert!(avail.present_on_1_4_dev);
        assert!(avail.present_on_master);
        assert!(avail.present_in_tags.contains(&"1.4.4-beta.10"));
        assert!(!avail.present_in_tags.iter().any(|t| t.contains("beta.100")));
        assert!(feature_present_on("1.4-dev", &avail));
    }

    #[test]
    fn version_feature_maps_differ() {
        let on_14 = journeys_for_version("1.4-dev");
        let on_master = journeys_for_version("master");
        assert!(on_14.len() < on_master.len());
        assert!(!on_14.contains(&JourneyId::InspectZenohNetwork.as_str()));
        assert!(on_master.contains(&JourneyId::InspectZenohNetwork.as_str()));
    }
}
