use std::collections::HashMap;

use schemars::JsonSchema;
use serde::Serialize;

use crate::catalog::Catalog;
use crate::catalog::CouplingMatrix;
use crate::edge::{Bus, Edge, FailureImpact};
use crate::id::{PathRef, ServiceId};
use crate::page::ConsumeTarget;
use crate::provenance::{Asserted, AssertedSet, Grounded, GroundedSet, ObservedSet};
use crate::resource::ResourceOwnership;
use crate::service::ServiceDefinition;

pub const WEIGHTS_VERSION: &str = "v1";

const STABILITY_SEED: u64 = 0xC47A_7005;
const STABILITY_THRESHOLD: f64 = 0.8;

/// Tunable default coupling weights; change `WEIGHTS_VERSION` when adjusting.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct CouplingWeights {
    pub bus: HashMap<Bus, f64>,
    pub failure_impact: HashMap<FailureImpact, f64>,
    pub required_at_boot_bonus: f64,
    pub shared_write_write: f64,
    pub shared_write_read: f64,
    pub shared_read_read: f64,
    pub trust_affinity_bonus: f64,
    pub domain_affinity_bonus: f64,
}

impl Default for CouplingWeights {
    fn default() -> Self {
        let mut bus = HashMap::new();
        bus.insert(Bus::Hardware, 1.0);
        bus.insert(Bus::Mavlink, 0.9);
        bus.insert(Bus::Subprocess, 0.8);
        bus.insert(Bus::File, 0.7);
        bus.insert(Bus::Zenoh, 0.6);
        bus.insert(Bus::Settings, 0.5);
        bus.insert(Bus::Rest, 0.4);
        bus.insert(Bus::Websocket, 0.4);
        bus.insert(Bus::HttpStream, 0.4);
        bus.insert(Bus::Docker, 0.3);

        let mut failure_impact = HashMap::new();
        failure_impact.insert(FailureImpact::VehicleUnsafe, 1.5);
        failure_impact.insert(FailureImpact::ServiceUnavailable, 1.2);
        failure_impact.insert(FailureImpact::Degraded, 1.0);
        failure_impact.insert(FailureImpact::None, 0.8);

        Self {
            bus,
            failure_impact,
            required_at_boot_bonus: 0.2,
            shared_write_write: 0.8,
            shared_write_read: 0.5,
            shared_read_read: 0.3,
            trust_affinity_bonus: 0.15,
            domain_affinity_bonus: 0.2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClusterPolicy {
    CouplingOnly,
    CouplingTrust,
    CouplingDomain,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct ClusterResult {
    pub policy: ClusterPolicy,
    pub communities: Vec<Vec<ServiceId>>,
    pub modularity: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct StabilityReport {
    pub runs: usize,
    pub jitter: f64,
    pub co_occurrence: Vec<Vec<f64>>,
    pub unstable_pairs: Vec<(ServiceId, ServiceId, f64)>,
}

/// One split "approach": a service partition produced by a single lens. The label
/// identifies the signal (structural coupling policy, page co-occurrence, or journey
/// co-occurrence) so partitions from different lenses can be compared side by side.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct LensPartition {
    pub lens: &'static str,
    pub communities: Vec<Vec<ServiceId>>,
    pub modularity: f64,
}

/// How often a service pair lands in the same community across the lenses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
pub struct PairAgreement {
    pub a: ServiceId,
    pub b: ServiceId,
    pub agree: usize,
    pub total: usize,
}

/// Cross-lens consensus: every lens partition, the pairwise agreement counts, and the
/// consensus clusters formed by keeping only pairs agreed on by a majority of lenses.
/// This is decision *input* for M4, not a boundary decision.
#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
pub struct SplitConsensus {
    pub lenses: Vec<LensPartition>,
    pub pair_agreement: Vec<PairAgreement>,
    pub majority_threshold: usize,
    pub consensus_clusters: Vec<Vec<ServiceId>>,
}

impl Catalog {
    pub fn coupling_matrix(&self, policy: ClusterPolicy) -> CouplingMatrix {
        build_coupling_matrix(self, policy, &CouplingWeights::default())
    }

    pub fn cluster(&self, policy: ClusterPolicy) -> ClusterResult {
        let matrix = self.coupling_matrix(policy);
        let (communities, modularity) = greedy_modularity_cluster(&matrix);
        ClusterResult {
            policy,
            communities,
            modularity,
        }
    }

    pub fn cluster_stability(
        &self,
        policy: ClusterPolicy,
        runs: usize,
        jitter: f64,
    ) -> StabilityReport {
        let base = self.cluster(policy);
        let matrix = self.coupling_matrix(policy);
        let n = matrix.service_ids.len();
        let mut co_counts = vec![vec![0usize; n]; n];
        let mut rng = Lcg::new(STABILITY_SEED);

        for _ in 0..runs {
            let jittered = jitter_matrix(&matrix, jitter, &mut rng);
            let (communities, _) = greedy_modularity_cluster(&jittered);
            for community in &communities {
                for (a, id_a) in community.iter().enumerate() {
                    let i = matrix
                        .service_ids
                        .iter()
                        .position(|id| id == id_a)
                        .expect("community member in matrix");
                    for id_b in community.iter().skip(a + 1) {
                        let j = matrix
                            .service_ids
                            .iter()
                            .position(|id| id == id_b)
                            .expect("community member in matrix");
                        co_counts[i][j] += 1;
                        co_counts[j][i] += 1;
                    }
                }
            }
        }

        let runs_f = runs as f64;
        let co_occurrence: Vec<Vec<f64>> = co_counts
            .iter()
            .map(|row| row.iter().map(|count| *count as f64 / runs_f).collect())
            .collect();

        let base_membership = membership_map(&base.communities, &matrix.service_ids);
        let mut unstable_pairs = Vec::new();
        for (i, row) in co_occurrence.iter().enumerate().take(n) {
            for (j, co_rate) in row
                .iter()
                .enumerate()
                .skip(i + 1)
                .take(n.saturating_sub(i + 1))
            {
                let together_base = same_community(&base_membership, i, j);
                let co_rate = *co_rate;
                let unstable = (together_base && co_rate < STABILITY_THRESHOLD)
                    || (!together_base && co_rate >= STABILITY_THRESHOLD);
                if unstable {
                    unstable_pairs.push((matrix.service_ids[i], matrix.service_ids[j], co_rate));
                }
            }
        }
        unstable_pairs.sort_by(|left, right| {
            left.0
                .as_str()
                .cmp(right.0.as_str())
                .then_with(|| left.1.as_str().cmp(right.1.as_str()))
        });

        StabilityReport {
            runs,
            jitter,
            co_occurrence,
            unstable_pairs,
        }
    }

    /// Candidate proposals for human review; NOT a final boundary decision (that is M4/human).
    pub fn boundary_proposals(&self) -> Vec<ClusterResult> {
        vec![
            self.cluster(ClusterPolicy::CouplingOnly),
            self.cluster(ClusterPolicy::CouplingTrust),
            self.cluster(ClusterPolicy::CouplingDomain),
        ]
    }

    /// Service affinity from the frontend: two services that a page consumes together
    /// gain +1 per shared page. Captures "which services the operator uses together",
    /// a signal invisible to the structural edge graph.
    pub fn page_cooccurrence_matrix(&self) -> CouplingMatrix {
        let service_ids = all_service_ids(self);
        let index = index_of(&service_ids);
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for page in self.pages() {
            if let ObservedSet::Known { items } = &page.consumes {
                let mut members: Vec<usize> = Vec::new();
                for item in items.iter() {
                    if let ConsumeTarget::Service(sid) = item.value.service {
                        if let Some(&idx) = index.get(&sid) {
                            if !members.contains(&idx) {
                                members.push(idx);
                            }
                        }
                    }
                }
                if members.len() >= 2 {
                    groups.push(members);
                }
            }
        }
        cooccurrence_matrix(service_ids, &groups)
    }

    /// Service affinity from workflows: two services that co-participate in a journey
    /// (via `services` or a step `route`) gain +1 per shared journey. Captures
    /// "which services collaborate to accomplish one operator goal".
    pub fn journey_cooccurrence_matrix(&self) -> CouplingMatrix {
        let service_ids = all_service_ids(self);
        let index = index_of(&service_ids);
        let mut groups: Vec<Vec<usize>> = Vec::new();
        for journey in self.journeys() {
            let mut members: Vec<usize> = Vec::new();
            if let GroundedSet::Known { items } = &journey.services {
                for item in items.iter() {
                    push_unique(&index, &mut members, item.value);
                }
            }
            if let GroundedSet::Known { items } = &journey.steps {
                for item in items.iter() {
                    if let Some(Grounded::Known { value: route, .. }) = &item.value.route {
                        push_unique(&index, &mut members, route.service);
                    }
                }
            }
            if members.len() >= 2 {
                groups.push(members);
            }
        }
        cooccurrence_matrix(service_ids, &groups)
    }

    /// All service-level split lenses: the three structural coupling policies plus the
    /// two co-occurrence lenses. Each returns a partition over the same 26 services.
    pub fn split_lenses(&self) -> Vec<LensPartition> {
        let mut lenses = Vec::new();
        for policy in [
            ClusterPolicy::CouplingOnly,
            ClusterPolicy::CouplingTrust,
            ClusterPolicy::CouplingDomain,
        ] {
            let result = self.cluster(policy);
            lenses.push(LensPartition {
                lens: policy_label(policy),
                communities: result.communities,
                modularity: result.modularity,
            });
        }
        let (communities, modularity) = greedy_modularity_cluster(&self.page_cooccurrence_matrix());
        lenses.push(LensPartition {
            lens: "page_cooccurrence",
            communities,
            modularity,
        });
        let (communities, modularity) =
            greedy_modularity_cluster(&self.journey_cooccurrence_matrix());
        lenses.push(LensPartition {
            lens: "journey_cooccurrence",
            communities,
            modularity,
        });
        lenses
    }

    /// Cross-lens consensus over all split lenses. Emits, per service pair, how many
    /// lenses grouped them together, and the consensus clusters formed by keeping only
    /// majority-agreed pairs (connected components). Input for M4, not a decision.
    pub fn split_consensus(&self) -> SplitConsensus {
        let lenses = self.split_lenses();
        let total = lenses.len();
        let service_ids = all_service_ids(self);
        let index = index_of(&service_ids);
        let n = service_ids.len();

        let memberships: Vec<Vec<usize>> = lenses
            .iter()
            .map(|lens| membership_vec(&lens.communities, &index, n))
            .collect();

        let mut pair_agreement = Vec::new();
        let mut agree_matrix = vec![vec![0usize; n]; n];
        for i in 0..n {
            for j in (i + 1)..n {
                let agree = memberships
                    .iter()
                    .filter(|m| m[i] == m[j] && m[i] != usize::MAX)
                    .count();
                agree_matrix[i][j] = agree;
                agree_matrix[j][i] = agree;
                if agree > 0 {
                    pair_agreement.push(PairAgreement {
                        a: service_ids[i],
                        b: service_ids[j],
                        agree,
                        total,
                    });
                }
            }
        }
        pair_agreement.sort_by(|left, right| {
            right
                .agree
                .cmp(&left.agree)
                .then_with(|| left.a.as_str().cmp(right.a.as_str()))
                .then_with(|| left.b.as_str().cmp(right.b.as_str()))
        });

        let majority_threshold = total / 2 + 1;
        let consensus_clusters =
            connected_components(&service_ids, &agree_matrix, majority_threshold);

        SplitConsensus {
            lenses,
            pair_agreement,
            majority_threshold,
            consensus_clusters,
        }
    }
}

fn all_service_ids(catalog: &Catalog) -> Vec<ServiceId> {
    catalog
        .services()
        .iter()
        .map(|service| service.id)
        .collect()
}

fn index_of(service_ids: &[ServiceId]) -> HashMap<ServiceId, usize> {
    service_ids
        .iter()
        .enumerate()
        .map(|(idx, id)| (*id, idx))
        .collect()
}

fn push_unique(index: &HashMap<ServiceId, usize>, members: &mut Vec<usize>, id: ServiceId) {
    if let Some(&idx) = index.get(&id) {
        if !members.contains(&idx) {
            members.push(idx);
        }
    }
}

fn cooccurrence_matrix(service_ids: Vec<ServiceId>, groups: &[Vec<usize>]) -> CouplingMatrix {
    let n = service_ids.len();
    let mut weights = vec![vec![0.0; n]; n];
    for group in groups {
        for (pos, &i) in group.iter().enumerate() {
            for &j in group.iter().skip(pos + 1) {
                weights[i][j] += 1.0;
                weights[j][i] += 1.0;
            }
        }
    }
    CouplingMatrix {
        service_ids,
        weights,
    }
}

fn policy_label(policy: ClusterPolicy) -> &'static str {
    match policy {
        ClusterPolicy::CouplingOnly => "coupling_only",
        ClusterPolicy::CouplingTrust => "coupling_trust",
        ClusterPolicy::CouplingDomain => "coupling_domain",
    }
}

/// Membership vector: `out[i]` is the community index of service `i` in a partition,
/// or `usize::MAX` if the service is absent (should not happen for full partitions).
fn membership_vec(
    communities: &[Vec<ServiceId>],
    index: &HashMap<ServiceId, usize>,
    n: usize,
) -> Vec<usize> {
    let mut out = vec![usize::MAX; n];
    for (community_idx, community) in communities.iter().enumerate() {
        for id in community {
            if let Some(&i) = index.get(id) {
                out[i] = community_idx;
            }
        }
    }
    out
}

/// Connected components over the agreement graph, keeping edges with `agree >= threshold`.
/// Every service appears in exactly one component (singletons included). Deterministic.
fn connected_components(
    service_ids: &[ServiceId],
    agree_matrix: &[Vec<usize>],
    threshold: usize,
) -> Vec<Vec<ServiceId>> {
    let n = service_ids.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    for (i, row) in agree_matrix.iter().enumerate() {
        for (j, &agree) in row.iter().enumerate().skip(i + 1) {
            if agree >= threshold {
                let ri = find(&mut parent, i);
                let rj = find(&mut parent, j);
                if ri != rj {
                    parent[ri.max(rj)] = ri.min(rj);
                }
            }
        }
    }
    let mut groups: HashMap<usize, Vec<ServiceId>> = HashMap::new();
    for (i, id) in service_ids.iter().enumerate() {
        let root = find(&mut parent, i);
        groups.entry(root).or_default().push(*id);
    }
    let mut clusters: Vec<Vec<ServiceId>> = groups
        .into_values()
        .map(|mut ids| {
            ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
            ids
        })
        .collect();
    clusters.sort_by(|left, right| left[0].as_str().cmp(right[0].as_str()));
    clusters
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_f64(&mut self) -> f64 {
        self.state = self
            .state
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        ((self.state >> 16) & 0xFFFF) as f64 / 65_536.0
    }
}

fn build_coupling_matrix(
    catalog: &Catalog,
    policy: ClusterPolicy,
    weights: &CouplingWeights,
) -> CouplingMatrix {
    let service_ids: Vec<ServiceId> = catalog
        .services()
        .iter()
        .map(|service| service.id)
        .collect();
    let index: HashMap<&ServiceId, usize> = service_ids
        .iter()
        .enumerate()
        .map(|(idx, id)| (id, idx))
        .collect();
    let size = service_ids.len();
    let mut matrix = vec![vec![0.0; size]; size];

    for service in catalog.services() {
        let Some(&from_idx) = index.get(&service.id) else {
            continue;
        };
        if let AssertedSet::Established { items } = &service.definition.edges {
            for rationaled in items.iter() {
                let edge = &rationaled.value;
                let Some(&to_idx) = index.get(&edge.to) else {
                    continue;
                };
                let edge_weight = edge_coupling_weight(edge, weights);
                matrix[from_idx][to_idx] += edge_weight;
                matrix[to_idx][from_idx] += edge_weight;
            }
        }
    }

    add_shared_resource_coupling(catalog, &index, &mut matrix, weights);
    add_policy_affinity(catalog, &index, &mut matrix, policy, weights);

    CouplingMatrix {
        service_ids,
        weights: matrix,
    }
}

fn edge_coupling_weight(edge: &Edge, weights: &CouplingWeights) -> f64 {
    let bus_weight = weights.bus.get(&edge.via).copied().unwrap_or(0.0);
    let failure_mult = weights
        .failure_impact
        .get(&edge.failure_impact)
        .copied()
        .unwrap_or(1.0);
    let mut weight = bus_weight * failure_mult;
    if edge.required_at_boot {
        weight += weights.required_at_boot_bonus;
    }
    weight
}

fn add_shared_resource_coupling(
    catalog: &Catalog,
    index: &HashMap<&ServiceId, usize>,
    matrix: &mut [Vec<f64>],
    weights: &CouplingWeights,
) {
    let mut path_owners: HashMap<&PathRef, Vec<(usize, ResourceOwnership)>> = HashMap::new();
    for service in catalog.services() {
        let Some(&service_idx) = index.get(&service.id) else {
            continue;
        };
        if let AssertedSet::Established { items } = &service.definition.resources {
            for rationaled in items.iter() {
                let resource = &rationaled.value;
                if resource.ownership == ResourceOwnership::Exclusive {
                    continue;
                }
                path_owners
                    .entry(&resource.path)
                    .or_default()
                    .push((service_idx, resource.ownership));
            }
        }
    }

    for owners in path_owners.values() {
        for left in 0..owners.len() {
            for right in (left + 1)..owners.len() {
                let (i, left_own) = owners[left];
                let (j, right_own) = owners[right];
                if i == j {
                    continue;
                }
                let shared_weight = shared_resource_weight(left_own, right_own, weights);
                if shared_weight > 0.0 {
                    matrix[i][j] += shared_weight;
                    matrix[j][i] += shared_weight;
                }
            }
        }
    }
}

fn shared_resource_weight(
    left: ResourceOwnership,
    right: ResourceOwnership,
    weights: &CouplingWeights,
) -> f64 {
    match (left, right) {
        (ResourceOwnership::SharedWrite, ResourceOwnership::SharedWrite) => {
            weights.shared_write_write
        }
        (ResourceOwnership::SharedRead, ResourceOwnership::SharedRead) => weights.shared_read_read,
        (ResourceOwnership::SharedWrite, ResourceOwnership::SharedRead)
        | (ResourceOwnership::SharedRead, ResourceOwnership::SharedWrite) => {
            weights.shared_write_read
        }
        (ResourceOwnership::Exclusive, _) | (_, ResourceOwnership::Exclusive) => 0.0,
    }
}

fn add_policy_affinity(
    catalog: &Catalog,
    index: &HashMap<&ServiceId, usize>,
    matrix: &mut [Vec<f64>],
    policy: ClusterPolicy,
    weights: &CouplingWeights,
) {
    let bonus = match policy {
        ClusterPolicy::CouplingOnly => return,
        ClusterPolicy::CouplingTrust => weights.trust_affinity_bonus,
        ClusterPolicy::CouplingDomain => weights.domain_affinity_bonus,
    };

    let services = catalog.services();
    for left in 0..services.len() {
        for right in (left + 1)..services.len() {
            if policy_matches(
                &services[left].definition,
                &services[right].definition,
                policy,
            ) {
                let i = index[&services[left].id];
                let j = index[&services[right].id];
                matrix[i][j] += bonus;
                matrix[j][i] += bonus;
            }
        }
    }
}

fn policy_matches(
    left: &ServiceDefinition,
    right: &ServiceDefinition,
    policy: ClusterPolicy,
) -> bool {
    match policy {
        ClusterPolicy::CouplingOnly => false,
        ClusterPolicy::CouplingTrust => match (&left.privilege_level, &right.privilege_level) {
            (
                Asserted::Established { value: left, .. },
                Asserted::Established { value: right, .. },
            ) => left == right,
            _ => false,
        },
        ClusterPolicy::CouplingDomain => match (&left.bounded_context, &right.bounded_context) {
            (
                Asserted::Established { value: left, .. },
                Asserted::Established { value: right, .. },
            ) => left == right,
            _ => false,
        },
    }
}

fn total_edge_mass(weights: &[Vec<f64>]) -> f64 {
    let mut total = 0.0;
    for (row_idx, row) in weights.iter().enumerate() {
        for (col_idx, weight) in row.iter().enumerate() {
            if col_idx > row_idx {
                total += weight;
            }
        }
    }
    total
}

pub(crate) fn greedy_modularity_communities(n: usize, weights: &[Vec<f64>]) -> Vec<Vec<usize>> {
    if n == 0 {
        return Vec::new();
    }

    let mut communities: Vec<Vec<usize>> = (0..n).map(|idx| vec![idx]).collect();
    let mass = total_edge_mass(weights);
    if mass == 0.0 {
        return communities;
    }

    loop {
        let mut best_delta = 0.0;
        let mut best_pair: Option<(usize, usize)> = None;

        for left in 0..communities.len() {
            for right in (left + 1)..communities.len() {
                let delta = merge_delta(&communities[left], &communities[right], weights, mass);
                if delta > best_delta + f64::EPSILON {
                    best_delta = delta;
                    best_pair = Some((left, right));
                } else if (delta - best_delta).abs() <= f64::EPSILON && delta > 0.0 {
                    if let Some((best_left, best_right)) = best_pair {
                        if merge_tie_break(
                            &communities[left],
                            &communities[right],
                            &communities[best_left],
                            &communities[best_right],
                        ) {
                            best_pair = Some((left, right));
                        }
                    } else {
                        best_pair = Some((left, right));
                    }
                }
            }
        }

        let Some((left_idx, right_idx)) = best_pair else {
            break;
        };
        let mut merged = communities.remove(right_idx);
        merged.extend(communities.remove(left_idx));
        communities.push(merged);
    }

    communities
}

pub(crate) fn modularity_q_indices(weights: &[Vec<f64>], communities: &[Vec<usize>]) -> f64 {
    let mass = total_edge_mass(weights);
    if mass == 0.0 {
        return 0.0;
    }
    let two_m = 2.0 * mass;
    let mut q = 0.0;
    for community in communities {
        let internal = community_internal(community, weights);
        let total = community_total(community, weights);
        q += internal / two_m - (total / two_m).powi(2);
    }
    q
}

fn greedy_modularity_cluster(matrix: &CouplingMatrix) -> (Vec<Vec<ServiceId>>, f64) {
    let n = matrix.service_ids.len();
    if n == 0 {
        return (Vec::new(), 0.0);
    }

    let communities = greedy_modularity_communities(n, &matrix.weights);
    if total_edge_mass(&matrix.weights) == 0.0 {
        let singletons = matrix.service_ids.iter().map(|id| vec![*id]).collect();
        return (singletons, 0.0);
    }

    let mut result: Vec<Vec<ServiceId>> = communities
        .into_iter()
        .map(|community| {
            let mut ids: Vec<ServiceId> = community
                .iter()
                .map(|idx| matrix.service_ids[*idx])
                .collect();
            ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
            ids
        })
        .collect();
    sort_communities(&mut result);
    let modularity = modularity_q(
        &matrix.weights,
        &result,
        &matrix.service_ids,
        total_edge_mass(&matrix.weights),
    );
    (result, modularity)
}

fn merge_tie_break(
    left_a: &[usize],
    left_b: &[usize],
    right_a: &[usize],
    right_b: &[usize],
) -> bool {
    let left_min = min_node_index(left_a, left_b);
    let right_min = min_node_index(right_a, right_b);
    if left_min != right_min {
        return left_min < right_min;
    }
    let left_pair = (min_node_index(left_a, &[]), min_node_index(left_b, &[]));
    let right_pair = (min_node_index(right_a, &[]), min_node_index(right_b, &[]));
    left_pair < right_pair
}

fn min_node_index(left: &[usize], right: &[usize]) -> usize {
    left.iter()
        .chain(right.iter())
        .copied()
        .min()
        .unwrap_or(usize::MAX)
}

fn community_internal(community: &[usize], weights: &[Vec<f64>]) -> f64 {
    let mut total = 0.0;
    for (left_pos, &left) in community.iter().enumerate() {
        for &right in community.iter().skip(left_pos + 1) {
            total += weights[left][right];
        }
    }
    total
}

fn community_cut(left: &[usize], right: &[usize], weights: &[Vec<f64>]) -> f64 {
    let mut total = 0.0;
    for &i in left {
        for &j in right {
            total += weights[i][j];
        }
    }
    total
}

fn community_total(community: &[usize], weights: &[Vec<f64>]) -> f64 {
    community
        .iter()
        .map(|&idx| weights[idx].iter().sum::<f64>())
        .sum()
}

fn merge_delta(left: &[usize], right: &[usize], weights: &[Vec<f64>], mass: f64) -> f64 {
    let two_m = 2.0 * mass;
    let internal_left = community_internal(left, weights);
    let internal_right = community_internal(right, weights);
    let cut = community_cut(left, right, weights);
    let total_left = community_total(left, weights);
    let total_right = community_total(right, weights);

    let internal_merged = internal_left + internal_right + cut;
    let total_merged = total_left + total_right;

    (internal_merged / two_m - (total_merged / two_m).powi(2))
        - (internal_left / two_m - (total_left / two_m).powi(2))
        - (internal_right / two_m - (total_right / two_m).powi(2))
}

fn modularity_q(
    weights: &[Vec<f64>],
    communities: &[Vec<ServiceId>],
    service_ids: &[ServiceId],
    mass: f64,
) -> f64 {
    if mass == 0.0 {
        return 0.0;
    }
    let index: HashMap<&ServiceId, usize> = service_ids
        .iter()
        .enumerate()
        .map(|(idx, id)| (id, idx))
        .collect();
    let two_m = 2.0 * mass;
    let mut q = 0.0;
    for community in communities {
        let nodes: Vec<usize> = community.iter().map(|id| index[id]).collect();
        let internal = community_internal(&nodes, weights);
        let total = community_total(&nodes, weights);
        q += internal / two_m - (total / two_m).powi(2);
    }
    q
}

fn sort_communities(communities: &mut [Vec<ServiceId>]) {
    communities.sort_by(|left, right| {
        right.len().cmp(&left.len()).then_with(|| {
            left.first()
                .map(|id| id.as_str())
                .cmp(&right.first().map(|id| id.as_str()))
        })
    });
}

fn jitter_matrix(matrix: &CouplingMatrix, jitter: f64, rng: &mut Lcg) -> CouplingMatrix {
    let size = matrix.service_ids.len();
    let mut weights = vec![vec![0.0; size]; size];
    for (i, row) in weights.iter_mut().enumerate().take(size) {
        for (j, cell) in row.iter_mut().enumerate().take(size) {
            if i == j {
                continue;
            }
            let factor = 1.0 - jitter + (2.0 * jitter) * rng.next_f64();
            *cell = matrix.weights[i][j] * factor;
        }
    }
    CouplingMatrix {
        service_ids: matrix.service_ids.clone(),
        weights,
    }
}

fn membership_map(communities: &[Vec<ServiceId>], service_ids: &[ServiceId]) -> Vec<usize> {
    let mut membership = vec![0; service_ids.len()];
    let index: HashMap<&ServiceId, usize> = service_ids
        .iter()
        .enumerate()
        .map(|(idx, id)| (id, idx))
        .collect();
    for (community_idx, community) in communities.iter().enumerate() {
        for id in community {
            membership[index[id]] = community_idx;
        }
    }
    membership
}

fn same_community(membership: &[usize], left: usize, right: usize) -> bool {
    membership[left] == membership[right]
}

pub(crate) fn bus_label(bus: Bus) -> &'static str {
    match bus {
        Bus::Rest => "rest",
        Bus::Zenoh => "zenoh",
        Bus::Mavlink => "mavlink",
        Bus::Websocket => "websocket",
        Bus::HttpStream => "http_stream",
        Bus::Subprocess => "subprocess",
        Bus::File => "file",
        Bus::Settings => "settings",
        Bus::Hardware => "hardware",
        Bus::Docker => "docker",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Catalog;

    fn service_index(catalog: &Catalog, id: &str) -> usize {
        catalog
            .coupling_matrix(ClusterPolicy::CouplingOnly)
            .service_ids
            .iter()
            .position(|service_id| service_id.as_str() == id)
            .unwrap_or_else(|| panic!("missing service {id}"))
    }

    #[test]
    fn clustering_groups_obviously_coupled_pair() {
        let catalog = Catalog::bootstrap();
        let result = catalog.cluster(ClusterPolicy::CouplingTrust);
        let mavlink = ServiceId::Mavlink2rest;
        let manager = ServiceId::ArdupilotManager;
        let together = result
            .communities
            .iter()
            .any(|community| community.contains(&mavlink) && community.contains(&manager));
        assert!(together);
    }

    #[test]
    fn coupling_matrix_is_symmetric_with_zero_diagonal() {
        let catalog = Catalog::bootstrap();
        let matrix = catalog.coupling_matrix(ClusterPolicy::CouplingOnly);
        let size = matrix.service_ids.len();
        for row in 0..size {
            assert_eq!(matrix.weights[row][row], 0.0);
            for col in (row + 1)..size {
                assert_eq!(matrix.weights[row][col], matrix.weights[col][row]);
            }
        }
    }

    #[test]
    fn coupling_matrix_has_known_edge_weight() {
        let catalog = Catalog::bootstrap();
        let matrix = catalog.coupling_matrix(ClusterPolicy::CouplingOnly);
        let mavlink = service_index(&catalog, "mavlink2rest");
        let manager = service_index(&catalog, "ardupilot_manager");
        assert!(matrix.weights[mavlink][manager] > 0.0);
    }

    #[test]
    fn unrelated_leaf_services_have_zero_coupling() {
        let catalog = Catalog::bootstrap();
        let matrix = catalog.coupling_matrix(ClusterPolicy::CouplingOnly);
        let iperf = service_index(&catalog, "iperf3");
        let terminal = service_index(&catalog, "user_terminal");
        assert_eq!(matrix.weights[iperf][terminal], 0.0);
    }

    #[test]
    fn clustering_is_deterministic() {
        let catalog = Catalog::bootstrap();
        let first = catalog.cluster(ClusterPolicy::CouplingOnly);
        let second = catalog.cluster(ClusterPolicy::CouplingOnly);
        assert_eq!(first, second);
    }

    #[test]
    fn stability_is_reproducible() {
        let catalog = Catalog::bootstrap();
        let first = catalog.cluster_stability(ClusterPolicy::CouplingOnly, 5, 0.1);
        let second = catalog.cluster_stability(ClusterPolicy::CouplingOnly, 5, 0.1);
        assert_eq!(first.co_occurrence, second.co_occurrence);
    }

    #[test]
    fn boundary_proposals_returns_three() {
        let catalog = Catalog::bootstrap();
        assert_eq!(catalog.boundary_proposals().len(), 3);
    }

    #[test]
    fn modularity_is_finite_and_bounded() {
        let catalog = Catalog::bootstrap();
        let result = catalog.cluster(ClusterPolicy::CouplingOnly);
        assert!(result.modularity.is_finite());
        assert!(result.modularity > -1.0);
        assert!(result.modularity <= 1.0);
    }

    fn assert_full_partition(communities: &[Vec<ServiceId>], n: usize) {
        let mut seen: Vec<ServiceId> = communities.iter().flatten().copied().collect();
        assert_eq!(seen.len(), n, "every service appears exactly once");
        seen.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        seen.dedup();
        assert_eq!(seen.len(), n, "no service appears twice");
    }

    #[test]
    fn cooccurrence_matrices_are_symmetric_zero_diagonal() {
        let catalog = Catalog::bootstrap();
        for matrix in [
            catalog.page_cooccurrence_matrix(),
            catalog.journey_cooccurrence_matrix(),
        ] {
            let n = matrix.service_ids.len();
            for i in 0..n {
                assert_eq!(matrix.weights[i][i], 0.0);
                for j in 0..n {
                    assert_eq!(matrix.weights[i][j], matrix.weights[j][i]);
                }
            }
        }
    }

    #[test]
    fn split_lenses_all_partition_every_service() {
        let catalog = Catalog::bootstrap();
        let n = catalog.services().len();
        let lenses = catalog.split_lenses();
        assert_eq!(lenses.len(), 5);
        for lens in &lenses {
            assert_full_partition(&lens.communities, n);
        }
    }

    #[test]
    fn consensus_partitions_and_bounds_agreement() {
        let catalog = Catalog::bootstrap();
        let n = catalog.services().len();
        let consensus = catalog.split_consensus();
        assert_eq!(consensus.majority_threshold, 3);
        for pair in &consensus.pair_agreement {
            assert!(pair.agree >= 1 && pair.agree <= pair.total);
            assert_eq!(pair.total, consensus.lenses.len());
        }
        assert_full_partition(&consensus.consensus_clusters, n);
    }

    #[test]
    fn consensus_is_deterministic() {
        let catalog = Catalog::bootstrap();
        assert_eq!(catalog.split_consensus(), catalog.split_consensus());
    }
}
