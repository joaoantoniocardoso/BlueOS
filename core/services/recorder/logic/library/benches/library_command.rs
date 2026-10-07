//! Wall-clock cost of one recording library Command step (D-33).

#![expect(missing_docs, reason = "Criterion bench harness macros")]

use core::{hint::black_box, time::Duration};

use criterion::{Criterion, criterion_group, criterion_main};

use blueos_domain::{Now, Outcome};
use blueos_jobs::JobId;
use blueos_recorder_library::{
    Library, LibraryIoResult, LibraryRequest, ScannedRecording, handle_io_result,
    handle_library_request,
};
use blueos_recorder_paths::RecordingRelativePath;

const NOW: Now = Now {
    wall: Duration::from_secs(20_000),
    monotonic: Duration::from_secs(100),
};

fn library_with_live_recording() -> Library {
    let mut library = Library::default();
    let recordings = vec![ScannedRecording {
        relative_path: "live.mcap".into(),
        name: "live.mcap".into(),
        size_bytes: 100,
        modified_unix_seconds: 1_000,
        indexed: true,
        contents: None,
    }];
    let outcome = handle_io_result(
        &mut library,
        LibraryIoResult::ScanCompleted { recordings },
        None,
        NOW,
    );
    assert!(matches!(outcome, Outcome::Applied { .. }));
    library
}

fn library_command(criterion: &mut Criterion) {
    let mut library = library_with_live_recording();

    criterion.bench_function("library_handle_delete_recording_command", |bencher| {
        bencher.iter(|| {
            let path = RecordingRelativePath::parse("live.mcap").expect("path");
            let outcome = handle_library_request(
                black_box(&mut library),
                LibraryRequest::DeleteRecording {
                    path,
                    job_id: JobId::from_u128(1),
                },
                Some("live.mcap"),
                NOW,
            );
            black_box(outcome);
        });
    });
}

criterion_group!(benches, library_command);
criterion_main!(benches);
