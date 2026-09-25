# blueos_recorder_msgs/msg/RecordingIndex
# Reply of the index query: one page of a walk over record headers, so the browser can fetch chunk
# bodies with HTTP ranges even when the file has no summary yet (still recording, needs repair).

uint64 size
# Where the next page starts.
uint64 offset
# A DataEnd or Footer record was reached: the walk is complete.
bool closed
blueos_recorder_msgs/ChunkIndexEntry[] chunks
blueos_recorder_msgs/ChannelMessageCount[] message_counts
# Raw Header, Schema, Channel and Metadata records met in this page, in file order.
uint8[] records
