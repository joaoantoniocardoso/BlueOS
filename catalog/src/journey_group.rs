use catalog_analysis::journey_group::{CatalogJourneyGrouping, JourneySplitConsensus};

use crate::catalog::Catalog;

impl CatalogJourneyGrouping for Catalog {
    fn journey_split_consensus(&self) -> JourneySplitConsensus {
        self.as_ref().journey_split_consensus()
    }
}
