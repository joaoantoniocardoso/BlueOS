# blueos_recorder_msgs/msg/RecordingIndexRequest
# Payload of the blueos/v1/recorder/query/index query.

string path
# 0 starts at the file magic; otherwise the `offset` of a previous RecordingIndex reply.
uint64 from_offset
# Maximum chunks in the reply (1..=20000).
uint32 limit
