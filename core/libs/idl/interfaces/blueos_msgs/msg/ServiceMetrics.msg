# blueos_msgs/msg/ServiceMetrics
# State published on blueos/v1/<service>/state/metrics (D-12, D-35): every counter, gauge and histogram that the
# Kernel, the Tasks and the adapters of the service recorded, each list sorted by name and then by labels. It is
# published at most once per second, and only when a value changed.

blueos_msgs/MetricCounter[] counters
blueos_msgs/MetricGauge[] gauges
blueos_msgs/MetricHistogram[] histograms
