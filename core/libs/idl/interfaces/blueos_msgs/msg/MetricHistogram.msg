# blueos_msgs/msg/MetricHistogram
# A histogram in ServiceMetrics (D-35), such as how long the Inbox took to apply each Command: how many values the
# service recorded, their sum, and how many fell in each bucket.

string name
blueos_msgs/MetricLabel[] labels
uint64 count
float64 sum
# The upper bound of each bucket, ascending. A bucket holds the values above the bound before it, up to its own.
float64[] bucket_bounds
# How many values fell in each bucket of bucket_bounds, plus one last entry for the values above the last bound.
uint64[] bucket_counts
