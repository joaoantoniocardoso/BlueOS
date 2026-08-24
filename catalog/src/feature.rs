use std::collections::{HashMap, HashSet};

use schemars::JsonSchema;
use serde::Serialize;

use crate::capability::{capability_def, Aggregate, CAPABILITIES, FRONTEND_CAPABILITIES};
use crate::catalog::Catalog;
use crate::cluster::{greedy_modularity_communities, modularity_q_indices};
use crate::id::{CapabilityId, JourneyId, ServiceId};
use crate::page::{ConsumeTarget, PageId};
use crate::provenance::{AssertedSet, GroundedSet, ObservedSet};
use crate::split::{clique_weights, consensus_over, group_by_key, modularity_partition};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    BackendService(ServiceId),
    FrontendPage(PageId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredCapability {
    pub id: CapabilityId,
    pub aggregate: Aggregate,
    pub origin: Origin,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct AggregateGroup {
    pub aggregate: Aggregate,
    pub capabilities: Vec<CapabilityId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct CapabilityCommunity {
    pub members: Vec<CapabilityId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct CapabilityJourneyView {
    pub communities: Vec<CapabilityCommunity>,
    pub modularity: f64,
    pub unreferenced_capabilities: Vec<CapabilityId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct Divergence {
    pub split_by_journey: Vec<(CapabilityId, CapabilityId, String)>,
    pub joined_by_journey: Vec<(CapabilityId, CapabilityId, String)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct CapabilityLens {
    pub lens: &'static str,
    pub groups: Vec<Vec<CapabilityId>>,
    pub modularity: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct CapabilityPairAgreement {
    pub a: CapabilityId,
    pub b: CapabilityId,
    pub agree: usize,
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct CapabilitySplitConsensus {
    pub lenses: Vec<CapabilityLens>,
    pub pair_agreement: Vec<CapabilityPairAgreement>,
    pub majority_threshold: usize,
    pub consensus_clusters: Vec<Vec<CapabilityId>>,
}

pub fn declared_capabilities(catalog: &Catalog) -> Vec<DeclaredCapability> {
    let mut capabilities = Vec::new();
    for service in catalog.services() {
        if let AssertedSet::Established { items } = &service.definition.capabilities {
            for cap in items.iter() {
                let id = cap.value;
                let def = capability_def(id)
                    .unwrap_or_else(|| panic!("unmapped capability: {}", id.as_str()));
                assert_eq!(
                    def.owner,
                    service.id,
                    "capability {} registry owner {:?} != declaring service {:?}",
                    id.as_str(),
                    def.owner,
                    service.id
                );
                capabilities.push(DeclaredCapability {
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
            )
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
        capabilities.push(DeclaredCapability {
            id: def.id,
            aggregate: def.aggregate,
            origin: Origin::FrontendPage(def.owner),
            rationale: rationaled.rationale.to_string(),
        });
    }
    capabilities
}

pub fn capability_aggregate_view(catalog: &Catalog) -> Vec<AggregateGroup> {
    let capabilities = declared_capabilities(catalog);
    let mut by_aggregate: HashMap<Aggregate, Vec<CapabilityId>> = HashMap::new();
    for capability in &capabilities {
        by_aggregate
            .entry(capability.aggregate)
            .or_default()
            .push(capability.id);
    }
    let mut groups: Vec<AggregateGroup> = by_aggregate
        .into_iter()
        .map(|(aggregate, mut capabilities)| {
            capabilities.sort_by_key(|id| id.as_str());
            AggregateGroup {
                aggregate,
                capabilities,
            }
        })
        .collect();
    groups.sort_by(|left, right| left.aggregate.as_str().cmp(right.aggregate.as_str()));
    groups
}

pub fn capability_journey_view(catalog: &Catalog) -> CapabilityJourneyView {
    let capabilities = declared_capabilities(catalog);
    let n = capabilities.len();
    let index: HashMap<CapabilityId, usize> = capabilities
        .iter()
        .enumerate()
        .map(|(idx, capability)| (capability.id, idx))
        .collect();
    let mut weights = vec![vec![0.0; n]; n];
    let mut referenced = HashSet::new();

    for journey in catalog.journeys() {
        let journey_capabilities = journey_capability_indices(journey, catalog, &index);
        for &idx in &journey_capabilities {
            referenced.insert(idx);
        }
        for left in 0..journey_capabilities.len() {
            for right in (left + 1)..journey_capabilities.len() {
                let a = journey_capabilities[left];
                let b = journey_capabilities[right];
                weights[a][b] += 1.0;
                weights[b][a] += 1.0;
            }
        }
    }

    let communities_idx = greedy_modularity_communities(n, &weights);
    let modularity = modularity_q_indices(&weights, &communities_idx);
    let mut communities: Vec<CapabilityCommunity> = communities_idx
        .into_iter()
        .map(|community| {
            let mut members: Vec<CapabilityId> =
                community.iter().map(|idx| capabilities[*idx].id).collect();
            members.sort_by_key(|id| id.as_str());
            CapabilityCommunity { members }
        })
        .collect();
    sort_capability_communities(&mut communities);

    let mut unreferenced_capabilities: Vec<CapabilityId> = capabilities
        .iter()
        .filter(|capability| !referenced.contains(index.get(&capability.id).expect("cap index")))
        .map(|capability| capability.id)
        .collect();
    unreferenced_capabilities.sort_by_key(|id| id.as_str());

    CapabilityJourneyView {
        communities,
        modularity,
        unreferenced_capabilities,
    }
}

pub fn capability_view_divergence(catalog: &Catalog) -> Divergence {
    let journey_view = capability_journey_view(catalog);
    let capabilities = declared_capabilities(catalog);
    let capability_aggregate: HashMap<&str, Aggregate> = capabilities
        .iter()
        .map(|capability| (capability.id.as_str(), capability.aggregate))
        .collect();
    let capability_community = capability_community_map(&journey_view.communities);
    let community_sizes: HashMap<usize, usize> = journey_view
        .communities
        .iter()
        .enumerate()
        .map(|(idx, community)| (idx, community.members.len()))
        .collect();
    let index = capability_index(&capabilities);
    let pair_journeys = journey_cooccurrence_pairs(catalog, &index);

    let mut joined_by_journey = Vec::new();
    for (left_id, right_id) in pair_journeys.keys() {
        let left_agg = capability_aggregate[left_id.as_str()];
        let right_agg = capability_aggregate[right_id.as_str()];
        if left_agg == right_agg {
            continue;
        }
        let left_comm = capability_community[left_id];
        let right_comm = capability_community[right_id];
        if left_comm != right_comm {
            continue;
        }
        if community_sizes[&left_comm] < 2 {
            continue;
        }
        let bridge = format_aggregate_bridge(left_agg.as_str(), right_agg.as_str());
        joined_by_journey.push((*left_id, *right_id, bridge));
    }
    joined_by_journey.sort_by(|left, right| {
        left.0
            .as_str()
            .cmp(right.0.as_str())
            .then_with(|| left.1.as_str().cmp(right.1.as_str()))
    });
    joined_by_journey.dedup_by(|left, right| left.0 == right.0 && left.1 == right.1);

    let mut split_by_journey = Vec::new();
    for aggregate_group in capability_aggregate_view(catalog) {
        for left in 0..aggregate_group.capabilities.len() {
            for right in (left + 1)..aggregate_group.capabilities.len() {
                let left_id = aggregate_group.capabilities[left];
                let right_id = aggregate_group.capabilities[right];
                let left_comm = capability_community[&left_id];
                let right_comm = capability_community[&right_id];
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
    split_by_journey.sort_by(|left, right| {
        left.0
            .as_str()
            .cmp(right.0.as_str())
            .then_with(|| left.1.as_str().cmp(right.1.as_str()))
    });

    Divergence {
        split_by_journey,
        joined_by_journey,
    }
}

pub fn capability_split_consensus(catalog: &Catalog) -> CapabilitySplitConsensus {
    let capabilities = declared_capabilities(catalog);
    let n = capabilities.len();
    let ids: Vec<CapabilityId> = capabilities
        .iter()
        .map(|capability| capability.id)
        .collect();
    let index: HashMap<CapabilityId, usize> =
        ids.iter().enumerate().map(|(idx, id)| (*id, idx)).collect();

    let aggregate_keys: Vec<&str> = capabilities
        .iter()
        .map(|capability| capability.aggregate.as_str())
        .collect();
    let aggregate_groups = group_by_key(&aggregate_keys);

    let origin_keys: Vec<String> = capabilities
        .iter()
        .map(|capability| origin_key(&capability.origin))
        .collect();
    let origin_groups = group_by_key(&origin_keys);

    let journey_weights = journey_capability_weights(catalog, &capabilities, &index);
    let (journey_groups, journey_q) = modularity_partition(n, &journey_weights);

    let page_weights = page_capability_weights(catalog, &capabilities, &index);
    let (page_groups, page_q) = modularity_partition(n, &page_weights);

    let partitions = vec![
        aggregate_groups.clone(),
        origin_groups.clone(),
        journey_groups.clone(),
        page_groups.clone(),
    ];
    let consensus = consensus_over(&partitions, n);

    let lenses = vec![
        CapabilityLens {
            lens: "aggregate",
            groups: map_capability_groups(&aggregate_groups, &ids),
            modularity: None,
        },
        CapabilityLens {
            lens: "origin",
            groups: map_capability_groups(&origin_groups, &ids),
            modularity: None,
        },
        CapabilityLens {
            lens: "journey_cooccurrence",
            groups: map_capability_groups(&journey_groups, &ids),
            modularity: Some(journey_q),
        },
        CapabilityLens {
            lens: "page_reachability",
            groups: map_capability_groups(&page_groups, &ids),
            modularity: Some(page_q),
        },
    ];

    let pair_agreement = consensus
        .pair_agreement
        .iter()
        .map(|(i, j, agree)| CapabilityPairAgreement {
            a: ids[*i],
            b: ids[*j],
            agree: *agree,
            total: consensus.total,
        })
        .collect();
    let consensus_clusters = map_capability_groups(&consensus.clusters, &ids);

    CapabilitySplitConsensus {
        lenses,
        pair_agreement,
        majority_threshold: consensus.majority_threshold,
        consensus_clusters,
    }
}

pub fn validate_capabilities(catalog: &Catalog) -> Result<(), Vec<String>> {
    let capabilities = declared_capabilities(catalog);
    let mut errors = Vec::new();
    let known_services: HashSet<&ServiceId> = catalog.services().iter().map(|s| &s.id).collect();
    let known_pages: HashSet<PageId> = catalog.pages().iter().map(|page| page.id).collect();
    let distinct_capabilities = distinct_capability_count(catalog);
    let expected_count = distinct_capabilities + FRONTEND_CAPABILITIES.len();

    let mut seen_ids = HashSet::new();
    for capability in &capabilities {
        match capability.origin {
            Origin::BackendService(service) => {
                if !known_services.contains(&service) {
                    errors.push(format!(
                        "capability {} references unknown backend origin service {}",
                        capability.id.as_str(),
                        service.as_str()
                    ));
                }
            }
            Origin::FrontendPage(page) => {
                if !known_pages.contains(&page) {
                    errors.push(format!(
                        "capability {} references unknown frontend origin page {}",
                        capability.id.as_str(),
                        page.as_str()
                    ));
                }
            }
        }
        if !seen_ids.insert(capability.id) {
            errors.push(format!(
                "duplicate capability id {}",
                capability.id.as_str()
            ));
        }
    }

    if capabilities.len() != expected_count {
        errors.push(format!(
            "capability count {} does not match expected count {expected_count} \
             ({distinct_capabilities} backend + {} frontend)",
            capabilities.len(),
            FRONTEND_CAPABILITIES.len()
        ));
    }

    for page in catalog.pages() {
        if let AssertedSet::Established { items } = &page.frontend_features {
            for item in items.iter() {
                if FRONTEND_CAPABILITIES.iter().all(|def| def.id != item.value) {
                    errors.push(format!(
                        "page frontend capability {} is not registered in FRONTEND_CAPABILITIES",
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

fn capability_index(capabilities: &[DeclaredCapability]) -> HashMap<CapabilityId, usize> {
    capabilities
        .iter()
        .enumerate()
        .map(|(idx, capability)| (capability.id, idx))
        .collect()
}

fn sort_capability_communities(communities: &mut [CapabilityCommunity]) {
    for community in communities.iter_mut() {
        community.members.sort_by_key(|id| id.as_str());
    }
    communities.sort_by(|left, right| {
        right.members.len().cmp(&left.members.len()).then_with(|| {
            left.members
                .first()
                .map(|id| id.as_str())
                .cmp(&right.members.first().map(|id| id.as_str()))
        })
    });
}

fn capability_community_map(communities: &[CapabilityCommunity]) -> HashMap<CapabilityId, usize> {
    let mut map = HashMap::new();
    for (idx, community) in communities.iter().enumerate() {
        for member in &community.members {
            map.insert(*member, idx);
        }
    }
    map
}

fn journey_capability_indices(
    journey: &crate::journey::UseCase,
    catalog: &Catalog,
    index: &HashMap<CapabilityId, usize>,
) -> Vec<usize> {
    let mut capabilities = Vec::new();
    if let GroundedSet::Known { items } = &journey.capability_refs {
        for item in items.iter() {
            if let Some(&idx) = index.get(&item.value) {
                capabilities.push(idx);
            }
        }
    }
    if let Some(chain_id) = &journey.chains_from {
        if let Some(parent) = catalog
            .journeys()
            .iter()
            .find(|candidate| &candidate.id == chain_id)
        {
            capabilities.extend(journey_capability_indices(parent, catalog, index));
        }
    }
    capabilities.sort_unstable();
    capabilities.dedup();
    capabilities
}

fn journey_cooccurrence_pairs(
    catalog: &Catalog,
    capability_index: &HashMap<CapabilityId, usize>,
) -> HashMap<(CapabilityId, CapabilityId), Vec<JourneyId>> {
    let known: HashSet<CapabilityId> = capability_index.keys().copied().collect();
    let reverse_index: HashMap<usize, CapabilityId> = capability_index
        .iter()
        .map(|(id, idx)| (*idx, *id))
        .collect();
    let forward_index: HashMap<CapabilityId, usize> = capability_index
        .iter()
        .map(|(id, idx)| (*id, *idx))
        .collect();
    let mut pairs: HashMap<(CapabilityId, CapabilityId), Vec<JourneyId>> = HashMap::new();

    for journey in catalog.journeys() {
        let journey_capabilities: Vec<CapabilityId> =
            journey_capability_indices(journey, catalog, &forward_index)
                .into_iter()
                .filter_map(|idx| reverse_index.get(&idx).copied())
                .filter(|id| known.contains(id))
                .collect();
        for left in 0..journey_capabilities.len() {
            for right in (left + 1)..journey_capabilities.len() {
                let (pair_left, pair_right) =
                    ordered_pair(journey_capabilities[left], journey_capabilities[right]);
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

fn map_capability_groups(groups: &[Vec<usize>], ids: &[CapabilityId]) -> Vec<Vec<CapabilityId>> {
    let mut out: Vec<Vec<CapabilityId>> = groups
        .iter()
        .map(|group| {
            let mut members: Vec<CapabilityId> = group.iter().map(|&idx| ids[idx]).collect();
            members.sort_by_key(|id| id.as_str());
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

fn ordered_pair(left: CapabilityId, right: CapabilityId) -> (CapabilityId, CapabilityId) {
    if left.as_str() <= right.as_str() {
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

fn journey_capability_weights(
    catalog: &Catalog,
    capabilities: &[DeclaredCapability],
    index: &HashMap<CapabilityId, usize>,
) -> Vec<Vec<f64>> {
    let sets: Vec<Vec<usize>> = catalog
        .journeys()
        .iter()
        .map(|journey| journey_capability_indices(journey, catalog, index))
        .filter(|set| set.len() >= 2)
        .collect();
    clique_weights(capabilities.len(), &sets)
}

fn page_capability_weights(
    catalog: &Catalog,
    capabilities: &[DeclaredCapability],
    index: &HashMap<CapabilityId, usize>,
) -> Vec<Vec<f64>> {
    let mut service_capabilities: HashMap<ServiceId, Vec<usize>> = HashMap::new();
    for (idx, capability) in capabilities.iter().enumerate() {
        if let Origin::BackendService(service) = capability.origin {
            service_capabilities.entry(service).or_default().push(idx);
        }
    }

    let mut sets: Vec<Vec<usize>> = Vec::new();
    for page in catalog.pages() {
        let mut members: Vec<usize> = Vec::new();
        if let AssertedSet::Established { items } = &page.frontend_features {
            for item in items.iter() {
                if let Some(&idx) = index.get(&item.value) {
                    if !members.contains(&idx) {
                        members.push(idx);
                    }
                }
            }
        }
        if let ObservedSet::Known { items } = &page.consumes {
            for item in items.iter() {
                if let ConsumeTarget::Service(service) = item.value.service {
                    if let Some(capability_idxs) = service_capabilities.get(&service) {
                        for &idx in capability_idxs {
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
    clique_weights(capabilities.len(), &sets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::FRONTEND_CAPABILITIES;
    use crate::domain::{domain_of, DOMAINS};
    use crate::provenance::AssertedSet;

    const EXPECTED_CAPABILITY_COUNT: usize = 143;
    const EXPECTED_AGGREGATE_COUNT: usize = 20;

    #[test]
    fn declared_capabilities_builds_expected_count() {
        let catalog = Catalog::bootstrap();
        let capabilities = declared_capabilities(&catalog);
        assert_eq!(capabilities.len(), EXPECTED_CAPABILITY_COUNT);
    }

    #[test]
    fn origin_split_backend_and_frontend() {
        let catalog = Catalog::bootstrap();
        let capabilities = declared_capabilities(&catalog);
        let backend = capabilities
            .iter()
            .filter(|cap| matches!(cap.origin, Origin::BackendService(_)))
            .count();
        let frontend = capabilities
            .iter()
            .filter(|cap| matches!(cap.origin, Origin::FrontendPage(_)))
            .count();
        assert_eq!(frontend, FRONTEND_CAPABILITIES.len());
        assert_eq!(backend + frontend, EXPECTED_CAPABILITY_COUNT);
        assert_eq!(frontend, 14);
    }

    #[test]
    fn aggregate_view_returns_expected_aggregates() {
        let catalog = Catalog::bootstrap();
        let view = capability_aggregate_view(&catalog);
        assert_eq!(view.len(), EXPECTED_AGGREGATE_COUNT);
        let total: usize = view.iter().map(|group| group.capabilities.len()).sum();
        assert_eq!(total, EXPECTED_CAPABILITY_COUNT);
    }

    #[test]
    fn validate_passes_on_bootstrap() {
        let catalog = Catalog::bootstrap();
        assert!(validate_capabilities(&catalog).is_ok());
    }

    #[test]
    fn every_capability_is_declared_once() {
        let catalog = Catalog::bootstrap();
        let mut distinct = HashSet::new();
        for service in catalog.services() {
            if let AssertedSet::Established { items } = &service.definition.capabilities {
                for item in items.iter() {
                    distinct.insert(item.value);
                }
            }
        }
        let capabilities = declared_capabilities(&catalog);
        assert_eq!(
            capabilities.len(),
            distinct.len() + FRONTEND_CAPABILITIES.len()
        );
        assert!(validate_capabilities(&catalog).is_ok());
    }

    #[test]
    fn capabilities_per_domain() {
        let catalog = Catalog::bootstrap();
        let capabilities = declared_capabilities(&catalog);
        assert_eq!(capabilities.len(), EXPECTED_CAPABILITY_COUNT);

        let mut total = 0;
        for domain_def in DOMAINS {
            let count = capabilities
                .iter()
                .filter(|capability| domain_of(capability.aggregate) == domain_def.id)
                .count();
            eprintln!("{}: {count}", domain_def.id);
            total += count;
        }
        assert_eq!(total, EXPECTED_CAPABILITY_COUNT);

        for capability in &capabilities {
            let _ = domain_of(capability.aggregate);
        }
    }

    #[test]
    fn journey_view_is_deterministic() {
        let catalog = Catalog::bootstrap();
        let first = capability_journey_view(&catalog);
        let second = capability_journey_view(&catalog);
        assert_eq!(first, second);
    }

    #[test]
    fn journey_view_co_referenced_capabilities_share_community() {
        let catalog = Catalog::bootstrap();
        let view = capability_journey_view(&catalog);
        let flash = CapabilityId::FlashFirmware;
        let detect = CapabilityId::DetectFlightControllers;
        let flash_community = view
            .communities
            .iter()
            .find(|community| community.members.contains(&flash))
            .expect("flash_firmware community");
        assert!(flash_community.members.contains(&detect));
    }

    #[test]
    fn journey_view_unreferenced_capabilities_are_absent_from_journeys() {
        let catalog = Catalog::bootstrap();
        let view = capability_journey_view(&catalog);
        assert!(!view.unreferenced_capabilities.is_empty());

        let mut referenced = HashSet::new();
        for journey in catalog.journeys() {
            let GroundedSet::Known { items } = &journey.capability_refs else {
                continue;
            };
            for item in items.iter() {
                referenced.insert(item.value.as_str());
            }
        }
        for id in &view.unreferenced_capabilities {
            assert!(!referenced.contains(id.as_str()));
        }
    }

    #[test]
    fn view_divergence_finds_cross_aggregate_joins() {
        let catalog = Catalog::bootstrap();
        let divergence = capability_view_divergence(&catalog);
        assert!(!divergence.joined_by_journey.is_empty());
    }

    #[test]
    fn capability_split_lenses_each_partition_all_capabilities() {
        let catalog = Catalog::bootstrap();
        let split = capability_split_consensus(&catalog);
        assert_eq!(split.lenses.len(), 4);
        assert_eq!(split.majority_threshold, 3);
        for lens in &split.lenses {
            let mut seen = HashSet::new();
            let mut total = 0;
            for group in &lens.groups {
                for id in group {
                    assert!(
                        seen.insert(*id),
                        "duplicate capability in lens {}",
                        lens.lens
                    );
                    total += 1;
                }
            }
            assert_eq!(
                total, EXPECTED_CAPABILITY_COUNT,
                "lens {} missing capabilities",
                lens.lens
            );
        }
    }

    #[test]
    fn capability_split_consensus_is_deterministic() {
        let catalog = Catalog::bootstrap();
        assert_eq!(
            capability_split_consensus(&catalog),
            capability_split_consensus(&catalog)
        );
    }

    #[test]
    fn capability_split_consensus_agreement_bounds_and_partition() {
        let catalog = Catalog::bootstrap();
        let split = capability_split_consensus(&catalog);
        for pair in &split.pair_agreement {
            assert!(pair.agree >= 1 && pair.agree <= pair.total);
            assert_eq!(pair.total, 4);
        }
        let total: usize = split.consensus_clusters.iter().map(|c| c.len()).sum();
        assert_eq!(total, EXPECTED_CAPABILITY_COUNT);
    }

    #[test]
    fn journey_view_and_divergence_round_trip_serde() {
        let catalog = Catalog::bootstrap();
        let view = capability_journey_view(&catalog);
        let divergence = capability_view_divergence(&catalog);

        let view_json = serde_json::to_string(&view).unwrap();
        let view_value: serde_json::Value = serde_json::from_str(&view_json).unwrap();
        assert!(view_value.is_object());

        let divergence_json = serde_json::to_string(&divergence).unwrap();
        let divergence_value: serde_json::Value = serde_json::from_str(&divergence_json).unwrap();
        assert!(divergence_value.is_object());
    }
}
