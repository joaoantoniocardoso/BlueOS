# blueos_recorder_msgs/msg/RecordingLibrary
# Published on blueos/v1/recorder/state/library, newest recording first.

blueos_recorder_msgs/RecordingFile[] files
# What the recordings of files hold, keyed by path: one entry per file whose summary was read, so a file without
# an entry (being written, needs repair, or from an older recorder) holds something unknown. Kept apart from
# RecordingFile, which is frozen as a sequence element (D-06).
blueos_recorder_msgs/RecordingContents[] contents
