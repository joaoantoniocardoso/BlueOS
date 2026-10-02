# blueos_msgs/msg/JobStatus
# One job entry; status values mirror blueos_jobs (D-12).

uint8 STATUS_QUEUED=0
uint8 STATUS_RUNNING=1
uint8 STATUS_CANCELLING=2
uint8 STATUS_SUCCEEDED=3
uint8 STATUS_FAILED=4
uint8 STATUS_CANCELLED=5

uint64 job_id
uint64 parent_job_id
uint8 status
string name
