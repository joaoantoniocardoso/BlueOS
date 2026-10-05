# blueos_msgs/msg/MetricGauge
# A gauge in ServiceMetrics (D-35): the last value the service set, such as how many Commands wait in the Inbox.

string name
blueos_msgs/MetricLabel[] labels
float64 value
