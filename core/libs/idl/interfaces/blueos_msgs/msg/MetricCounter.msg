# blueos_msgs/msg/MetricCounter
# A counter in ServiceMetrics (D-35): a total that only grows while the service runs.

string name
blueos_msgs/MetricLabel[] labels
uint64 value
