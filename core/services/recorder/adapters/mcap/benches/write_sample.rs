//! Wall-clock cost of one MCAP sample write on the data-plane path (D-33).

#![expect(missing_docs, reason = "Criterion bench harness macros")]

use core::hint::black_box;
use std::sync::Arc;

use bytes::Bytes;
use criterion::{Criterion, criterion_group, criterion_main};
use tempfile::tempdir;

use blueos_comms::Payload;
use blueos_recorder_mcap::{
    ChannelDescriptor, ChannelRoute, McapFile, MessageEncoding, WriteSampleRequest,
};

const PAYLOAD_BYTES: &[u8] = &[0xAB; 256];

fn sample_write_request(descriptor: &Arc<ChannelDescriptor>) -> WriteSampleRequest {
    WriteSampleRequest {
        topic: "mavlink/from_vehicle".into(),
        route: ChannelRoute::for_topic("mavlink/from_vehicle"),
        log_time: 1,
        publish_time: 1,
        payload: Payload::new(Bytes::from_static(PAYLOAD_BYTES)),
        descriptor: Arc::clone(descriptor),
    }
}

fn write_sample(criterion: &mut Criterion) {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("recorder_bench.mcap");
    let descriptor = Arc::new(ChannelDescriptor {
        topic: "mavlink/from_vehicle".into(),
        schema: None,
        message_encoding: MessageEncoding::OctetStream,
    });
    let request = sample_write_request(&descriptor);

    let mut file = McapFile::open(path, "recorder_bench.mcap".into()).expect("open");
    file.write_sample_request(&request).expect("prime channel");

    criterion.bench_function("mcap_write_sample_request", |bencher| {
        bencher.iter(|| {
            file.write_sample_request(black_box(&request))
                .expect("write sample");
        });
    });
}

criterion_group!(benches, write_sample);
criterion_main!(benches);
