//! Snapshot output path from injected wall time.

use alloc::{format, string::String};
use core::time::Duration;

use blueos_domain::Now;
use blueos_recorder_paths::civil_date_from_days_since_epoch;

/// Relative path for an indexed copy next to a recording (`<stem>.snapshot-<UTC>Z.mcap`).
pub fn snapshot_output_relative_path(source_relative: &str, now: Now) -> String {
    let source = source_relative
        .rsplit('/')
        .next()
        .unwrap_or(source_relative);
    let stem = source.strip_suffix(".mcap").unwrap_or(source);
    let timestamp = format_snapshot_timestamp(now.wall);
    let parent = source_relative.rsplit_once('/').map(|(parent, _)| parent);
    let file_name = format!("{stem}.snapshot-{timestamp}Z.mcap");
    match parent {
        Some(prefix) if !prefix.is_empty() => format!("{prefix}/{file_name}"),
        _ => file_name,
    }
}

fn format_snapshot_timestamp(wall: Duration) -> String {
    let unix_seconds = i64::try_from(wall.as_secs()).unwrap_or(i64::MAX);
    let second = unix_seconds.rem_euclid(60);
    let minute = (unix_seconds / 60).rem_euclid(60);
    let hour = (unix_seconds / 3_600).rem_euclid(24);
    let days = unix_seconds.div_euclid(86_400);
    let (year, month, day) = civil_date_from_days_since_epoch(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}-{minute:02}-{second:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_name_matches_shared_timestamp_vector() {
        let now = Now {
            wall: Duration::from_secs(1_704_168_306),
            monotonic: Duration::ZERO,
        };
        assert_eq!(
            snapshot_output_relative_path("recorder_20240102_030405.mcap", now,),
            "recorder_20240102_030405.snapshot-2024-01-02T04-05-06Z.mcap"
        );
    }
}
