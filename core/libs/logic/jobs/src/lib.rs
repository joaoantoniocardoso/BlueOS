//! Deterministic job graphs for multi-step sans-IO workflows.
//!
//! A service models long-running work as a tree of jobs: leaves are units the kernel runs (IO, blocking
//! CPU); `Sequence` runs children one after another; `Parallel` runs children concurrently. Cancellation
//! propagates to queued descendants; a running leaf moves to `Cancelling` until `complete` finishes it.
//!
//! The specification type `Spec` is defined by your domain (MAVLink steps, file paths, and so on), not by
//! this crate. The kernel only schedules and tracks status; it never interprets `Spec`.
//!
//! ## Example shape (domain-defined `Spec`)
//!
//! ```ignore
//! jobs.enqueue(JobGraph::Sequence(vec![
//!     JobGraph::Leaf(Spec::ReadOffsets),
//!     JobGraph::Leaf(Spec::FitEllipsoid),
//! ]));
//! ```
//!
//! Time limits are not handled here: the domain arms a timer via `Effect::Schedule` in `blueos_cqrs` when a
//! step needs a deadline.

#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use core::fmt;

use thiserror::Error;

/// Opaque identifier for a node in the job graph. Allocated sequentially from zero (`JobId(0)` is the first
/// enqueued root). Indices are never reused even when jobs finish.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct JobId(pub u64);

/// Lifecycle state of one node in the graph (leaf or composite).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobStatus {
    Queued,
    Running,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
}

/// Tree of work to enqueue. Composite nodes have no `Spec`; only leaves carry the domain payload.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JobGraph<Spec> {
    Leaf(Spec),
    Sequence(Vec<JobGraph<Spec>>),
    Parallel(Vec<JobGraph<Spec>>),
}

/// One row in [`JobsSnapshot`]: identity, optional leaf spec, parent link, and status.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobView<Spec> {
    pub job_id: JobId,
    pub parent: Option<JobId>,
    pub job_spec: Option<Spec>,
    pub status: JobStatus,
}

/// Point-in-time view of every node ever allocated in this [`Jobs`] instance (including finished jobs).
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct JobsSnapshot<Spec> {
    pub jobs: Vec<JobView<Spec>>,
}

/// Errors from invalid job identifiers or cancel requests on terminal jobs.
#[derive(Debug, Error)]
pub enum JobsError {
    #[error("unknown job {0:?}")]
    Unknown(JobId),
    #[error("cannot cancel {0:?} in status {1:?}")]
    NotCancellable(JobId, JobStatus),
}

/// In-memory job graph executor. Pure scheduling: no IO, clocks, or interpretation of `Spec`.
pub struct Jobs<Spec> {
    // ponytail: completed nodes stay in the vec for snapshot identity; compact if JobId space matters.
    nodes: Vec<Node<Spec>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Leaf,
    Sequence,
    Parallel,
}

struct Node<Spec> {
    parent: Option<JobId>,
    job_spec: Option<Spec>,
    children: Vec<JobId>,
    kind: Kind,
    status: JobStatus,
    sequence_next: usize,
}

impl<Spec> Default for Jobs<Spec> {
    fn default() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl<Spec> fmt::Debug for Jobs<Spec> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Jobs")
            .field("nodes", &self.nodes.len())
            .finish()
    }
}

impl<Spec> Jobs<Spec>
where
    Spec: Clone,
{
    /// Empty graph with no allocated job ids.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts `graph` and returns the root job id. Runnable leaves become eligible for
    /// [`poll_runnable`] immediately (subject to sequence ordering).
    pub fn enqueue(&mut self, graph: JobGraph<Spec>) -> JobId {
        let job_id = self.insert(graph, None);
        self.settle_from(job_id);
        job_id
    }

    /// Requests cancellation. Queued descendants are marked cancelled; a running leaf enters
    /// [`JobStatus::Cancelling`] until [`complete`].
    pub fn cancel(&mut self, job_id: JobId) -> Result<(), JobsError> {
        let status = self.get(job_id).ok_or(JobsError::Unknown(job_id))?.status;
        match status {
            JobStatus::Succeeded | JobStatus::Failed | JobStatus::Cancelled => {
                Err(JobsError::NotCancellable(job_id, status))
            }
            JobStatus::Cancelling => Ok(()),
            JobStatus::Queued | JobStatus::Running => {
                self.cancel_node(job_id);
                if Self::is_terminal(self.node(job_id).status)
                    && let Some(parent) = self.node(job_id).parent
                {
                    self.propagate(parent);
                }
                Ok(())
            }
        }
    }

    /// Current status of `job_id`, if it was ever allocated on this instance.
    pub fn status(&self, job_id: JobId) -> Option<JobStatus> {
        self.get(job_id).map(|node| node.status)
    }

    /// Snapshot suitable for publishing to clients (see D-12 `jobs` state).
    pub fn snapshot(&self) -> JobsSnapshot<Spec> {
        JobsSnapshot {
            jobs: self
                .nodes
                .iter()
                .enumerate()
                .map(|(index, node)| JobView {
                    job_id: JobId(index as u64),
                    parent: node.parent,
                    job_spec: node.job_spec.clone(),
                    status: node.status,
                })
                .collect(),
        }
    }

    /// Marks every currently runnable leaf as [`JobStatus::Running`] and returns its id and spec.
    /// Call once per scheduling round; the kernel turns each pair into an IO effect.
    pub fn poll_runnable(&mut self) -> Vec<(JobId, Spec)> {
        let job_ids: Vec<JobId> = (0..self.nodes.len())
            .map(|index| JobId(index as u64))
            .collect();
        let mut runnable = Vec::new();
        for job_id in job_ids {
            if !self.is_runnable_leaf(job_id) {
                continue;
            }
            let job_spec = self
                .node(job_id)
                .job_spec
                .clone()
                .expect("leaf job specification");
            self.node_mut(job_id).status = JobStatus::Running;
            self.mark_ancestors_running(job_id);
            runnable.push((job_id, job_spec));
        }
        runnable
    }

    /// Reports completion of a leaf started by [`poll_runnable`]. `succeeded == false` fails a sequence
    /// parent and can fail a parallel parent once all siblings finish.
    pub fn complete(&mut self, job_id: JobId, succeeded: bool) -> Result<(), JobsError> {
        let node = self.get(job_id).ok_or(JobsError::Unknown(job_id))?;
        if node.kind != Kind::Leaf {
            return Err(JobsError::Unknown(job_id));
        }
        match node.status {
            JobStatus::Cancelling => self.node_mut(job_id).status = JobStatus::Cancelled,
            JobStatus::Running => {
                self.node_mut(job_id).status = if succeeded {
                    JobStatus::Succeeded
                } else {
                    JobStatus::Failed
                };
            }
            JobStatus::Succeeded | JobStatus::Failed | JobStatus::Cancelled => return Ok(()),
            JobStatus::Queued => return Err(JobsError::Unknown(job_id)),
        }
        if let Some(parent) = self.node(job_id).parent {
            self.propagate(parent);
        }
        Ok(())
    }

    fn insert(&mut self, graph: JobGraph<Spec>, parent: Option<JobId>) -> JobId {
        let (kind, items) = match graph {
            JobGraph::Leaf(job_spec) => {
                return self.alloc(Node {
                    parent,
                    job_spec: Some(job_spec),
                    children: Vec::new(),
                    kind: Kind::Leaf,
                    status: JobStatus::Queued,
                    sequence_next: 0,
                });
            }
            JobGraph::Sequence(items) => (Kind::Sequence, items),
            JobGraph::Parallel(items) => (Kind::Parallel, items),
        };
        let job_id = self.alloc(Node {
            parent,
            job_spec: None,
            children: Vec::new(),
            kind,
            status: JobStatus::Queued,
            sequence_next: 0,
        });
        let children: Vec<JobId> = items
            .into_iter()
            .map(|item| self.insert(item, Some(job_id)))
            .collect();
        self.node_mut(job_id).children = children;
        job_id
    }

    fn alloc(&mut self, node: Node<Spec>) -> JobId {
        let job_id = JobId(self.nodes.len() as u64);
        self.nodes.push(node);
        job_id
    }

    fn get(&self, job_id: JobId) -> Option<&Node<Spec>> {
        self.nodes.get(job_id.0 as usize)
    }

    fn node(&self, job_id: JobId) -> &Node<Spec> {
        &self.nodes[job_id.0 as usize]
    }

    fn node_mut(&mut self, job_id: JobId) -> &mut Node<Spec> {
        &mut self.nodes[job_id.0 as usize]
    }

    fn is_terminal(status: JobStatus) -> bool {
        matches!(
            status,
            JobStatus::Succeeded | JobStatus::Failed | JobStatus::Cancelled
        )
    }

    fn settle_from(&mut self, job_id: JobId) {
        let children = self.node(job_id).children.clone();
        for child in children {
            self.settle_from(child);
        }
        self.propagate(job_id);
    }

    fn is_runnable_leaf(&self, job_id: JobId) -> bool {
        let node = self.node(job_id);
        if node.kind != Kind::Leaf || node.status != JobStatus::Queued {
            return false;
        }
        let mut child = job_id;
        let mut parent = node.parent;
        while let Some(parent_id) = parent {
            let parent_node = self.node(parent_id);
            if matches!(
                parent_node.status,
                JobStatus::Cancelling
                    | JobStatus::Succeeded
                    | JobStatus::Failed
                    | JobStatus::Cancelled
            ) {
                return false;
            }
            if parent_node.kind == Kind::Sequence {
                match parent_node.children.get(parent_node.sequence_next) {
                    Some(&current) if current == child => {}
                    _ => return false,
                }
            }
            child = parent_id;
            parent = parent_node.parent;
        }
        true
    }

    fn mark_ancestors_running(&mut self, mut job_id: JobId) {
        while let Some(parent_id) = self.node(job_id).parent {
            let node = self.node_mut(parent_id);
            if node.status == JobStatus::Queued {
                node.status = JobStatus::Running;
            }
            job_id = parent_id;
        }
    }

    fn propagate(&mut self, job_id: JobId) {
        if Self::is_terminal(self.node(job_id).status) {
            return;
        }
        match self.node(job_id).kind {
            Kind::Leaf => {}
            Kind::Sequence => loop {
                let sequence_next = self.node(job_id).sequence_next;
                let children = self.node(job_id).children.clone();
                if sequence_next >= children.len() {
                    self.finish_composite(job_id, JobStatus::Succeeded);
                    break;
                }
                match self.node(children[sequence_next]).status {
                    JobStatus::Succeeded => self.node_mut(job_id).sequence_next += 1,
                    JobStatus::Failed => {
                        self.cancel_later_siblings(job_id);
                        self.finish_composite(job_id, JobStatus::Failed);
                        break;
                    }
                    JobStatus::Cancelled => {
                        self.cancel_later_siblings(job_id);
                        self.finish_composite(job_id, JobStatus::Cancelled);
                        break;
                    }
                    JobStatus::Queued | JobStatus::Running | JobStatus::Cancelling => break,
                }
            },
            Kind::Parallel => {
                let children = self.node(job_id).children.clone();
                if children
                    .iter()
                    .any(|child| !Self::is_terminal(self.node(*child).status))
                {
                    return;
                }
                let status = if children
                    .iter()
                    .any(|child| self.node(*child).status == JobStatus::Failed)
                {
                    JobStatus::Failed
                } else if children
                    .iter()
                    .any(|child| self.node(*child).status == JobStatus::Cancelled)
                {
                    JobStatus::Cancelled
                } else {
                    JobStatus::Succeeded
                };
                self.finish_composite(job_id, status);
            }
        }
    }

    fn finish_composite(&mut self, job_id: JobId, status: JobStatus) {
        {
            let node = self.node_mut(job_id);
            if Self::is_terminal(node.status) {
                return;
            }
            node.status = status;
        }
        if let Some(parent) = self.node(job_id).parent {
            self.propagate(parent);
        }
    }

    fn cancel_later_siblings(&mut self, job_id: JobId) {
        let start = self.node(job_id).sequence_next.saturating_add(1);
        let later: Vec<JobId> = self
            .node(job_id)
            .children
            .get(start..)
            .unwrap_or(&[])
            .to_vec();
        for child in later {
            self.cancel_node(child);
        }
    }

    fn cancel_node(&mut self, job_id: JobId) {
        let status = self.node(job_id).status;
        if Self::is_terminal(status) || status == JobStatus::Cancelling {
            return;
        }
        if self.node(job_id).kind == Kind::Leaf {
            self.node_mut(job_id).status = if status == JobStatus::Running {
                JobStatus::Cancelling
            } else {
                JobStatus::Cancelled
            };
            return;
        }
        let children = self.node(job_id).children.clone();
        for child in &children {
            self.cancel_node(*child);
        }
        let cancelling = children
            .iter()
            .any(|child| self.node(*child).status == JobStatus::Cancelling);
        self.node_mut(job_id).status = if cancelling {
            JobStatus::Cancelling
        } else {
            JobStatus::Cancelled
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum TestJobSpec {
        Step(alloc::string::String),
    }

    fn leaf(name: &str) -> JobGraph<TestJobSpec> {
        JobGraph::Leaf(TestJobSpec::Step(name.to_string()))
    }

    fn names(jobs: &[(JobId, TestJobSpec)]) -> Vec<&str> {
        jobs.iter()
            .map(|(_job_id, job_spec)| match job_spec {
                TestJobSpec::Step(name) => name.as_str(),
            })
            .collect()
    }

    fn view(jobs: &Jobs<TestJobSpec>, name: &str) -> JobView<TestJobSpec> {
        jobs.snapshot()
            .jobs
            .into_iter()
            .find(|job| {
                job.job_spec.as_ref().map(|job_spec| match job_spec {
                    TestJobSpec::Step(step_name) => step_name.as_str(),
                }) == Some(name)
            })
            .unwrap()
    }

    #[test]
    fn leaf_enqueue_poll_complete() {
        let mut jobs = Jobs::new();
        let root = jobs.enqueue(leaf("read_offsets"));
        assert_eq!(jobs.status(root), Some(JobStatus::Queued));
        let runnable = jobs.poll_runnable();
        assert_eq!(names(&runnable), ["read_offsets"]);
        assert_eq!(jobs.status(root), Some(JobStatus::Running));
        assert!(jobs.poll_runnable().is_empty());
        jobs.complete(root, true).unwrap();
        assert_eq!(jobs.status(root), Some(JobStatus::Succeeded));
    }

    #[test]
    fn sequence_runs_one_child_then_succeeds() {
        let mut jobs = Jobs::new();
        let root = jobs.enqueue(JobGraph::Sequence(vec![leaf("a"), leaf("b")]));
        assert_eq!(names(&jobs.poll_runnable()), ["a"]);
        jobs.complete(view(&jobs, "a").job_id, true).unwrap();
        assert_eq!(names(&jobs.poll_runnable()), ["b"]);
        jobs.complete(view(&jobs, "b").job_id, true).unwrap();
        assert_eq!(jobs.status(root), Some(JobStatus::Succeeded));
        assert!(jobs.poll_runnable().is_empty());
    }

    #[test]
    fn sequence_failure_stops_the_rest() {
        let mut jobs = Jobs::new();
        let root = jobs.enqueue(JobGraph::Sequence(vec![leaf("a"), leaf("b")]));
        jobs.poll_runnable();
        jobs.complete(view(&jobs, "a").job_id, false).unwrap();
        assert_eq!(jobs.status(root), Some(JobStatus::Failed));
        assert_eq!(view(&jobs, "b").status, JobStatus::Cancelled);
        assert!(jobs.poll_runnable().is_empty());
    }

    #[test]
    fn parallel_all_runnable_failure_lets_sibling_finish() {
        let mut jobs = Jobs::new();
        let root = jobs.enqueue(JobGraph::Parallel(vec![leaf("a"), leaf("b")]));
        let runnable = jobs.poll_runnable();
        assert_eq!(names(&runnable), ["a", "b"]);
        jobs.complete(runnable[0].0, false).unwrap();
        assert_eq!(jobs.status(root), Some(JobStatus::Running));
        jobs.complete(runnable[1].0, true).unwrap();
        assert_eq!(jobs.status(root), Some(JobStatus::Failed));
    }

    #[test]
    fn cancel_running_leaf_waits_then_cancels_descendants() {
        let mut jobs = Jobs::new();
        let root = jobs.enqueue(JobGraph::Sequence(vec![leaf("a"), leaf("b")]));
        let runnable = jobs.poll_runnable();
        jobs.cancel(root).unwrap();
        assert_eq!(jobs.status(runnable[0].0), Some(JobStatus::Cancelling));
        assert_eq!(jobs.status(root), Some(JobStatus::Cancelling));
        assert_eq!(view(&jobs, "b").status, JobStatus::Cancelled);
        jobs.complete(runnable[0].0, true).unwrap();
        assert_eq!(jobs.status(runnable[0].0), Some(JobStatus::Cancelled));
        assert_eq!(jobs.status(root), Some(JobStatus::Cancelled));
        assert!(matches!(
            jobs.cancel(root),
            Err(JobsError::NotCancellable(_, JobStatus::Cancelled))
        ));
    }

    #[test]
    fn nested_sequence_then_next_graph() {
        let mut jobs = Jobs::new();
        let gyro = JobGraph::Sequence(vec![leaf("start"), leaf("ack"), leaf("read")]);
        let baro = JobGraph::Sequence(vec![leaf("start"), leaf("ack"), leaf("read")]);
        let root = jobs.enqueue(JobGraph::Sequence(vec![gyro, baro]));
        let mut seen = Vec::new();
        loop {
            let runnable = jobs.poll_runnable();
            if runnable.is_empty() {
                break;
            }
            for (job_id, job_spec) in runnable {
                seen.push(match job_spec {
                    TestJobSpec::Step(name) => name,
                });
                jobs.complete(job_id, true).unwrap();
            }
        }
        assert_eq!(
            seen,
            [
                "start".to_string(),
                "ack".to_string(),
                "read".to_string(),
                "start".to_string(),
                "ack".to_string(),
                "read".to_string(),
            ]
        );
        assert_eq!(jobs.status(root), Some(JobStatus::Succeeded));
    }

    #[test]
    fn unknown_job_errors() {
        let mut jobs = Jobs::<TestJobSpec>::new();
        let missing = JobId(9);
        assert!(matches!(jobs.cancel(missing), Err(JobsError::Unknown(_))));
        assert!(matches!(
            jobs.complete(missing, true),
            Err(JobsError::Unknown(_))
        ));
        assert_eq!(jobs.status(missing), None);
    }
}
