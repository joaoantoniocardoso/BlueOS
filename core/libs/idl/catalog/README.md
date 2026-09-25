# Schema catalog

Schema text of the ROS 2 and Foxglove messages, exposed as `blueos_idl::catalog::schema` by the `catalog`
feature, so the Recorder can write an MCAP `ros2msg` schema for any CDR topic, including those published outside
the BlueOS IDL (e.g. `foxglove.CompressedVideo` from mavlink-camera-manager). It has no message types and is not
part of the versioned BlueOS API.

`interfaces/` holds unmodified `msg/*.msg` files from:

| Source | Version | License |
|---|---|---|
| [ros2/common_interfaces](https://github.com/ros2/common_interfaces) | 5.3.8 (Jazzy) | Apache-2.0 |
| [ros2/rcl_interfaces](https://github.com/ros2/rcl_interfaces), without `test_msgs` | 2.0.4 (Jazzy) | Apache-2.0 |
| [ros2/unique_identifier_msgs](https://github.com/ros2/unique_identifier_msgs) | 2.5.1 (Jazzy) | BSD |
| [ros2/geometry2](https://github.com/ros2/geometry2) (`tf2_msgs`) | 0.36.23 (Jazzy) | BSD |
| [foxglove/foxglove-sdk](https://github.com/foxglove/foxglove-sdk) `schemas/ros2`, as `foxglove_msgs/msg` | sdk/v0.27.0 | MIT |

Their licenses are in `licenses/`. To update, replace a package's `msg` directory with the one from the new
release and update the table.
