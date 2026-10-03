//! The Jobs of a Service: the work clients submit and watch, with its lifecycle and the controls they send (D-36).
//!
//! The Kernel keeps one [`Jobs`] table per Service: it submits a Job for every client Request and applies every
//! control to it. A Domain whose Job types last beyond the step that accepts them keeps the table in its Snapshot
//! through [`DomainJobs`], so a rejected Command rolls it back with the rest of the DomainState, follows the status
//! the controls set (D-27), and reports how each Job ended with [`Jobs::end`].

#![no_std]

extern crate alloc;

use alloc::{
    borrow::ToOwned,
    collections::VecDeque,
    string::{String, ToString},
    vec::Vec,
};
use core::{fmt, str::FromStr};

use blueos_domain::Domain;

/// How many ended Jobs [`Jobs::default`] keeps, so clients see how they ended and a retry finds them.
pub const DEFAULT_RETENTION: usize = 16;

/// The reason of a Job that was running when the Service restarted (D-28).
const INTERRUPTED: &str = "interrupted";
/// The reason of a Job whose permission request was denied.
const PERMISSION_DENIED: &str = "permission denied";

/// The Jobs of a Service: every active Job in the order it was submitted, and the last few that ended.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Jobs {
    retention: usize,
    active: Vec<Job>,
    /// The ended Jobs in the order they ended, at most `retention` of them.
    finished: VecDeque<Job>,
}

/// Why [`Jobs`] refused a submit, a control or an end. Its text is the reason a client gets.
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum JobsError {
    /// No Job in the table has this id.
    #[error("there is no Job {0}")]
    Unknown(JobId),
    /// A Job with this id exists with another type or Goal.
    #[error("id reused")]
    IdReused(JobId),
    /// The nature of the Job type does not allow the control.
    #[error("{job_type} does not allow {control}")]
    NotAllowed {
        /// The Job type.
        job_type: String,
        /// The control it does not allow.
        control: JobControl,
    },
    /// The control does not apply to a Job in this status.
    #[error("{control} does not apply to the Job {job_id}, which is {status:?}")]
    Refused {
        /// The Job.
        job_id: JobId,
        /// The control.
        control: JobControl,
        /// The status of the Job, which the control left unchanged.
        status: JobStatus,
    },
    /// The Job has already ended.
    #[error("the Job {0} has already ended")]
    AlreadyEnded(JobId),
}

/// One Job in the table.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Job {
    /// The id the client generated.
    pub job_id: JobId,
    /// The Job type, the name of its submit endpoint.
    pub job_type: String,
    /// The Goal as the client encoded it, so a retry is told from a reused id.
    pub goal: Vec<u8>,
    /// What the Job type allows.
    pub nature: JobNature,
    /// Where the Job is in its lifecycle.
    pub status: JobStatus,
    /// Why the Job was canceled or aborted, or empty.
    pub reason: String,
}

/// Identifies a Job: the UUID the client generated when it submitted it, written as text on the wire.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(try_from = "String", into = "String")
)]
pub struct JobId(u128);

/// The text is not a UUID such as `0b5e8f5c-6f0a-4c4e-9a52-2f1e7d3c9b10`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("the Job id is not a UUID")]
pub struct InvalidJobId;

/// What a Job type allows, declared in code with the Job type (D-36). [`JobNature::INSTANT`] allows nothing, and a
/// declaration sets what its Job type allows on top of it.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JobNature {
    /// The Job runs until its Domain ends it with [`Jobs::end`]. Otherwise it succeeds in the step that executes
    /// it.
    pub lasting: bool,
    /// A client may cancel it.
    pub cancellable: bool,
    /// A client may pause and resume it.
    pub pausable: bool,
    /// It waits for a client to grant permission before it executes.
    pub needs_permission: bool,
}

/// Where a Job is in its lifecycle: a ROS 2 action goal status, plus pause and the two waits (D-36).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JobStatus {
    /// Accepted and not yet waiting or executing.
    Accepted,
    /// Waiting for a client to answer its permission request.
    WaitingForPermission,
    /// Waiting for the resources it needs (D-37).
    WaitingForResource,
    /// Its Domain is doing its work.
    Executing,
    /// A client paused it; its Domain holds the work until it is resumed.
    Paused,
    /// A client cancelled it; its Domain stops the work and ends it.
    Canceling,
    /// It did its work. It never changes again.
    Succeeded,
    /// It stopped because a client cancelled it or denied its permission. It never changes again.
    Canceled,
    /// It stopped on an error. It never changes again.
    Aborted,
}

/// How a Job ended.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobEnd {
    /// It did its work.
    Succeeded,
    /// It stopped because it was cancelled.
    Canceled,
    /// It stopped on an error.
    Aborted,
}

/// What a client asks of an existing Job. Its text is the name of its endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JobControl {
    /// Stop the Job.
    Cancel,
    /// Hold the Job until it is resumed.
    Pause,
    /// Continue a paused Job.
    Resume,
    /// Answer the Job's permission request.
    AnswerPermission {
        /// True lets the Job execute; false cancels it.
        granted: bool,
    },
}

/// Whether [`Jobs::submit`] added a Job or found the one a retry names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Submitted {
    /// A new Job, waiting for permission or executing.
    New,
    /// The Job with this id and Goal already exists, and nothing changed.
    Retry,
}

/// A Domain whose Job types last beyond the step that accepts them. It keeps its [`Jobs`] in its Snapshot, so the
/// Projections its Tasks follow see the status the controls set, and it ends each Job with [`Jobs::end`].
pub trait DomainJobs: Domain {
    /// The Jobs in the Snapshot.
    fn jobs(snapshot: &Self::Snapshot) -> &Jobs;

    /// The Jobs in the Snapshot, for the Kernel to submit and control them.
    fn jobs_mut(snapshot: &mut Self::Snapshot) -> &mut Jobs;
}

impl Default for Jobs {
    /// No Jobs, keeping the last [`DEFAULT_RETENTION`] ended ones.
    fn default() -> Self {
        Self::with_retention(DEFAULT_RETENTION)
    }
}

impl Jobs {
    /// No Jobs, keeping the last `retention` ended ones. An active Job is always kept.
    pub const fn with_retention(retention: usize) -> Self {
        Self {
            retention,
            active: Vec::new(),
            finished: VecDeque::new(),
        }
    }

    /// Adds the Job a client submitted, waiting for permission when its nature needs it and executing otherwise.
    /// A Job already in the table with the same type and Goal is a retry and changes nothing.
    ///
    /// # Errors
    ///
    /// [`JobsError::IdReused`] when a Job with this id has another type or Goal.
    pub fn submit(
        &mut self,
        job_id: JobId,
        job_type: &str,
        goal: &[u8],
        nature: JobNature,
    ) -> Result<Submitted, JobsError> {
        if let Some(job) = self.job(job_id) {
            return if job.job_type == job_type && job.goal == goal {
                Ok(Submitted::Retry)
            } else {
                Err(JobsError::IdReused(job_id))
            };
        }
        self.active.push(Job {
            job_id,
            job_type: job_type.to_owned(),
            goal: goal.to_owned(),
            nature,
            status: if nature.needs_permission {
                JobStatus::WaitingForPermission
            } else {
                JobStatus::Executing
            },
            reason: String::new(),
        });
        Ok(Submitted::New)
    }

    /// Applies a control a client sent and returns the status it left the Job in. Cancelling a Job that has not
    /// executed yet, or denying its permission, ends it at once; cancelling an executing or paused Job leaves it
    /// Canceling until its Domain ends it.
    ///
    /// # Errors
    ///
    /// [`JobsError::Unknown`] when no Job has this id, [`JobsError::NotAllowed`] when the Job type does not allow
    /// the control, and [`JobsError::Refused`] when the control does not apply to the Job's status.
    pub fn control(&mut self, job_id: JobId, control: JobControl) -> Result<JobStatus, JobsError> {
        let job = self.job(job_id).ok_or(JobsError::Unknown(job_id))?;
        if !job.nature.allows(control) {
            return Err(JobsError::NotAllowed {
                job_type: job.job_type.clone(),
                control,
            });
        }
        let status = job.status;
        let (next, reason) = match (control, status) {
            (
                JobControl::Cancel,
                JobStatus::Accepted
                | JobStatus::WaitingForPermission
                | JobStatus::WaitingForResource,
            ) => (JobStatus::Canceled, ""),
            (
                JobControl::Cancel,
                JobStatus::Executing | JobStatus::Paused | JobStatus::Canceling,
            ) => (JobStatus::Canceling, ""),
            (JobControl::Pause, JobStatus::Executing | JobStatus::Paused) => {
                (JobStatus::Paused, "")
            }
            (JobControl::Resume, JobStatus::Executing | JobStatus::Paused) => {
                (JobStatus::Executing, "")
            }
            (JobControl::AnswerPermission { granted: true }, JobStatus::WaitingForPermission) => {
                (JobStatus::Executing, "")
            }
            (JobControl::AnswerPermission { granted: false }, JobStatus::WaitingForPermission) => {
                (JobStatus::Canceled, PERMISSION_DENIED)
            }
            _ => {
                return Err(JobsError::Refused {
                    job_id,
                    control,
                    status,
                });
            }
        };
        self.set(job_id, next, reason);
        Ok(next)
    }

    /// Ends an active Job with `reason`, which clients see when it is canceled or aborted.
    ///
    /// # Errors
    ///
    /// [`JobsError::Unknown`] when no Job has this id, and [`JobsError::AlreadyEnded`] when it has ended.
    pub fn end(&mut self, job_id: JobId, end: JobEnd, reason: &str) -> Result<(), JobsError> {
        match self.job(job_id) {
            None => Err(JobsError::Unknown(job_id)),
            Some(job) if job.status.has_ended() => Err(JobsError::AlreadyEnded(job_id)),
            Some(_) => {
                let status = match end {
                    JobEnd::Succeeded => JobStatus::Succeeded,
                    JobEnd::Canceled => JobStatus::Canceled,
                    JobEnd::Aborted => JobStatus::Aborted,
                };
                self.set(job_id, status, reason);
                Ok(())
            }
        }
    }

    /// Aborts every Job that was executing, paused or canceling, with the reason "interrupted", for restore after
    /// a restart (D-28). Jobs that had not executed yet keep waiting.
    pub fn interrupt(&mut self) {
        let interrupted: Vec<JobId> = self
            .active
            .iter()
            .filter(|job| {
                matches!(
                    job.status,
                    JobStatus::Executing | JobStatus::Paused | JobStatus::Canceling
                )
            })
            .map(|job| job.job_id)
            .collect();
        for job_id in interrupted {
            self.set(job_id, JobStatus::Aborted, INTERRUPTED);
        }
    }

    /// The Job with this id, or `None` when it is not in the table or has left the history.
    pub fn job(&self, job_id: JobId) -> Option<&Job> {
        self.list().find(|job| job.job_id == job_id)
    }

    /// Every Job, as the Kernel publishes them in the `jobs` State: the active ones in the order they were
    /// submitted, then the retained ended ones in the order they ended.
    pub fn list(&self) -> impl Iterator<Item = &Job> {
        self.active.iter().chain(&self.finished)
    }

    /// Changes the status of an active Job, and moves it to the history when the status ends it.
    fn set(&mut self, job_id: JobId, status: JobStatus, reason: &str) {
        let Some(index) = self.active.iter().position(|job| job.job_id == job_id) else {
            return;
        };
        let job = &mut self.active[index];
        job.status = status;
        reason.clone_into(&mut job.reason);
        if status.has_ended() {
            self.finished.push_back(self.active.remove(index));
            let excess = self.finished.len().saturating_sub(self.retention);
            self.finished.drain(..excess);
        }
    }
}

impl fmt::Display for JobId {
    /// The UUID in its hyphenated lowercase form.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = self.0;
        write!(
            formatter,
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            value >> 96,
            (value >> 80) & 0xffff,
            (value >> 64) & 0xffff,
            (value >> 48) & 0xffff,
            value & 0xffff_ffff_ffff,
        )
    }
}

impl FromStr for JobId {
    type Err = InvalidJobId;

    /// Parses the hyphenated form of a UUID, in either case.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        if text.len() != 36 {
            return Err(InvalidJobId);
        }
        let mut value = 0_u128;
        for (index, character) in text.chars().enumerate() {
            if matches!(index, 8 | 13 | 18 | 23) {
                if character != '-' {
                    return Err(InvalidJobId);
                }
                continue;
            }
            let digit = character.to_digit(16).ok_or(InvalidJobId)?;
            value = (value << 4) | u128::from(digit);
        }
        Ok(Self(value))
    }
}

impl TryFrom<String> for JobId {
    type Error = InvalidJobId;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        text.parse()
    }
}

impl JobId {
    /// The id whose UUID is the 128 bits of `value`.
    pub const fn from_u128(value: u128) -> Self {
        Self(value)
    }
}

impl From<JobId> for String {
    fn from(job_id: JobId) -> Self {
        job_id.to_string()
    }
}

impl JobNature {
    /// A Job that succeeds in the step that executes it, and that a client cannot control.
    pub const INSTANT: Self = Self {
        lasting: false,
        cancellable: false,
        pausable: false,
        needs_permission: false,
    };

    const fn allows(self, control: JobControl) -> bool {
        match control {
            JobControl::Cancel => self.cancellable,
            JobControl::Pause | JobControl::Resume => self.pausable,
            JobControl::AnswerPermission { .. } => self.needs_permission,
        }
    }
}

impl JobStatus {
    /// Whether the Job has ended and never changes again.
    pub const fn has_ended(self) -> bool {
        matches!(self, Self::Succeeded | Self::Canceled | Self::Aborted)
    }
}

impl fmt::Display for JobControl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Cancel => "CancelJob",
            Self::Pause => "PauseJob",
            Self::Resume => "ResumeJob",
            Self::AnswerPermission { .. } => "AnswerPermission",
        })
    }
}
