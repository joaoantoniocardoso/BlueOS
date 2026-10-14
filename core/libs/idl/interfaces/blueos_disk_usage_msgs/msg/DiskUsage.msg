# blueos_disk_usage_msgs/msg/DiskUsage
# Result of a usage scan: flat nodes list plus the parameters that produced it.

blueos_disk_usage_msgs/DiskNode[] nodes
builtin_interfaces/Time generated_at
uint8 depth
bool include_files
uint64 min_size_bytes
