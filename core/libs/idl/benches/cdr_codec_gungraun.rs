//! Instruction counts for representative IDL CDR encode and decode (D-33).

#![expect(missing_docs, reason = "Gungraun bench harness macros")]

use core::hint::black_box;

use gungraun::prelude::*;

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

#[library_benchmark]
fn cdr_encode_command_ack() {
    let command_ack = sample_command_ack();
    black_box(command_ack.encode().expect("encode"));
}

#[library_benchmark]
fn cdr_decode_command_ack() {
    let command_ack = sample_command_ack();
    let payload = command_ack.encode().expect("encode");
    black_box(CommandAck::decode(black_box(&payload)).expect("decode"));
}

#[library_benchmark]
fn cdr_encode_log() {
    let log = sample_log();
    black_box(log.encode().expect("encode"));
}

#[library_benchmark]
fn cdr_decode_log() {
    let log = sample_log();
    let payload = log.encode().expect("encode");
    black_box(Log::decode(black_box(&payload)).expect("decode"));
}

#[library_benchmark]
fn cdr_encode_recording_file() {
    let recording_file = sample_recording_file();
    black_box(recording_file.encode().expect("encode"));
}

#[library_benchmark]
fn cdr_decode_recording_file() {
    let recording_file = sample_recording_file();
    let payload = recording_file.encode().expect("encode");
    black_box(RecordingFile::decode(black_box(&payload)).expect("decode"));
}

library_benchmark_group!(
    name = cdr_codec,
    benchmarks = [
        cdr_encode_command_ack,
        cdr_decode_command_ack,
        cdr_encode_log,
        cdr_decode_log,
        cdr_encode_recording_file,
        cdr_decode_recording_file,
    ]
);

gungraun::main!(library_benchmark_groups = cdr_codec);
