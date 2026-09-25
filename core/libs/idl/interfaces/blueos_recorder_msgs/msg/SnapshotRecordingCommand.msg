# blueos_recorder_msgs/msg/SnapshotRecordingCommand
# Writes an indexed copy of a recording (typically the one being written) next to it, named
# <stem>.snapshot-<UTC ISO time>Z.mcap. The copy is announced by a RecordingOperation event.

string path
