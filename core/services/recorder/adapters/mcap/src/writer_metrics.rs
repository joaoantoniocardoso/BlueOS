//! Counters of the writer actor, one per lane (D-35).

use metrics::Counter;

const MAVLINK_TOPIC_PREFIXES: [&str; 2] = ["mavlink/", "mavlink_raw/"];
const VIDEO_TOPIC_PREFIX: &str = "video/";

/// The kinds of recorded topics a counter tells apart. Topics are open-ended, so a lane is the only label.
#[derive(Clone, Copy)]
enum Lane {
    Mavlink,
    Video,
    Other,
}

/// One counter handle per lane, registered once so a sample costs no label building.
struct LaneCounters {
    mavlink: Counter,
    video: Counter,
    other: Counter,
}

/// The recording health counters of the writer: payload bytes and samples written, and samples dropped.
pub(crate) struct WriterMetrics {
    bytes_written: LaneCounters,
    samples_written: LaneCounters,
    samples_dropped: LaneCounters,
}

impl Lane {
    fn of(topic: &str) -> Self {
        if MAVLINK_TOPIC_PREFIXES
            .iter()
            .any(|prefix| topic.starts_with(prefix))
        {
            Self::Mavlink
        } else if topic.starts_with(VIDEO_TOPIC_PREFIX) {
            Self::Video
        } else {
            Self::Other
        }
    }
}

impl LaneCounters {
    fn register(name: &'static str) -> Self {
        Self {
            mavlink: metrics::counter!(name, "lane" => "mavlink"),
            video: metrics::counter!(name, "lane" => "video"),
            other: metrics::counter!(name, "lane" => "other"),
        }
    }

    fn of(&self, lane: Lane) -> &Counter {
        match lane {
            Lane::Mavlink => &self.mavlink,
            Lane::Video => &self.video,
            Lane::Other => &self.other,
        }
    }
}

impl WriterMetrics {
    /// Registers the counters with the recorder in scope; the handles then record into it from any thread.
    pub(crate) fn register() -> Self {
        Self {
            bytes_written: LaneCounters::register("bytes_written"),
            samples_written: LaneCounters::register("samples_written"),
            samples_dropped: LaneCounters::register("samples_dropped"),
        }
    }

    pub(crate) fn record_written(&self, topic: &str, payload_bytes: usize) {
        let lane = Lane::of(topic);
        self.bytes_written.of(lane).increment(payload_bytes as u64);
        self.samples_written.of(lane).increment(1);
    }

    pub(crate) fn record_dropped(&self, topic: &str) {
        self.samples_dropped.of(Lane::of(topic)).increment(1);
    }
}
