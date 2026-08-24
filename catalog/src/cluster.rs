use catalog_analysis::cluster::{
    CatalogClustering, ClusterPolicy, ClusterResult, LensPartition, SplitConsensus, StabilityReport,
};

use crate::catalog::Catalog;

impl CatalogClustering for Catalog {
    fn coupling_matrix(&self, policy: ClusterPolicy) -> catalog_core::catalog::CouplingMatrix {
        self.as_ref().coupling_matrix(policy)
    }

    fn cluster(&self, policy: ClusterPolicy) -> ClusterResult {
        self.as_ref().cluster(policy)
    }

    fn cluster_stability(
        &self,
        policy: ClusterPolicy,
        runs: usize,
        jitter: f64,
    ) -> StabilityReport {
        self.as_ref().cluster_stability(policy, runs, jitter)
    }

    fn boundary_proposals(&self) -> Vec<ClusterResult> {
        self.as_ref().boundary_proposals()
    }

    fn page_cooccurrence_matrix(&self) -> catalog_core::catalog::CouplingMatrix {
        self.as_ref().page_cooccurrence_matrix()
    }

    fn journey_cooccurrence_matrix(&self) -> catalog_core::catalog::CouplingMatrix {
        self.as_ref().journey_cooccurrence_matrix()
    }

    fn split_lenses(&self) -> Vec<LensPartition> {
        self.as_ref().split_lenses()
    }

    fn split_consensus(&self) -> SplitConsensus {
        self.as_ref().split_consensus()
    }
}
