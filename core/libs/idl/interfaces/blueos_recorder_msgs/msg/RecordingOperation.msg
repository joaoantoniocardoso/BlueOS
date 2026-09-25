# blueos_recorder_msgs/msg/RecordingOperation
# Published on blueos/v1/recorder/event/operation when a repair, snapshot or delete ends.

uint8 OPERATION_REPAIR=0
uint8 OPERATION_SNAPSHOT=1
uint8 OPERATION_DELETE=2

uint8 operation
string path
# The snapshot copy; empty for other operations.
string output_path
bool succeeded
bool cancelled
string error
