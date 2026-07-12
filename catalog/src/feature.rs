use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

use schemars::JsonSchema;
use serde::Serialize;

use crate::capability::{capability_def, Aggregate, CAPABILITIES, FRONTEND_CAPABILITIES};
use crate::catalog::Catalog;
use crate::cluster::{greedy_modularity_communities, modularity_q_indices};
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::page::{ConsumeTarget, PageId};
use crate::provenance::{AssertedSet, GroundedSet, ObservedSet};
use crate::split::{clique_weights, consensus_over, group_by_key, modularity_partition};

#[derive(Debug, Clone, Copy, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct FeatureId(pub CapabilityId);

impl PartialEq for FeatureId {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

impl Eq for FeatureId {}

impl PartialOrd for FeatureId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FeatureId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.as_str().cmp(other.0.as_str())
    }
}

impl Hash for FeatureId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.as_str().hash(state);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    BackendService(ServiceId),
    FrontendPage(PageId),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Feature {
    pub id: FeatureId,
    pub aggregate: Aggregate,
    pub origin: Origin,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct FeatureCatalog {
    features: Vec<Feature>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct AggregateGroup {
    pub aggregate: Aggregate,
    pub features: Vec<FeatureId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct FeatureCommunity {
    pub members: Vec<FeatureId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct JourneyView {
    pub communities: Vec<FeatureCommunity>,
    pub modularity: f64,
    pub unreferenced_features: Vec<FeatureId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Divergence {
    pub split_by_journey: Vec<(FeatureId, FeatureId, String)>,
    pub joined_by_journey: Vec<(FeatureId, FeatureId, String)>,
}

/// One feature-grouping approach: a partition of all features by a single lens.
/// `modularity` is `Some` for clustering lenses, `None` for label lenses.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct FeatureLens {
    pub lens: &'static str,
    pub groups: Vec<Vec<FeatureId>>,
    pub modularity: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct FeaturePairAgreement {
    pub a: FeatureId,
    pub b: FeatureId,
    pub agree: usize,
    pub total: usize,
}

/// Cross-lens consensus over feature groupings. Input for M4, not a decision.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct FeatureSplitConsensus {
    pub lenses: Vec<FeatureLens>,
    pub pair_agreement: Vec<FeaturePairAgreement>,
    pub majority_threshold: usize,
    pub consensus_clusters: Vec<Vec<FeatureId>>,
}

impl FeatureCatalog {
    pub fn bootstrap() -> Self {
        Self::from_catalog(&Catalog::bootstrap())
    }

    pub fn from_catalog(catalog: &Catalog) -> Self {
        let mut features = Vec::new();
        for service in catalog.services() {
            if let AssertedSet::Established { items } = &service.definition.capabilities {
                for cap in items.iter() {
                    let id = FeatureId(cap.value);
                    let def = capability_def(cap.value)
                        .unwrap_or_else(|| panic!("unmapped capability: {}", cap.value.as_str()));
                    assert_eq!(
                        def.owner,
                        service.id,
                        "capability {} registry owner {:?} != declaring service {:?}",
                        cap.value.as_str(),
                        def.owner,
                        service.id
                    );
                    features.push(Feature {
                        id,
                        aggregate: def.aggregate,
                        origin: Origin::BackendService(service.id),
                        rationale: cap.rationale.to_string(),
                    });
                }
            }
        }
        for def in FRONTEND_CAPABILITIES {
            let page = catalog
                .pages()
                .iter()
                .find(|page| page.id == def.owner)
                .unwrap_or_else(|| {
                    panic!(
                        "frontend capability {} owner page {:?} not found in catalog",
                        def.id.as_str(),
                        def.owner
                    )
                });
            let AssertedSet::Established { items } = &page.frontend_features else {
                panic!(
                    "owner page {:?} has no established frontend_features for capability {}",
                    def.owner,
                    def.id.as_str()
                );
            };
            let rationaled = items
                .iter()
                .find(|item| item.value == def.id)
                .unwrap_or_else(|| {
                    panic!(
                        "owner page {:?} does not list frontend capability {}",
                        def.owner,
                        def.id.as_str()
                    )
                });
            features.push(Feature {
                id: FeatureId(def.id),
                aggregate: def.aggregate,
                origin: Origin::FrontendPage(def.owner),
                rationale: rationaled.rationale.to_string(),
            });
        }
        Self { features }
    }

    pub fn features(&self) -> &[Feature] {
        &self.features
    }

    pub fn aggregate_view(&self) -> Vec<AggregateGroup> {
        let mut by_aggregate: HashMap<Aggregate, Vec<FeatureId>> = HashMap::new();
        for feature in &self.features {
            by_aggregate
                .entry(feature.aggregate)
                .or_default()
                .push(feature.id);
        }
        let mut groups: Vec<AggregateGroup> = by_aggregate
            .into_iter()
            .map(|(aggregate, mut features)| {
                features.sort();
                AggregateGroup {
                    aggregate,
                    features,
                }
            })
            .collect();
        groups.sort_by(|left, right| left.aggregate.as_str().cmp(right.aggregate.as_str()));
        groups
    }

    pub fn journey_view(&self, catalog: &Catalog) -> JourneyView {
        let n = self.features.len();
        let index: HashMap<FeatureId, usize> = self
            .features
            .iter()
            .enumerate()
            .map(|(idx, feature)| (feature.id, idx))
            .collect();
        let mut weights = vec![vec![0.0; n]; n];
        let mut referenced = HashSet::new();

        for journey in catalog.journeys() {
            let journey_features = journey_feature_indices(journey, catalog, &index);
            for &idx in &journey_features {
                referenced.insert(idx);
            }
            for left in 0..journey_features.len() {
                for right in (left + 1)..journey_features.len() {
                    let a = journey_features[left];
                    let b = journey_features[right];
                    weights[a][b] += 1.0;
                    weights[b][a] += 1.0;
                }
            }
        }

        let communities_idx = greedy_modularity_communities(n, &weights);
        let modularity = modularity_q_indices(&weights, &communities_idx);
        let mut communities: Vec<FeatureCommunity> = communities_idx
            .into_iter()
            .map(|community| {
                let mut members: Vec<FeatureId> =
                    community.iter().map(|idx| self.features[*idx].id).collect();
                members.sort();
                FeatureCommunity { members }
            })
            .collect();
        sort_feature_communities(&mut communities);

        let mut unreferenced_features: Vec<FeatureId> = self
            .features
            .iter()
            .filter(|feature| !referenced.contains(index.get(&feature.id).expect("feature index")))
            .map(|feature| feature.id)
            .collect();
        unreferenced_features.sort();

        JourneyView {
            communities,
            modularity,
            unreferenced_features,
        }
    }

    pub fn view_divergence(&self, catalog: &Catalog) -> Divergence {
        let journey_view = self.journey_view(catalog);
        let feature_aggregate: HashMap<&str, Aggregate> = self
            .features
            .iter()
            .map(|feature| (feature.id.0.as_str(), feature.aggregate))
            .collect();
        let feature_community = feature_community_map(&journey_view.communities);
        let community_sizes: HashMap<usize, usize> = journey_view
            .communities
            .iter()
            .enumerate()
            .map(|(idx, community)| (idx, community.members.len()))
            .collect();
        let pair_journeys = journey_cooccurrence_pairs(catalog, &index_from_features(self));

        let mut joined_by_journey = Vec::new();
        for (left_id, right_id) in pair_journeys.keys() {
            let left_agg = feature_aggregate[left_id.0.as_str()];
            let right_agg = feature_aggregate[right_id.0.as_str()];
            if left_agg == right_agg {
                continue;
            }
            let left_comm = feature_community[left_id];
            let right_comm = feature_community[right_id];
            if left_comm != right_comm {
                continue;
            }
            if community_sizes[&left_comm] < 2 {
                continue;
            }
            let bridge = format_aggregate_bridge(left_agg.as_str(), right_agg.as_str());
            joined_by_journey.push((*left_id, *right_id, bridge));
        }
        joined_by_journey
            .sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
        joined_by_journey.dedup_by(|left, right| left.0 == right.0 && left.1 == right.1);

        let mut split_by_journey = Vec::new();
        for aggregate_group in self.aggregate_view() {
            for left in 0..aggregate_group.features.len() {
                for right in (left + 1)..aggregate_group.features.len() {
                    let left_id = aggregate_group.features[left];
                    let right_id = aggregate_group.features[right];
                    let left_comm = feature_community[&left_id];
                    let right_comm = feature_community[&right_id];
                    if left_comm == right_comm {
                        continue;
                    }
                    if community_sizes[&left_comm] < 2 || community_sizes[&right_comm] < 2 {
                        continue;
                    }
                    let (left_id, right_id) = ordered_pair(left_id, right_id);
                    split_by_journey.push((
                        left_id,
                        right_id,
                        aggregate_group.aggregate.as_str().to_string(),
                    ));
                }
            }
        }
        split_by_journey
            .sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));

        Divergence {
            split_by_journey,
            joined_by_journey,
        }
    }

    /// Group features through four independent lenses and reconcile them by consensus:
    /// `aggregate` (entity the feature acts on), `origin` (owning service/page today),
    /// `journey_cooccurrence` (features used together in a workflow), and
    /// `page_reachability` (features surfaced together through one frontend page).
    pub fn split_consensus(&self, catalog: &Catalog) -> FeatureSplitConsensus {
        let n = self.features.len();
        let ids: Vec<FeatureId> = self.features.iter().map(|feature| feature.id).collect();
        let index: HashMap<FeatureId, usize> =
            ids.iter().enumerate().map(|(idx, id)| (*id, idx)).collect();

        let aggregate_keys: Vec<&str> = self
            .features
            .iter()
            .map(|feature| feature.aggregate.as_str())
            .collect();
        let aggregate_groups = group_by_key(&aggregate_keys);

        let origin_keys: Vec<String> = self
            .features
            .iter()
            .map(|feature| origin_key(&feature.origin))
            .collect();
        let origin_groups = group_by_key(&origin_keys);

        let journey_weights = self.journey_feature_weights(catalog, &index);
        let (journey_groups, journey_q) = modularity_partition(n, &journey_weights);

        let page_weights = self.page_feature_weights(catalog, &index);
        let (page_groups, page_q) = modularity_partition(n, &page_weights);

        let partitions = vec![
            aggregate_groups.clone(),
            origin_groups.clone(),
            journey_groups.clone(),
            page_groups.clone(),
        ];
        let consensus = consensus_over(&partitions, n);

        let lenses = vec![
            FeatureLens {
                lens: "aggregate",
                groups: map_feature_groups(&aggregate_groups, &ids),
                modularity: None,
            },
            FeatureLens {
                lens: "origin",
                groups: map_feature_groups(&origin_groups, &ids),
                modularity: None,
            },
            FeatureLens {
                lens: "journey_cooccurrence",
                groups: map_feature_groups(&journey_groups, &ids),
                modularity: Some(journey_q),
            },
            FeatureLens {
                lens: "page_reachability",
                groups: map_feature_groups(&page_groups, &ids),
                modularity: Some(page_q),
            },
        ];

        let pair_agreement = consensus
            .pair_agreement
            .iter()
            .map(|(i, j, agree)| FeaturePairAgreement {
                a: ids[*i],
                b: ids[*j],
                agree: *agree,
                total: consensus.total,
            })
            .collect();
        let consensus_clusters = map_feature_groups(&consensus.clusters, &ids);

        FeatureSplitConsensus {
            lenses,
            pair_agreement,
            majority_threshold: consensus.majority_threshold,
            consensus_clusters,
        }
    }

    fn journey_feature_weights(
        &self,
        catalog: &Catalog,
        index: &HashMap<FeatureId, usize>,
    ) -> Vec<Vec<f64>> {
        let sets: Vec<Vec<usize>> = catalog
            .journeys()
            .iter()
            .map(|journey| journey_feature_indices(journey, catalog, index))
            .filter(|set| set.len() >= 2)
            .collect();
        clique_weights(self.features.len(), &sets)
    }

    fn page_feature_weights(
        &self,
        catalog: &Catalog,
        index: &HashMap<FeatureId, usize>,
    ) -> Vec<Vec<f64>> {
        let mut service_features: HashMap<ServiceId, Vec<usize>> = HashMap::new();
        for (idx, feature) in self.features.iter().enumerate() {
            if let Origin::BackendService(service) = feature.origin {
                service_features.entry(service).or_default().push(idx);
            }
        }

        let mut sets: Vec<Vec<usize>> = Vec::new();
        for page in catalog.pages() {
            let mut members: Vec<usize> = Vec::new();
            if let AssertedSet::Established { items } = &page.frontend_features {
                for item in items.iter() {
                    if let Some(&idx) = index.get(&FeatureId(item.value)) {
                        if !members.contains(&idx) {
                            members.push(idx);
                        }
                    }
                }
            }
            if let ObservedSet::Known { items } = &page.consumes {
                for item in items.iter() {
                    if let ConsumeTarget::Service(service) = item.value.service {
                        if let Some(feature_idxs) = service_features.get(&service) {
                            for &idx in feature_idxs {
                                if !members.contains(&idx) {
                                    members.push(idx);
                                }
                            }
                        }
                    }
                }
            }
            if members.len() >= 2 {
                sets.push(members);
            }
        }
        clique_weights(self.features.len(), &sets)
    }

    pub fn validate(&self, catalog: &Catalog) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let known_services: HashSet<&ServiceId> =
            catalog.services().iter().map(|s| &s.id).collect();
        let known_pages: HashSet<PageId> = catalog.pages().iter().map(|page| page.id).collect();
        let distinct_capabilities = distinct_capability_count(catalog);
        let expected_count = distinct_capabilities + FRONTEND_CAPABILITIES.len();

        let mut seen_ids = HashSet::new();
        for feature in &self.features {
            match feature.origin {
                Origin::BackendService(service) => {
                    if !known_services.contains(&service) {
                        errors.push(format!(
                            "feature {} references unknown backend origin service {}",
                            feature.id.0.as_str(),
                            service.as_str()
                        ));
                    }
                }
                Origin::FrontendPage(page) => {
                    if !known_pages.contains(&page) {
                        errors.push(format!(
                            "feature {} references unknown frontend origin page {}",
                            feature.id.0.as_str(),
                            page.as_str()
                        ));
                    }
                }
            }
            if !seen_ids.insert(feature.id) {
                errors.push(format!("duplicate feature id {}", feature.id.0.as_str()));
            }
        }

        if self.features.len() != expected_count {
            errors.push(format!(
                "feature count {} does not match expected count {expected_count} \
                 ({distinct_capabilities} backend + {} frontend)",
                self.features.len(),
                FRONTEND_CAPABILITIES.len()
            ));
        }

        for page in catalog.pages() {
            if let AssertedSet::Established { items } = &page.frontend_features {
                for item in items.iter() {
                    if FRONTEND_CAPABILITIES.iter().all(|def| def.id != item.value) {
                        errors.push(format!(
                            "page frontend feature {} is not registered in FRONTEND_CAPABILITIES",
                            item.value.as_str()
                        ));
                    }
                }
            }
        }

        for def in FRONTEND_CAPABILITIES {
            if CAPABILITIES.iter().any(|cap| cap.id == def.id) {
                errors.push(format!(
                    "frontend capability {} collides with backend registry",
                    def.id.as_str()
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn distinct_capability_count(catalog: &Catalog) -> usize {
    let mut seen = HashSet::new();
    for service in catalog.services() {
        if let AssertedSet::Established { items } = &service.definition.capabilities {
            for item in items.iter() {
                seen.insert(&item.value);
            }
        }
    }
    seen.len()
}

fn index_from_features(catalog: &FeatureCatalog) -> HashMap<FeatureId, usize> {
    catalog
        .features
        .iter()
        .enumerate()
        .map(|(idx, feature)| (feature.id, idx))
        .collect()
}

fn sort_feature_communities(communities: &mut [FeatureCommunity]) {
    for community in communities.iter_mut() {
        community.members.sort();
    }
    communities.sort_by(|left, right| {
        right.members.len().cmp(&left.members.len()).then_with(|| {
            left.members
                .first()
                .map(|id| id.0.as_str())
                .cmp(&right.members.first().map(|id| id.0.as_str()))
        })
    });
}

fn feature_community_map(communities: &[FeatureCommunity]) -> HashMap<FeatureId, usize> {
    let mut map = HashMap::new();
    for (idx, community) in communities.iter().enumerate() {
        for member in &community.members {
            map.insert(*member, idx);
        }
    }
    map
}

fn journey_feature_indices(
    journey: &crate::journey::UserJourney,
    catalog: &Catalog,
    index: &HashMap<FeatureId, usize>,
) -> Vec<usize> {
    let mut features = Vec::new();
    if let GroundedSet::Known { items } = &journey.capability_refs {
        for item in items.iter() {
            let id = FeatureId(item.value);
            if let Some(&idx) = index.get(&id) {
                features.push(idx);
            }
        }
    }
    if let Some(chain_id) = &journey.chains_from {
        if let Some(parent) = catalog
            .journeys()
            .iter()
            .find(|candidate| &candidate.id == chain_id)
        {
            features.extend(journey_feature_indices(parent, catalog, index));
        }
    }
    features.sort_unstable();
    features.dedup();
    features
}

fn journey_cooccurrence_pairs(
    catalog: &Catalog,
    feature_index: &HashMap<FeatureId, usize>,
) -> HashMap<(FeatureId, FeatureId), Vec<JourneyId>> {
    let known: HashSet<FeatureId> = feature_index.keys().copied().collect();
    let reverse_index: HashMap<usize, FeatureId> =
        feature_index.iter().map(|(id, idx)| (*idx, *id)).collect();
    let forward_index: HashMap<FeatureId, usize> =
        feature_index.iter().map(|(id, idx)| (*id, *idx)).collect();
    let mut pairs: HashMap<(FeatureId, FeatureId), Vec<JourneyId>> = HashMap::new();

    for journey in catalog.journeys() {
        let journey_features: Vec<FeatureId> =
            journey_feature_indices(journey, catalog, &forward_index)
                .into_iter()
                .filter_map(|idx| reverse_index.get(&idx).copied())
                .filter(|id| known.contains(id))
                .collect();
        for left in 0..journey_features.len() {
            for right in (left + 1)..journey_features.len() {
                let (pair_left, pair_right) =
                    ordered_pair(journey_features[left], journey_features[right]);
                pairs
                    .entry((pair_left, pair_right))
                    .or_default()
                    .push(journey.id);
            }
        }
    }

    for journeys in pairs.values_mut() {
        journeys.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        journeys.dedup();
    }
    pairs
}

fn origin_key(origin: &Origin) -> String {
    match origin {
        Origin::BackendService(service) => format!("service:{}", service.as_str()),
        Origin::FrontendPage(page) => format!("page:{}", page.as_str()),
    }
}

fn map_feature_groups(groups: &[Vec<usize>], ids: &[FeatureId]) -> Vec<Vec<FeatureId>> {
    let mut out: Vec<Vec<FeatureId>> = groups
        .iter()
        .map(|group| {
            let mut members: Vec<FeatureId> = group.iter().map(|&idx| ids[idx]).collect();
            members.sort();
            members
        })
        .collect();
    out.sort_by(|left, right| {
        right.len().cmp(&left.len()).then_with(|| {
            left.first()
                .map(|id| id.0.as_str())
                .cmp(&right.first().map(|id| id.0.as_str()))
        })
    });
    out
}

fn ordered_pair(left: FeatureId, right: FeatureId) -> (FeatureId, FeatureId) {
    if left <= right {
        (left, right)
    } else {
        (right, left)
    }
}

fn format_aggregate_bridge(left: &str, right: &str) -> String {
    if left <= right {
        format!("{left} + {right}")
    } else {
        format!("{right} + {left}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::FRONTEND_CAPABILITIES;
    use crate::id::CapabilityId;
    use crate::provenance::AssertedSet;

    const EXPECTED_FEATURE_COUNT: usize = 143;
    const EXPECTED_AGGREGATE_COUNT: usize = 19;

    #[test]
    fn from_catalog_builds_expected_features() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        assert_eq!(features.features().len(), EXPECTED_FEATURE_COUNT);
    }

    #[test]
    fn origin_split_backend_and_frontend() {
        let features = FeatureCatalog::bootstrap();
        let backend = features
            .features()
            .iter()
            .filter(|f| matches!(f.origin, Origin::BackendService(_)))
            .count();
        let frontend = features
            .features()
            .iter()
            .filter(|f| matches!(f.origin, Origin::FrontendPage(_)))
            .count();
        assert_eq!(frontend, FRONTEND_CAPABILITIES.len());
        assert_eq!(backend + frontend, EXPECTED_FEATURE_COUNT);
        assert_eq!(frontend, 14);
    }

    #[test]
    fn aggregate_view_returns_expected_aggregates() {
        let features = FeatureCatalog::bootstrap();
        let view = features.aggregate_view();
        assert_eq!(view.len(), EXPECTED_AGGREGATE_COUNT);
        let total: usize = view.iter().map(|group| group.features.len()).sum();
        assert_eq!(total, EXPECTED_FEATURE_COUNT);
    }

    #[test]
    fn validate_passes_on_bootstrap() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        assert!(features.validate(&catalog).is_ok());
    }

    #[test]
    fn round_trips_through_serde_json() {
        let features = FeatureCatalog::bootstrap();
        let json = serde_json::to_string(&features).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value.is_object());
    }

    #[test]
    fn every_capability_becomes_exactly_one_feature() {
        let catalog = Catalog::bootstrap();
        let mut distinct = HashSet::new();
        for service in catalog.services() {
            if let AssertedSet::Established { items } = &service.definition.capabilities {
                for item in items.iter() {
                    distinct.insert(item.value);
                }
            }
        }
        let features = FeatureCatalog::from_catalog(&catalog);
        assert_eq!(
            features.features().len(),
            distinct.len() + FRONTEND_CAPABILITIES.len()
        );
        assert!(features.validate(&catalog).is_ok());
    }

    #[test]
    fn journey_view_is_deterministic() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        let first = features.journey_view(&catalog);
        let second = features.journey_view(&catalog);
        assert_eq!(first, second);
    }

    #[test]
    fn journey_view_co_referenced_capabilities_share_community() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        let view = features.journey_view(&catalog);
        let flash = FeatureId(CapabilityId::FlashFirmware);
        let detect = FeatureId(CapabilityId::DetectFlightControllers);
        let flash_community = view
            .communities
            .iter()
            .find(|community| community.members.contains(&flash))
            .expect("flash_firmware community");
        assert!(flash_community.members.contains(&detect));
    }

    #[test]
    fn journey_view_unreferenced_features_are_absent_from_journeys() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        let view = features.journey_view(&catalog);
        assert!(!view.unreferenced_features.is_empty());

        let mut referenced = HashSet::new();
        for journey in catalog.journeys() {
            let GroundedSet::Known { items } = &journey.capability_refs else {
                continue;
            };
            for item in items.iter() {
                referenced.insert(item.value.as_str());
            }
        }
        for id in &view.unreferenced_features {
            assert!(!referenced.contains(id.0.as_str()));
        }
    }

    #[test]
    fn view_divergence_finds_cross_aggregate_joins() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        let divergence = features.view_divergence(&catalog);
        assert!(!divergence.joined_by_journey.is_empty());
    }

    #[test]
    fn feature_split_lenses_each_partition_all_features() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        let split = features.split_consensus(&catalog);
        assert_eq!(split.lenses.len(), 4);
        assert_eq!(split.majority_threshold, 3);
        for lens in &split.lenses {
            let mut seen = HashSet::new();
            let mut total = 0;
            for group in &lens.groups {
                for id in group {
                    assert!(seen.insert(*id), "duplicate feature in lens {}", lens.lens);
                    total += 1;
                }
            }
            assert_eq!(
                total, EXPECTED_FEATURE_COUNT,
                "lens {} missing features",
                lens.lens
            );
        }
    }

    #[test]
    fn feature_split_consensus_is_deterministic() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        assert_eq!(
            features.split_consensus(&catalog),
            features.split_consensus(&catalog)
        );
    }

    #[test]
    fn feature_split_consensus_agreement_bounds_and_partition() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        let split = features.split_consensus(&catalog);
        for pair in &split.pair_agreement {
            assert!(pair.agree >= 1 && pair.agree <= pair.total);
            assert_eq!(pair.total, 4);
        }
        let total: usize = split.consensus_clusters.iter().map(|c| c.len()).sum();
        assert_eq!(total, EXPECTED_FEATURE_COUNT);
    }

    #[test]
    fn journey_view_and_divergence_round_trip_serde() {
        let catalog = Catalog::bootstrap();
        let features = FeatureCatalog::from_catalog(&catalog);
        let view = features.journey_view(&catalog);
        let divergence = features.view_divergence(&catalog);

        let view_json = serde_json::to_string(&view).unwrap();
        let view_value: serde_json::Value = serde_json::from_str(&view_json).unwrap();
        assert!(view_value.is_object());

        let divergence_json = serde_json::to_string(&divergence).unwrap();
        let divergence_value: serde_json::Value = serde_json::from_str(&divergence_json).unwrap();
        assert!(divergence_value.is_object());
    }
}
