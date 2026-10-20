//! Conversions between the Disk Usage Domain and its public Messages.

#![no_std]

use core::convert::Infallible;

use blueos_disk_usage_domain::DiskUsageDomain;

use crate::endpoints::Conversions;

pub mod endpoints;

impl Conversions for DiskUsageDomain {
    type ComputeTargetUsageError = Infallible;
    type DeleteTargetError = Infallible;

    fn compute_target_usage(
        job_id: blueos_jobs::JobId,
        goal: blueos_idl::msg::blueos_disk_usage_msgs::ComputeTargetUsageGoal,
    ) -> Result<Self::Request, Self::ComputeTargetUsageError> {
        todo!()
    }

    fn compute_target_usage_feedback(
        snapshot: &Self::Snapshot,
        job_id: blueos_jobs::JobId,
    ) -> Option<blueos_idl::msg::blueos_disk_usage_msgs::ComputeTargetUsageFeedback> {
        todo!()
    }

    fn compute_target_usage_result(
        snapshot: &Self::Snapshot,
        job_id: blueos_jobs::JobId,
    ) -> blueos_idl::msg::blueos_disk_usage_msgs::ComputeTargetUsageResult {
        todo!()
    }

    fn delete_target(
        job_id: blueos_jobs::JobId,
        goal: blueos_idl::msg::blueos_disk_usage_msgs::DeleteTargetGoal,
    ) -> Result<Self::Request, Self::DeleteTargetError> {
        todo!()
    }

    fn delete_target_feedback(
        snapshot: &Self::Snapshot,
        job_id: blueos_jobs::JobId,
    ) -> Option<blueos_idl::msg::blueos_disk_usage_msgs::DeleteTargetFeedback> {
        todo!()
    }

    fn delete_target_result(
        snapshot: &Self::Snapshot,
        job_id: blueos_jobs::JobId,
    ) -> blueos_idl::msg::blueos_disk_usage_msgs::DeleteTargetResult {
        todo!()
    }

    fn usage(snapshot: &Self::Snapshot) -> blueos_idl::msg::blueos_disk_usage_msgs::DiskUsage {
        todo!()
    }
}
