# blueos_recorder_msgs/msg/RecordingState
# Published on blueos/v1/recorder/state/recording.

bool armed
bool session_active
string current_file
uint64 session_bytes_written
string[] recording_video_topics
# Samples left out of current_file because they arrived faster than the disk took them.
uint64 samples_dropped
