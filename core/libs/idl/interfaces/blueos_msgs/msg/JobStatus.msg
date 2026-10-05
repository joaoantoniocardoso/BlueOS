# blueos_msgs/msg/JobStatus
# One Job in the jobs State (D-12, D-36). STATUS_ values 0 to 6 are those of ROS 2 action_msgs/GoalStatus; a ROS 2
# client sees the two waiting statuses as STATUS_ACCEPTED, and STATUS_PAUSED as STATUS_EXECUTING (D-38).

uint8 STATUS_UNKNOWN=0
uint8 STATUS_ACCEPTED=1
uint8 STATUS_EXECUTING=2
uint8 STATUS_CANCELING=3
uint8 STATUS_SUCCEEDED=4
uint8 STATUS_CANCELED=5
uint8 STATUS_ABORTED=6
uint8 STATUS_WAITING_FOR_PERMISSION=7
uint8 STATUS_WAITING_FOR_RESOURCE=8
uint8 STATUS_PAUSED=9

# The UUID the client generated for the Job, as text.
string job_id
# The Job type, the name of its submit endpoint.
string job_type
uint8 status
# Why the Job was canceled or aborted, when the Kernel ended it.
string reason
