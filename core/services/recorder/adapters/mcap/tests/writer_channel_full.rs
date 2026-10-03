//! A sample the full writer channel rejects is dropped, and counted in the metrics of its lane (D-35).

use core::sync::atomic::Ordering;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use bytes::Bytes;
use metrics::{
    Counter, Gauge, Histogram, Key, KeyName, Label, Metadata, Recorder, SharedString, Unit,
    atomics::AtomicU64,
};

use blueos_comms::Payload;
use blueos_recorder_mcap::{ChannelDescriptor, ChannelRoute, McapWriterHandle, MessageEncoding};

/// Keeps every counter it is asked for, by name and labels.
#[derive(Default)]
struct CountingRecorder {
    counters: Mutex<BTreeMap<Key, Arc<AtomicU64>>>,
}

impl Recorder for CountingRecorder {
    fn describe_counter(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}

    fn describe_gauge(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}

    fn describe_histogram(&self, _key: KeyName, _unit: Option<Unit>, _description: SharedString) {}

    fn register_counter(&self, key: &Key, _metadata: &Metadata<'_>) -> Counter {
        let mut counters = self.counters.lock().expect("the counters lock");
        Counter::from_arc(Arc::clone(counters.entry(key.clone()).or_default()))
    }

    fn register_gauge(&self, _key: &Key, _metadata: &Metadata<'_>) -> Gauge {
        Gauge::noop()
    }

    fn register_histogram(&self, _key: &Key, _metadata: &Metadata<'_>) -> Histogram {
        Histogram::noop()
    }
}

impl CountingRecorder {
    fn value(&self, name: &'static str, lane: &'static str) -> u64 {
        let counters = self.counters.lock().expect("the counters lock");
        counters
            .get(&Key::from_parts(name, vec![Label::new("lane", lane)]))
            .map_or(0, |counter| counter.load(Ordering::Acquire))
    }
}

#[tokio::test]
async fn a_sample_the_full_channel_rejects_is_dropped_and_counted_in_its_lane() {
    let recorder = CountingRecorder::default();
    let writer = metrics::with_local_recorder(&recorder, || {
        McapWriterHandle::spawn_with_queue_bytes(usize::MAX / 2)
    });
    let topic = "video/camera1/stream";
    let descriptor = Arc::new(ChannelDescriptor {
        topic: topic.into(),
        schema: None,
        message_encoding: MessageEncoding::OctetStream,
    });

    // The runtime cannot run the writer actor while this loop does not yield, so the channel only fills.
    for _ in 0..10_000 {
        writer.try_write_sample(
            topic.into(),
            ChannelRoute::for_topic(topic),
            0,
            0,
            Payload::new(Bytes::from_static(&[0])),
            Arc::clone(&descriptor),
        );
    }

    let dropped = writer.take_dropped_samples();
    assert!(dropped > 0, "the channel never filled");
    assert_eq!(recorder.value("samples_dropped", "video"), dropped);
    assert_eq!(recorder.value("samples_dropped", "other"), 0);
}
