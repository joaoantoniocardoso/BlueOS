//! `api.lock` endpoint key lines.

use alloc::collections::BTreeSet;
use std::path::Path;

use crate::endpoints::{Endpoint, EndpointsError, Kind, MANIFEST_FILE, STANDARD_ENDPOINTS};

use super::{
    manifest_parse::{endpoints, read_manifest},
    workspace::workspace_members,
};

/// Builds sorted `api.lock` lines for every endpoint key in the workspace.
pub fn collect_endpoint_lock_lines(
    core_dir: &Path,
    messages: &BTreeSet<String>,
) -> Result<Vec<String>, EndpointsError> {
    let mut lines = Vec::new();
    for (service, endpoint_list) in collect_service_endpoints(core_dir, messages)? {
        for standard in &STANDARD_ENDPOINTS {
            lines.push(crate::format_lock_line(
                &format!("blueos/v1/{service}/{}", standard.key),
                1,
                &format!("type={}", standard.interface_type),
            ));
        }
        let mut job_types = vec!["UpdateSettings".to_owned()];
        for endpoint in endpoint_list {
            lines.push(crate::format_lock_line(
                &endpoint.key(&service),
                1,
                &format!("type={}", endpoint.interface.schema_name()),
            ));
            if endpoint.kind == Kind::Job {
                job_types.push(endpoint.name);
            }
        }
        for job_type in job_types {
            for (output, interface_type) in [
                ("feedback", "blueos_msgs/msg/JobFeedbackList"),
                ("result", "blueos_msgs/msg/JobResult"),
                ("history", "blueos_msgs/msg/JobList"),
            ] {
                lines.push(crate::format_lock_line(
                    &format!("blueos/v1/{service}/jobs/{job_type}/{output}"),
                    1,
                    &format!("type={interface_type}"),
                ));
            }
        }
    }
    lines.sort();
    Ok(lines)
}

fn collect_service_endpoints(
    core_dir: &Path,
    messages: &BTreeSet<String>,
) -> Result<Vec<(String, Vec<Endpoint>)>, EndpointsError> {
    let mut collected = Vec::new();
    for member in workspace_members(core_dir)? {
        let manifest_path = core_dir.join(&member).join(MANIFEST_FILE);
        if !manifest_path.is_file() {
            continue;
        }
        let manifest = read_manifest(&manifest_path)?;
        let (service, service_endpoints) =
            endpoints(manifest, messages).map_err(|error| EndpointsError::Manifest {
                path: manifest_path,
                error,
            })?;
        collected.push((service, service_endpoints));
    }
    Ok(collected)
}
