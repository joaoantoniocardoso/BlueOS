//! Journey-first grouping: cluster the 94 user journeys through several independent
//! lenses and reconcile them by cross-lens consensus. This is the journey analogue of
//! the service split in [`crate::cluster`] and the feature split in [`crate::feature`].
//! Output is an M4 input, not an architectural decision.

use std::collections::{BTreeMap, HashMap, HashSet};

use schemars::JsonSchema;
use serde::Serialize;

use crate::capability::{capability_def, frontend_capability_def, Aggregate};
use crate::catalog::Catalog;
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::provenance::GroundedSet;
use crate::split::{
    clique_weights, components, consensus_over, group_by_key, modularity_partition,
};

/// One journey-grouping approach: a partition of all journeys by a single lens.
/// `modularity` is `Some` for clustering lenses, `None` for label/structural lenses.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct JourneyLens {
    pub lens: &'static str,
    pub groups: Vec<Vec<JourneyId>>,
    pub modularity: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct JourneyPairAgreement {
    pub a: JourneyId,
    pub b: JourneyId,
    pub agree: usize,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct JourneySplitConsensus {
    pub lenses: Vec<JourneyLens>,
    pub pair_agreement: Vec<JourneyPairAgreement>,
    pub majority_threshold: usize,
    pub consensus_clusters: Vec<Vec<JourneyId>>,
}

impl Catalog {
    /// Group journeys through four independent lenses and reconcile by consensus:
    /// `shared_service` (journeys touching the same services), `shared_capability`
    /// (journeys referencing the same capabilities), `chain` (journeys linked by
    /// `chains_from`), and `dominant_aggregate` (the entity a journey mostly acts on).
    pub fn journey_split_consensus(&self) -> JourneySplitConsensus {
        let ids: Vec<JourneyId> = self.journeys().iter().map(|journey| journey.id).collect();
        let n = ids.len();
        let index: HashMap<JourneyId, usize> =
            ids.iter().enumerate().map(|(idx, id)| (*id, idx)).collect();

        let service_weights = self.shared_service_weights(n);
        let (service_groups, service_q) = modularity_partition(n, &service_weights);

        let capability_weights = self.shared_capability_weights(n);
        let (capability_groups, capability_q) = modularity_partition(n, &capability_weights);

        let chain_groups = self.chain_components(n, &index);

        let aggregate_keys: Vec<String> =
            self.journeys().iter().map(dominant_aggregate_key).collect();
        let aggregate_groups = group_by_key(&aggregate_keys);

        let partitions = vec![
            service_groups.clone(),
            capability_groups.clone(),
            chain_groups.clone(),
            aggregate_groups.clone(),
        ];
        let consensus = consensus_over(&partitions, n);

        let lenses = vec![
            JourneyLens {
                lens: "shared_service",
                groups: map_journey_groups(&service_groups, &ids),
                modularity: Some(service_q),
            },
            JourneyLens {
                lens: "shared_capability",
                groups: map_journey_groups(&capability_groups, &ids),
                modularity: Some(capability_q),
            },
            JourneyLens {
                lens: "chain",
                groups: map_journey_groups(&chain_groups, &ids),
                modularity: None,
            },
            JourneyLens {
                lens: "dominant_aggregate",
                groups: map_journey_groups(&aggregate_groups, &ids),
                modularity: None,
            },
        ];

        let pair_agreement = consensus
            .pair_agreement
            .iter()
            .map(|(i, j, agree)| JourneyPairAgreement {
                a: ids[*i],
                b: ids[*j],
                agree: *agree,
                total: consensus.total,
            })
            .collect();
        let consensus_clusters = map_journey_groups(&consensus.clusters, &ids);

        JourneySplitConsensus {
            lenses,
            pair_agreement,
            majority_threshold: consensus.majority_threshold,
            consensus_clusters,
        }
    }

    fn shared_service_weights(&self, n: usize) -> Vec<Vec<f64>> {
        let mut by_service: HashMap<ServiceId, Vec<usize>> = HashMap::new();
        for (idx, journey) in self.journeys().iter().enumerate() {
            for service in journey_services(journey) {
                by_service.entry(service).or_default().push(idx);
            }
        }
        let sets: Vec<Vec<usize>> = by_service.into_values().collect();
        clique_weights(n, &sets)
    }

    fn shared_capability_weights(&self, n: usize) -> Vec<Vec<f64>> {
        let mut by_capability: HashMap<CapabilityId, Vec<usize>> = HashMap::new();
        for (idx, journey) in self.journeys().iter().enumerate() {
            if let GroundedSet::Known { items } = &journey.capability_refs {
                for item in items.iter() {
                    by_capability.entry(item.value).or_default().push(idx);
                }
            }
        }
        let sets: Vec<Vec<usize>> = by_capability.into_values().collect();
        clique_weights(n, &sets)
    }

    fn chain_components(&self, n: usize, index: &HashMap<JourneyId, usize>) -> Vec<Vec<usize>> {
        let mut adjacency = vec![vec![0usize; n]; n];
        for (idx, journey) in self.journeys().iter().enumerate() {
            if let Some(parent) = &journey.chains_from {
                if let Some(&parent_idx) = index.get(parent) {
                    adjacency[idx][parent_idx] = 1;
                    adjacency[parent_idx][idx] = 1;
                }
            }
        }
        components(n, &adjacency, 1)
    }
}

/// Participating services of a journey: the declared `services` set augmented with the
/// services targeted by any grounded step route.
fn journey_services(journey: &crate::journey::UserJourney) -> Vec<ServiceId> {
    let mut seen: HashSet<ServiceId> = HashSet::new();
    let mut out: Vec<ServiceId> = Vec::new();
    if let GroundedSet::Known { items } = &journey.services {
        for item in items.iter() {
            if seen.insert(item.value) {
                out.push(item.value);
            }
        }
    }
    if let GroundedSet::Known { items } = &journey.steps {
        for step in items.iter() {
            if let Some(crate::provenance::Grounded::Known { value, .. }) = &step.value.route {
                if seen.insert(value.service) {
                    out.push(value.service);
                }
            }
        }
    }
    out
}

fn dominant_aggregate_key(journey: &crate::journey::UserJourney) -> String {
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    if let GroundedSet::Known { items } = &journey.capability_refs {
        for item in items.iter() {
            if let Some(aggregate) = aggregate_of(item.value) {
                *counts.entry(aggregate.as_str()).or_default() += 1;
            }
        }
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(left.0)))
        .map(|(name, _)| name.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn aggregate_of(capability: CapabilityId) -> Option<Aggregate> {
    capability_def(capability)
        .map(|def| def.aggregate)
        .or_else(|| frontend_capability_def(capability).map(|def| def.aggregate))
}

fn map_journey_groups(groups: &[Vec<usize>], ids: &[JourneyId]) -> Vec<Vec<JourneyId>> {
    let mut out: Vec<Vec<JourneyId>> = groups
        .iter()
        .map(|group| {
            let mut members: Vec<JourneyId> = group.iter().map(|&idx| ids[idx]).collect();
            members.sort_by(|left, right| left.as_str().cmp(right.as_str()));
            members
        })
        .collect();
    out.sort_by(|left, right| {
        right.len().cmp(&left.len()).then_with(|| {
            left.first()
                .map(|id| id.as_str())
                .cmp(&right.first().map(|id| id.as_str()))
        })
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn journey_count() -> usize {
        Catalog::bootstrap().journeys().len()
    }

    #[test]
    fn journey_split_lenses_each_partition_all_journeys() {
        let catalog = Catalog::bootstrap();
        let n = journey_count();
        let split = catalog.journey_split_consensus();
        assert_eq!(split.lenses.len(), 4);
        assert_eq!(split.majority_threshold, 3);
        for lens in &split.lenses {
            let mut seen = HashSet::new();
            let mut total = 0;
            for group in &lens.groups {
                for id in group {
                    assert!(
                        seen.insert(id.as_str()),
                        "duplicate journey in lens {}",
                        lens.lens
                    );
                    total += 1;
                }
            }
            assert_eq!(total, n, "lens {} missing journeys", lens.lens);
        }
    }

    #[test]
    fn journey_split_consensus_is_deterministic() {
        let catalog = Catalog::bootstrap();
        assert_eq!(
            catalog.journey_split_consensus(),
            catalog.journey_split_consensus()
        );
    }

    #[test]
    fn journey_split_consensus_agreement_bounds_and_partition() {
        let catalog = Catalog::bootstrap();
        let n = journey_count();
        let split = catalog.journey_split_consensus();
        for pair in &split.pair_agreement {
            assert!(pair.agree >= 1 && pair.agree <= pair.total);
            assert_eq!(pair.total, 4);
        }
        let total: usize = split.consensus_clusters.iter().map(|c| c.len()).sum();
        assert_eq!(total, n);
    }

    #[test]
    fn journey_split_round_trips_through_serde_json() {
        let catalog = Catalog::bootstrap();
        let split = catalog.journey_split_consensus();
        let json = serde_json::to_string(&split).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value.is_object());
    }
}
