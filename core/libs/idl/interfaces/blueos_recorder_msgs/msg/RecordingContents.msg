# blueos_recorder_msgs/msg/RecordingContents
# What one recording of RecordingLibrary holds, read from its MCAP summary.

# The path of the RecordingFile it describes.
string path
# Time from the first message to the last, from the Statistics record; zero without one.
builtin_interfaces/Duration duration
# Topics of the video channels (foxglove.CompressedVideo), sorted; empty when it has no video.
string[] video_topics
# How many other topics it has, such as telemetry.
uint32 other_topic_count
