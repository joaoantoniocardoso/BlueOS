//! The Disk Usage domain: TODO

#![no_std]

extern crate alloc;

use core::convert::Infallible;

use blueos_domain::{Domain, IoError};
use blueos_jobs::{DomainJobs, JobId, Jobs};

/// The Disk Usage domain
pub struct DiskUsageDomain;

/// Root Snapshot composed from Blocks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DiskUsageSnapshot {
    /// Long-running Jobs.
    pub jobs: Jobs,
}

/// Client Commands for the DiskUsage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiskUsageRequests {
    /// The Job type on blueos/v1/disk_usage/command/ComputeTargetUsage: compute disk usage for a target folder recursively.
    ComputeTargetUsage {
        /// The Job the compute disk usage runs as.
        job_id: JobId,
        /// The target (file or folder) to inspect.
        target: alloc::string::String,
        /// The maximum depth to inspect.
        depth: u8,
        /// Whether to include files in the output.
        include_files: bool,
        /// Filter out entries smaller than this size (except for the root).
        min_size_bytes: u64,
    },
    /// The Job type on blueos/v1/disk_usage/command/DeleteTarget: deletes a target.
    /// Rejected when the target is protected, not found, outside root, is already being deleted by another Job.
    /// The Job succeeds once the target is gone.
    DeleteTarget {
        /// The Job the delete target runs as.
        job_id: JobId,
        /// The target the delete Job executed. Why it failed is the reason of the Job.
        target: alloc::string::String,
    },
}

/// IO requests scheduled by the DiskUsage Domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiskUsageIoRequest {
    /// Inspect target disk usage
    Inspect {
        /// The Job the compute disk usage runs as.
        job_id: JobId,
        /// The target (file or folder) to inspect.
        target: alloc::string::String,
        /// The maximum depth to inspect.
        depth: u8,
        /// Whether to include files in the output.
        include_files: bool,
        /// Filter out entries smaller than this size (except for the root).
        min_size_bytes: u64,
    },
    /// Delete a target
    Delete {
        /// The Job the delete target runs as.
        job_id: JobId,
        /// The target (file or folder) to delete.
        target: alloc::string::String,
    },
}

/// IO results delivered back to the Domain.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiskUsageIoResult {
    /// An inspection attempt compelted.
    InspectionCompleted {
        /// Target that was inspected or attempted.
        target: alloc::string::String,
        /// The Job the inspect ran as.
        job_id: JobId,
        /// Set when usage inspection failed.
        error: Option<IoError>,
    },
    /// A deletion attempt completed.
    DeletionCompleted {
        /// Target that was deleted or attempted.
        target: alloc::string::String,
        /// The Job the delete ran as.
        job_id: JobId,
        /// Set when removal failed.
        error: Option<IoError>,
    },
}

impl Domain for DiskUsageDomain {
    type Snapshot = DiskUsageSnapshot;
    type Request = DiskUsageRequests;
    type IoResult = DiskUsageIoResult;
    type Tick = Infallible;
    type ObservedFact = Infallible;
    type Event = Infallible;
    type IoRequest = DiskUsageIoRequest;
    type TimerKey = Infallible;

    fn handle(
        snapshot: &mut Self::Snapshot,
        command: blueos_domain::Command<
            Self::Request,
            Self::IoResult,
            Self::Tick,
            Self::ObservedFact,
        >,
        now: blueos_domain::Now,
    ) -> blueos_domain::Decision<Self> {
        todo!()
    }

    fn io_failed(
        request: Self::IoRequest,
        error: blueos_domain::IoError,
    ) -> blueos_domain::Command<Self::Request, Self::IoResult, Self::Tick, Self::ObservedFact> {
        todo!()
    }
}

impl DomainJobs for DiskUsageDomain {
    fn jobs(snapshot: &Self::Snapshot) -> &Jobs {
        &snapshot.jobs
    }

    fn jobs_mut(snapshot: &mut Self::Snapshot) -> &mut Jobs {
        &mut snapshot.jobs
    }
}
