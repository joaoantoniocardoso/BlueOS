//! What a finished recording holds, read from its MCAP summary for the library.

use core::time::Duration;
use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Seek},
    path::Path,
};

use mcap::sans_io::summary_reader::{SummaryReadEvent, SummaryReader};

/// Schema name of the video channels, as the Records page finds them.
pub const COMPRESSED_VIDEO_SCHEMA: &str = "foxglove.CompressedVideo";

/// What a recording holds, from its MCAP summary.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RecordingContents {
    /// Time from the first message to the last, from the Statistics record; zero without one.
    pub duration: Duration,
    /// Topics of the [`COMPRESSED_VIDEO_SCHEMA`] channels, sorted.
    pub video_topics: Vec<String>,
    /// How many other topics the recording has, such as telemetry.
    pub other_topic_count: u32,
}

/// Reads what the recording at `path` holds from its summary, or `None` when it has no summary.
pub fn read_recording_contents(path: &Path) -> Result<Option<RecordingContents>, mcap::McapError> {
    let mut file = File::open(path)?;
    let mut reader = SummaryReader::new();
    while let Some(event) = reader.next_event() {
        match event? {
            SummaryReadEvent::ReadRequest(length) => {
                let read = file.read(reader.insert(length))?;
                reader.notify_read(read);
            }
            SummaryReadEvent::SeekRequest(position) => {
                reader.notify_seeked(file.seek(position)?);
            }
        }
    }
    Ok(reader.finish().map(|summary| {
        let (video_topics, other_topics): (BTreeSet<_>, BTreeSet<_>) = summary
            .channels
            .values()
            .map(|channel| {
                let video = channel
                    .schema
                    .as_ref()
                    .is_some_and(|schema| schema.name == COMPRESSED_VIDEO_SCHEMA);
                (video, channel.topic.clone())
            })
            .partition(|(video, _topic)| *video);
        RecordingContents {
            duration: summary
                .stats
                .map(|statistics| {
                    Duration::from_nanos(
                        statistics
                            .message_end_time
                            .saturating_sub(statistics.message_start_time),
                    )
                })
                .unwrap_or_default(),
            video_topics: video_topics
                .into_iter()
                .map(|(_video, topic)| topic)
                .collect(),
            other_topic_count: u32::try_from(other_topics.len()).unwrap_or(u32::MAX),
        }
    }))
}
