//! Repair progress publishing and repair outcome rows.

#[path = "support/library_common.rs"]
mod common;

use core::time::Duration;

use blueos_domain::Now;
use blueos_recorder_library::{
    LibraryObservedFact, LibraryRepairProgress, REPAIR_PROGRESS_PUBLISH_INTERVAL,
    handle_observed_fact, start_repair,
};

use common::{NOW, job_id, repair_spec, scan_snapshot};

#[test]
fn the_library_takes_repair_progress_once_per_publish_interval_and_always_its_end() {
    let mut library = scan_snapshot(&[("file.mcap", false)], 1_000);
    let path = blueos_recorder_paths::RecordingRelativePath::parse("file.mcap").expect("path");
    assert!(matches!(
        start_repair(&mut library, repair_spec(path.clone(), job_id(1), None)),
        blueos_domain::Outcome::Applied { .. }
    ));
    let mut report = |bytes_processed: u64, after: Duration| {
        handle_observed_fact(
            &mut library,
            LibraryObservedFact::RepairProgress(LibraryRepairProgress {
                path: path.clone(),
                bytes_processed,
                total_bytes: 100,
            }),
            None,
            Now {
                wall: NOW.wall + after,
                monotonic: NOW.monotonic + after,
            },
        );
        let published = library.catalog().entries()[0].repair_bytes_processed;
        let latest = library
            .work()
            .queue()
            .repair_progress(job_id(1))
            .expect("progress")
            .bytes_processed;
        (published, latest)
    };

    assert_eq!(
        report(10, REPAIR_PROGRESS_PUBLISH_INTERVAL / 5),
        (0, 10),
        "progress right after the repair started waits for the publish interval"
    );
    assert_eq!(report(20, REPAIR_PROGRESS_PUBLISH_INTERVAL), (20, 20));
    assert_eq!(
        report(30, REPAIR_PROGRESS_PUBLISH_INTERVAL * 6 / 5),
        (20, 30),
        "progress within the publish interval waits"
    );
    assert_eq!(
        report(100, REPAIR_PROGRESS_PUBLISH_INTERVAL * 7 / 5),
        (100, 100),
        "the end of the read is published at once"
    );
}
