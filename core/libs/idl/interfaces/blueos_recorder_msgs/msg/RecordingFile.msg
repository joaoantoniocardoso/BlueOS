# blueos_recorder_msgs/msg/RecordingFile
# One MCAP recording in the recorder folder, as listed in RecordingLibrary.

# Being written by the recorder; bytes are readable but the file has no summary yet.
uint8 STATE_RECORDING=0
# Finished and indexed; seekable through HTTP ranges on /userdata/recorder/<path>.
uint8 STATE_READY=1
# Finished without a summary (power loss, crash); RepairRecording gives it back.
uint8 STATE_NEEDS_REPAIR=2
uint8 STATE_REPAIRING=3

# Relative to the recorder folder, forward slashes. Identifies the recording in every command.
string path
string name
uint64 size_bytes
# From the timestamp embedded in the file name, falling back to the file time.
builtin_interfaces/Time created
uint8 state
# Repair progress while STATE_REPAIRING; zero otherwise.
uint64 repair_bytes_processed
uint64 repair_total_bytes
float64 repair_bytes_per_second
# Reason the last repair failed; empty when it did not fail. Cleared by the next repair.
string repair_error
