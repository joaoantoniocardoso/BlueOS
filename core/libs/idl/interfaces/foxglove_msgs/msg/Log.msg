# foxglove_msgs/msg/Log
# A log message for service tracing output (D-13).

builtin_interfaces/Time timestamp

uint8 UNKNOWN=0
uint8 DEBUG=1
uint8 INFO=2
uint8 WARNING=3
uint8 ERROR=4
uint8 FATAL=5

uint8 level
string message
string name
string file
uint32 line
