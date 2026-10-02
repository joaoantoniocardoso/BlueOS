//! Multi-step work that a Domain runs and clients watch: Jobs composed in sequence or in parallel.
//!
//! A Domain keeps its [`Jobs`] in its Snapshot, so a rejected Command rolls them back with the rest of the
//! DomainState. It starts a [`JobGraph`] from `handle`, turns every [`LeafJob`] that [`Jobs::start`] or
//! [`Jobs::finish`] returns into an Effect, and reports how each one ended with [`Jobs::finish`].

#![no_std]

extern crate alloc;

use alloc::{collections::VecDeque, vec::Vec};
use core::{fmt::Display, num::NonZeroU64};

use blueos_domain::Domain;

/// How many finished root Jobs [`Jobs::default`] keeps for clients to see how they ended.
pub const DEFAULT_RETENTION: usize = 16;

/// The work of a Job, as a Domain starts it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum JobGraph<Step> {
    /// One step the Domain runs, usually as IO.
    Leaf(Step),
    /// Runs its Jobs one after another. The first one that does not succeed ends the Sequence and cancels the rest.
    Sequence(Vec<JobGraph<Step>>),
    /// Runs its Jobs at once and finishes when every one of them has finished.
    Parallel(Vec<JobGraph<Step>>),
}

/// The Jobs of a Domain: every root Job that has not finished, and the last few that have.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Jobs<Step> {
    next_id: JobId,
    latest_root: Option<JobId>,
    retention: usize,
    /// The root Jobs that have not finished, in the order they started.
    live: Vec<Node<Step>>,
    /// The root Jobs that have finished, in the order they finished, at most `retention` of them.
    finished: VecDeque<Node<Step>>,
}

/// A root Job that [`Jobs::start`] started.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Started<Step> {
    /// The root Job, which a Command acknowledges.
    pub job_id: JobId,
    /// The leaves that are now running, for the Domain to run.
    pub leaves: Vec<LeafJob<Step>>,
}

/// A leaf Job and the step it runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeafJob<Step> {
    /// The leaf, which the Domain reports to [`Jobs::finish`].
    pub job_id: JobId,
    /// What the leaf does.
    pub step: Step,
}

/// One Job as clients see it in the `jobs` State.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct JobView<'jobs, Step> {
    /// The Job.
    pub job_id: JobId,
    /// The Sequence or Parallel the Job is part of, or `None` for a root Job.
    pub parent: Option<JobId>,
    /// Where the Job is in its life.
    pub status: JobStatus,
    /// What the Job is.
    pub kind: JobKind<'jobs, Step>,
}

/// What a Job is: a step, or a composition of Jobs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobKind<'jobs, Step> {
    /// A leaf, which runs this step.
    Leaf(&'jobs Step),
    /// A [`JobGraph::Sequence`].
    Sequence,
    /// A [`JobGraph::Parallel`].
    Parallel,
}

/// Where a Job is in its life. The status of a Sequence or a Parallel follows from the Jobs in it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JobStatus {
    /// Waiting for an earlier Job of a Sequence.
    Queued,
    /// Its steps are being run.
    Running,
    /// It was cancelled while a step was running, and that step has not reported how it ended yet.
    Cancelling,
    /// Its step was running when the Service restarted; the Domain decides what to do next.
    Interrupted,
    /// It has ended and never changes again.
    Finished(JobEnd),
}

/// How a finished Job ended. A Sequence or a Parallel failed when any Job in it failed, else was cancelled when any
/// was cancelled, else succeeded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JobEnd {
    /// It did all its work.
    Succeeded,
    /// It stopped on an error.
    Failed,
    /// It stopped because it was cancelled, or never ran because an earlier step of its Sequence did not succeed.
    Cancelled,
}

/// Why [`Jobs`] refused to change a Job.
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum JobsError {
    /// No Job has this id.
    #[error("there is no Job {0:?}")]
    Unknown(JobId),
    /// The Job has already finished.
    #[error("the Job {0:?} has already finished")]
    Finished(JobId),
    /// The Job is not a leaf that is running or cancelling, so it cannot finish.
    #[error("the Job {0:?} is not a running leaf")]
    NotRunning(JobId),
}

/// Identifies a Job. Ids start at 1 and are never reused, so 0 means "no Job" on the wire.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JobId(NonZeroU64);

/// A Domain whose Commands start Jobs. It keeps its [`Jobs`] in its Snapshot, and the Kernel publishes them as the
/// `jobs` State and acknowledges a Command with the root Job it started. A Domain without Jobs does not implement
/// it and publishes no `jobs` State.
pub trait DomainJobs: Domain {
    /// What one step of a Job does. Its text names the leaf in the `jobs` State.
    type Step: Clone + Display;

    /// The Jobs in the Snapshot.
    fn jobs(snapshot: &Self::Snapshot) -> &Jobs<Self::Step>;

    /// The Jobs in the Snapshot, for restore after a restart.
    fn jobs_mut(snapshot: &mut Self::Snapshot) -> &mut Jobs<Self::Step>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct Node<Step> {
    job_id: JobId,
    work: Work<Step>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
enum Work<Step> {
    Leaf { step: Step, status: JobStatus },
    Sequence(Vec<Node<Step>>),
    Parallel(Vec<Node<Step>>),
}

impl<Step> Default for Jobs<Step> {
    /// No Jobs, keeping the last [`DEFAULT_RETENTION`] finished root Jobs.
    fn default() -> Self {
        Self::with_retention(DEFAULT_RETENTION)
    }
}

impl<Step: Clone> Jobs<Step> {
    /// Starts `graph` as a new root Job and returns its id with the leaves that are now running.
    pub fn start(&mut self, graph: JobGraph<Step>) -> Started<Step> {
        let mut root = self.build(graph);
        let mut leaves = Vec::new();
        root.run(&mut leaves);
        let job_id = root.job_id;
        self.latest_root = Some(job_id);
        self.live.push(root);
        self.settle(self.live.len() - 1);
        Started { job_id, leaves }
    }

    /// Records how a running, cancelling, or interrupted leaf ended and returns the leaves that this starts.
    ///
    /// # Errors
    ///
    /// [`JobsError::Unknown`] when no Job has this id, and [`JobsError::NotRunning`] when the Job is not a leaf
    /// that is running, cancelling, or interrupted.
    pub fn finish(&mut self, job_id: JobId, end: JobEnd) -> Result<Vec<LeafJob<Step>>, JobsError> {
        let Some(index) = self
            .live
            .iter()
            .position(|root| root.find(job_id).is_some())
        else {
            return Err(match self.status(job_id) {
                Some(_) => JobsError::NotRunning(job_id),
                None => JobsError::Unknown(job_id),
            });
        };
        let root = &mut self.live[index];
        match root.find_mut(job_id).map(|job| &mut job.work) {
            Some(Work::Leaf {
                status:
                    status @ (JobStatus::Running | JobStatus::Cancelling | JobStatus::Interrupted),
                ..
            }) => *status = JobStatus::Finished(end),
            _ => return Err(JobsError::NotRunning(job_id)),
        }
        let mut leaves = Vec::new();
        root.run(&mut leaves);
        self.settle(index);
        Ok(leaves)
    }

    /// Turns an interrupted leaf back into a running one and returns it for the Domain to run again.
    ///
    /// # Errors
    ///
    /// [`JobsError::Unknown`] when no Job has this id, and [`JobsError::NotRunning`] when the Job is not an
    /// interrupted leaf.
    pub fn retry(&mut self, job_id: JobId) -> Result<LeafJob<Step>, JobsError> {
        let Some(index) = self
            .live
            .iter()
            .position(|root| root.find(job_id).is_some())
        else {
            return Err(match self.status(job_id) {
                Some(_) => JobsError::NotRunning(job_id),
                None => JobsError::Unknown(job_id),
            });
        };
        let root = &mut self.live[index];
        match root.find_mut(job_id).map(|job| &mut job.work) {
            Some(Work::Leaf {
                status: status @ JobStatus::Interrupted,
                step,
            }) => {
                *status = JobStatus::Running;
                Ok(LeafJob {
                    job_id,
                    step: step.clone(),
                })
            }
            _ => Err(JobsError::NotRunning(job_id)),
        }
    }

    /// Cancels a Job and every Job in it. A queued leaf is cancelled at once; a running leaf becomes Cancelling
    /// until the Domain reports how it ended. Returns the leaves that became Cancelling, for the Domain to stop.
    ///
    /// # Errors
    ///
    /// [`JobsError::Unknown`] when no Job has this id, and [`JobsError::Finished`] when the Job has finished.
    pub fn cancel(&mut self, job_id: JobId) -> Result<Vec<LeafJob<Step>>, JobsError> {
        let Some(job) = self.live.iter_mut().find_map(|root| root.find_mut(job_id)) else {
            return Err(match self.status(job_id) {
                Some(_) => JobsError::Finished(job_id),
                None => JobsError::Unknown(job_id),
            });
        };
        if let JobStatus::Finished(_) = job.status() {
            return Err(JobsError::Finished(job_id));
        }
        // A live root always has a running leaf, so cancelling never finishes it here.
        let mut cancelling = Vec::new();
        job.cancel(&mut cancelling);
        Ok(cancelling)
    }

    fn build(&mut self, graph: JobGraph<Step>) -> Node<Step> {
        let job_id = self.next_id;
        self.next_id = JobId(job_id.0.saturating_add(1));
        let work = match graph {
            JobGraph::Leaf(step) => Work::Leaf {
                step,
                status: JobStatus::Queued,
            },
            JobGraph::Sequence(graphs) => {
                Work::Sequence(graphs.into_iter().map(|child| self.build(child)).collect())
            }
            JobGraph::Parallel(graphs) => {
                Work::Parallel(graphs.into_iter().map(|child| self.build(child)).collect())
            }
        };
        Node { job_id, work }
    }
}

impl<Step> Jobs<Step> {
    /// No Jobs, keeping the last `retention` finished root Jobs. A root Job that has not finished is always kept.
    pub const fn with_retention(retention: usize) -> Self {
        Self {
            next_id: JobId(NonZeroU64::MIN),
            latest_root: None,
            retention,
            live: Vec::new(),
            finished: VecDeque::new(),
        }
    }

    /// The root Job started last, even if it has been dropped from the history. The Kernel acknowledges a Command
    /// with it when the Command changed it.
    pub const fn latest_root(&self) -> Option<JobId> {
        self.latest_root
    }

    /// The status of a Job, or `None` when no Job has this id or its root was dropped from the history.
    pub fn status(&self, job_id: JobId) -> Option<JobStatus> {
        self.roots()
            .find_map(|root| root.find(job_id))
            .map(Node::status)
    }

    /// Marks every running or cancelling leaf Interrupted, for restore after a restart (D-28).
    pub fn interrupt_running_leaves(&mut self) {
        for root in &mut self.live {
            root.interrupt_running_leaves();
        }
    }

    /// Every Job, as the Kernel publishes them in the `jobs` State: the root Jobs that have not finished in the
    /// order they started, then the finished ones in the order they finished, each followed by the Jobs in it.
    pub fn list(&self) -> Vec<JobView<'_, Step>> {
        let mut views = Vec::new();
        for root in self.roots() {
            root.view(None, &mut views);
        }
        views
    }

    fn roots(&self) -> impl Iterator<Item = &Node<Step>> {
        self.live.iter().chain(&self.finished)
    }

    /// Moves the live root at `index` to the history once it has finished, and drops the oldest finished roots
    /// beyond the retention count.
    fn settle(&mut self, index: usize) {
        if let JobStatus::Finished(_) = self.live[index].status() {
            self.finished.push_back(self.live.remove(index));
            let excess = self.finished.len().saturating_sub(self.retention);
            self.finished.drain(..excess);
        }
    }
}

impl JobId {
    /// The Job id `raw` from the wire, or `None` for 0, which means "no Job".
    pub const fn new(raw: u64) -> Option<Self> {
        match NonZeroU64::new(raw) {
            Some(raw) => Some(Self(raw)),
            None => None,
        }
    }

    /// The id as a number for the wire, never 0.
    pub const fn get(self) -> u64 {
        self.0.get()
    }
}

impl<Step> Node<Step> {
    fn status(&self) -> JobStatus {
        let children = match &self.work {
            Work::Leaf { status, .. } => return *status,
            Work::Sequence(children) | Work::Parallel(children) => children,
        };
        let statuses: Vec<JobStatus> = children.iter().map(Self::status).collect();
        if statuses
            .iter()
            .all(|status| matches!(status, JobStatus::Finished(_)))
        {
            let end = [JobEnd::Failed, JobEnd::Cancelled]
                .into_iter()
                .find(|end| statuses.contains(&JobStatus::Finished(*end)))
                .unwrap_or(JobEnd::Succeeded);
            JobStatus::Finished(end)
        } else if statuses.contains(&JobStatus::Cancelling) {
            JobStatus::Cancelling
        } else if statuses.contains(&JobStatus::Interrupted) {
            JobStatus::Interrupted
        } else if statuses.iter().all(|status| *status == JobStatus::Queued) {
            JobStatus::Queued
        } else {
            JobStatus::Running
        }
    }

    // ponytail: finding a Job walks every graph, which is fine for the few Jobs a Domain runs at once; index the
    // nodes by JobId if a Domain ever keeps thousands.
    fn find(&self, job_id: JobId) -> Option<&Self> {
        if self.job_id == job_id {
            return Some(self);
        }
        match &self.work {
            Work::Leaf { .. } => None,
            Work::Sequence(children) | Work::Parallel(children) => {
                children.iter().find_map(|child| child.find(job_id))
            }
        }
    }

    fn find_mut(&mut self, job_id: JobId) -> Option<&mut Self> {
        if self.job_id == job_id {
            return Some(self);
        }
        match &mut self.work {
            Work::Leaf { .. } => None,
            Work::Sequence(children) | Work::Parallel(children) => {
                children.iter_mut().find_map(|child| child.find_mut(job_id))
            }
        }
    }

    fn view<'jobs>(&'jobs self, parent: Option<JobId>, views: &mut Vec<JobView<'jobs, Step>>) {
        let (kind, children) = match &self.work {
            Work::Leaf { step, .. } => (JobKind::Leaf(step), &[][..]),
            Work::Sequence(children) => (JobKind::Sequence, children.as_slice()),
            Work::Parallel(children) => (JobKind::Parallel, children.as_slice()),
        };
        views.push(JobView {
            job_id: self.job_id,
            parent,
            status: self.status(),
            kind,
        });
        for child in children {
            child.view(Some(self.job_id), views);
        }
    }

    fn interrupt_running_leaves(&mut self) {
        match &mut self.work {
            Work::Leaf { status, .. } => {
                if matches!(*status, JobStatus::Running | JobStatus::Cancelling) {
                    *status = JobStatus::Interrupted;
                }
            }
            Work::Sequence(children) | Work::Parallel(children) => {
                for child in children {
                    child.interrupt_running_leaves();
                }
            }
        }
    }
}

impl<Step: Clone> Node<Step> {
    /// Starts every queued leaf this Job may run now: all of a Parallel, and the first unfinished Job of a Sequence.
    fn run(&mut self, started: &mut Vec<LeafJob<Step>>) {
        match &mut self.work {
            Work::Leaf { step, status } => {
                if *status == JobStatus::Queued {
                    *status = JobStatus::Running;
                    started.push(LeafJob {
                        job_id: self.job_id,
                        step: step.clone(),
                    });
                }
            }
            Work::Parallel(children) => {
                for child in children {
                    child.run(started);
                }
            }
            Work::Sequence(children) => {
                let mut rest = children.iter_mut();
                while let Some(child) = rest.next() {
                    child.run(started);
                    match child.status() {
                        JobStatus::Finished(JobEnd::Succeeded) => {}
                        JobStatus::Finished(JobEnd::Failed | JobEnd::Cancelled) => {
                            // The Jobs after it never started, so none of their leaves is running.
                            rest.for_each(|later| later.cancel(&mut Vec::new()));
                            break;
                        }
                        JobStatus::Queued
                        | JobStatus::Running
                        | JobStatus::Cancelling
                        | JobStatus::Interrupted => break,
                    }
                }
            }
        }
    }

    /// Cancels every queued leaf of this Job, and marks every running one Cancelling and adds it to `cancelling`.
    fn cancel(&mut self, cancelling: &mut Vec<LeafJob<Step>>) {
        match &mut self.work {
            Work::Leaf { step, status } => match status {
                JobStatus::Queued => *status = JobStatus::Finished(JobEnd::Cancelled),
                JobStatus::Running => {
                    *status = JobStatus::Cancelling;
                    cancelling.push(LeafJob {
                        job_id: self.job_id,
                        step: step.clone(),
                    });
                }
                JobStatus::Cancelling | JobStatus::Interrupted | JobStatus::Finished(_) => {}
            },
            Work::Sequence(children) | Work::Parallel(children) => {
                for child in children {
                    child.cancel(cancelling);
                }
            }
        }
    }
}
