//! Generic, index-based cross-lens consensus.
//!
//! Given several partitions of the same `n` items (each partition produced by a
//! different "lens"), compute how often each pair lands together and the consensus
//! clusters formed by keeping only majority-agreed pairs. Domain modules (services,
//! features, journeys) map their typed ids to indices and back.

use crate::cluster::{greedy_modularity_communities, modularity_q_indices};

/// Modularity clustering over an affinity matrix; returns a full partition (isolated
/// items remain singletons) and its modularity Q.
pub fn modularity_partition(n: usize, weights: &[Vec<f64>]) -> (Vec<Vec<usize>>, f64) {
    if n == 0 {
        return (Vec::new(), 0.0);
    }
    let communities = greedy_modularity_communities(n, weights);
    let modularity = modularity_q_indices(weights, &communities);
    (communities, modularity)
}

/// Pairwise agreement and the majority-consensus clustering over index partitions.
pub struct ConsensusIdx {
    /// `(i, j, agree)` for `i < j` with `agree >= 1`, sorted by agreement descending.
    pub pair_agreement: Vec<(usize, usize, usize)>,
    pub total: usize,
    pub majority_threshold: usize,
    /// Connected components of the "agree >= majority" graph; every item appears once.
    pub clusters: Vec<Vec<usize>>,
}

/// `out[i]` = the group index of item `i` in a partition, or `usize::MAX` if absent.
fn membership(groups: &[Vec<usize>], n: usize) -> Vec<usize> {
    let mut out = vec![usize::MAX; n];
    for (group_idx, group) in groups.iter().enumerate() {
        for &item in group {
            out[item] = group_idx;
        }
    }
    out
}

pub fn consensus_over(partitions: &[Vec<Vec<usize>>], n: usize) -> ConsensusIdx {
    let total = partitions.len();
    let memberships: Vec<Vec<usize>> = partitions.iter().map(|p| membership(p, n)).collect();

    let mut agree = vec![vec![0usize; n]; n];
    let mut pair_agreement = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let count = memberships
                .iter()
                .filter(|m| m[i] != usize::MAX && m[i] == m[j])
                .count();
            agree[i][j] = count;
            agree[j][i] = count;
            if count > 0 {
                pair_agreement.push((i, j, count));
            }
        }
    }
    pair_agreement.sort_by(|left, right| {
        right
            .2
            .cmp(&left.2)
            .then_with(|| left.0.cmp(&right.0))
            .then_with(|| left.1.cmp(&right.1))
    });

    let majority_threshold = total / 2 + 1;
    let clusters = components(n, &agree, majority_threshold);

    ConsensusIdx {
        pair_agreement,
        total,
        majority_threshold,
        clusters,
    }
}

/// Connected components over the agreement graph, keeping `agree >= threshold` edges.
/// Deterministic: components and their members are returned in ascending index order.
pub(crate) fn components(n: usize, agree: &[Vec<usize>], threshold: usize) -> Vec<Vec<usize>> {
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }
    for (i, row) in agree.iter().enumerate() {
        for (j, &count) in row.iter().enumerate().skip(i + 1) {
            if count >= threshold {
                let ri = find(&mut parent, i);
                let rj = find(&mut parent, j);
                if ri != rj {
                    parent[ri.max(rj)] = ri.min(rj);
                }
            }
        }
    }
    let mut by_root: std::collections::BTreeMap<usize, Vec<usize>> =
        std::collections::BTreeMap::new();
    for i in 0..n {
        let root = find(&mut parent, i);
        by_root.entry(root).or_default().push(i);
    }
    by_root.into_values().collect()
}

/// Label lens: group item indices by an opaque key, returning index groups sorted by key.
pub fn group_by_key<K: Ord + Clone>(keys: &[K]) -> Vec<Vec<usize>> {
    let mut by_key: std::collections::BTreeMap<K, Vec<usize>> = std::collections::BTreeMap::new();
    for (idx, key) in keys.iter().enumerate() {
        by_key.entry(key.clone()).or_default().push(idx);
    }
    by_key.into_values().collect()
}

/// Clique-affinity matrix: each membership set contributes +1 to every pair inside it.
pub fn clique_weights(n: usize, sets: &[Vec<usize>]) -> Vec<Vec<f64>> {
    let mut weights = vec![vec![0.0; n]; n];
    for set in sets {
        for (pos, &i) in set.iter().enumerate() {
            for &j in set.iter().skip(pos + 1) {
                weights[i][j] += 1.0;
                weights[j][i] += 1.0;
            }
        }
    }
    weights
}
