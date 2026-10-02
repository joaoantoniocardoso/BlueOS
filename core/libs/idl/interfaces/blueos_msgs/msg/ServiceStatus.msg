# blueos_msgs/msg/ServiceStatus
# High-level service health on the status state key (D-12).

uint8 STATUS_UNKNOWN=0
uint8 STATUS_STARTING=1
uint8 STATUS_READY=2
uint8 STATUS_DEGRADED=3
uint8 STATUS_STOPPING=4

uint8 status
string detail
