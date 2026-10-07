//! Wall-clock encode and decode cost for representative IDL messages (D-33).

#![expect(missing_docs, reason = "Criterion bench harness macros")]

use core::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};

use blueos_idl::{
    Message,
    msg::{blueos_msgs::CommandAck, blueos_recorder_msgs::RecordingFile, foxglove_msgs::Log},
};

fn sample_command_ack() -> CommandAck {
    CommandAck {
        accepted: true,
        job_id: "job-7".into(),
        status: blueos_idl::msg::blueos_msgs::CommandAckStatus::Executing,
        reason: "queued".into(),
    }
}

fn sample_log() -> Log {
    Log {
        timestamp: blueos_idl::msg::builtin_interfaces::Time { sec: 1, nanosec: 2 },
        level: 2,
        message: "hello".into(),
        name: "recorder".into(),
        file: "capture.rs".into(),
        line: 42,
    }
}

fn sample_recording_file() -> RecordingFile {
    RecordingFile::default()
}

fn cdr_codec(criterion: &mut Criterion) {
    let command_ack = sample_command_ack();
    let log = sample_log();
    let recording_file = sample_recording_file();

    criterion.bench_function("cdr_encode_command_ack", |bencher| {
        bencher.iter(|| black_box(command_ack.encode().expect("encode")));
    });
    criterion.bench_function("cdr_decode_command_ack", |bencher| {
        let payload = command_ack.encode().expect("encode");
        bencher.iter(|| black_box(CommandAck::decode(black_box(&payload)).expect("decode")));
    });
    criterion.bench_function("cdr_encode_log", |bencher| {
        bencher.iter(|| black_box(log.encode().expect("encode")));
    });
    criterion.bench_function("cdr_decode_log", |bencher| {
        let payload = log.encode().expect("encode");
        bencher.iter(|| black_box(Log::decode(black_box(&payload)).expect("decode")));
    });
    criterion.bench_function("cdr_encode_recording_file", |bencher| {
        bencher.iter(|| black_box(recording_file.encode().expect("encode")));
    });
    criterion.bench_function("cdr_decode_recording_file", |bencher| {
        let payload = recording_file.encode().expect("encode");
        bencher.iter(|| black_box(RecordingFile::decode(black_box(&payload)).expect("decode")));
    });
}

criterion_group!(benches, cdr_codec);
criterion_main!(benches);
