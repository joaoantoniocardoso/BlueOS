// @generated
export const CATALOG_SCHEMAS: Record<string, string> = {
  "action_msgs/msg/GoalInfo": `# Goal ID
unique_identifier_msgs/UUID goal_id

# Time when the goal was accepted
builtin_interfaces/Time stamp
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: unique_identifier_msgs/UUID
# A universally unique identifier (UUID).
#
#  http://en.wikipedia.org/wiki/Universally_unique_identifier
#  http://tools.ietf.org/html/rfc4122.html

uint8[16] uuid`,
  "action_msgs/msg/GoalStatus": `# An action goal can be in one of these states after it is accepted by an action
# server.
#
# For more information, see http://design.ros2.org/articles/actions.html

# Indicates status has not been properly set.
int8 STATUS_UNKNOWN   = 0

# The goal has been accepted and is awaiting execution.
int8 STATUS_ACCEPTED  = 1

# The goal is currently being executed by the action server.
int8 STATUS_EXECUTING = 2

# The client has requested that the goal be canceled and the action server has
# accepted the cancel request.
int8 STATUS_CANCELING = 3

# The goal was achieved successfully by the action server.
int8 STATUS_SUCCEEDED = 4

# The goal was canceled after an external request from an action client.
int8 STATUS_CANCELED  = 5

# The goal was terminated by the action server without an external request.
int8 STATUS_ABORTED   = 6

# Goal info (contains ID and timestamp).
GoalInfo goal_info

# Action goal state-machine status.
int8 status
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: unique_identifier_msgs/UUID
# A universally unique identifier (UUID).
#
#  http://en.wikipedia.org/wiki/Universally_unique_identifier
#  http://tools.ietf.org/html/rfc4122.html

uint8[16] uuid
================================================================================
MSG: action_msgs/GoalInfo
# Goal ID
unique_identifier_msgs/UUID goal_id

# Time when the goal was accepted
builtin_interfaces/Time stamp`,
  "action_msgs/msg/GoalStatusArray": `# An array of goal statuses.
GoalStatus[] status_list
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: unique_identifier_msgs/UUID
# A universally unique identifier (UUID).
#
#  http://en.wikipedia.org/wiki/Universally_unique_identifier
#  http://tools.ietf.org/html/rfc4122.html

uint8[16] uuid
================================================================================
MSG: action_msgs/GoalInfo
# Goal ID
unique_identifier_msgs/UUID goal_id

# Time when the goal was accepted
builtin_interfaces/Time stamp
================================================================================
MSG: action_msgs/GoalStatus
# An action goal can be in one of these states after it is accepted by an action
# server.
#
# For more information, see http://design.ros2.org/articles/actions.html

# Indicates status has not been properly set.
int8 STATUS_UNKNOWN   = 0

# The goal has been accepted and is awaiting execution.
int8 STATUS_ACCEPTED  = 1

# The goal is currently being executed by the action server.
int8 STATUS_EXECUTING = 2

# The client has requested that the goal be canceled and the action server has
# accepted the cancel request.
int8 STATUS_CANCELING = 3

# The goal was achieved successfully by the action server.
int8 STATUS_SUCCEEDED = 4

# The goal was canceled after an external request from an action client.
int8 STATUS_CANCELED  = 5

# The goal was terminated by the action server without an external request.
int8 STATUS_ABORTED   = 6

# Goal info (contains ID and timestamp).
GoalInfo goal_info

# Action goal state-machine status.
int8 status`,
  "actionlib_msgs/msg/GoalID": `
# The stamp should store the time at which this goal was requested.
# It is used by an action server when it tries to preempt all
# goals that were requested before a certain time
builtin_interfaces/Time stamp

# The id provides a way to associate feedback and
# result message with specific goal requests. The id
# specified must be unique.
string id
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "actionlib_msgs/msg/GoalStatus": `GoalID goal_id
uint8 status
uint8 PENDING         = 0   # The goal has yet to be processed by the action server.
uint8 ACTIVE          = 1   # The goal is currently being processed by the action server.
uint8 PREEMPTED       = 2   # The goal received a cancel request after it started executing
                            #   and has since completed its execution (Terminal State).
uint8 SUCCEEDED       = 3   # The goal was achieved successfully by the action server
                            #   (Terminal State).
uint8 ABORTED         = 4   # The goal was aborted during execution by the action server due
                            #    to some failure (Terminal State).
uint8 REJECTED        = 5   # The goal was rejected by the action server without being processed,
                            #    because the goal was unattainable or invalid (Terminal State).
uint8 PREEMPTING      = 6   # The goal received a cancel request after it started executing
                            #    and has not yet completed execution.
uint8 RECALLING       = 7   # The goal received a cancel request before it started executing, but
                            #    the action server has not yet confirmed that the goal is canceled.
uint8 RECALLED        = 8   # The goal received a cancel request before it started executing
                            #    and was successfully cancelled (Terminal State).
uint8 LOST            = 9   # An action client can determine that a goal is LOST. This should not
                            #    be sent over the wire by an action server.

# Allow for the user to associate a string with GoalStatus for debugging.
string text
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: actionlib_msgs/GoalID

# The stamp should store the time at which this goal was requested.
# It is used by an action server when it tries to preempt all
# goals that were requested before a certain time
builtin_interfaces/Time stamp

# The id provides a way to associate feedback and
# result message with specific goal requests. The id
# specified must be unique.
string id`,
  "actionlib_msgs/msg/GoalStatusArray": `# Stores the statuses for goals that are currently being tracked
# by an action server
std_msgs/Header header
GoalStatus[] status_list
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: actionlib_msgs/GoalID

# The stamp should store the time at which this goal was requested.
# It is used by an action server when it tries to preempt all
# goals that were requested before a certain time
builtin_interfaces/Time stamp

# The id provides a way to associate feedback and
# result message with specific goal requests. The id
# specified must be unique.
string id
================================================================================
MSG: actionlib_msgs/GoalStatus
GoalID goal_id
uint8 status
uint8 PENDING         = 0   # The goal has yet to be processed by the action server.
uint8 ACTIVE          = 1   # The goal is currently being processed by the action server.
uint8 PREEMPTED       = 2   # The goal received a cancel request after it started executing
                            #   and has since completed its execution (Terminal State).
uint8 SUCCEEDED       = 3   # The goal was achieved successfully by the action server
                            #   (Terminal State).
uint8 ABORTED         = 4   # The goal was aborted during execution by the action server due
                            #    to some failure (Terminal State).
uint8 REJECTED        = 5   # The goal was rejected by the action server without being processed,
                            #    because the goal was unattainable or invalid (Terminal State).
uint8 PREEMPTING      = 6   # The goal received a cancel request after it started executing
                            #    and has not yet completed execution.
uint8 RECALLING       = 7   # The goal received a cancel request before it started executing, but
                            #    the action server has not yet confirmed that the goal is canceled.
uint8 RECALLED        = 8   # The goal received a cancel request before it started executing
                            #    and was successfully cancelled (Terminal State).
uint8 LOST            = 9   # An action client can determine that a goal is LOST. This should not
                            #    be sent over the wire by an action server.

# Allow for the user to associate a string with GoalStatus for debugging.
string text
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "builtin_interfaces/msg/Duration": `# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "builtin_interfaces/msg/Time": `# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "diagnostic_msgs/msg/DiagnosticArray": `# This message is used to send diagnostic information about the state of the robot.
std_msgs/Header header # for timestamp
DiagnosticStatus[] status # an array of components being reported on
================================================================================
MSG: diagnostic_msgs/KeyValue
# What to label this value when viewing.
string key
# A value to track over time.
string value
================================================================================
MSG: diagnostic_msgs/DiagnosticStatus
# This message holds the status of an individual component of the robot.

# Possible levels of operations.
byte OK=0
byte WARN=1
byte ERROR=2
byte STALE=3

# Level of operation enumerated above.
byte level
# A description of the test/component reporting.
string name
# A description of the status.
string message
# A hardware unique string.
string hardware_id
# An array of values associated with the status.
KeyValue[] values
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "diagnostic_msgs/msg/DiagnosticStatus": `# This message holds the status of an individual component of the robot.

# Possible levels of operations.
byte OK=0
byte WARN=1
byte ERROR=2
byte STALE=3

# Level of operation enumerated above.
byte level
# A description of the test/component reporting.
string name
# A description of the status.
string message
# A hardware unique string.
string hardware_id
# An array of values associated with the status.
KeyValue[] values
================================================================================
MSG: diagnostic_msgs/KeyValue
# What to label this value when viewing.
string key
# A value to track over time.
string value`,
  "diagnostic_msgs/msg/KeyValue": `# What to label this value when viewing.
string key
# A value to track over time.
string value`,
  "foxglove_msgs/msg/ArrowPrimitive": `# foxglove_msgs/msg/ArrowPrimitive
# A primitive representing an arrow

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the arrow's tail and orientation of the arrow. Identity orientation means the arrow points in the +x direction.
geometry_msgs/Pose pose

# Length of the arrow shaft
float64 shaft_length

# Diameter of the arrow shaft
float64 shaft_diameter

# Length of the arrow head
float64 head_length

# Diameter of the arrow head
float64 head_diameter

# Color of the arrow
foxglove_msgs/Color color
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation`,
  "foxglove_msgs/msg/CameraCalibration": `# foxglove_msgs/msg/CameraCalibration
# Camera calibration parameters

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of calibration data
builtin_interfaces/Time timestamp

# Frame of reference for the camera. The origin of the frame is the optical center of the camera. +x points to the right in the image, +y points down, and +z points into the plane of the image.
string frame_id

# Image width
uint32 width

# Image height
uint32 height

# Name of distortion model
# 
# Supported parameters: \`plumb_bob\` (k1, k2, p1, p2, k3), \`rational_polynomial\` (k1, k2, p1, p2, k3, k4, k5, k6), and \`kannala_brandt\` (k1, k2, k3, k4), and \`fisheye62\` (k0, k1, k2, k3, p0, p1, crit_theta [optional]). \`plumb_bob\` and \`rational_polynomial\` models are based on the pinhole model [OpenCV's](https://docs.opencv.org/4.11.0/d9/d0c/group__calib3d.html) [pinhole camera model](https://en.wikipedia.org/wiki/Distortion_%28optics%29#Software_correction). The \`kannala_brandt\` model matches the [OpenvCV fisheye](https://docs.opencv.org/4.11.0/db/d58/group__calib3d__fisheye.html) model. The \`fisheye62\` model matches the [Project Aria's Fisheye62 Model](https://facebookresearch.github.io/projectaria_tools/docs/tech_insights/camera_intrinsic_models).
string distortion_model

# Distortion parameters
float64[] d

# Intrinsic camera matrix (3x3 row-major matrix)
# 
# A 3x3 row-major matrix for the raw (distorted) image.
# 
# Projects 3D points in the camera coordinate frame to 2D pixel coordinates using the focal lengths (fx, fy) and principal point (cx, cy).
# 
# \`\`\`
#     [fx  0 cx]
# K = [ 0 fy cy]
#     [ 0  0  1]
# \`\`\`
# 
# **Uncalibrated cameras:** Following ROS conventions for [CameraInfo](https://docs.ros.org/en/noetic/api/sensor_msgs/html/msg/CameraInfo.html), Foxglove also treats K[0] == 0.0 as indicating an uncalibrated camera, and calibration data will be ignored.
float64[9] k

# Rectification matrix (stereo cameras only, 3x3 row-major matrix)
# 
# A rotation matrix aligning the camera coordinate system to the ideal stereo image plane so that epipolar lines in both stereo images are parallel.
float64[9] r

# Projection/camera matrix (3x4 row-major matrix)
# 
# \`\`\`
#     [fx'  0  cx' Tx]
# P = [ 0  fy' cy' Ty]
#     [ 0   0   1   0]
# \`\`\`
# 
# By convention, this matrix specifies the intrinsic (camera) matrix of the processed (rectified) image. That is, the left 3x3 portion is the normal camera intrinsic matrix for the rectified image.
# 
# It projects 3D points in the camera coordinate frame to 2D pixel coordinates using the focal lengths (fx', fy') and principal point (cx', cy') - these may differ from the values in K.
# 
# For monocular cameras, Tx = Ty = 0. Normally, monocular cameras will also have R = the identity and P[1:3,1:3] = K.
# 
# Foxglove currently does not support displaying stereo images, so Tx and Ty are ignored.
# 
# Given a 3D point [X Y Z]', the projection (x, y) of the point onto the rectified image is given by:
# 
# \`\`\`
# [u v w]' = P * [X Y Z 1]'
#        x = u / w
#        y = v / w
# \`\`\`
# 
# This holds for both images of a stereo pair.
float64[12] p
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "foxglove_msgs/msg/CircleAnnotation": `# foxglove_msgs/msg/CircleAnnotation
# A circle annotation on a 2D image

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of circle
builtin_interfaces/Time timestamp

# Center of the circle in 2D image coordinates (pixels).
# The coordinate uses the top-left corner of the top-left pixel of the image as the origin.
foxglove_msgs/Point2 position

# Circle diameter in pixels
float64 diameter

# Line thickness in pixels
float64 thickness

# Fill color
foxglove_msgs/Color fill_color

# Outline color
foxglove_msgs/Color outline_color

# Additional user-provided metadata associated with this annotation. Keys must be unique.
foxglove_msgs/KeyValuePair[] metadata
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: foxglove_msgs/KeyValuePair
# foxglove_msgs/msg/KeyValuePair
# A key with its associated value

# Generated by https://github.com/foxglove/foxglove-sdk

# Key
string key

# Value
string value
================================================================================
MSG: foxglove_msgs/Point2
# foxglove_msgs/msg/Point2
# A point representing a position in 2D space

# Generated by https://github.com/foxglove/foxglove-sdk

# x coordinate position
float64 x

# y coordinate position
float64 y`,
  "foxglove_msgs/msg/Color": `# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a`,
  "foxglove_msgs/msg/CompressedAudio": `# foxglove_msgs/msg/CompressedAudio
# A single chunk of a compressed audio bitstream

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of the start of the audio chunk
builtin_interfaces/Time timestamp

# Compressed audio data. Packet duration is determined by the codec during encoding. Messages should generally contain approximately 20 ms of audio.
# 
# - \`opus\`
#   - Each message must contain a complete raw Opus packet, without Ogg, WebM, or other container framing, as described in [RFC 6716 section 3](https://datatracker.ietf.org/doc/html/rfc6716#section-3).
#   - Each packet contains all information necessary for decoding, and may be decoded at any sample rate supported by Opus (8, 12, 16, 24, or 48 kHz).
#   - A single raw Opus packet represents mono or stereo audio; multichannel Opus requires multistream or container metadata and is not supported by this schema.
# - \`mp4a.40.2\`
#   - Each message must contain a complete MPEG-4 AAC-LC ADTS frame, including the ADTS header, as described in section 1.A.3.2 of ISO/IEC 14496-3:2019.
#   - The ADTS header supplies stream parameters such as sample rate and channel configuration.
uint8[] data

# Audio format. Values supported by Foxglove are \`opus\` for raw Opus packets and \`mp4a.40.2\` for AAC-LC ADTS frames.
string format
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "foxglove_msgs/msg/CompressedImage": `# foxglove_msgs/msg/CompressedImage
# A compressed image

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of image
builtin_interfaces/Time timestamp

# Frame of reference for the image. The origin of the frame is the optical center of the camera. +x points to the right in the image, +y points down, and +z points into the plane of the image.
string frame_id

# Compressed image data
uint8[] data

# Image format
# 
# Supported values: \`jpeg\`, \`png\`, \`webp\`, \`avif\`
string format
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "foxglove_msgs/msg/CompressedPointCloud": `# foxglove_msgs/msg/CompressedPointCloud
# A compressed point cloud. A decoder for \`format\` must decompress \`data\`, using metadata stored in the compressed payload to recover point positions and any additional per-point attributes. The decoded point cloud must include at least 2 coordinate fields from \`x\`, \`y\`, and \`z\`; \`red\`, \`green\`, \`blue\`, and \`alpha\` are optional for customizing each point's color.

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of point cloud
builtin_interfaces/Time timestamp

# Frame of reference
string frame_id

# The origin of the point cloud relative to the frame of reference
geometry_msgs/Pose pose

# Compressed point cloud data for exactly one point cloud, including any format-specific metadata needed to describe the decoded point attributes.
uint8[] data

# Point cloud compression format.
# 
# Supported values: \`draco\` ([Google Draco](https://google.github.io/draco/)).
string format
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation`,
  "foxglove_msgs/msg/CompressedVideo": `# foxglove_msgs/msg/CompressedVideo
# A single frame of a compressed video bitstream

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of video frame
builtin_interfaces/Time timestamp

# Frame of reference for the video.
# 
# The origin of the frame is the optical center of the camera. +x points to the right in the video, +y points down, and +z points into the plane of the video.
string frame_id

# Compressed video frame data.
# 
# For packet-based video codecs this data must begin and end on packet boundaries (no partial packets), and must contain enough video packets to decode exactly one image (either a keyframe or delta frame). Note: Foxglove does not support video streams that include B frames because they require lookahead.
# 
# Specifically, the requirements for different \`format\` values are:
# 
# - \`h264\`
#   - Use Annex B formatted data
#   - Each CompressedVideo message should contain enough NAL units to decode exactly one video frame
#   - Each message containing a key frame (IDR) must also include a SPS NAL unit
# 
# - \`h265\` (HEVC)
#   - Use Annex B formatted data
#   - Each CompressedVideo message should contain enough NAL units to decode exactly one video frame
#   - Each message containing a key frame (IRAP) must also include relevant VPS/SPS/PPS NAL units
# 
# - \`vp9\`
#   - Each CompressedVideo message should contain exactly one video frame
# 
# - \`av1\`
#   - Use the "Low overhead bitstream format" (section 5.2)
#   - Each CompressedVideo message should contain enough OBUs to decode exactly one video frame
#   - Each message containing a key frame must also include a Sequence Header OBU
uint8[] data

# Video format.
# 
# Supported values: \`h264\`, \`h265\`, \`vp9\`, \`av1\`.
# 
# Note: compressed video support is subject to hardware limitations and patent licensing, so not all encodings may be supported on all platforms. See more about [H.265 support](https://caniuse.com/hevc), [VP9 support](https://caniuse.com/webm), and [AV1 support](https://caniuse.com/av1).
string format
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "foxglove_msgs/msg/CubePrimitive": `# foxglove_msgs/msg/CubePrimitive
# A primitive representing a cube or rectangular prism

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the center of the cube and orientation of the cube
geometry_msgs/Pose pose

# Size of the cube along each axis
geometry_msgs/Vector3 size

# Color of the cube
foxglove_msgs/Color color
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "foxglove_msgs/msg/CylinderPrimitive": `# foxglove_msgs/msg/CylinderPrimitive
# A primitive representing a cylinder, elliptic cylinder, or truncated cone

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the center of the cylinder and orientation of the cylinder. The flat face(s) are perpendicular to the z-axis.
geometry_msgs/Pose pose

# Size of the cylinder's bounding box
geometry_msgs/Vector3 size

# 0-1, ratio of the diameter of the cylinder's bottom face (min z) to the bottom of the bounding box
float64 bottom_scale

# 0-1, ratio of the diameter of the cylinder's top face (max z) to the top of the bounding box
float64 top_scale

# Color of the cylinder
foxglove_msgs/Color color
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "foxglove_msgs/msg/Event": `# foxglove_msgs/msg/Event
# A discrete event that occurred over a time range

# Generated by https://github.com/foxglove/foxglove-sdk

# Event start time (inclusive)
builtin_interfaces/Time start_time

# Event end time (inclusive)
builtin_interfaces/Time end_time

# Additional key-value metadata. Keys must be unique.
foxglove_msgs/KeyValuePair[] metadata
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/KeyValuePair
# foxglove_msgs/msg/KeyValuePair
# A key with its associated value

# Generated by https://github.com/foxglove/foxglove-sdk

# Key
string key

# Value
string value`,
  "foxglove_msgs/msg/FrameTransform": `# foxglove_msgs/msg/FrameTransform
# A transform between two reference frames in 3D space. The transform defines the position and orientation of a child frame within a parent frame. Translation moves the origin of the child frame relative to the parent origin. The rotation changes the orientation of the child frame around its origin.
# 
# Examples:
# 
# - With translation (x=1, y=0, z=0) and identity rotation (x=0, y=0, z=0, w=1), a point at (x=0, y=0, z=0) in the child frame maps to (x=1, y=0, z=0) in the parent frame.
# 
# - With translation (x=1, y=2, z=0) and a 90-degree rotation around the z-axis (x=0, y=0, z=0.707, w=0.707), a point at (x=1, y=0, z=0) in the child frame maps to (x=-1, y=3, z=0) in the parent frame.

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of transform
builtin_interfaces/Time timestamp

# Name of the parent frame
string parent_frame_id

# Name of the child frame
string child_frame_id

# Translation component of the transform, representing the position of the child frame's origin in the parent frame.
geometry_msgs/Vector3 translation

# Rotation component of the transform, representing the orientation of the child frame in the parent frame
geometry_msgs/Quaternion rotation
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "foxglove_msgs/msg/FrameTransforms": `# foxglove_msgs/msg/FrameTransforms
# An array of FrameTransform messages

# Generated by https://github.com/foxglove/foxglove-sdk

# Array of transforms
foxglove_msgs/FrameTransform[] transforms
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: foxglove_msgs/FrameTransform
# foxglove_msgs/msg/FrameTransform
# A transform between two reference frames in 3D space. The transform defines the position and orientation of a child frame within a parent frame. Translation moves the origin of the child frame relative to the parent origin. The rotation changes the orientation of the child frame around its origin.
# 
# Examples:
# 
# - With translation (x=1, y=0, z=0) and identity rotation (x=0, y=0, z=0, w=1), a point at (x=0, y=0, z=0) in the child frame maps to (x=1, y=0, z=0) in the parent frame.
# 
# - With translation (x=1, y=2, z=0) and a 90-degree rotation around the z-axis (x=0, y=0, z=0.707, w=0.707), a point at (x=1, y=0, z=0) in the child frame maps to (x=-1, y=3, z=0) in the parent frame.

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of transform
builtin_interfaces/Time timestamp

# Name of the parent frame
string parent_frame_id

# Name of the child frame
string child_frame_id

# Translation component of the transform, representing the position of the child frame's origin in the parent frame.
geometry_msgs/Vector3 translation

# Rotation component of the transform, representing the orientation of the child frame in the parent frame
geometry_msgs/Quaternion rotation`,
  "foxglove_msgs/msg/GeoJSON": `# foxglove_msgs/msg/GeoJSON
# GeoJSON data for annotating maps

# Generated by https://github.com/foxglove/foxglove-sdk

# GeoJSON data encoded as a UTF-8 string
string geojson`,
  "foxglove_msgs/msg/Grid": `# foxglove_msgs/msg/Grid
# A 2D grid of data

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of grid
builtin_interfaces/Time timestamp

# Frame of reference
string frame_id

# Origin of grid's corner relative to frame of reference; grid is positioned in the x-y plane relative to this origin
geometry_msgs/Pose pose

# Number of grid columns
uint32 column_count

# Size of single grid cell along x and y axes, relative to \`pose\`
foxglove_msgs/Vector2 cell_size

# Number of bytes between rows in \`data\`
uint32 row_stride

# Number of bytes between cells within a row in \`data\`
uint32 cell_stride

# Fields in \`data\`. \`red\`, \`green\`, \`blue\`, and \`alpha\` are optional for customizing the grid's color.
# To enable RGB color visualization in the [3D panel](https://docs.foxglove.dev/docs/visualization/panels/3d#rgba-separate-fields-color-mode), include **all four** of these fields in your \`fields\` array:
# 
# - \`red\` - Red channel value
# - \`green\` - Green channel value
# - \`blue\` - Blue channel value
# - \`alpha\` - Alpha/transparency channel value
# 
# **note:** All four fields must be present with these exact names for RGB visualization to work. The order of fields doesn't matter, but the names must match exactly.
# 
# Recommended type: \`UINT8\` (0-255 range) for standard 8-bit color channels.
# 
# Example field definitions:
# 
# **RGB color only:**
# 
# \`\`\`javascript
# fields: [
#  { name: "red", offset: 0, type: NumericType.UINT8 },
#  { name: "green", offset: 1, type: NumericType.UINT8 },
#  { name: "blue", offset: 2, type: NumericType.UINT8 },
#  { name: "alpha", offset: 3, type: NumericType.UINT8 },
# ];
# \`\`\`
# 
# **RGB color with elevation (for 3D terrain visualization):**
# 
# \`\`\`javascript
# fields: [
#  { name: "red", offset: 0, type: NumericType.UINT8 },
#  { name: "green", offset: 1, type: NumericType.UINT8 },
#  { name: "blue", offset: 2, type: NumericType.UINT8 },
#  { name: "alpha", offset: 3, type: NumericType.UINT8 },
#  { name: "elevation", offset: 4, type: NumericType.FLOAT32 },
# ];
# \`\`\`
# 
# When these fields are present, the 3D panel will offer additional "Color Mode" options including "RGBA (separate fields)" to visualize the RGB data directly. For elevation visualization, set the "Elevation field" to your elevation layer name.
foxglove_msgs/PackedElementField[] fields

# Grid cell data, interpreted using \`fields\`, in row-major (y-major) order.
# For the data element starting at byte offset i, the coordinates of its corner closest to the origin will be:
# 
# - y = i / row_stride * cell_size.y
# - x = (i % row_stride) / cell_stride * cell_size.x
uint8[] data
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/PackedElementField
# foxglove_msgs/msg/PackedElementField
# A field present within each element in a byte array of packed elements.

# Generated by https://github.com/foxglove/foxglove-sdk

# Name of the field
string name

# Byte offset from start of data buffer
uint32 offset

# Unknown numeric type
uint8 UNKNOWN=0

# Unsigned 8-bit integer
uint8 UINT8=1

# Signed 8-bit integer
uint8 INT8=2

# Unsigned 16-bit integer
uint8 UINT16=3

# Signed 16-bit integer
uint8 INT16=4

# Unsigned 32-bit integer
uint8 UINT32=5

# Signed 32-bit integer
uint8 INT32=6

# 32-bit floating-point number
uint8 FLOAT32=7

# 64-bit floating-point number
uint8 FLOAT64=8

# Type of data in the field. Integers are stored using little-endian byte order.
uint8 type
================================================================================
MSG: foxglove_msgs/Vector2
# foxglove_msgs/msg/Vector2
# A vector in 2D space that represents a direction only

# Generated by https://github.com/foxglove/foxglove-sdk

# x component
float64 x

# y component
float64 y
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation`,
  "foxglove_msgs/msg/ImageAnnotations": `# foxglove_msgs/msg/ImageAnnotations
# Array of annotations for a 2D image

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of the image annotations. When set, individual annotation timestamps will be ignored.
builtin_interfaces/Time timestamp

# Circle annotations
foxglove_msgs/CircleAnnotation[] circles

# Points annotations
foxglove_msgs/PointsAnnotation[] points

# Text annotations
foxglove_msgs/TextAnnotation[] texts

# Additional user-provided metadata associated with the image annotations. Keys must be unique within this object. Per-annotation metadata takes precedence over these values.
foxglove_msgs/KeyValuePair[] metadata
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: foxglove_msgs/KeyValuePair
# foxglove_msgs/msg/KeyValuePair
# A key with its associated value

# Generated by https://github.com/foxglove/foxglove-sdk

# Key
string key

# Value
string value
================================================================================
MSG: foxglove_msgs/Point2
# foxglove_msgs/msg/Point2
# A point representing a position in 2D space

# Generated by https://github.com/foxglove/foxglove-sdk

# x coordinate position
float64 x

# y coordinate position
float64 y
================================================================================
MSG: foxglove_msgs/CircleAnnotation
# foxglove_msgs/msg/CircleAnnotation
# A circle annotation on a 2D image

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of circle
builtin_interfaces/Time timestamp

# Center of the circle in 2D image coordinates (pixels).
# The coordinate uses the top-left corner of the top-left pixel of the image as the origin.
foxglove_msgs/Point2 position

# Circle diameter in pixels
float64 diameter

# Line thickness in pixels
float64 thickness

# Fill color
foxglove_msgs/Color fill_color

# Outline color
foxglove_msgs/Color outline_color

# Additional user-provided metadata associated with this annotation. Keys must be unique.
foxglove_msgs/KeyValuePair[] metadata
================================================================================
MSG: foxglove_msgs/PointsAnnotation
# foxglove_msgs/msg/PointsAnnotation
# An array of points on a 2D image

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of annotation
builtin_interfaces/Time timestamp

# Unknown points annotation type
uint8 UNKNOWN=0

# Individual points: 0, 1, 2, ...
uint8 POINTS=1

# Closed polygon: 0-1, 1-2, ..., (n-1)-n, n-0
uint8 LINE_LOOP=2

# Connected line segments: 0-1, 1-2, ..., (n-1)-n
uint8 LINE_STRIP=3

# Individual line segments: 0-1, 2-3, 4-5, ...
uint8 LINE_LIST=4

# Type of points annotation to draw
uint8 type

# Points in 2D image coordinates (pixels).
# These coordinates use the top-left corner of the top-left pixel of the image as the origin.
foxglove_msgs/Point2[] points

# Outline color
foxglove_msgs/Color outline_color

# Per-point colors, if \`type\` is \`POINTS\`, or per-segment stroke colors, if \`type\` is \`LINE_LIST\`, \`LINE_STRIP\` or \`LINE_LOOP\`.
foxglove_msgs/Color[] outline_colors

# Fill color
foxglove_msgs/Color fill_color

# Stroke thickness in pixels
float64 thickness

# Additional user-provided metadata associated with this annotation. Keys must be unique.
foxglove_msgs/KeyValuePair[] metadata
================================================================================
MSG: foxglove_msgs/TextAnnotation
# foxglove_msgs/msg/TextAnnotation
# A text label on a 2D image

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of annotation
builtin_interfaces/Time timestamp

# Bottom-left origin of the text label in 2D image coordinates (pixels).
# The coordinate uses the top-left corner of the top-left pixel of the image as the origin.
foxglove_msgs/Point2 position

# Text to display
string text

# Font size in pixels
float64 font_size

# Text color
foxglove_msgs/Color text_color

# Background fill color
foxglove_msgs/Color background_color

# Additional user-provided metadata associated with this annotation. Keys must be unique.
foxglove_msgs/KeyValuePair[] metadata`,
  "foxglove_msgs/msg/JointState": `# foxglove_msgs/msg/JointState
# The state of a single joint (revolute or prismatic).

# Generated by https://github.com/foxglove/foxglove-sdk

# Joint name
string name

# Joint position. Radians for revolute joints, meters for prismatic joints. (NaN indicates this value is not set)
float64 position

# Joint velocity. Rad/s for revolute joints, m/s for prismatic joints. (NaN indicates this value is not set)
float64 velocity

# Joint acceleration. Rad/s² for revolute joints, m/s² for prismatic joints. (NaN indicates this value is not set)
float64 acceleration

# Joint effort (force or torque). Nm for revolute joints, N for prismatic joints. (NaN indicates this value is not set)
float64 effort`,
  "foxglove_msgs/msg/JointStates": `# foxglove_msgs/msg/JointStates
# The state of a set of joints at a given time.

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of the joint states
builtin_interfaces/Time timestamp

# Joint states
foxglove_msgs/JointState[] joints
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/JointState
# foxglove_msgs/msg/JointState
# The state of a single joint (revolute or prismatic).

# Generated by https://github.com/foxglove/foxglove-sdk

# Joint name
string name

# Joint position. Radians for revolute joints, meters for prismatic joints. (NaN indicates this value is not set)
float64 position

# Joint velocity. Rad/s for revolute joints, m/s for prismatic joints. (NaN indicates this value is not set)
float64 velocity

# Joint acceleration. Rad/s² for revolute joints, m/s² for prismatic joints. (NaN indicates this value is not set)
float64 acceleration

# Joint effort (force or torque). Nm for revolute joints, N for prismatic joints. (NaN indicates this value is not set)
float64 effort`,
  "foxglove_msgs/msg/KeyValuePair": `# foxglove_msgs/msg/KeyValuePair
# A key with its associated value

# Generated by https://github.com/foxglove/foxglove-sdk

# Key
string key

# Value
string value`,
  "foxglove_msgs/msg/LaserScan": `# foxglove_msgs/msg/LaserScan
# A single scan from a planar laser range-finder

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of scan
builtin_interfaces/Time timestamp

# Frame of reference
string frame_id

# Origin of scan relative to frame of reference; points are positioned in the x-y plane relative to this origin; angles are interpreted as counterclockwise rotations around the z axis with 0 rad being in the +x direction
geometry_msgs/Pose pose

# Bearing of first point, in radians
float64 start_angle

# Bearing of last point, in radians
float64 end_angle

# Distance of detections from origin; assumed to be at equally-spaced angles between \`start_angle\` and \`end_angle\`
float64[] ranges

# Intensity of detections
float64[] intensities
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation`,
  "foxglove_msgs/msg/LinePrimitive": `# foxglove_msgs/msg/LinePrimitive
# A primitive representing a series of points connected by lines

# Generated by https://github.com/foxglove/foxglove-sdk

# Connected line segments: 0-1, 1-2, ..., (n-1)-n
uint8 LINE_STRIP=0

# Closed polygon: 0-1, 1-2, ..., (n-1)-n, n-0
uint8 LINE_LOOP=1

# Individual line segments: 0-1, 2-3, 4-5, ...
uint8 LINE_LIST=2

# Drawing primitive to use for lines
uint8 type

# Origin of lines relative to reference frame
geometry_msgs/Pose pose

# Line thickness
float64 thickness

# Indicates whether \`thickness\` is a fixed size in screen pixels (true), or specified in world coordinates and scales with distance from the camera (false)
bool scale_invariant

# Points along the line
geometry_msgs/Point[] points

# Solid color to use for the whole line. Ignored if \`colors\` is non-empty.
foxglove_msgs/Color color

# Per-point colors (if non-empty, must have the same length as \`points\`).
foxglove_msgs/Color[] colors

# Indices into the \`points\` and \`colors\` attribute arrays, which can be used to avoid duplicating attribute data.
# 
# If omitted or empty, indexing will not be used. This default behavior is equivalent to specifying [0, 1, ..., N-1] for the indices (where N is the number of \`points\` provided).
uint32[] indices
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation`,
  "foxglove_msgs/msg/LocationFix": `# foxglove_msgs/msg/LocationFix
# A navigation satellite fix for any Global Navigation Satellite System

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of the message
builtin_interfaces/Time timestamp

# Frame for the sensor. Latitude and longitude readings are at the origin of the frame.
string frame_id

# Latitude in degrees
float64 latitude

# Longitude in degrees
float64 longitude

# Altitude in meters
float64 altitude

# Position covariance (m^2) defined relative to a tangential plane through the reported position. The components are East, North, and Up (ENU), in row-major order.
float64[9] position_covariance

# Unknown position covariance type
uint8 UNKNOWN=0

# Position covariance is approximated
uint8 APPROXIMATED=1

# Position covariance is per-axis, so put it along the diagonal
uint8 DIAGONAL_KNOWN=2

# Position covariance of the fix is known
uint8 KNOWN=3

# If \`position_covariance\` is available, \`position_covariance_type\` must be set to indicate the type of covariance.
uint8 position_covariance_type

# Heading (yaw angle), in radians, measured clockwise from north (NaN indicates this value is not set)
float64 heading

# Velocity in local East-North-Up (ENU) frame in m/s
geometry_msgs/Vector3 velocity

# Color used to visualize the location
foxglove_msgs/Color color

# Additional user-provided metadata associated with the location fix. Keys must be unique.
foxglove_msgs/KeyValuePair[] metadata
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: foxglove_msgs/KeyValuePair
# foxglove_msgs/msg/KeyValuePair
# A key with its associated value

# Generated by https://github.com/foxglove/foxglove-sdk

# Key
string key

# Value
string value
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "foxglove_msgs/msg/LocationFixes": `# foxglove_msgs/msg/LocationFixes
# A group of LocationFix messages

# Generated by https://github.com/foxglove/foxglove-sdk

# An array of location fixes
foxglove_msgs/LocationFix[] fixes
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: foxglove_msgs/KeyValuePair
# foxglove_msgs/msg/KeyValuePair
# A key with its associated value

# Generated by https://github.com/foxglove/foxglove-sdk

# Key
string key

# Value
string value
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: foxglove_msgs/LocationFix
# foxglove_msgs/msg/LocationFix
# A navigation satellite fix for any Global Navigation Satellite System

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of the message
builtin_interfaces/Time timestamp

# Frame for the sensor. Latitude and longitude readings are at the origin of the frame.
string frame_id

# Latitude in degrees
float64 latitude

# Longitude in degrees
float64 longitude

# Altitude in meters
float64 altitude

# Position covariance (m^2) defined relative to a tangential plane through the reported position. The components are East, North, and Up (ENU), in row-major order.
float64[9] position_covariance

# Unknown position covariance type
uint8 UNKNOWN=0

# Position covariance is approximated
uint8 APPROXIMATED=1

# Position covariance is per-axis, so put it along the diagonal
uint8 DIAGONAL_KNOWN=2

# Position covariance of the fix is known
uint8 KNOWN=3

# If \`position_covariance\` is available, \`position_covariance_type\` must be set to indicate the type of covariance.
uint8 position_covariance_type

# Heading (yaw angle), in radians, measured clockwise from north (NaN indicates this value is not set)
float64 heading

# Velocity in local East-North-Up (ENU) frame in m/s
geometry_msgs/Vector3 velocity

# Color used to visualize the location
foxglove_msgs/Color color

# Additional user-provided metadata associated with the location fix. Keys must be unique.
foxglove_msgs/KeyValuePair[] metadata`,
  "foxglove_msgs/msg/Log": `# foxglove_msgs/msg/Log
# A log message

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of log message
builtin_interfaces/Time timestamp

# Unknown log level
uint8 UNKNOWN=0

# Debug log level
uint8 DEBUG=1

# Info log level
uint8 INFO=2

# Warning log level
uint8 WARNING=3

# Error log level
uint8 ERROR=4

# Fatal log level
uint8 FATAL=5

# Log level
uint8 level

# Log message
string message

# Process or node name
string name

# Filename
string file

# Line number in the file
uint32 line
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "foxglove_msgs/msg/ModelPrimitive": `# foxglove_msgs/msg/ModelPrimitive
# A primitive representing a 3D model file loaded from an external URL or embedded data

# Generated by https://github.com/foxglove/foxglove-sdk

# Origin of model relative to reference frame
geometry_msgs/Pose pose

# Scale factor to apply to the model along each axis
geometry_msgs/Vector3 scale

# Solid color to use for the whole model if \`override_color\` is true.
foxglove_msgs/Color color

# Whether to use the color specified in \`color\` instead of any materials embedded in the original model.
bool override_color

# URL pointing to model file. One of \`url\` or \`data\` should be non-empty.
string url

# [Media type](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types) of embedded model (e.g. \`model/gltf-binary\`). Required if \`data\` is provided instead of \`url\`. Overrides the inferred media type if \`url\` is provided.
string media_type

# Embedded model. One of \`url\` or \`data\` should be non-empty. If \`data\` is non-empty, \`media_type\` must be set to indicate the type of the data.
uint8[] data
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "foxglove_msgs/msg/Odometry": `# foxglove_msgs/msg/Odometry
# An estimate of position, orientation, and velocity for an object or reference frame in 3D space

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of the message
builtin_interfaces/Time timestamp

# Reference coordinate frame (e.g. \`map\` or \`odom\`)
string frame_id

# Coordinate frame of the body whose motion is being estimated (e.g. \`base_link\`)
string body_frame_id

# Position and orientation of body_frame_id in frame_id
geometry_msgs/Pose pose

# Linear velocity in m/s in body_frame_id
geometry_msgs/Vector3 linear_velocity

# Angular velocity in rad/s in body_frame_id
geometry_msgs/Vector3 angular_velocity

# Row-major 6x6 covariance matrix (x, y, z, rotation about x, rotation about y, rotation about z). Set to zero if unknown. (NaN indicates this value is not set)
float64[36] pose_covariance

# Row-major 6x6 covariance matrix (vx, vy, vz, angular rate about x, angular rate about y, angular rate about z). Set to zero if unknown. (NaN indicates this value is not set)
float64[36] velocity_covariance

# Additional user-provided metadata associated with the odometry message. Keys must be unique.
foxglove_msgs/KeyValuePair[] metadata
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/KeyValuePair
# foxglove_msgs/msg/KeyValuePair
# A key with its associated value

# Generated by https://github.com/foxglove/foxglove-sdk

# Key
string key

# Value
string value
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "foxglove_msgs/msg/PackedElementField": `# foxglove_msgs/msg/PackedElementField
# A field present within each element in a byte array of packed elements.

# Generated by https://github.com/foxglove/foxglove-sdk

# Name of the field
string name

# Byte offset from start of data buffer
uint32 offset

# Unknown numeric type
uint8 UNKNOWN=0

# Unsigned 8-bit integer
uint8 UINT8=1

# Signed 8-bit integer
uint8 INT8=2

# Unsigned 16-bit integer
uint8 UINT16=3

# Signed 16-bit integer
uint8 INT16=4

# Unsigned 32-bit integer
uint8 UINT32=5

# Signed 32-bit integer
uint8 INT32=6

# 32-bit floating-point number
uint8 FLOAT32=7

# 64-bit floating-point number
uint8 FLOAT64=8

# Type of data in the field. Integers are stored using little-endian byte order.
uint8 type`,
  "foxglove_msgs/msg/Point2": `# foxglove_msgs/msg/Point2
# A point representing a position in 2D space

# Generated by https://github.com/foxglove/foxglove-sdk

# x coordinate position
float64 x

# y coordinate position
float64 y`,
  "foxglove_msgs/msg/PointCloud": `# foxglove_msgs/msg/PointCloud
# A collection of N-dimensional points, which may contain additional fields with information like normals, intensity, etc.

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of point cloud
builtin_interfaces/Time timestamp

# Frame of reference
string frame_id

# The origin of the point cloud relative to the frame of reference
geometry_msgs/Pose pose

# Number of bytes between points in the \`data\`
uint32 point_stride

# Fields in \`data\`. At least 2 coordinate fields from \`x\`, \`y\`, and \`z\` are required for each point's position; \`red\`, \`green\`, \`blue\`, and \`alpha\` are optional for customizing each point's color.
foxglove_msgs/PackedElementField[] fields

# Point data, interpreted using \`fields\`
uint8[] data
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/PackedElementField
# foxglove_msgs/msg/PackedElementField
# A field present within each element in a byte array of packed elements.

# Generated by https://github.com/foxglove/foxglove-sdk

# Name of the field
string name

# Byte offset from start of data buffer
uint32 offset

# Unknown numeric type
uint8 UNKNOWN=0

# Unsigned 8-bit integer
uint8 UINT8=1

# Signed 8-bit integer
uint8 INT8=2

# Unsigned 16-bit integer
uint8 UINT16=3

# Signed 16-bit integer
uint8 INT16=4

# Unsigned 32-bit integer
uint8 UINT32=5

# Signed 32-bit integer
uint8 INT32=6

# 32-bit floating-point number
uint8 FLOAT32=7

# 64-bit floating-point number
uint8 FLOAT64=8

# Type of data in the field. Integers are stored using little-endian byte order.
uint8 type
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation`,
  "foxglove_msgs/msg/PointsAnnotation": `# foxglove_msgs/msg/PointsAnnotation
# An array of points on a 2D image

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of annotation
builtin_interfaces/Time timestamp

# Unknown points annotation type
uint8 UNKNOWN=0

# Individual points: 0, 1, 2, ...
uint8 POINTS=1

# Closed polygon: 0-1, 1-2, ..., (n-1)-n, n-0
uint8 LINE_LOOP=2

# Connected line segments: 0-1, 1-2, ..., (n-1)-n
uint8 LINE_STRIP=3

# Individual line segments: 0-1, 2-3, 4-5, ...
uint8 LINE_LIST=4

# Type of points annotation to draw
uint8 type

# Points in 2D image coordinates (pixels).
# These coordinates use the top-left corner of the top-left pixel of the image as the origin.
foxglove_msgs/Point2[] points

# Outline color
foxglove_msgs/Color outline_color

# Per-point colors, if \`type\` is \`POINTS\`, or per-segment stroke colors, if \`type\` is \`LINE_LIST\`, \`LINE_STRIP\` or \`LINE_LOOP\`.
foxglove_msgs/Color[] outline_colors

# Fill color
foxglove_msgs/Color fill_color

# Stroke thickness in pixels
float64 thickness

# Additional user-provided metadata associated with this annotation. Keys must be unique.
foxglove_msgs/KeyValuePair[] metadata
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: foxglove_msgs/KeyValuePair
# foxglove_msgs/msg/KeyValuePair
# A key with its associated value

# Generated by https://github.com/foxglove/foxglove-sdk

# Key
string key

# Value
string value
================================================================================
MSG: foxglove_msgs/Point2
# foxglove_msgs/msg/Point2
# A point representing a position in 2D space

# Generated by https://github.com/foxglove/foxglove-sdk

# x coordinate position
float64 x

# y coordinate position
float64 y`,
  "foxglove_msgs/msg/PoseInFrame": `# foxglove_msgs/msg/PoseInFrame
# A timestamped pose for an object or reference frame in 3D space

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of pose
builtin_interfaces/Time timestamp

# Frame of reference for pose position and orientation
string frame_id

# Pose in 3D space
geometry_msgs/Pose pose
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation`,
  "foxglove_msgs/msg/PosesInFrame": `# foxglove_msgs/msg/PosesInFrame
# An array of timestamped poses for an object or reference frame in 3D space

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of pose
builtin_interfaces/Time timestamp

# Frame of reference for pose position and orientation
string frame_id

# Poses in 3D space
geometry_msgs/Pose[] poses
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation`,
  "foxglove_msgs/msg/RawAudio": `# foxglove_msgs/msg/RawAudio
# A single block of an audio bitstream

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of the start of the audio block
builtin_interfaces/Time timestamp

# Audio data. The samples in the data must be interleaved and little-endian
uint8[] data

# Audio format. Only 'pcm-s16' is currently supported
string format

# Sample rate in Hz
uint32 sample_rate

# Number of channels in the audio block
uint32 number_of_channels
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "foxglove_msgs/msg/RawImage": `# foxglove_msgs/msg/RawImage
# A raw image

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of image
builtin_interfaces/Time timestamp

# Frame of reference for the image. The origin of the frame is the optical center of the camera. +x points to the right in the image, +y points down, and +z points into the plane of the image.
string frame_id

# Image width in pixels
uint32 width

# Image height in pixels
uint32 height

# Encoding of the raw image data. See the \`data\` field description for supported values.
string encoding

# Byte length of a single row. This is usually some multiple of \`width\` depending on the encoding, but can be greater to incorporate padding.
uint32 step

# Raw image data.
# 
# For each \`encoding\` value, the \`data\` field contains image pixel data serialized as follows:
# 
# - \`yuv422\` or \`uyvy\`:
#   - Pixel colors are decomposed into [Y'UV](https://en.wikipedia.org/wiki/Y%E2%80%B2UV) channels.
#   - Pixel channel values are represented as unsigned 8-bit integers.
#   - U and V values are shared between horizontal pairs of pixels. Each pair of output pixels is serialized as [U, Y1, V, Y2].
#   - \`step\` must be greater than or equal to \`width\` * 2.
# - \`yuv422_yuy2\` or  \`yuyv\`:
#   - Pixel colors are decomposed into [Y'UV](https://en.wikipedia.org/wiki/Y%E2%80%B2UV) channels.
#   - Pixel channel values are represented as unsigned 8-bit integers.
#   - U and V values are shared between horizontal pairs of pixels. Each pair of output pixels is encoded as [Y1, U, Y2, V].
#   - \`step\` must be greater than or equal to \`width\` * 2.
# - \`nv12\`:
#   - Pixel colors are decomposed into [Y'UV](https://en.wikipedia.org/wiki/Y%E2%80%B2UV) channels using 4:2:0 chroma subsampling. The data is stored in [NV12](https://www.kernel.org/doc/html/v4.10/media/uapi/v4l/pixfmt-nv12.html) semi-planar layout with two contiguous planes: a Y (luma) plane followed by an interleaved UV (chroma) plane.
#   - All channel values are represented as unsigned 8-bit integers.
#   - Both planes use \`step\` as their row stride.
#   - The Y plane contains one luma value per pixel (\`step\` * \`height\` bytes).
#   - The UV plane contains interleaved U, V chroma pairs, subsampled by a factor of 2 in both dimensions (\`width\`/2 pairs per row, \`height\`/2 rows, \`step\` * \`height\`/2 bytes). Each U, V pair is shared by a 2x2 block of pixels.
#   - \`width\` and \`height\` must be even.
#   - \`step\` must be greater than or equal to \`width\`.
#   - Total \`data\` length is \`step\` * \`height\` * 3/2 bytes.
# - \`rgb8\`:
#   - Pixel colors are decomposed into Red, Green, and Blue channels.
#   - Pixel channel values are represented as unsigned 8-bit integers.
#   - Each output pixel is serialized as [R, G, B].
#   - \`step\` must be greater than or equal to \`width\` * 3.
# - \`rgba8\`:
#   - Pixel colors are decomposed into Red, Green, Blue, and Alpha channels.
#   - Pixel channel values are represented as unsigned 8-bit integers.
#   - Each output pixel is serialized as [R, G, B, Alpha].
#   - \`step\` must be greater than or equal to \`width\` * 4.
# - \`bgr8\` or \`8UC3\`:
#   - Pixel colors are decomposed into Blue, Green, and Red channels.
#   - Pixel channel values are represented as unsigned 8-bit integers.
#   - Each output pixel is serialized as [B, G, R].
#   - \`step\` must be greater than or equal to \`width\` * 3.
# - \`bgra8\`:
#   - Pixel colors are decomposed into Blue, Green, Red, and Alpha channels.
#   - Pixel channel values are represented as unsigned 8-bit integers.
#   - Each output pixel is encoded as [B, G, R, Alpha].
#   - \`step\` must be greater than or equal to \`width\` * 4.
# - \`32FC1\`:
#   - Pixel brightness is represented as a single-channel, 32-bit little-endian IEEE 754 floating-point value, ranging from 0.0 (black) to 1.0 (white).
#   - \`step\` must be greater than or equal to \`width\` * 4.
# - \`bayer_rggb8\`, \`bayer_bggr8\`, \`bayer_gbrg8\`, or \`bayer_grbg8\`:
#   - Pixel colors are decomposed into Red, Blue and Green channels.
#   - Pixel channel values are represented as unsigned 8-bit integers, and serialized in a 2x2 bayer filter pattern.
#   - The order of the four letters after \`bayer_\` determine the layout, so for \`bayer_wxyz8\` the pattern is:
#   \`\`\`text
#   w | x
#   - + -
#   y | z
#   \`\`\`
#   - \`step\` must be greater than or equal to \`width\`.
# - \`mono8\` or \`8UC1\`:
#   - Pixel brightness is represented as unsigned 8-bit integers.
#   - \`step\` must be greater than or equal to \`width\`.
# - \`mono16\` or \`16UC1\`:
#   - Pixel brightness is represented as 16-bit unsigned little-endian integers. Rendering of these values is controlled in [Image panel color mode settings](https://docs.foxglove.dev/docs/visualization/panels/image#general).
#   - \`step\` must be greater than or equal to \`width\` * 2.
uint8[] data
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "foxglove_msgs/msg/SceneEntity": `# foxglove_msgs/msg/SceneEntity
# A visual element in a 3D scene. An entity may be composed of multiple primitives which all share the same frame of reference.

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of the entity
builtin_interfaces/Time timestamp

# Frame of reference
string frame_id

# Identifier for the entity. A entity will replace any prior entity on the same topic with the same \`id\`.
string id

# Length of time (relative to \`timestamp\`) after which the entity should be automatically removed. Zero value indicates the entity should remain visible until it is replaced or deleted.
builtin_interfaces/Duration lifetime

# False indicates the entity should keep its location in the fixed frame until a new entity is published. True indicates the entity should follow the frame specified in \`frame_id\` as it moves relative to the fixed frame when new transform messages are received.
bool frame_locked

# Additional user-provided metadata associated with the entity. Keys must be unique.
foxglove_msgs/KeyValuePair[] metadata

# Arrow primitives
foxglove_msgs/ArrowPrimitive[] arrows

# Cube primitives
foxglove_msgs/CubePrimitive[] cubes

# Sphere primitives
foxglove_msgs/SpherePrimitive[] spheres

# Cylinder primitives
foxglove_msgs/CylinderPrimitive[] cylinders

# Line primitives
foxglove_msgs/LinePrimitive[] lines

# Triangle list primitives
foxglove_msgs/TriangleListPrimitive[] triangles

# Text primitives
foxglove_msgs/TextPrimitive[] texts

# Model primitives
foxglove_msgs/ModelPrimitive[] models
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: foxglove_msgs/ArrowPrimitive
# foxglove_msgs/msg/ArrowPrimitive
# A primitive representing an arrow

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the arrow's tail and orientation of the arrow. Identity orientation means the arrow points in the +x direction.
geometry_msgs/Pose pose

# Length of the arrow shaft
float64 shaft_length

# Diameter of the arrow shaft
float64 shaft_diameter

# Length of the arrow head
float64 head_length

# Diameter of the arrow head
float64 head_diameter

# Color of the arrow
foxglove_msgs/Color color
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: foxglove_msgs/CubePrimitive
# foxglove_msgs/msg/CubePrimitive
# A primitive representing a cube or rectangular prism

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the center of the cube and orientation of the cube
geometry_msgs/Pose pose

# Size of the cube along each axis
geometry_msgs/Vector3 size

# Color of the cube
foxglove_msgs/Color color
================================================================================
MSG: foxglove_msgs/CylinderPrimitive
# foxglove_msgs/msg/CylinderPrimitive
# A primitive representing a cylinder, elliptic cylinder, or truncated cone

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the center of the cylinder and orientation of the cylinder. The flat face(s) are perpendicular to the z-axis.
geometry_msgs/Pose pose

# Size of the cylinder's bounding box
geometry_msgs/Vector3 size

# 0-1, ratio of the diameter of the cylinder's bottom face (min z) to the bottom of the bounding box
float64 bottom_scale

# 0-1, ratio of the diameter of the cylinder's top face (max z) to the top of the bounding box
float64 top_scale

# Color of the cylinder
foxglove_msgs/Color color
================================================================================
MSG: foxglove_msgs/KeyValuePair
# foxglove_msgs/msg/KeyValuePair
# A key with its associated value

# Generated by https://github.com/foxglove/foxglove-sdk

# Key
string key

# Value
string value
================================================================================
MSG: foxglove_msgs/LinePrimitive
# foxglove_msgs/msg/LinePrimitive
# A primitive representing a series of points connected by lines

# Generated by https://github.com/foxglove/foxglove-sdk

# Connected line segments: 0-1, 1-2, ..., (n-1)-n
uint8 LINE_STRIP=0

# Closed polygon: 0-1, 1-2, ..., (n-1)-n, n-0
uint8 LINE_LOOP=1

# Individual line segments: 0-1, 2-3, 4-5, ...
uint8 LINE_LIST=2

# Drawing primitive to use for lines
uint8 type

# Origin of lines relative to reference frame
geometry_msgs/Pose pose

# Line thickness
float64 thickness

# Indicates whether \`thickness\` is a fixed size in screen pixels (true), or specified in world coordinates and scales with distance from the camera (false)
bool scale_invariant

# Points along the line
geometry_msgs/Point[] points

# Solid color to use for the whole line. Ignored if \`colors\` is non-empty.
foxglove_msgs/Color color

# Per-point colors (if non-empty, must have the same length as \`points\`).
foxglove_msgs/Color[] colors

# Indices into the \`points\` and \`colors\` attribute arrays, which can be used to avoid duplicating attribute data.
# 
# If omitted or empty, indexing will not be used. This default behavior is equivalent to specifying [0, 1, ..., N-1] for the indices (where N is the number of \`points\` provided).
uint32[] indices
================================================================================
MSG: foxglove_msgs/ModelPrimitive
# foxglove_msgs/msg/ModelPrimitive
# A primitive representing a 3D model file loaded from an external URL or embedded data

# Generated by https://github.com/foxglove/foxglove-sdk

# Origin of model relative to reference frame
geometry_msgs/Pose pose

# Scale factor to apply to the model along each axis
geometry_msgs/Vector3 scale

# Solid color to use for the whole model if \`override_color\` is true.
foxglove_msgs/Color color

# Whether to use the color specified in \`color\` instead of any materials embedded in the original model.
bool override_color

# URL pointing to model file. One of \`url\` or \`data\` should be non-empty.
string url

# [Media type](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types) of embedded model (e.g. \`model/gltf-binary\`). Required if \`data\` is provided instead of \`url\`. Overrides the inferred media type if \`url\` is provided.
string media_type

# Embedded model. One of \`url\` or \`data\` should be non-empty. If \`data\` is non-empty, \`media_type\` must be set to indicate the type of the data.
uint8[] data
================================================================================
MSG: foxglove_msgs/SpherePrimitive
# foxglove_msgs/msg/SpherePrimitive
# A primitive representing a sphere or ellipsoid

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the center of the sphere and orientation of the sphere
geometry_msgs/Pose pose

# Size (diameter) of the sphere along each axis
geometry_msgs/Vector3 size

# Color of the sphere
foxglove_msgs/Color color
================================================================================
MSG: foxglove_msgs/TextPrimitive
# foxglove_msgs/msg/TextPrimitive
# A primitive representing a text label

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the center of the text box and orientation of the text. Identity orientation means the text is oriented in the xy-plane and flows from -x to +x.
geometry_msgs/Pose pose

# Whether the text should respect \`pose.orientation\` (false) or always face the camera (true)
bool billboard

# Font size (height of one line of text)
float64 font_size

# Indicates whether \`font_size\` is a fixed size in screen pixels (true), or specified in world coordinates and scales with distance from the camera (false)
bool scale_invariant

# Color of the text
foxglove_msgs/Color color

# Text
string text
================================================================================
MSG: foxglove_msgs/TriangleListPrimitive
# foxglove_msgs/msg/TriangleListPrimitive
# A primitive representing a set of triangles or a surface tiled by triangles

# Generated by https://github.com/foxglove/foxglove-sdk

# Origin of triangles relative to reference frame
geometry_msgs/Pose pose

# Vertices to use for triangles, interpreted as a list of triples (0-1-2, 3-4-5, ...)
geometry_msgs/Point[] points

# Solid color to use for the whole shape. Ignored if \`colors\` is non-empty.
foxglove_msgs/Color color

# Per-vertex colors (if specified, must have the same length as \`points\`).
foxglove_msgs/Color[] colors

# Indices into the \`points\` and \`colors\` attribute arrays, which can be used to avoid duplicating attribute data.
# 
# If omitted or empty, indexing will not be used. This default behavior is equivalent to specifying [0, 1, ..., N-1] for the indices (where N is the number of \`points\` provided).
uint32[] indices`,
  "foxglove_msgs/msg/SceneEntityDeletion": `# foxglove_msgs/msg/SceneEntityDeletion
# Command to remove previously published entities

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of the deletion. Only matching entities earlier than this timestamp will be deleted.
builtin_interfaces/Time timestamp

# Delete the existing entity on the same topic that has the provided \`id\`
uint8 MATCHING_ID=0

# Delete all existing entities on the same topic
uint8 ALL=1

# Type of deletion action to perform
uint8 type

# Identifier which must match if \`type\` is \`MATCHING_ID\`.
string id
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "foxglove_msgs/msg/SceneUpdate": `# foxglove_msgs/msg/SceneUpdate
# An update to the entities displayed in a 3D scene

# Generated by https://github.com/foxglove/foxglove-sdk

# Scene entities to delete
foxglove_msgs/SceneEntityDeletion[] deletions

# Scene entities to add or replace
foxglove_msgs/SceneEntity[] entities
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: foxglove_msgs/ArrowPrimitive
# foxglove_msgs/msg/ArrowPrimitive
# A primitive representing an arrow

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the arrow's tail and orientation of the arrow. Identity orientation means the arrow points in the +x direction.
geometry_msgs/Pose pose

# Length of the arrow shaft
float64 shaft_length

# Diameter of the arrow shaft
float64 shaft_diameter

# Length of the arrow head
float64 head_length

# Diameter of the arrow head
float64 head_diameter

# Color of the arrow
foxglove_msgs/Color color
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: foxglove_msgs/CubePrimitive
# foxglove_msgs/msg/CubePrimitive
# A primitive representing a cube or rectangular prism

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the center of the cube and orientation of the cube
geometry_msgs/Pose pose

# Size of the cube along each axis
geometry_msgs/Vector3 size

# Color of the cube
foxglove_msgs/Color color
================================================================================
MSG: foxglove_msgs/CylinderPrimitive
# foxglove_msgs/msg/CylinderPrimitive
# A primitive representing a cylinder, elliptic cylinder, or truncated cone

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the center of the cylinder and orientation of the cylinder. The flat face(s) are perpendicular to the z-axis.
geometry_msgs/Pose pose

# Size of the cylinder's bounding box
geometry_msgs/Vector3 size

# 0-1, ratio of the diameter of the cylinder's bottom face (min z) to the bottom of the bounding box
float64 bottom_scale

# 0-1, ratio of the diameter of the cylinder's top face (max z) to the top of the bounding box
float64 top_scale

# Color of the cylinder
foxglove_msgs/Color color
================================================================================
MSG: foxglove_msgs/KeyValuePair
# foxglove_msgs/msg/KeyValuePair
# A key with its associated value

# Generated by https://github.com/foxglove/foxglove-sdk

# Key
string key

# Value
string value
================================================================================
MSG: foxglove_msgs/LinePrimitive
# foxglove_msgs/msg/LinePrimitive
# A primitive representing a series of points connected by lines

# Generated by https://github.com/foxglove/foxglove-sdk

# Connected line segments: 0-1, 1-2, ..., (n-1)-n
uint8 LINE_STRIP=0

# Closed polygon: 0-1, 1-2, ..., (n-1)-n, n-0
uint8 LINE_LOOP=1

# Individual line segments: 0-1, 2-3, 4-5, ...
uint8 LINE_LIST=2

# Drawing primitive to use for lines
uint8 type

# Origin of lines relative to reference frame
geometry_msgs/Pose pose

# Line thickness
float64 thickness

# Indicates whether \`thickness\` is a fixed size in screen pixels (true), or specified in world coordinates and scales with distance from the camera (false)
bool scale_invariant

# Points along the line
geometry_msgs/Point[] points

# Solid color to use for the whole line. Ignored if \`colors\` is non-empty.
foxglove_msgs/Color color

# Per-point colors (if non-empty, must have the same length as \`points\`).
foxglove_msgs/Color[] colors

# Indices into the \`points\` and \`colors\` attribute arrays, which can be used to avoid duplicating attribute data.
# 
# If omitted or empty, indexing will not be used. This default behavior is equivalent to specifying [0, 1, ..., N-1] for the indices (where N is the number of \`points\` provided).
uint32[] indices
================================================================================
MSG: foxglove_msgs/ModelPrimitive
# foxglove_msgs/msg/ModelPrimitive
# A primitive representing a 3D model file loaded from an external URL or embedded data

# Generated by https://github.com/foxglove/foxglove-sdk

# Origin of model relative to reference frame
geometry_msgs/Pose pose

# Scale factor to apply to the model along each axis
geometry_msgs/Vector3 scale

# Solid color to use for the whole model if \`override_color\` is true.
foxglove_msgs/Color color

# Whether to use the color specified in \`color\` instead of any materials embedded in the original model.
bool override_color

# URL pointing to model file. One of \`url\` or \`data\` should be non-empty.
string url

# [Media type](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types) of embedded model (e.g. \`model/gltf-binary\`). Required if \`data\` is provided instead of \`url\`. Overrides the inferred media type if \`url\` is provided.
string media_type

# Embedded model. One of \`url\` or \`data\` should be non-empty. If \`data\` is non-empty, \`media_type\` must be set to indicate the type of the data.
uint8[] data
================================================================================
MSG: foxglove_msgs/SpherePrimitive
# foxglove_msgs/msg/SpherePrimitive
# A primitive representing a sphere or ellipsoid

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the center of the sphere and orientation of the sphere
geometry_msgs/Pose pose

# Size (diameter) of the sphere along each axis
geometry_msgs/Vector3 size

# Color of the sphere
foxglove_msgs/Color color
================================================================================
MSG: foxglove_msgs/TextPrimitive
# foxglove_msgs/msg/TextPrimitive
# A primitive representing a text label

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the center of the text box and orientation of the text. Identity orientation means the text is oriented in the xy-plane and flows from -x to +x.
geometry_msgs/Pose pose

# Whether the text should respect \`pose.orientation\` (false) or always face the camera (true)
bool billboard

# Font size (height of one line of text)
float64 font_size

# Indicates whether \`font_size\` is a fixed size in screen pixels (true), or specified in world coordinates and scales with distance from the camera (false)
bool scale_invariant

# Color of the text
foxglove_msgs/Color color

# Text
string text
================================================================================
MSG: foxglove_msgs/TriangleListPrimitive
# foxglove_msgs/msg/TriangleListPrimitive
# A primitive representing a set of triangles or a surface tiled by triangles

# Generated by https://github.com/foxglove/foxglove-sdk

# Origin of triangles relative to reference frame
geometry_msgs/Pose pose

# Vertices to use for triangles, interpreted as a list of triples (0-1-2, 3-4-5, ...)
geometry_msgs/Point[] points

# Solid color to use for the whole shape. Ignored if \`colors\` is non-empty.
foxglove_msgs/Color color

# Per-vertex colors (if specified, must have the same length as \`points\`).
foxglove_msgs/Color[] colors

# Indices into the \`points\` and \`colors\` attribute arrays, which can be used to avoid duplicating attribute data.
# 
# If omitted or empty, indexing will not be used. This default behavior is equivalent to specifying [0, 1, ..., N-1] for the indices (where N is the number of \`points\` provided).
uint32[] indices
================================================================================
MSG: foxglove_msgs/SceneEntity
# foxglove_msgs/msg/SceneEntity
# A visual element in a 3D scene. An entity may be composed of multiple primitives which all share the same frame of reference.

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of the entity
builtin_interfaces/Time timestamp

# Frame of reference
string frame_id

# Identifier for the entity. A entity will replace any prior entity on the same topic with the same \`id\`.
string id

# Length of time (relative to \`timestamp\`) after which the entity should be automatically removed. Zero value indicates the entity should remain visible until it is replaced or deleted.
builtin_interfaces/Duration lifetime

# False indicates the entity should keep its location in the fixed frame until a new entity is published. True indicates the entity should follow the frame specified in \`frame_id\` as it moves relative to the fixed frame when new transform messages are received.
bool frame_locked

# Additional user-provided metadata associated with the entity. Keys must be unique.
foxglove_msgs/KeyValuePair[] metadata

# Arrow primitives
foxglove_msgs/ArrowPrimitive[] arrows

# Cube primitives
foxglove_msgs/CubePrimitive[] cubes

# Sphere primitives
foxglove_msgs/SpherePrimitive[] spheres

# Cylinder primitives
foxglove_msgs/CylinderPrimitive[] cylinders

# Line primitives
foxglove_msgs/LinePrimitive[] lines

# Triangle list primitives
foxglove_msgs/TriangleListPrimitive[] triangles

# Text primitives
foxglove_msgs/TextPrimitive[] texts

# Model primitives
foxglove_msgs/ModelPrimitive[] models
================================================================================
MSG: foxglove_msgs/SceneEntityDeletion
# foxglove_msgs/msg/SceneEntityDeletion
# Command to remove previously published entities

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of the deletion. Only matching entities earlier than this timestamp will be deleted.
builtin_interfaces/Time timestamp

# Delete the existing entity on the same topic that has the provided \`id\`
uint8 MATCHING_ID=0

# Delete all existing entities on the same topic
uint8 ALL=1

# Type of deletion action to perform
uint8 type

# Identifier which must match if \`type\` is \`MATCHING_ID\`.
string id`,
  "foxglove_msgs/msg/SpherePrimitive": `# foxglove_msgs/msg/SpherePrimitive
# A primitive representing a sphere or ellipsoid

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the center of the sphere and orientation of the sphere
geometry_msgs/Pose pose

# Size (diameter) of the sphere along each axis
geometry_msgs/Vector3 size

# Color of the sphere
foxglove_msgs/Color color
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "foxglove_msgs/msg/TextAnnotation": `# foxglove_msgs/msg/TextAnnotation
# A text label on a 2D image

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of annotation
builtin_interfaces/Time timestamp

# Bottom-left origin of the text label in 2D image coordinates (pixels).
# The coordinate uses the top-left corner of the top-left pixel of the image as the origin.
foxglove_msgs/Point2 position

# Text to display
string text

# Font size in pixels
float64 font_size

# Text color
foxglove_msgs/Color text_color

# Background fill color
foxglove_msgs/Color background_color

# Additional user-provided metadata associated with this annotation. Keys must be unique.
foxglove_msgs/KeyValuePair[] metadata
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: foxglove_msgs/KeyValuePair
# foxglove_msgs/msg/KeyValuePair
# A key with its associated value

# Generated by https://github.com/foxglove/foxglove-sdk

# Key
string key

# Value
string value
================================================================================
MSG: foxglove_msgs/Point2
# foxglove_msgs/msg/Point2
# A point representing a position in 2D space

# Generated by https://github.com/foxglove/foxglove-sdk

# x coordinate position
float64 x

# y coordinate position
float64 y`,
  "foxglove_msgs/msg/TextPrimitive": `# foxglove_msgs/msg/TextPrimitive
# A primitive representing a text label

# Generated by https://github.com/foxglove/foxglove-sdk

# Position of the center of the text box and orientation of the text. Identity orientation means the text is oriented in the xy-plane and flows from -x to +x.
geometry_msgs/Pose pose

# Whether the text should respect \`pose.orientation\` (false) or always face the camera (true)
bool billboard

# Font size (height of one line of text)
float64 font_size

# Indicates whether \`font_size\` is a fixed size in screen pixels (true), or specified in world coordinates and scales with distance from the camera (false)
bool scale_invariant

# Color of the text
foxglove_msgs/Color color

# Text
string text
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation`,
  "foxglove_msgs/msg/TriangleListPrimitive": `# foxglove_msgs/msg/TriangleListPrimitive
# A primitive representing a set of triangles or a surface tiled by triangles

# Generated by https://github.com/foxglove/foxglove-sdk

# Origin of triangles relative to reference frame
geometry_msgs/Pose pose

# Vertices to use for triangles, interpreted as a list of triples (0-1-2, 3-4-5, ...)
geometry_msgs/Point[] points

# Solid color to use for the whole shape. Ignored if \`colors\` is non-empty.
foxglove_msgs/Color color

# Per-vertex colors (if specified, must have the same length as \`points\`).
foxglove_msgs/Color[] colors

# Indices into the \`points\` and \`colors\` attribute arrays, which can be used to avoid duplicating attribute data.
# 
# If omitted or empty, indexing will not be used. This default behavior is equivalent to specifying [0, 1, ..., N-1] for the indices (where N is the number of \`points\` provided).
uint32[] indices
================================================================================
MSG: foxglove_msgs/Color
# foxglove_msgs/msg/Color
# A color in RGBA format

# Generated by https://github.com/foxglove/foxglove-sdk

# Red value between 0 and 1
float64 r

# Green value between 0 and 1
float64 g

# Blue value between 0 and 1
float64 b

# Alpha value between 0 and 1
float64 a
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation`,
  "foxglove_msgs/msg/Vector2": `# foxglove_msgs/msg/Vector2
# A vector in 2D space that represents a direction only

# Generated by https://github.com/foxglove/foxglove-sdk

# x component
float64 x

# y component
float64 y`,
  "foxglove_msgs/msg/VoxelGrid": `# foxglove_msgs/msg/VoxelGrid
# A 3D grid of data

# Generated by https://github.com/foxglove/foxglove-sdk

# Timestamp of grid
builtin_interfaces/Time timestamp

# Frame of reference
string frame_id

# Origin of the grid’s lower-front-left corner in the reference frame. The grid’s pose is defined relative to this corner, so an untransformed grid with an identity orientation has this corner at the origin.
geometry_msgs/Pose pose

# Number of grid rows
uint32 row_count

# Number of grid columns
uint32 column_count

# Size of single grid cell along x, y, and z axes, relative to \`pose\`
geometry_msgs/Vector3 cell_size

# Number of bytes between depth slices in \`data\`
uint32 slice_stride

# Number of bytes between rows in \`data\`
uint32 row_stride

# Number of bytes between cells within a row in \`data\`
uint32 cell_stride

# Fields in \`data\`. \`red\`, \`green\`, \`blue\`, and \`alpha\` are optional for customizing the grid's color.
foxglove_msgs/PackedElementField[] fields

# Grid cell data, interpreted using \`fields\`, in depth-major, row-major (Z-Y-X) order.
# For the data element starting at byte offset i, the coordinates of its corner closest to the origin will be:
# 
# - z = i / slice_stride * cell_size.z
# - y = (i % slice_stride) / row_stride * cell_size.y
# - x = (i % row_stride) / cell_stride * cell_size.x
uint8[] data
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: foxglove_msgs/PackedElementField
# foxglove_msgs/msg/PackedElementField
# A field present within each element in a byte array of packed elements.

# Generated by https://github.com/foxglove/foxglove-sdk

# Name of the field
string name

# Byte offset from start of data buffer
uint32 offset

# Unknown numeric type
uint8 UNKNOWN=0

# Unsigned 8-bit integer
uint8 UINT8=1

# Signed 8-bit integer
uint8 INT8=2

# Unsigned 16-bit integer
uint8 UINT16=3

# Signed 16-bit integer
uint8 INT16=4

# Unsigned 32-bit integer
uint8 UINT32=5

# Signed 32-bit integer
uint8 INT32=6

# 32-bit floating-point number
uint8 FLOAT32=7

# 64-bit floating-point number
uint8 FLOAT64=8

# Type of data in the field. Integers are stored using little-endian byte order.
uint8 type
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "geometry_msgs/msg/Accel": `# This expresses acceleration in free space broken into its linear and angular parts.
Vector3  linear
Vector3  angular
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "geometry_msgs/msg/AccelStamped": `# An accel with reference coordinate frame and timestamp
std_msgs/Header header
Accel accel
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Accel
# This expresses acceleration in free space broken into its linear and angular parts.
Vector3  linear
Vector3  angular
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/AccelWithCovariance": `# This expresses acceleration in free space with uncertainty.

Accel accel

# Row-major representation of the 6x6 covariance matrix
# The orientation parameters use a fixed-axis representation.
# In order, the parameters are:
# (x, y, z, rotation about X axis, rotation about Y axis, rotation about Z axis)
float64[36] covariance
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Accel
# This expresses acceleration in free space broken into its linear and angular parts.
Vector3  linear
Vector3  angular`,
  "geometry_msgs/msg/AccelWithCovarianceStamped": `# This represents an estimated accel with reference coordinate frame and timestamp.
std_msgs/Header header
AccelWithCovariance accel
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Accel
# This expresses acceleration in free space broken into its linear and angular parts.
Vector3  linear
Vector3  angular
================================================================================
MSG: geometry_msgs/AccelWithCovariance
# This expresses acceleration in free space with uncertainty.

Accel accel

# Row-major representation of the 6x6 covariance matrix
# The orientation parameters use a fixed-axis representation.
# In order, the parameters are:
# (x, y, z, rotation about X axis, rotation about Y axis, rotation about Z axis)
float64[36] covariance
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/Inertia": `# Mass [kg]
float64 m

# Center of mass [m]
geometry_msgs/Vector3 com

# Inertia Tensor [kg-m^2] about the center of mass
#     | ixx ixy ixz |
# I = | ixy iyy iyz |
#     | ixz iyz izz |
float64 ixx
float64 ixy
float64 ixz
float64 iyy
float64 iyz
float64 izz
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "geometry_msgs/msg/InertiaStamped": `# An Inertia with a time stamp and reference frame.

std_msgs/Header header
Inertia inertia
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Inertia
# Mass [kg]
float64 m

# Center of mass [m]
geometry_msgs/Vector3 com

# Inertia Tensor [kg-m^2] about the center of mass
#     | ixx ixy ixz |
# I = | ixy iyy iyz |
#     | ixz iyz izz |
float64 ixx
float64 ixy
float64 ixz
float64 iyy
float64 iyz
float64 izz
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/Point": `# This contains the position of a point in free space
float64 x
float64 y
float64 z`,
  "geometry_msgs/msg/Point32": `# This contains the position of a point in free space(with 32 bits of precision).
# It is recommended to use Point wherever possible instead of Point32.
#
# This recommendation is to promote interoperability.
#
# This message is designed to take up less space when sending
# lots of points at once, as in the case of a PointCloud.

float32 x
float32 y
float32 z`,
  "geometry_msgs/msg/PointStamped": `# This represents a Point with reference coordinate frame and timestamp

std_msgs/Header header
Point point
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/Polygon": `# A specification of a polygon where the first and last points are assumed to be connected

Point32[] points
================================================================================
MSG: geometry_msgs/Point32
# This contains the position of a point in free space(with 32 bits of precision).
# It is recommended to use Point wherever possible instead of Point32.
#
# This recommendation is to promote interoperability.
#
# This message is designed to take up less space when sending
# lots of points at once, as in the case of a PointCloud.

float32 x
float32 y
float32 z`,
  "geometry_msgs/msg/PolygonInstance": `# A specification of a polygon where the first and last points are assumed to be connected
# It includes a unique identification field for disambiguating multiple instances

geometry_msgs/Polygon polygon
int64 id
================================================================================
MSG: geometry_msgs/Point32
# This contains the position of a point in free space(with 32 bits of precision).
# It is recommended to use Point wherever possible instead of Point32.
#
# This recommendation is to promote interoperability.
#
# This message is designed to take up less space when sending
# lots of points at once, as in the case of a PointCloud.

float32 x
float32 y
float32 z
================================================================================
MSG: geometry_msgs/Polygon
# A specification of a polygon where the first and last points are assumed to be connected

Point32[] points`,
  "geometry_msgs/msg/PolygonInstanceStamped": `# This represents a Polygon with reference coordinate frame and timestamp
# It includes a unique identification field for disambiguating multiple instances

std_msgs/Header header
geometry_msgs/PolygonInstance polygon
================================================================================
MSG: geometry_msgs/Point32
# This contains the position of a point in free space(with 32 bits of precision).
# It is recommended to use Point wherever possible instead of Point32.
#
# This recommendation is to promote interoperability.
#
# This message is designed to take up less space when sending
# lots of points at once, as in the case of a PointCloud.

float32 x
float32 y
float32 z
================================================================================
MSG: geometry_msgs/Polygon
# A specification of a polygon where the first and last points are assumed to be connected

Point32[] points
================================================================================
MSG: geometry_msgs/PolygonInstance
# A specification of a polygon where the first and last points are assumed to be connected
# It includes a unique identification field for disambiguating multiple instances

geometry_msgs/Polygon polygon
int64 id
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/PolygonStamped": `# This represents a Polygon with reference coordinate frame and timestamp

std_msgs/Header header
Polygon polygon
================================================================================
MSG: geometry_msgs/Point32
# This contains the position of a point in free space(with 32 bits of precision).
# It is recommended to use Point wherever possible instead of Point32.
#
# This recommendation is to promote interoperability.
#
# This message is designed to take up less space when sending
# lots of points at once, as in the case of a PointCloud.

float32 x
float32 y
float32 z
================================================================================
MSG: geometry_msgs/Polygon
# A specification of a polygon where the first and last points are assumed to be connected

Point32[] points
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/Pose": `# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1`,
  "geometry_msgs/msg/Pose2D": `# Deprecated as of Foxy and will potentially be removed in any following release.
# Please use the full 3D pose.

# In general our recommendation is to use a full 3D representation of everything and for 2D specific applications make the appropriate projections into the plane for their calculations but optimally will preserve the 3D information during processing.

# If we have parallel copies of 2D datatypes every UI and other pipeline will end up needing to have dual interfaces to plot everything. And you will end up with not being able to use 3D tools for 2D use cases even if they're completely valid, as you'd have to reimplement it with different inputs and outputs. It's not particularly hard to plot the 2D pose or compute the yaw error for the Pose message and there are already tools and libraries that can do this for you.# This expresses a position and orientation on a 2D manifold.

float64 x
float64 y
float64 theta`,
  "geometry_msgs/msg/PoseArray": `# An array of poses with a header for global reference.

std_msgs/Header header

Pose[] poses
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/PoseStamped": `# A Pose with reference coordinate frame and timestamp

std_msgs/Header header
Pose pose
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/PoseWithCovariance": `# This represents a pose in free space with uncertainty.

Pose pose

# Row-major representation of the 6x6 covariance matrix
# The orientation parameters use a fixed-axis representation.
# In order, the parameters are:
# (x, y, z, rotation about X axis, rotation about Y axis, rotation about Z axis)
float64[36] covariance
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation`,
  "geometry_msgs/msg/PoseWithCovarianceStamped": `# This expresses an estimated pose with a reference coordinate frame and timestamp

std_msgs/Header header
PoseWithCovariance pose
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/PoseWithCovariance
# This represents a pose in free space with uncertainty.

Pose pose

# Row-major representation of the 6x6 covariance matrix
# The orientation parameters use a fixed-axis representation.
# In order, the parameters are:
# (x, y, z, rotation about X axis, rotation about Y axis, rotation about Z axis)
float64[36] covariance
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/Quaternion": `# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1`,
  "geometry_msgs/msg/QuaternionStamped": `# This represents an orientation with reference coordinate frame and timestamp.

std_msgs/Header header
Quaternion quaternion
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/Transform": `# This represents the transform between two coordinate frames in free space.

Vector3 translation
Quaternion rotation
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "geometry_msgs/msg/TransformStamped": `# This expresses a transform from coordinate frame header.frame_id
# to the coordinate frame child_frame_id at the time of header.stamp
#
# This message is mostly used by the
# <a href="https://docs.ros.org/en/rolling/p/tf2/">tf2</a> package.
# See its documentation for more information.
#
# The child_frame_id is necessary in addition to the frame_id
# in the Header to communicate the full reference for the transform
# in a self contained message.

# The frame id in the header is used as the reference frame of this transform.
std_msgs/Header header

# The frame id of the child frame to which this transform points.
string child_frame_id

# Translation and rotation in 3-dimensions of child_frame_id from header.frame_id.
Transform transform
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Transform
# This represents the transform between two coordinate frames in free space.

Vector3 translation
Quaternion rotation
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/Twist": `# This expresses velocity in free space broken into its linear and angular parts.

Vector3  linear
Vector3  angular
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "geometry_msgs/msg/TwistStamped": `# A twist with reference coordinate frame and timestamp

std_msgs/Header header
Twist twist
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Twist
# This expresses velocity in free space broken into its linear and angular parts.

Vector3  linear
Vector3  angular
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/TwistWithCovariance": `# This expresses velocity in free space with uncertainty.

Twist twist

# Row-major representation of the 6x6 covariance matrix
# The orientation parameters use a fixed-axis representation.
# In order, the parameters are:
# (x, y, z, rotation about X axis, rotation about Y axis, rotation about Z axis)
float64[36] covariance
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Twist
# This expresses velocity in free space broken into its linear and angular parts.

Vector3  linear
Vector3  angular`,
  "geometry_msgs/msg/TwistWithCovarianceStamped": `# This represents an estimated twist with reference coordinate frame and timestamp.

std_msgs/Header header
TwistWithCovariance twist
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Twist
# This expresses velocity in free space broken into its linear and angular parts.

Vector3  linear
Vector3  angular
================================================================================
MSG: geometry_msgs/TwistWithCovariance
# This expresses velocity in free space with uncertainty.

Twist twist

# Row-major representation of the 6x6 covariance matrix
# The orientation parameters use a fixed-axis representation.
# In order, the parameters are:
# (x, y, z, rotation about X axis, rotation about Y axis, rotation about Z axis)
float64[36] covariance
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/Vector3": `# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "geometry_msgs/msg/Vector3Stamped": `# This represents a Vector3 with reference coordinate frame and timestamp

# Note that this follows vector semantics with it always anchored at the origin,
# so the rotational elements of a transform are the only parts applied when transforming.

std_msgs/Header header
Vector3 vector
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/VelocityStamped": `# This expresses the timestamped velocity vector of a frame 'body_frame_id' in the reference frame 'reference_frame_id' expressed from arbitrary observation frame 'header.frame_id'.
# - If the 'body_frame_id' and 'header.frame_id' are identical, the velocity is observed and defined in the local coordinates system of the body
#   which is the usual use-case in mobile robotics and is also known as a body twist.

std_msgs/Header header
string body_frame_id
string reference_frame_id
Twist velocity
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Twist
# This expresses velocity in free space broken into its linear and angular parts.

Vector3  linear
Vector3  angular
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/VelocityWithCovarianceStamped": `# A timestamped velocity of a body whose frame is 'body_frame_id', measured
# relative to the reference frame 'reference_frame_id', with the velocity and
# covariance both expressed in the basis of the observation frame
# 'header.frame_id'.
#
# - If 'body_frame_id' and 'header.frame_id' are identical, the velocity and
#   covariance are expressed in the body's own basis. This is functionally
#   equivalent to the body-twist convention used by
#   'geometry_msgs/TwistStamped'.
#
# This message is the covariance-bearing analogue of
# 'geometry_msgs/VelocityStamped'.

std_msgs/Header header
string body_frame_id
string reference_frame_id
TwistWithCovariance velocity
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Twist
# This expresses velocity in free space broken into its linear and angular parts.

Vector3  linear
Vector3  angular
================================================================================
MSG: geometry_msgs/TwistWithCovariance
# This expresses velocity in free space with uncertainty.

Twist twist

# Row-major representation of the 6x6 covariance matrix
# The orientation parameters use a fixed-axis representation.
# In order, the parameters are:
# (x, y, z, rotation about X axis, rotation about Y axis, rotation about Z axis)
float64[36] covariance
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "geometry_msgs/msg/Wrench": `# This represents force in free space, separated into its linear and angular parts.

Vector3  force
Vector3  torque
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z`,
  "geometry_msgs/msg/WrenchStamped": `# A wrench with reference coordinate frame and timestamp

std_msgs/Header header
Wrench wrench
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Wrench
# This represents force in free space, separated into its linear and angular parts.

Vector3  force
Vector3  torque
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "lifecycle_msgs/msg/State": `# Primary state definitions as depicted in:
# http://design.ros2.org/articles/node_lifecycle.html

# These are the primary states. State changes can only be requested when the
# node is in one of these states.

# Indicates state has not yet been set.
uint8 PRIMARY_STATE_UNKNOWN = 0

# This is the life cycle state the node is in immediately after being
# instantiated.
uint8 PRIMARY_STATE_UNCONFIGURED = 1

# This state represents a node that is not currently performing any processing.
uint8 PRIMARY_STATE_INACTIVE = 2

# This is the main state of the node's life cycle. While in this state, the node
# performs any processing, responds to service requests, reads and processes
# data, produces output, etc.
uint8 PRIMARY_STATE_ACTIVE = 3

# The finalized state is the state in which the node ends immediately before
# being destroyed.
uint8 PRIMARY_STATE_FINALIZED = 4

# Temporary intermediate states. When a transition is requested, the node
# changes its state into one of these states.

# In this transition state the node's onConfigure callback will be called to
# allow the node to load its configuration and conduct any required setup.
uint8 TRANSITION_STATE_CONFIGURING = 10

# In this transition state the node's callback onCleanup will be called to clear
# all state and return the node to a functionally equivalent state as when
# first created.
uint8 TRANSITION_STATE_CLEANINGUP = 11

# In this transition state the callback onShutdown will be executed to do any
# cleanup necessary before destruction.
uint8 TRANSITION_STATE_SHUTTINGDOWN = 12

# In this transition state the callback onActivate will be executed to do any
# final preparations to start executing.
uint8 TRANSITION_STATE_ACTIVATING = 13

# In this transition state the callback onDeactivate will be executed to do any
# cleanup to start executing, and reverse the onActivate changes.
uint8 TRANSITION_STATE_DEACTIVATING = 14

# This transition state is where any error may be cleaned up.
uint8 TRANSITION_STATE_ERRORPROCESSING = 15

# The state id value from the above definitions.
uint8 id

# A text label of the state.
string label`,
  "lifecycle_msgs/msg/Transition": `# Default values for transitions as described in:
# http://design.ros2.org/articles/node_lifecycle.html

# Reserved [0-9], publicly available transitions.
# When a node is in one of these primary states, these transitions can be
# invoked.

# This transition will instantiate the node, but will not run any code beyond
# the constructor.
uint8 TRANSITION_CREATE = 0

# The node's onConfigure callback will be called to allow the node to load its
# configuration and conduct any required setup.
uint8 TRANSITION_CONFIGURE = 1

# The node's callback onCleanup will be called in this transition to allow the
# node to load its configuration and conduct any required setup.
uint8 TRANSITION_CLEANUP = 2

# The node's callback onActivate will be executed to do any final preparations
# to start executing.
uint8 TRANSITION_ACTIVATE = 3

# The node's callback onDeactivate will be executed to do any cleanup to start
# executing, and reverse the onActivate changes.
uint8 TRANSITION_DEACTIVATE = 4

# This signals shutdown during an unconfigured state, the node's callback
# onShutdown will be executed to do any cleanup necessary before destruction.
uint8 TRANSITION_UNCONFIGURED_SHUTDOWN  = 5

# This signals shutdown during an inactive state, the node's callback onShutdown
# will be executed to do any cleanup necessary before destruction.
uint8 TRANSITION_INACTIVE_SHUTDOWN = 6

# This signals shutdown during an active state, the node's callback onShutdown
# will be executed to do any cleanup necessary before destruction.
uint8 TRANSITION_ACTIVE_SHUTDOWN = 7

# This transition will simply cause the deallocation of the node.
uint8 TRANSITION_DESTROY = 8

# Reserved [10-69], private transitions
# These transitions are not publicly available and cannot be invoked by a user.
# The following transitions are implicitly invoked based on the callback
# feedback of the intermediate transition states.
uint8 TRANSITION_ON_CONFIGURE_SUCCESS = 10
uint8 TRANSITION_ON_CONFIGURE_FAILURE = 11
uint8 TRANSITION_ON_CONFIGURE_ERROR = 12

uint8 TRANSITION_ON_CLEANUP_SUCCESS = 20
uint8 TRANSITION_ON_CLEANUP_FAILURE = 21
uint8 TRANSITION_ON_CLEANUP_ERROR = 22

uint8 TRANSITION_ON_ACTIVATE_SUCCESS = 30
uint8 TRANSITION_ON_ACTIVATE_FAILURE = 31
uint8 TRANSITION_ON_ACTIVATE_ERROR = 32

uint8 TRANSITION_ON_DEACTIVATE_SUCCESS = 40
uint8 TRANSITION_ON_DEACTIVATE_FAILURE = 41
uint8 TRANSITION_ON_DEACTIVATE_ERROR = 42

uint8 TRANSITION_ON_SHUTDOWN_SUCCESS = 50
uint8 TRANSITION_ON_SHUTDOWN_FAILURE = 51
uint8 TRANSITION_ON_SHUTDOWN_ERROR = 52

uint8 TRANSITION_ON_ERROR_SUCCESS = 60
uint8 TRANSITION_ON_ERROR_FAILURE = 61
uint8 TRANSITION_ON_ERROR_ERROR = 62

# Reserved [90-99]. Transition callback success values.
# These return values ought to be set as a return value for each callback.
# Depending on which return value, the transition will be executed correctly or
# fallback/error callbacks will be triggered.

# The transition callback successfully performed its required functionality.
uint8 TRANSITION_CALLBACK_SUCCESS = 97

# The transition callback failed to perform its required functionality.
uint8 TRANSITION_CALLBACK_FAILURE = 98

# The transition callback encountered an error that requires special cleanup, if
# possible.
uint8 TRANSITION_CALLBACK_ERROR = 99

##
## Fields
##

# The transition id from above definitions.
uint8 id

# A text label of the transition.
string label`,
  "lifecycle_msgs/msg/TransitionDescription": `# The transition id and label of this description.
Transition transition

# The current state from which this transition transitions.
State start_state

# The desired target state of this transition.
State goal_state
================================================================================
MSG: lifecycle_msgs/State
# Primary state definitions as depicted in:
# http://design.ros2.org/articles/node_lifecycle.html

# These are the primary states. State changes can only be requested when the
# node is in one of these states.

# Indicates state has not yet been set.
uint8 PRIMARY_STATE_UNKNOWN = 0

# This is the life cycle state the node is in immediately after being
# instantiated.
uint8 PRIMARY_STATE_UNCONFIGURED = 1

# This state represents a node that is not currently performing any processing.
uint8 PRIMARY_STATE_INACTIVE = 2

# This is the main state of the node's life cycle. While in this state, the node
# performs any processing, responds to service requests, reads and processes
# data, produces output, etc.
uint8 PRIMARY_STATE_ACTIVE = 3

# The finalized state is the state in which the node ends immediately before
# being destroyed.
uint8 PRIMARY_STATE_FINALIZED = 4

# Temporary intermediate states. When a transition is requested, the node
# changes its state into one of these states.

# In this transition state the node's onConfigure callback will be called to
# allow the node to load its configuration and conduct any required setup.
uint8 TRANSITION_STATE_CONFIGURING = 10

# In this transition state the node's callback onCleanup will be called to clear
# all state and return the node to a functionally equivalent state as when
# first created.
uint8 TRANSITION_STATE_CLEANINGUP = 11

# In this transition state the callback onShutdown will be executed to do any
# cleanup necessary before destruction.
uint8 TRANSITION_STATE_SHUTTINGDOWN = 12

# In this transition state the callback onActivate will be executed to do any
# final preparations to start executing.
uint8 TRANSITION_STATE_ACTIVATING = 13

# In this transition state the callback onDeactivate will be executed to do any
# cleanup to start executing, and reverse the onActivate changes.
uint8 TRANSITION_STATE_DEACTIVATING = 14

# This transition state is where any error may be cleaned up.
uint8 TRANSITION_STATE_ERRORPROCESSING = 15

# The state id value from the above definitions.
uint8 id

# A text label of the state.
string label
================================================================================
MSG: lifecycle_msgs/Transition
# Default values for transitions as described in:
# http://design.ros2.org/articles/node_lifecycle.html

# Reserved [0-9], publicly available transitions.
# When a node is in one of these primary states, these transitions can be
# invoked.

# This transition will instantiate the node, but will not run any code beyond
# the constructor.
uint8 TRANSITION_CREATE = 0

# The node's onConfigure callback will be called to allow the node to load its
# configuration and conduct any required setup.
uint8 TRANSITION_CONFIGURE = 1

# The node's callback onCleanup will be called in this transition to allow the
# node to load its configuration and conduct any required setup.
uint8 TRANSITION_CLEANUP = 2

# The node's callback onActivate will be executed to do any final preparations
# to start executing.
uint8 TRANSITION_ACTIVATE = 3

# The node's callback onDeactivate will be executed to do any cleanup to start
# executing, and reverse the onActivate changes.
uint8 TRANSITION_DEACTIVATE = 4

# This signals shutdown during an unconfigured state, the node's callback
# onShutdown will be executed to do any cleanup necessary before destruction.
uint8 TRANSITION_UNCONFIGURED_SHUTDOWN  = 5

# This signals shutdown during an inactive state, the node's callback onShutdown
# will be executed to do any cleanup necessary before destruction.
uint8 TRANSITION_INACTIVE_SHUTDOWN = 6

# This signals shutdown during an active state, the node's callback onShutdown
# will be executed to do any cleanup necessary before destruction.
uint8 TRANSITION_ACTIVE_SHUTDOWN = 7

# This transition will simply cause the deallocation of the node.
uint8 TRANSITION_DESTROY = 8

# Reserved [10-69], private transitions
# These transitions are not publicly available and cannot be invoked by a user.
# The following transitions are implicitly invoked based on the callback
# feedback of the intermediate transition states.
uint8 TRANSITION_ON_CONFIGURE_SUCCESS = 10
uint8 TRANSITION_ON_CONFIGURE_FAILURE = 11
uint8 TRANSITION_ON_CONFIGURE_ERROR = 12

uint8 TRANSITION_ON_CLEANUP_SUCCESS = 20
uint8 TRANSITION_ON_CLEANUP_FAILURE = 21
uint8 TRANSITION_ON_CLEANUP_ERROR = 22

uint8 TRANSITION_ON_ACTIVATE_SUCCESS = 30
uint8 TRANSITION_ON_ACTIVATE_FAILURE = 31
uint8 TRANSITION_ON_ACTIVATE_ERROR = 32

uint8 TRANSITION_ON_DEACTIVATE_SUCCESS = 40
uint8 TRANSITION_ON_DEACTIVATE_FAILURE = 41
uint8 TRANSITION_ON_DEACTIVATE_ERROR = 42

uint8 TRANSITION_ON_SHUTDOWN_SUCCESS = 50
uint8 TRANSITION_ON_SHUTDOWN_FAILURE = 51
uint8 TRANSITION_ON_SHUTDOWN_ERROR = 52

uint8 TRANSITION_ON_ERROR_SUCCESS = 60
uint8 TRANSITION_ON_ERROR_FAILURE = 61
uint8 TRANSITION_ON_ERROR_ERROR = 62

# Reserved [90-99]. Transition callback success values.
# These return values ought to be set as a return value for each callback.
# Depending on which return value, the transition will be executed correctly or
# fallback/error callbacks will be triggered.

# The transition callback successfully performed its required functionality.
uint8 TRANSITION_CALLBACK_SUCCESS = 97

# The transition callback failed to perform its required functionality.
uint8 TRANSITION_CALLBACK_FAILURE = 98

# The transition callback encountered an error that requires special cleanup, if
# possible.
uint8 TRANSITION_CALLBACK_ERROR = 99

##
## Fields
##

# The transition id from above definitions.
uint8 id

# A text label of the transition.
string label`,
  "lifecycle_msgs/msg/TransitionEvent": `# The time point at which this event occurred.
uint64 timestamp

# The id and label of this transition event.
Transition transition

# The starting state from which this event transitioned.
State start_state

# The end state of this transition event.
State goal_state
================================================================================
MSG: lifecycle_msgs/State
# Primary state definitions as depicted in:
# http://design.ros2.org/articles/node_lifecycle.html

# These are the primary states. State changes can only be requested when the
# node is in one of these states.

# Indicates state has not yet been set.
uint8 PRIMARY_STATE_UNKNOWN = 0

# This is the life cycle state the node is in immediately after being
# instantiated.
uint8 PRIMARY_STATE_UNCONFIGURED = 1

# This state represents a node that is not currently performing any processing.
uint8 PRIMARY_STATE_INACTIVE = 2

# This is the main state of the node's life cycle. While in this state, the node
# performs any processing, responds to service requests, reads and processes
# data, produces output, etc.
uint8 PRIMARY_STATE_ACTIVE = 3

# The finalized state is the state in which the node ends immediately before
# being destroyed.
uint8 PRIMARY_STATE_FINALIZED = 4

# Temporary intermediate states. When a transition is requested, the node
# changes its state into one of these states.

# In this transition state the node's onConfigure callback will be called to
# allow the node to load its configuration and conduct any required setup.
uint8 TRANSITION_STATE_CONFIGURING = 10

# In this transition state the node's callback onCleanup will be called to clear
# all state and return the node to a functionally equivalent state as when
# first created.
uint8 TRANSITION_STATE_CLEANINGUP = 11

# In this transition state the callback onShutdown will be executed to do any
# cleanup necessary before destruction.
uint8 TRANSITION_STATE_SHUTTINGDOWN = 12

# In this transition state the callback onActivate will be executed to do any
# final preparations to start executing.
uint8 TRANSITION_STATE_ACTIVATING = 13

# In this transition state the callback onDeactivate will be executed to do any
# cleanup to start executing, and reverse the onActivate changes.
uint8 TRANSITION_STATE_DEACTIVATING = 14

# This transition state is where any error may be cleaned up.
uint8 TRANSITION_STATE_ERRORPROCESSING = 15

# The state id value from the above definitions.
uint8 id

# A text label of the state.
string label
================================================================================
MSG: lifecycle_msgs/Transition
# Default values for transitions as described in:
# http://design.ros2.org/articles/node_lifecycle.html

# Reserved [0-9], publicly available transitions.
# When a node is in one of these primary states, these transitions can be
# invoked.

# This transition will instantiate the node, but will not run any code beyond
# the constructor.
uint8 TRANSITION_CREATE = 0

# The node's onConfigure callback will be called to allow the node to load its
# configuration and conduct any required setup.
uint8 TRANSITION_CONFIGURE = 1

# The node's callback onCleanup will be called in this transition to allow the
# node to load its configuration and conduct any required setup.
uint8 TRANSITION_CLEANUP = 2

# The node's callback onActivate will be executed to do any final preparations
# to start executing.
uint8 TRANSITION_ACTIVATE = 3

# The node's callback onDeactivate will be executed to do any cleanup to start
# executing, and reverse the onActivate changes.
uint8 TRANSITION_DEACTIVATE = 4

# This signals shutdown during an unconfigured state, the node's callback
# onShutdown will be executed to do any cleanup necessary before destruction.
uint8 TRANSITION_UNCONFIGURED_SHUTDOWN  = 5

# This signals shutdown during an inactive state, the node's callback onShutdown
# will be executed to do any cleanup necessary before destruction.
uint8 TRANSITION_INACTIVE_SHUTDOWN = 6

# This signals shutdown during an active state, the node's callback onShutdown
# will be executed to do any cleanup necessary before destruction.
uint8 TRANSITION_ACTIVE_SHUTDOWN = 7

# This transition will simply cause the deallocation of the node.
uint8 TRANSITION_DESTROY = 8

# Reserved [10-69], private transitions
# These transitions are not publicly available and cannot be invoked by a user.
# The following transitions are implicitly invoked based on the callback
# feedback of the intermediate transition states.
uint8 TRANSITION_ON_CONFIGURE_SUCCESS = 10
uint8 TRANSITION_ON_CONFIGURE_FAILURE = 11
uint8 TRANSITION_ON_CONFIGURE_ERROR = 12

uint8 TRANSITION_ON_CLEANUP_SUCCESS = 20
uint8 TRANSITION_ON_CLEANUP_FAILURE = 21
uint8 TRANSITION_ON_CLEANUP_ERROR = 22

uint8 TRANSITION_ON_ACTIVATE_SUCCESS = 30
uint8 TRANSITION_ON_ACTIVATE_FAILURE = 31
uint8 TRANSITION_ON_ACTIVATE_ERROR = 32

uint8 TRANSITION_ON_DEACTIVATE_SUCCESS = 40
uint8 TRANSITION_ON_DEACTIVATE_FAILURE = 41
uint8 TRANSITION_ON_DEACTIVATE_ERROR = 42

uint8 TRANSITION_ON_SHUTDOWN_SUCCESS = 50
uint8 TRANSITION_ON_SHUTDOWN_FAILURE = 51
uint8 TRANSITION_ON_SHUTDOWN_ERROR = 52

uint8 TRANSITION_ON_ERROR_SUCCESS = 60
uint8 TRANSITION_ON_ERROR_FAILURE = 61
uint8 TRANSITION_ON_ERROR_ERROR = 62

# Reserved [90-99]. Transition callback success values.
# These return values ought to be set as a return value for each callback.
# Depending on which return value, the transition will be executed correctly or
# fallback/error callbacks will be triggered.

# The transition callback successfully performed its required functionality.
uint8 TRANSITION_CALLBACK_SUCCESS = 97

# The transition callback failed to perform its required functionality.
uint8 TRANSITION_CALLBACK_FAILURE = 98

# The transition callback encountered an error that requires special cleanup, if
# possible.
uint8 TRANSITION_CALLBACK_ERROR = 99

##
## Fields
##

# The transition id from above definitions.
uint8 id

# A text label of the transition.
string label`,
  "nav_msgs/msg/Goals": `# An array of navigation goals


# This header will store the time at which the poses were computed (not to be confused with the stamps of the poses themselves)
# In the case that individual poses do not have their frame_id set or their timetamp set they will use the default value here.
std_msgs/Header header

# An array of goals to for navigation to achieve.
# The goals should be executed in the order of the array.
# The header and stamp are intended to be used for computing the position of the goals.
# They may vary to support cases of goals that are moving with respect to the robot.
geometry_msgs/PoseStamped[] goals
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: geometry_msgs/PoseStamped
# A Pose with reference coordinate frame and timestamp

std_msgs/Header header
Pose pose`,
  "nav_msgs/msg/GridCells": `# An array of cells in a 2D grid

std_msgs/Header header

# Width of each cell
float32 cell_width

# Height of each cell
float32 cell_height

# Each cell is represented by the Point at the center of the cell
geometry_msgs/Point[] cells
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "nav_msgs/msg/MapMetaData": `# This hold basic information about the characteristics of the OccupancyGrid

# The time at which the map was loaded
builtin_interfaces/Time map_load_time

# The map resolution [m/cell]
float32 resolution

# Map width [cells]
uint32 width

# Map height [cells]
uint32 height

# The origin of the map [m, m, rad].  This is the real-world pose of the
# bottom left corner of cell (0,0) in the map.
geometry_msgs/Pose origin
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation`,
  "nav_msgs/msg/OccupancyGrid": `# This represents a 2-D grid map
std_msgs/Header header

# MetaData for the map
MapMetaData info

# The map data, in row-major order, starting with (0,0). 
# Cell (1, 0) will be listed second, representing the next cell in the x direction. 
# Cell (0, 1) will be at the index equal to info.width, followed by (1, 1).
# The values inside are application dependent, but frequently, 
# 0 represents unoccupied, 1 represents definitely occupied, and
# -1 represents unknown. 
int8[] data
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: nav_msgs/MapMetaData
# This hold basic information about the characteristics of the OccupancyGrid

# The time at which the map was loaded
builtin_interfaces/Time map_load_time

# The map resolution [m/cell]
float32 resolution

# Map width [cells]
uint32 width

# Map height [cells]
uint32 height

# The origin of the map [m, m, rad].  This is the real-world pose of the
# bottom left corner of cell (0,0) in the map.
geometry_msgs/Pose origin
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "nav_msgs/msg/Odometry": `# This represents an estimate of a position and velocity in free space.
# The pose in this message should be specified in the coordinate frame given by header.frame_id
# The twist in this message should be specified in the coordinate frame given by the child_frame_id

# Includes the frame id of the pose parent.
std_msgs/Header header

# Frame id the pose points to. The twist is in this coordinate frame.
string child_frame_id

# Estimated pose that is typically relative to a fixed world frame.
geometry_msgs/PoseWithCovariance pose

# Estimated linear and angular velocity relative to child_frame_id.
geometry_msgs/TwistWithCovariance twist
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/PoseWithCovariance
# This represents a pose in free space with uncertainty.

Pose pose

# Row-major representation of the 6x6 covariance matrix
# The orientation parameters use a fixed-axis representation.
# In order, the parameters are:
# (x, y, z, rotation about X axis, rotation about Y axis, rotation about Z axis)
float64[36] covariance
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Twist
# This expresses velocity in free space broken into its linear and angular parts.

Vector3  linear
Vector3  angular
================================================================================
MSG: geometry_msgs/TwistWithCovariance
# This expresses velocity in free space with uncertainty.

Twist twist

# Row-major representation of the 6x6 covariance matrix
# The orientation parameters use a fixed-axis representation.
# In order, the parameters are:
# (x, y, z, rotation about X axis, rotation about Y axis, rotation about Z axis)
float64[36] covariance
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "nav_msgs/msg/Path": `# An array of poses that represents a Path for a robot to follow.

# Indicates the frame_id of the path.
std_msgs/Header header

# Array of poses to follow.
geometry_msgs/PoseStamped[] poses
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: geometry_msgs/PoseStamped
# A Pose with reference coordinate frame and timestamp

std_msgs/Header header
Pose pose`,
  "nav_msgs/msg/Trajectory": `# An array of trajectory points that represents a trajectory for a robot to follow.

# Indicates the frame_id in which the planner ran and timestamp represents when the trajectory was generated.
std_msgs/Header header

# Array of trajectory points to follow.
TrajectoryPoint[] points
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Accel
# This expresses acceleration in free space broken into its linear and angular parts.
Vector3  linear
Vector3  angular
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/Twist
# This expresses velocity in free space broken into its linear and angular parts.

Vector3  linear
Vector3  angular
================================================================================
MSG: geometry_msgs/Wrench
# This represents force in free space, separated into its linear and angular parts.

Vector3  force
Vector3  torque
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: nav_msgs/TrajectoryPoint
# Trajectory point state

# Absolute time and frame of reference of the point along a trajectory
std_msgs/Header header

# Pose of the trajectory sample.
geometry_msgs/Pose pose

# Velocity of the trajectory sample.
geometry_msgs/Twist velocity

# Acceleration of the trajectory (optional, linear.x component is NaN if not set).
geometry_msgs/Accel acceleration

# Force/Torque to apply at trajectory sample (optional, force.x component is NaN if not set).
geometry_msgs/Wrench effort`,
  "nav_msgs/msg/TrajectoryPoint": `# Trajectory point state

# Absolute time and frame of reference of the point along a trajectory
std_msgs/Header header

# Pose of the trajectory sample.
geometry_msgs/Pose pose

# Velocity of the trajectory sample.
geometry_msgs/Twist velocity

# Acceleration of the trajectory (optional, linear.x component is NaN if not set).
geometry_msgs/Accel acceleration

# Force/Torque to apply at trajectory sample (optional, force.x component is NaN if not set).
geometry_msgs/Wrench effort
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Accel
# This expresses acceleration in free space broken into its linear and angular parts.
Vector3  linear
Vector3  angular
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/Twist
# This expresses velocity in free space broken into its linear and angular parts.

Vector3  linear
Vector3  angular
================================================================================
MSG: geometry_msgs/Wrench
# This represents force in free space, separated into its linear and angular parts.

Vector3  force
Vector3  torque
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "rcl_interfaces/msg/FloatingPointRange": `# Represents bounds and a step value for a floating point typed parameter.

# Start value for valid values, inclusive.
float64 from_value

# End value for valid values, inclusive.
float64 to_value

# Size of valid steps between the from and to bound.
# 
# Step is considered to be a magnitude, therefore negative values are treated
# the same as positive values, and a step value of zero implies a continuous
# range of values.
#
# Ideally, the step would be less than or equal to the distance between the
# bounds, as well as an even multiple of the distance between the bounds, but
# neither are required.
#
# If the absolute value of the step is larger than or equal to the distance
# between the two bounds, then the bounds will be the only valid values. e.g. if
# the range is defined as {from_value: 1.0, to_value: 2.0, step: 5.0} then the
# valid values will be 1.0 and 2.0.
#
# If the step is less than the distance between the bounds, but the distance is
# not a multiple of the step, then the "to" bound will always be a valid value,
# e.g. if the range is defined as {from_value: 2.0, to_value: 5.0, step: 2.0}
# then the valid values will be 2.0, 4.0, and 5.0.
float64 step`,
  "rcl_interfaces/msg/IntegerRange": `# Represents bounds and a step value for an integer typed parameter.

# Start value for valid values, inclusive.
int64 from_value

# End value for valid values, inclusive.
int64 to_value

# Size of valid steps between the from and to bound.
#
# A step value of zero implies a continuous range of values. Ideally, the step
# would be less than or equal to the distance between the bounds, as well as an
# even multiple of the distance between the bounds, but neither are required.
#
# If the absolute value of the step is larger than or equal to the distance
# between the two bounds, then the bounds will be the only valid values. e.g. if
# the range is defined as {from_value: 1, to_value: 2, step: 5} then the valid
# values will be 1 and 2.
# 
# If the step is less than the distance between the bounds, but the distance is
# not a multiple of the step, then the "to" bound will always be a valid value,
# e.g. if the range is defined as {from_value: 2, to_value: 5, step: 2} then
# the valid values will be 2, 4, and 5.
uint64 step`,
  "rcl_interfaces/msg/ListParametersResult": `# The resulting parameters under the given prefixes.
string[] names

# The resulting prefixes under the given prefixes.
# TODO(wjwwood): link to prefix definition and rules.
string[] prefixes`,
  "rcl_interfaces/msg/Log": `##
## Severity level constants
## 
## These logging levels follow the Python Standard
## https://docs.python.org/3/library/logging.html#logging-levels
## And are implemented in rcutils as well
## https://github.com/ros2/rcutils/blob/35f29850064e0c33a4063cbc947ebbfeada11dba/include/rcutils/logging.h#L164-L172
## This leaves space for other standard logging levels to be inserted in the middle in the future,
## as well as custom user defined levels.
## Since there are several other logging enumeration standard for different implementations,
## other logging implementations may need to provide level mappings to match their internal implementations.
##

# Debug is for pedantic information, which is useful when debugging issues.
uint8 DEBUG=10

# Info is the standard informational level and is used to report expected
# information.
uint8 INFO=20

# Warning is for information that may potentially cause issues or possibly unexpected
# behavior.
uint8 WARN=30

# Error is for information that this node cannot resolve.
uint8 ERROR=40

# Information about a impending node shutdown.
uint8 FATAL=50

##
## Fields
##

# Timestamp when this message was generated by the node.
builtin_interfaces/Time stamp

# Corresponding log level, see above definitions.
uint8 level

# The name representing the logger this message came from.
string name

# The full log message.
string msg

# The file the message came from.
string file

# The function the message came from.
string function

# The line in the file the message came from.
uint32 line
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "rcl_interfaces/msg/LoggerLevel": `# All available logger levels; these correspond to the enum in rcutils/logger.h

uint8 LOG_LEVEL_UNKNOWN = 0
uint8 LOG_LEVEL_DEBUG = 10
uint8 LOG_LEVEL_INFO = 20
uint8 LOG_LEVEL_WARN = 30
uint8 LOG_LEVEL_ERROR = 40
uint8 LOG_LEVEL_FATAL = 50

# The logger name.
string name

# The logger level
uint32 level`,
  "rcl_interfaces/msg/Parameter": `# This is the message to communicate a parameter. It is an open struct with an enum in
# the descriptor to select which value is active.

# The full name of the parameter.
string name

# The parameter's value which can be one of several types, see
# \`ParameterValue.msg\` and \`ParameterType.msg\`.
ParameterValue value
================================================================================
MSG: rcl_interfaces/ParameterValue
# Used to determine which of the next *_value fields are set.
# ParameterType.PARAMETER_NOT_SET indicates that the parameter was not set
# (if gotten) or is uninitialized.
# Values are enumerated in \`ParameterType.msg\`.

# The type of this parameter, which corresponds to the appropriate field below.
uint8 type

# "Variant" style storage of the parameter value. Only the value corresponding
# the type field will have valid information.

# Boolean value, can be either true or false.
bool bool_value

# Integer value ranging from -9,223,372,036,854,775,808 to
# 9,223,372,036,854,775,807.
int64 integer_value

# A double precision floating point value following IEEE 754.
float64 double_value

# A textual value with no practical length limit.
string string_value

# An array of bytes, used for non-textual information.
byte[] byte_array_value

# An array of boolean values.
bool[] bool_array_value

# An array of 64-bit integer values.
int64[] integer_array_value

# An array of 64-bit floating point values.
float64[] double_array_value

# An array of string values.
string[] string_array_value`,
  "rcl_interfaces/msg/ParameterDescriptor": `# This is the message to communicate a parameter's descriptor.

# The name of the parameter.
string name

# Enum values are defined in the \`ParameterType.msg\` message.
uint8 type

# Description of the parameter, visible from introspection tools.
string description

# Parameter constraints

# Plain English description of additional constraints which cannot be expressed
# with the available constraints, e.g. "only prime numbers".
#
# By convention, this should only be used to clarify constraints which cannot
# be completely expressed with the parameter constraints below.
string additional_constraints

# If 'true' then the value cannot change after it has been initialized.
bool read_only false

# If true, the parameter is allowed to change type.
bool dynamic_typing false

# If any of the following sequences are not empty, then the constraint inside of
# them apply to this parameter.
#
# FloatingPointRange and IntegerRange are mutually exclusive.

# FloatingPointRange consists of a from_value, a to_value, and a step.
FloatingPointRange[<=1] floating_point_range

# IntegerRange consists of a from_value, a to_value, and a step.
IntegerRange[<=1] integer_range
================================================================================
MSG: rcl_interfaces/FloatingPointRange
# Represents bounds and a step value for a floating point typed parameter.

# Start value for valid values, inclusive.
float64 from_value

# End value for valid values, inclusive.
float64 to_value

# Size of valid steps between the from and to bound.
# 
# Step is considered to be a magnitude, therefore negative values are treated
# the same as positive values, and a step value of zero implies a continuous
# range of values.
#
# Ideally, the step would be less than or equal to the distance between the
# bounds, as well as an even multiple of the distance between the bounds, but
# neither are required.
#
# If the absolute value of the step is larger than or equal to the distance
# between the two bounds, then the bounds will be the only valid values. e.g. if
# the range is defined as {from_value: 1.0, to_value: 2.0, step: 5.0} then the
# valid values will be 1.0 and 2.0.
#
# If the step is less than the distance between the bounds, but the distance is
# not a multiple of the step, then the "to" bound will always be a valid value,
# e.g. if the range is defined as {from_value: 2.0, to_value: 5.0, step: 2.0}
# then the valid values will be 2.0, 4.0, and 5.0.
float64 step
================================================================================
MSG: rcl_interfaces/IntegerRange
# Represents bounds and a step value for an integer typed parameter.

# Start value for valid values, inclusive.
int64 from_value

# End value for valid values, inclusive.
int64 to_value

# Size of valid steps between the from and to bound.
#
# A step value of zero implies a continuous range of values. Ideally, the step
# would be less than or equal to the distance between the bounds, as well as an
# even multiple of the distance between the bounds, but neither are required.
#
# If the absolute value of the step is larger than or equal to the distance
# between the two bounds, then the bounds will be the only valid values. e.g. if
# the range is defined as {from_value: 1, to_value: 2, step: 5} then the valid
# values will be 1 and 2.
# 
# If the step is less than the distance between the bounds, but the distance is
# not a multiple of the step, then the "to" bound will always be a valid value,
# e.g. if the range is defined as {from_value: 2, to_value: 5, step: 2} then
# the valid values will be 2, 4, and 5.
uint64 step`,
  "rcl_interfaces/msg/ParameterEvent": `# This message contains a parameter event.
# Because the parameter event was an atomic update, a specific parameter name
# can only be in one of the three sets.

# The time stamp when this parameter event occurred.
builtin_interfaces/Time stamp

# Fully qualified ROS path to node.
string node

# New parameters that have been set for this node.
Parameter[] new_parameters

# Parameters that have been changed during this event.
Parameter[] changed_parameters

# Parameters that have been deleted during this event.
Parameter[] deleted_parameters
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: rcl_interfaces/ParameterValue
# Used to determine which of the next *_value fields are set.
# ParameterType.PARAMETER_NOT_SET indicates that the parameter was not set
# (if gotten) or is uninitialized.
# Values are enumerated in \`ParameterType.msg\`.

# The type of this parameter, which corresponds to the appropriate field below.
uint8 type

# "Variant" style storage of the parameter value. Only the value corresponding
# the type field will have valid information.

# Boolean value, can be either true or false.
bool bool_value

# Integer value ranging from -9,223,372,036,854,775,808 to
# 9,223,372,036,854,775,807.
int64 integer_value

# A double precision floating point value following IEEE 754.
float64 double_value

# A textual value with no practical length limit.
string string_value

# An array of bytes, used for non-textual information.
byte[] byte_array_value

# An array of boolean values.
bool[] bool_array_value

# An array of 64-bit integer values.
int64[] integer_array_value

# An array of 64-bit floating point values.
float64[] double_array_value

# An array of string values.
string[] string_array_value
================================================================================
MSG: rcl_interfaces/Parameter
# This is the message to communicate a parameter. It is an open struct with an enum in
# the descriptor to select which value is active.

# The full name of the parameter.
string name

# The parameter's value which can be one of several types, see
# \`ParameterValue.msg\` and \`ParameterType.msg\`.
ParameterValue value`,
  "rcl_interfaces/msg/ParameterEventDescriptors": `# This message contains descriptors of a parameter event.
# It was an atomic update.
# A specific parameter name can only be in one of the three sets.

ParameterDescriptor[] new_parameters
ParameterDescriptor[] changed_parameters
ParameterDescriptor[] deleted_parameters
================================================================================
MSG: rcl_interfaces/FloatingPointRange
# Represents bounds and a step value for a floating point typed parameter.

# Start value for valid values, inclusive.
float64 from_value

# End value for valid values, inclusive.
float64 to_value

# Size of valid steps between the from and to bound.
# 
# Step is considered to be a magnitude, therefore negative values are treated
# the same as positive values, and a step value of zero implies a continuous
# range of values.
#
# Ideally, the step would be less than or equal to the distance between the
# bounds, as well as an even multiple of the distance between the bounds, but
# neither are required.
#
# If the absolute value of the step is larger than or equal to the distance
# between the two bounds, then the bounds will be the only valid values. e.g. if
# the range is defined as {from_value: 1.0, to_value: 2.0, step: 5.0} then the
# valid values will be 1.0 and 2.0.
#
# If the step is less than the distance between the bounds, but the distance is
# not a multiple of the step, then the "to" bound will always be a valid value,
# e.g. if the range is defined as {from_value: 2.0, to_value: 5.0, step: 2.0}
# then the valid values will be 2.0, 4.0, and 5.0.
float64 step
================================================================================
MSG: rcl_interfaces/IntegerRange
# Represents bounds and a step value for an integer typed parameter.

# Start value for valid values, inclusive.
int64 from_value

# End value for valid values, inclusive.
int64 to_value

# Size of valid steps between the from and to bound.
#
# A step value of zero implies a continuous range of values. Ideally, the step
# would be less than or equal to the distance between the bounds, as well as an
# even multiple of the distance between the bounds, but neither are required.
#
# If the absolute value of the step is larger than or equal to the distance
# between the two bounds, then the bounds will be the only valid values. e.g. if
# the range is defined as {from_value: 1, to_value: 2, step: 5} then the valid
# values will be 1 and 2.
# 
# If the step is less than the distance between the bounds, but the distance is
# not a multiple of the step, then the "to" bound will always be a valid value,
# e.g. if the range is defined as {from_value: 2, to_value: 5, step: 2} then
# the valid values will be 2, 4, and 5.
uint64 step
================================================================================
MSG: rcl_interfaces/ParameterDescriptor
# This is the message to communicate a parameter's descriptor.

# The name of the parameter.
string name

# Enum values are defined in the \`ParameterType.msg\` message.
uint8 type

# Description of the parameter, visible from introspection tools.
string description

# Parameter constraints

# Plain English description of additional constraints which cannot be expressed
# with the available constraints, e.g. "only prime numbers".
#
# By convention, this should only be used to clarify constraints which cannot
# be completely expressed with the parameter constraints below.
string additional_constraints

# If 'true' then the value cannot change after it has been initialized.
bool read_only false

# If true, the parameter is allowed to change type.
bool dynamic_typing false

# If any of the following sequences are not empty, then the constraint inside of
# them apply to this parameter.
#
# FloatingPointRange and IntegerRange are mutually exclusive.

# FloatingPointRange consists of a from_value, a to_value, and a step.
FloatingPointRange[<=1] floating_point_range

# IntegerRange consists of a from_value, a to_value, and a step.
IntegerRange[<=1] integer_range`,
  "rcl_interfaces/msg/ParameterType": `# These types correspond to the value that is set in the ParameterValue message.

# Default value, which implies this is not a valid parameter.
uint8 PARAMETER_NOT_SET=0

uint8 PARAMETER_BOOL=1
uint8 PARAMETER_INTEGER=2
uint8 PARAMETER_DOUBLE=3
uint8 PARAMETER_STRING=4
uint8 PARAMETER_BYTE_ARRAY=5
uint8 PARAMETER_BOOL_ARRAY=6
uint8 PARAMETER_INTEGER_ARRAY=7
uint8 PARAMETER_DOUBLE_ARRAY=8
uint8 PARAMETER_STRING_ARRAY=9`,
  "rcl_interfaces/msg/ParameterValue": `# Used to determine which of the next *_value fields are set.
# ParameterType.PARAMETER_NOT_SET indicates that the parameter was not set
# (if gotten) or is uninitialized.
# Values are enumerated in \`ParameterType.msg\`.

# The type of this parameter, which corresponds to the appropriate field below.
uint8 type

# "Variant" style storage of the parameter value. Only the value corresponding
# the type field will have valid information.

# Boolean value, can be either true or false.
bool bool_value

# Integer value ranging from -9,223,372,036,854,775,808 to
# 9,223,372,036,854,775,807.
int64 integer_value

# A double precision floating point value following IEEE 754.
float64 double_value

# A textual value with no practical length limit.
string string_value

# An array of bytes, used for non-textual information.
byte[] byte_array_value

# An array of boolean values.
bool[] bool_array_value

# An array of 64-bit integer values.
int64[] integer_array_value

# An array of 64-bit floating point values.
float64[] double_array_value

# An array of string values.
string[] string_array_value`,
  "rcl_interfaces/msg/SetLoggerLevelsResult": `# True when succeed, false when failed.
bool successful

# Reason why the setting was either successful or a failure.
string reason`,
  "rcl_interfaces/msg/SetParametersResult": `# A true value of the same index indicates that the parameter was set
# successfully. A false value indicates the change was rejected.
bool successful

# Reason why the setting was a failure. On success, the contents of this field
# are undefined.  This should only be used for logging and user interfaces.
string reason`,
  "rosgraph_msgs/msg/Action": `# Describes a single Action endpoint, which may be a Server or Client

# Fully qualified name of the Action
string name

# An action is actually a composition of the following fundamental ROS entities
Service send_goal
Service get_result
Service cancel_goal
Topic feedback
Topic status
================================================================================
MSG: rosgraph_msgs/TypeHash
# RIHS spec version
uint8 version 1
# ROSIDL_TYPE_HASH_SIZE == 32
uint8[32] value
================================================================================
MSG: rosgraph_msgs/InterfaceType
# Represent a type of a ROS Graph Interface

# The plaintext namespaced name of the type - e.g. sensor_msgs/Image
string name

# The hash uniquely identifies the exact structure of the type,
# the definition of which may change between package version
TypeHash hash
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: rosgraph_msgs/QoSProfile
# Message-based representation of ROS 2 Quality of Service settings
# Default values are kept in sync with RMW by integration test
# Note that SYSTEM_DEFAULT and BEST_AVAILABLE values cannot be an observed value,
# because they resolve concretely at runtime.
# They are included here for completeness to match the data structures in RMW

# Depth of the message queue (only meaningful when history==KEEP_LAST)
uint32 depth

# Deadline between messages (0 for no deadline)
builtin_interfaces/Duration deadline

# Lifespan of each message (0 for infinite)
builtin_interfaces/Duration lifespan

# History policy
uint8 HISTORY_SYSTEM_DEFAULT=0
uint8 HISTORY_KEEP_LAST=1
uint8 HISTORY_KEEP_ALL=2
uint8 HISTORY_UNKNOWN=3
uint8 history

# Reliability policy
uint8 RELIABILITY_SYSTEM_DEFAULT=0
uint8 RELIABILITY_RELIABLE=1
uint8 RELIABILITY_BEST_EFFORT=2
uint8 RELIABILITY_UNKNOWN=3
uint8 RELIABILITY_BEST_AVAILABLE=4
uint8 reliability

# Durability policy
uint8 DURABILITY_SYSTEM_DEFAULT=0
uint8 DURABILITY_TRANSIENT_LOCAL=1
uint8 DURABILITY_VOLATILE=2
uint8 DURABILITY_UNKNOWN=3
uint8 DURABILITY_BEST_AVAILABLE=4
uint8 durability

# Liveliness policy
uint8 LIVELINESS_SYSTEM_DEFAULT=0
uint8 LIVELINESS_AUTOMATIC=1
uint8 LIVELINESS_MANUAL_BY_TOPIC=3
uint8 LIVELINESS_UNKNOWN=4
uint8 LIVELINESS_BEST_AVAILABLE=5
uint8 liveliness

# Lease duration for liveliness (0 for infinite)
builtin_interfaces/Duration liveliness_lease_duration
================================================================================
MSG: rosgraph_msgs/Service
# Describes a single Service endpoint, which may be a Server or Client

# Fully qualified name of the Service
string name

# Type and actual QoS of the request publisher (Client) or subscription (Server)
InterfaceType request_type
QoSProfile request_qos

# Type and actual QoS of the request subscription (Client) or publisher (Server)
InterfaceType response_type
QoSProfile response_qos
================================================================================
MSG: rosgraph_msgs/Topic
# Describes a single topic endpoint, which may be a Publisher or Subscription

# Fully qualified name of the topic
string name

# Type of the topic
InterfaceType type

# Observed QoS of the endpoint
QoSProfile qos`,
  "rosgraph_msgs/msg/Clock": `# This message communicates the current time.
#
# For more information, see https://design.ros2.org/articles/clock_and_time.html.
builtin_interfaces/Time clock
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "rosgraph_msgs/msg/Graph": `# Represents a ROS node graph, which is only a collection of nodes
Node[] nodes
================================================================================
MSG: rcl_interfaces/FloatingPointRange
# Represents bounds and a step value for a floating point typed parameter.

# Start value for valid values, inclusive.
float64 from_value

# End value for valid values, inclusive.
float64 to_value

# Size of valid steps between the from and to bound.
# 
# Step is considered to be a magnitude, therefore negative values are treated
# the same as positive values, and a step value of zero implies a continuous
# range of values.
#
# Ideally, the step would be less than or equal to the distance between the
# bounds, as well as an even multiple of the distance between the bounds, but
# neither are required.
#
# If the absolute value of the step is larger than or equal to the distance
# between the two bounds, then the bounds will be the only valid values. e.g. if
# the range is defined as {from_value: 1.0, to_value: 2.0, step: 5.0} then the
# valid values will be 1.0 and 2.0.
#
# If the step is less than the distance between the bounds, but the distance is
# not a multiple of the step, then the "to" bound will always be a valid value,
# e.g. if the range is defined as {from_value: 2.0, to_value: 5.0, step: 2.0}
# then the valid values will be 2.0, 4.0, and 5.0.
float64 step
================================================================================
MSG: rcl_interfaces/IntegerRange
# Represents bounds and a step value for an integer typed parameter.

# Start value for valid values, inclusive.
int64 from_value

# End value for valid values, inclusive.
int64 to_value

# Size of valid steps between the from and to bound.
#
# A step value of zero implies a continuous range of values. Ideally, the step
# would be less than or equal to the distance between the bounds, as well as an
# even multiple of the distance between the bounds, but neither are required.
#
# If the absolute value of the step is larger than or equal to the distance
# between the two bounds, then the bounds will be the only valid values. e.g. if
# the range is defined as {from_value: 1, to_value: 2, step: 5} then the valid
# values will be 1 and 2.
# 
# If the step is less than the distance between the bounds, but the distance is
# not a multiple of the step, then the "to" bound will always be a valid value,
# e.g. if the range is defined as {from_value: 2, to_value: 5, step: 2} then
# the valid values will be 2, 4, and 5.
uint64 step
================================================================================
MSG: rcl_interfaces/ParameterDescriptor
# This is the message to communicate a parameter's descriptor.

# The name of the parameter.
string name

# Enum values are defined in the \`ParameterType.msg\` message.
uint8 type

# Description of the parameter, visible from introspection tools.
string description

# Parameter constraints

# Plain English description of additional constraints which cannot be expressed
# with the available constraints, e.g. "only prime numbers".
#
# By convention, this should only be used to clarify constraints which cannot
# be completely expressed with the parameter constraints below.
string additional_constraints

# If 'true' then the value cannot change after it has been initialized.
bool read_only false

# If true, the parameter is allowed to change type.
bool dynamic_typing false

# If any of the following sequences are not empty, then the constraint inside of
# them apply to this parameter.
#
# FloatingPointRange and IntegerRange are mutually exclusive.

# FloatingPointRange consists of a from_value, a to_value, and a step.
FloatingPointRange[<=1] floating_point_range

# IntegerRange consists of a from_value, a to_value, and a step.
IntegerRange[<=1] integer_range
================================================================================
MSG: rcl_interfaces/ParameterValue
# Used to determine which of the next *_value fields are set.
# ParameterType.PARAMETER_NOT_SET indicates that the parameter was not set
# (if gotten) or is uninitialized.
# Values are enumerated in \`ParameterType.msg\`.

# The type of this parameter, which corresponds to the appropriate field below.
uint8 type

# "Variant" style storage of the parameter value. Only the value corresponding
# the type field will have valid information.

# Boolean value, can be either true or false.
bool bool_value

# Integer value ranging from -9,223,372,036,854,775,808 to
# 9,223,372,036,854,775,807.
int64 integer_value

# A double precision floating point value following IEEE 754.
float64 double_value

# A textual value with no practical length limit.
string string_value

# An array of bytes, used for non-textual information.
byte[] byte_array_value

# An array of boolean values.
bool[] bool_array_value

# An array of 64-bit integer values.
int64[] integer_array_value

# An array of 64-bit floating point values.
float64[] double_array_value

# An array of string values.
string[] string_array_value
================================================================================
MSG: rosgraph_msgs/TypeHash
# RIHS spec version
uint8 version 1
# ROSIDL_TYPE_HASH_SIZE == 32
uint8[32] value
================================================================================
MSG: rosgraph_msgs/InterfaceType
# Represent a type of a ROS Graph Interface

# The plaintext namespaced name of the type - e.g. sensor_msgs/Image
string name

# The hash uniquely identifies the exact structure of the type,
# the definition of which may change between package version
TypeHash hash
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: rosgraph_msgs/QoSProfile
# Message-based representation of ROS 2 Quality of Service settings
# Default values are kept in sync with RMW by integration test
# Note that SYSTEM_DEFAULT and BEST_AVAILABLE values cannot be an observed value,
# because they resolve concretely at runtime.
# They are included here for completeness to match the data structures in RMW

# Depth of the message queue (only meaningful when history==KEEP_LAST)
uint32 depth

# Deadline between messages (0 for no deadline)
builtin_interfaces/Duration deadline

# Lifespan of each message (0 for infinite)
builtin_interfaces/Duration lifespan

# History policy
uint8 HISTORY_SYSTEM_DEFAULT=0
uint8 HISTORY_KEEP_LAST=1
uint8 HISTORY_KEEP_ALL=2
uint8 HISTORY_UNKNOWN=3
uint8 history

# Reliability policy
uint8 RELIABILITY_SYSTEM_DEFAULT=0
uint8 RELIABILITY_RELIABLE=1
uint8 RELIABILITY_BEST_EFFORT=2
uint8 RELIABILITY_UNKNOWN=3
uint8 RELIABILITY_BEST_AVAILABLE=4
uint8 reliability

# Durability policy
uint8 DURABILITY_SYSTEM_DEFAULT=0
uint8 DURABILITY_TRANSIENT_LOCAL=1
uint8 DURABILITY_VOLATILE=2
uint8 DURABILITY_UNKNOWN=3
uint8 DURABILITY_BEST_AVAILABLE=4
uint8 durability

# Liveliness policy
uint8 LIVELINESS_SYSTEM_DEFAULT=0
uint8 LIVELINESS_AUTOMATIC=1
uint8 LIVELINESS_MANUAL_BY_TOPIC=3
uint8 LIVELINESS_UNKNOWN=4
uint8 LIVELINESS_BEST_AVAILABLE=5
uint8 liveliness

# Lease duration for liveliness (0 for infinite)
builtin_interfaces/Duration liveliness_lease_duration
================================================================================
MSG: rosgraph_msgs/Service
# Describes a single Service endpoint, which may be a Server or Client

# Fully qualified name of the Service
string name

# Type and actual QoS of the request publisher (Client) or subscription (Server)
InterfaceType request_type
QoSProfile request_qos

# Type and actual QoS of the request subscription (Client) or publisher (Server)
InterfaceType response_type
QoSProfile response_qos
================================================================================
MSG: rosgraph_msgs/Topic
# Describes a single topic endpoint, which may be a Publisher or Subscription

# Fully qualified name of the topic
string name

# Type of the topic
InterfaceType type

# Observed QoS of the endpoint
QoSProfile qos
================================================================================
MSG: rosgraph_msgs/Action
# Describes a single Action endpoint, which may be a Server or Client

# Fully qualified name of the Action
string name

# An action is actually a composition of the following fundamental ROS entities
Service send_goal
Service get_result
Service cancel_goal
Topic feedback
Topic status
================================================================================
MSG: rosgraph_msgs/Node
# Represents the observable runtime state of a ROS Node
# Therefore, does not perfectly align with the abstract specification which created it.

# Fully qualified node name (FQN)
string name

# Parameter specifications for the node
rcl_interfaces/ParameterDescriptor[] parameters

# Current values of the node's parameters
# NOTE:
#   parameter_values[] must be empty, or the same size as parameters[]
#   When set, parameter_values[] match 1:1 with the same index in parameters[]
rcl_interfaces/ParameterValue[] parameter_values

# Communications endpoints - Topics, Services, and Actions
Topic[] publishers
Topic[] subscriptions

Service[] service_clients
Service[] service_servers

Action[] action_clients
Action[] action_servers`,
  "rosgraph_msgs/msg/InterfaceType": `# Represent a type of a ROS Graph Interface

# The plaintext namespaced name of the type - e.g. sensor_msgs/Image
string name

# The hash uniquely identifies the exact structure of the type,
# the definition of which may change between package version
TypeHash hash
================================================================================
MSG: rosgraph_msgs/TypeHash
# RIHS spec version
uint8 version 1
# ROSIDL_TYPE_HASH_SIZE == 32
uint8[32] value`,
  "rosgraph_msgs/msg/Node": `# Represents the observable runtime state of a ROS Node
# Therefore, does not perfectly align with the abstract specification which created it.

# Fully qualified node name (FQN)
string name

# Parameter specifications for the node
rcl_interfaces/ParameterDescriptor[] parameters

# Current values of the node's parameters
# NOTE:
#   parameter_values[] must be empty, or the same size as parameters[]
#   When set, parameter_values[] match 1:1 with the same index in parameters[]
rcl_interfaces/ParameterValue[] parameter_values

# Communications endpoints - Topics, Services, and Actions
Topic[] publishers
Topic[] subscriptions

Service[] service_clients
Service[] service_servers

Action[] action_clients
Action[] action_servers
================================================================================
MSG: rcl_interfaces/FloatingPointRange
# Represents bounds and a step value for a floating point typed parameter.

# Start value for valid values, inclusive.
float64 from_value

# End value for valid values, inclusive.
float64 to_value

# Size of valid steps between the from and to bound.
# 
# Step is considered to be a magnitude, therefore negative values are treated
# the same as positive values, and a step value of zero implies a continuous
# range of values.
#
# Ideally, the step would be less than or equal to the distance between the
# bounds, as well as an even multiple of the distance between the bounds, but
# neither are required.
#
# If the absolute value of the step is larger than or equal to the distance
# between the two bounds, then the bounds will be the only valid values. e.g. if
# the range is defined as {from_value: 1.0, to_value: 2.0, step: 5.0} then the
# valid values will be 1.0 and 2.0.
#
# If the step is less than the distance between the bounds, but the distance is
# not a multiple of the step, then the "to" bound will always be a valid value,
# e.g. if the range is defined as {from_value: 2.0, to_value: 5.0, step: 2.0}
# then the valid values will be 2.0, 4.0, and 5.0.
float64 step
================================================================================
MSG: rcl_interfaces/IntegerRange
# Represents bounds and a step value for an integer typed parameter.

# Start value for valid values, inclusive.
int64 from_value

# End value for valid values, inclusive.
int64 to_value

# Size of valid steps between the from and to bound.
#
# A step value of zero implies a continuous range of values. Ideally, the step
# would be less than or equal to the distance between the bounds, as well as an
# even multiple of the distance between the bounds, but neither are required.
#
# If the absolute value of the step is larger than or equal to the distance
# between the two bounds, then the bounds will be the only valid values. e.g. if
# the range is defined as {from_value: 1, to_value: 2, step: 5} then the valid
# values will be 1 and 2.
# 
# If the step is less than the distance between the bounds, but the distance is
# not a multiple of the step, then the "to" bound will always be a valid value,
# e.g. if the range is defined as {from_value: 2, to_value: 5, step: 2} then
# the valid values will be 2, 4, and 5.
uint64 step
================================================================================
MSG: rcl_interfaces/ParameterDescriptor
# This is the message to communicate a parameter's descriptor.

# The name of the parameter.
string name

# Enum values are defined in the \`ParameterType.msg\` message.
uint8 type

# Description of the parameter, visible from introspection tools.
string description

# Parameter constraints

# Plain English description of additional constraints which cannot be expressed
# with the available constraints, e.g. "only prime numbers".
#
# By convention, this should only be used to clarify constraints which cannot
# be completely expressed with the parameter constraints below.
string additional_constraints

# If 'true' then the value cannot change after it has been initialized.
bool read_only false

# If true, the parameter is allowed to change type.
bool dynamic_typing false

# If any of the following sequences are not empty, then the constraint inside of
# them apply to this parameter.
#
# FloatingPointRange and IntegerRange are mutually exclusive.

# FloatingPointRange consists of a from_value, a to_value, and a step.
FloatingPointRange[<=1] floating_point_range

# IntegerRange consists of a from_value, a to_value, and a step.
IntegerRange[<=1] integer_range
================================================================================
MSG: rcl_interfaces/ParameterValue
# Used to determine which of the next *_value fields are set.
# ParameterType.PARAMETER_NOT_SET indicates that the parameter was not set
# (if gotten) or is uninitialized.
# Values are enumerated in \`ParameterType.msg\`.

# The type of this parameter, which corresponds to the appropriate field below.
uint8 type

# "Variant" style storage of the parameter value. Only the value corresponding
# the type field will have valid information.

# Boolean value, can be either true or false.
bool bool_value

# Integer value ranging from -9,223,372,036,854,775,808 to
# 9,223,372,036,854,775,807.
int64 integer_value

# A double precision floating point value following IEEE 754.
float64 double_value

# A textual value with no practical length limit.
string string_value

# An array of bytes, used for non-textual information.
byte[] byte_array_value

# An array of boolean values.
bool[] bool_array_value

# An array of 64-bit integer values.
int64[] integer_array_value

# An array of 64-bit floating point values.
float64[] double_array_value

# An array of string values.
string[] string_array_value
================================================================================
MSG: rosgraph_msgs/TypeHash
# RIHS spec version
uint8 version 1
# ROSIDL_TYPE_HASH_SIZE == 32
uint8[32] value
================================================================================
MSG: rosgraph_msgs/InterfaceType
# Represent a type of a ROS Graph Interface

# The plaintext namespaced name of the type - e.g. sensor_msgs/Image
string name

# The hash uniquely identifies the exact structure of the type,
# the definition of which may change between package version
TypeHash hash
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: rosgraph_msgs/QoSProfile
# Message-based representation of ROS 2 Quality of Service settings
# Default values are kept in sync with RMW by integration test
# Note that SYSTEM_DEFAULT and BEST_AVAILABLE values cannot be an observed value,
# because they resolve concretely at runtime.
# They are included here for completeness to match the data structures in RMW

# Depth of the message queue (only meaningful when history==KEEP_LAST)
uint32 depth

# Deadline between messages (0 for no deadline)
builtin_interfaces/Duration deadline

# Lifespan of each message (0 for infinite)
builtin_interfaces/Duration lifespan

# History policy
uint8 HISTORY_SYSTEM_DEFAULT=0
uint8 HISTORY_KEEP_LAST=1
uint8 HISTORY_KEEP_ALL=2
uint8 HISTORY_UNKNOWN=3
uint8 history

# Reliability policy
uint8 RELIABILITY_SYSTEM_DEFAULT=0
uint8 RELIABILITY_RELIABLE=1
uint8 RELIABILITY_BEST_EFFORT=2
uint8 RELIABILITY_UNKNOWN=3
uint8 RELIABILITY_BEST_AVAILABLE=4
uint8 reliability

# Durability policy
uint8 DURABILITY_SYSTEM_DEFAULT=0
uint8 DURABILITY_TRANSIENT_LOCAL=1
uint8 DURABILITY_VOLATILE=2
uint8 DURABILITY_UNKNOWN=3
uint8 DURABILITY_BEST_AVAILABLE=4
uint8 durability

# Liveliness policy
uint8 LIVELINESS_SYSTEM_DEFAULT=0
uint8 LIVELINESS_AUTOMATIC=1
uint8 LIVELINESS_MANUAL_BY_TOPIC=3
uint8 LIVELINESS_UNKNOWN=4
uint8 LIVELINESS_BEST_AVAILABLE=5
uint8 liveliness

# Lease duration for liveliness (0 for infinite)
builtin_interfaces/Duration liveliness_lease_duration
================================================================================
MSG: rosgraph_msgs/Service
# Describes a single Service endpoint, which may be a Server or Client

# Fully qualified name of the Service
string name

# Type and actual QoS of the request publisher (Client) or subscription (Server)
InterfaceType request_type
QoSProfile request_qos

# Type and actual QoS of the request subscription (Client) or publisher (Server)
InterfaceType response_type
QoSProfile response_qos
================================================================================
MSG: rosgraph_msgs/Topic
# Describes a single topic endpoint, which may be a Publisher or Subscription

# Fully qualified name of the topic
string name

# Type of the topic
InterfaceType type

# Observed QoS of the endpoint
QoSProfile qos
================================================================================
MSG: rosgraph_msgs/Action
# Describes a single Action endpoint, which may be a Server or Client

# Fully qualified name of the Action
string name

# An action is actually a composition of the following fundamental ROS entities
Service send_goal
Service get_result
Service cancel_goal
Topic feedback
Topic status`,
  "rosgraph_msgs/msg/QoSProfile": `# Message-based representation of ROS 2 Quality of Service settings
# Default values are kept in sync with RMW by integration test
# Note that SYSTEM_DEFAULT and BEST_AVAILABLE values cannot be an observed value,
# because they resolve concretely at runtime.
# They are included here for completeness to match the data structures in RMW

# Depth of the message queue (only meaningful when history==KEEP_LAST)
uint32 depth

# Deadline between messages (0 for no deadline)
builtin_interfaces/Duration deadline

# Lifespan of each message (0 for infinite)
builtin_interfaces/Duration lifespan

# History policy
uint8 HISTORY_SYSTEM_DEFAULT=0
uint8 HISTORY_KEEP_LAST=1
uint8 HISTORY_KEEP_ALL=2
uint8 HISTORY_UNKNOWN=3
uint8 history

# Reliability policy
uint8 RELIABILITY_SYSTEM_DEFAULT=0
uint8 RELIABILITY_RELIABLE=1
uint8 RELIABILITY_BEST_EFFORT=2
uint8 RELIABILITY_UNKNOWN=3
uint8 RELIABILITY_BEST_AVAILABLE=4
uint8 reliability

# Durability policy
uint8 DURABILITY_SYSTEM_DEFAULT=0
uint8 DURABILITY_TRANSIENT_LOCAL=1
uint8 DURABILITY_VOLATILE=2
uint8 DURABILITY_UNKNOWN=3
uint8 DURABILITY_BEST_AVAILABLE=4
uint8 durability

# Liveliness policy
uint8 LIVELINESS_SYSTEM_DEFAULT=0
uint8 LIVELINESS_AUTOMATIC=1
uint8 LIVELINESS_MANUAL_BY_TOPIC=3
uint8 LIVELINESS_UNKNOWN=4
uint8 LIVELINESS_BEST_AVAILABLE=5
uint8 liveliness

# Lease duration for liveliness (0 for infinite)
builtin_interfaces/Duration liveliness_lease_duration
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "rosgraph_msgs/msg/Service": `# Describes a single Service endpoint, which may be a Server or Client

# Fully qualified name of the Service
string name

# Type and actual QoS of the request publisher (Client) or subscription (Server)
InterfaceType request_type
QoSProfile request_qos

# Type and actual QoS of the request subscription (Client) or publisher (Server)
InterfaceType response_type
QoSProfile response_qos
================================================================================
MSG: rosgraph_msgs/TypeHash
# RIHS spec version
uint8 version 1
# ROSIDL_TYPE_HASH_SIZE == 32
uint8[32] value
================================================================================
MSG: rosgraph_msgs/InterfaceType
# Represent a type of a ROS Graph Interface

# The plaintext namespaced name of the type - e.g. sensor_msgs/Image
string name

# The hash uniquely identifies the exact structure of the type,
# the definition of which may change between package version
TypeHash hash
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: rosgraph_msgs/QoSProfile
# Message-based representation of ROS 2 Quality of Service settings
# Default values are kept in sync with RMW by integration test
# Note that SYSTEM_DEFAULT and BEST_AVAILABLE values cannot be an observed value,
# because they resolve concretely at runtime.
# They are included here for completeness to match the data structures in RMW

# Depth of the message queue (only meaningful when history==KEEP_LAST)
uint32 depth

# Deadline between messages (0 for no deadline)
builtin_interfaces/Duration deadline

# Lifespan of each message (0 for infinite)
builtin_interfaces/Duration lifespan

# History policy
uint8 HISTORY_SYSTEM_DEFAULT=0
uint8 HISTORY_KEEP_LAST=1
uint8 HISTORY_KEEP_ALL=2
uint8 HISTORY_UNKNOWN=3
uint8 history

# Reliability policy
uint8 RELIABILITY_SYSTEM_DEFAULT=0
uint8 RELIABILITY_RELIABLE=1
uint8 RELIABILITY_BEST_EFFORT=2
uint8 RELIABILITY_UNKNOWN=3
uint8 RELIABILITY_BEST_AVAILABLE=4
uint8 reliability

# Durability policy
uint8 DURABILITY_SYSTEM_DEFAULT=0
uint8 DURABILITY_TRANSIENT_LOCAL=1
uint8 DURABILITY_VOLATILE=2
uint8 DURABILITY_UNKNOWN=3
uint8 DURABILITY_BEST_AVAILABLE=4
uint8 durability

# Liveliness policy
uint8 LIVELINESS_SYSTEM_DEFAULT=0
uint8 LIVELINESS_AUTOMATIC=1
uint8 LIVELINESS_MANUAL_BY_TOPIC=3
uint8 LIVELINESS_UNKNOWN=4
uint8 LIVELINESS_BEST_AVAILABLE=5
uint8 liveliness

# Lease duration for liveliness (0 for infinite)
builtin_interfaces/Duration liveliness_lease_duration`,
  "rosgraph_msgs/msg/Topic": `# Describes a single topic endpoint, which may be a Publisher or Subscription

# Fully qualified name of the topic
string name

# Type of the topic
InterfaceType type

# Observed QoS of the endpoint
QoSProfile qos
================================================================================
MSG: rosgraph_msgs/TypeHash
# RIHS spec version
uint8 version 1
# ROSIDL_TYPE_HASH_SIZE == 32
uint8[32] value
================================================================================
MSG: rosgraph_msgs/InterfaceType
# Represent a type of a ROS Graph Interface

# The plaintext namespaced name of the type - e.g. sensor_msgs/Image
string name

# The hash uniquely identifies the exact structure of the type,
# the definition of which may change between package version
TypeHash hash
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: rosgraph_msgs/QoSProfile
# Message-based representation of ROS 2 Quality of Service settings
# Default values are kept in sync with RMW by integration test
# Note that SYSTEM_DEFAULT and BEST_AVAILABLE values cannot be an observed value,
# because they resolve concretely at runtime.
# They are included here for completeness to match the data structures in RMW

# Depth of the message queue (only meaningful when history==KEEP_LAST)
uint32 depth

# Deadline between messages (0 for no deadline)
builtin_interfaces/Duration deadline

# Lifespan of each message (0 for infinite)
builtin_interfaces/Duration lifespan

# History policy
uint8 HISTORY_SYSTEM_DEFAULT=0
uint8 HISTORY_KEEP_LAST=1
uint8 HISTORY_KEEP_ALL=2
uint8 HISTORY_UNKNOWN=3
uint8 history

# Reliability policy
uint8 RELIABILITY_SYSTEM_DEFAULT=0
uint8 RELIABILITY_RELIABLE=1
uint8 RELIABILITY_BEST_EFFORT=2
uint8 RELIABILITY_UNKNOWN=3
uint8 RELIABILITY_BEST_AVAILABLE=4
uint8 reliability

# Durability policy
uint8 DURABILITY_SYSTEM_DEFAULT=0
uint8 DURABILITY_TRANSIENT_LOCAL=1
uint8 DURABILITY_VOLATILE=2
uint8 DURABILITY_UNKNOWN=3
uint8 DURABILITY_BEST_AVAILABLE=4
uint8 durability

# Liveliness policy
uint8 LIVELINESS_SYSTEM_DEFAULT=0
uint8 LIVELINESS_AUTOMATIC=1
uint8 LIVELINESS_MANUAL_BY_TOPIC=3
uint8 LIVELINESS_UNKNOWN=4
uint8 LIVELINESS_BEST_AVAILABLE=5
uint8 liveliness

# Lease duration for liveliness (0 for infinite)
builtin_interfaces/Duration liveliness_lease_duration`,
  "rosgraph_msgs/msg/TypeHash": `# RIHS spec version
uint8 version 1
# ROSIDL_TYPE_HASH_SIZE == 32
uint8[32] value`,
  "sensor_msgs/msg/BatteryState": `
# Constants are chosen to match the enums in the linux kernel
# defined in include/linux/power_supply.h as of version 3.7
# The one difference is for style reasons the constants are
# all uppercase not mixed case.

# Power supply status constants
uint8 POWER_SUPPLY_STATUS_UNKNOWN = 0
uint8 POWER_SUPPLY_STATUS_CHARGING = 1
uint8 POWER_SUPPLY_STATUS_DISCHARGING = 2
uint8 POWER_SUPPLY_STATUS_NOT_CHARGING = 3
uint8 POWER_SUPPLY_STATUS_FULL = 4

# Power supply health constants
uint8 POWER_SUPPLY_HEALTH_UNKNOWN = 0
uint8 POWER_SUPPLY_HEALTH_GOOD = 1
uint8 POWER_SUPPLY_HEALTH_OVERHEAT = 2
uint8 POWER_SUPPLY_HEALTH_DEAD = 3
uint8 POWER_SUPPLY_HEALTH_OVERVOLTAGE = 4
uint8 POWER_SUPPLY_HEALTH_UNSPEC_FAILURE = 5
uint8 POWER_SUPPLY_HEALTH_COLD = 6
uint8 POWER_SUPPLY_HEALTH_WATCHDOG_TIMER_EXPIRE = 7
uint8 POWER_SUPPLY_HEALTH_SAFETY_TIMER_EXPIRE = 8

# Power supply technology (chemistry) constants
uint8 POWER_SUPPLY_TECHNOLOGY_UNKNOWN = 0 # Unknown battery technology
uint8 POWER_SUPPLY_TECHNOLOGY_NIMH = 1    # Nickel-Metal Hydride battery
uint8 POWER_SUPPLY_TECHNOLOGY_LION = 2    # Lithium-ion battery
uint8 POWER_SUPPLY_TECHNOLOGY_LIPO = 3    # Lithium Polymer battery
uint8 POWER_SUPPLY_TECHNOLOGY_LIFE = 4    # Lithium Iron Phosphate battery
uint8 POWER_SUPPLY_TECHNOLOGY_NICD = 5    # Nickel-Cadmium battery
uint8 POWER_SUPPLY_TECHNOLOGY_LIMN = 6    # Lithium Manganese Dioxide battery
uint8 POWER_SUPPLY_TECHNOLOGY_TERNARY = 7 # Ternary Lithium battery
uint8 POWER_SUPPLY_TECHNOLOGY_VRLA = 8    # Valve Regulated Lead-Acid battery

std_msgs/Header  header
float32 voltage          # Voltage in Volts (Mandatory)
float32 temperature      # Temperature in Degrees Celsius (If unmeasured NaN)
float32 current          # Negative when discharging (A)  (If unmeasured NaN)
float32 charge           # Current charge in Ah  (If unmeasured NaN)
float32 capacity         # Capacity in Ah (last full capacity)  (If unmeasured NaN)
float32 design_capacity  # Capacity in Ah (design capacity)  (If unmeasured NaN)
float32 percentage       # Charge percentage on 0 to 1 range  (If unmeasured NaN)
uint8   power_supply_status     # The charging status as reported. Values defined above
uint8   power_supply_health     # The battery health metric. Values defined above
uint8   power_supply_technology # The battery chemistry. Values defined above
bool    present          # True if the battery is present

float32[] cell_voltage   # An array of individual cell voltages for each cell in the pack
                         # If individual voltages unknown but number of cells known set each to NaN
float32[] cell_temperature # An array of individual cell temperatures for each cell in the pack
                           # If individual temperatures unknown but number of cells known set each to NaN
string location          # The location into which the battery is inserted. (slot number or plug)
string serial_number     # The best approximation of the battery serial number
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/CameraInfo": `# This message defines meta information for a camera. It should be in a
# camera namespace on topic "camera_info" and accompanied by up to five
# image topics named:
#
#   image_raw - raw data from the camera driver, possibly Bayer encoded
#   image            - monochrome, distorted
#   image_color      - color, distorted
#   image_rect       - monochrome, rectified
#   image_rect_color - color, rectified
#
# The image_pipeline contains packages (image_proc, stereo_image_proc)
# for producing the four processed image topics from image_raw and
# camera_info. The meaning of the camera parameters are described in
# detail at http://www.ros.org/wiki/image_pipeline/CameraInfo.
#
# The image_geometry package provides a user-friendly interface to
# common operations using this meta information. If you want to, e.g.,
# project a 3d point into image coordinates, we strongly recommend
# using image_geometry.
#
# If the camera is uncalibrated, the matrices D, K, R, P should be left
# zeroed out. In particular, clients may assume that K[0] == 0.0
# indicates an uncalibrated camera.

#######################################################################
#                     Image acquisition info                          #
#######################################################################

# Time of image acquisition, camera coordinate frame ID
std_msgs/Header header # Header timestamp should be acquisition time of image
                             # Header frame_id should be optical frame of camera
                             # origin of frame should be optical center of camera
                             # +x should point to the right in the image
                             # +y should point down in the image
                             # +z should point into the plane of the image


#######################################################################
#                      Calibration Parameters                         #
#######################################################################
# These are fixed during camera calibration. Their values will be the #
# same in all messages until the camera is recalibrated. Note that    #
# self-calibrating systems may "recalibrate" frequently.              #
#                                                                     #
# The internal parameters can be used to warp a raw (distorted) image #
# to:                                                                 #
#   1. An undistorted image (requires D and K)                        #
#   2. A rectified image (requires D, K, R)                           #
# The projection matrix P projects 3D points into the rectified image.#
#######################################################################

# The image dimensions with which the camera was calibrated.
# Normally this will be the full camera resolution in pixels.
uint32 height
uint32 width

# The distortion model used. Supported models are listed in
# sensor_msgs/distortion_models.hpp. For most cameras, "plumb_bob" - a
# simple model of radial and tangential distortion - is sufficent.
string distortion_model

# The distortion parameters, size depending on the distortion model.
# For "plumb_bob", the 5 parameters are: (k1, k2, t1, t2, k3).
float64[] d

# Intrinsic camera matrix for the raw (distorted) images.
#     [fx  0 cx]
# K = [ 0 fy cy]
#     [ 0  0  1]
# Projects 3D points in the camera coordinate frame to 2D pixel
# coordinates using the focal lengths (fx, fy) and principal point
# (cx, cy).
float64[9]  k # 3x3 row-major matrix

# Rectification matrix (stereo cameras only)
# A rotation matrix aligning the camera coordinate system to the ideal
# stereo image plane so that epipolar lines in both stereo images are
# parallel.
float64[9]  r # 3x3 row-major matrix

# Projection/camera matrix
#     [fx'  0  cx' Tx]
# P = [ 0  fy' cy' Ty]
#     [ 0   0   1   0]
# By convention, this matrix specifies the intrinsic (camera) matrix
#  of the processed (rectified) image. That is, the left 3x3 portion
#  is the normal camera intrinsic matrix for the rectified image.
# It projects 3D points in the camera coordinate frame to 2D pixel
#  coordinates using the focal lengths (fx', fy') and principal point
#  (cx', cy') - these may differ from the values in K.
# For monocular cameras, Tx = Ty = 0. Normally, monocular cameras will
#  also have R = the identity and P[1:3,1:3] = K.
# For a stereo pair, the fourth column [Tx Ty 0]' is related to the
#  position of the optical center of the second camera in the first
#  camera's frame. We assume Tz = 0 so both cameras are in the same
#  stereo image plane. The first camera always has Tx = Ty = 0. For
#  the right (second) camera of a horizontal stereo pair, Ty = 0 and
#  Tx = -fx' * B, where B is the baseline between the cameras.
# Given a 3D point [X Y Z]', the projection (x, y) of the point onto
#  the rectified image is given by:
#  [u v w]' = P * [X Y Z 1]'
#         x = u / w
#         y = v / w
#  This holds for both images of a stereo pair.
float64[12] p # 3x4 row-major matrix


#######################################################################
#                      Operational Parameters                         #
#######################################################################
# These define the image region actually captured by the camera       #
# driver. Although they affect the geometry of the output image, they #
# may be changed freely without recalibrating the camera.             #
#######################################################################

# Binning refers here to any camera setting which combines rectangular
#  neighborhoods of pixels into larger "super-pixels." It reduces the
#  resolution of the output image to
#  (width / binning_x) x (height / binning_y).
# The default values binning_x = binning_y = 0 is considered the same
#  as binning_x = binning_y = 1 (no subsampling).
uint32 binning_x
uint32 binning_y

# Region of interest (subwindow of full camera resolution), given in
#  full resolution (unbinned) image coordinates. A particular ROI
#  always denotes the same window of pixels on the camera sensor,
#  regardless of binning settings.
# The default setting of roi (all values 0) is considered the same as
#  full resolution (roi.width = width, roi.height = height).
RegionOfInterest roi
================================================================================
MSG: sensor_msgs/RegionOfInterest
# This message is used to specify a region of interest within an image.
#
# When used to specify the ROI setting of the camera when the image was
# taken, the height and width fields should either match the height and
# width fields for the associated image; or height = width = 0
# indicates that the full resolution image was captured.

uint32 x_offset  # Leftmost pixel of the ROI
                 # (0 if the ROI includes the left edge of the image)
uint32 y_offset  # Topmost pixel of the ROI
                 # (0 if the ROI includes the top edge of the image)
uint32 height    # Height of ROI
uint32 width     # Width of ROI

# True if a distinct rectified ROI should be calculated from the "raw"
# ROI in this message. Typically this should be False if the full image
# is captured (ROI not used), and True if a subwindow is captured (ROI
# used).
bool do_rectify
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/ChannelFloat32": `# This message is used by the PointCloud message to hold optional data
# associated with each point in the cloud. The length of the values
# array should be the same as the length of the points array in the
# PointCloud, and each value should be associated with the corresponding
# point.
#
# Channel names in existing practice include:
#   "u", "v" - row and column (respectively) in the left stereo image.
#              This is opposite to usual conventions but remains for
#              historical reasons. The newer PointCloud2 message has no
#              such problem.
#   "rgb" - For point clouds produced by color stereo cameras. uint8
#           (R,G,B) values packed into the least significant 24 bits,
#           in order.
#   "intensity" - laser or pixel intensity.
#   "distance"

# The channel name should give semantics of the channel (e.g.
# "intensity" instead of "value").
string name

# The values array should be 1-1 with the elements of the associated
# PointCloud.
float32[] values`,
  "sensor_msgs/msg/CompressedImage": `# This message contains a compressed image.

std_msgs/Header header # Header timestamp should be acquisition time of image
                             # Header frame_id should be optical frame of camera
                             # origin of frame should be optical center of cameara
                             # +x should point to the right in the image
                             # +y should point down in the image
                             # +z should point into to plane of the image

string format                # Specifies the format of the data
                             # Acceptable values differ by the image transport used:
                             # - compressed_image_transport:
                             #     ORIG_PIXFMT; CODEC compressed [COMPRESSED_PIXFMT]
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #   - CODEC is one of [jpeg, png, tiff]
                             #   - COMPRESSED_PIXFMT is only appended for color images
                             #     and is the pixel format used by the compression
                             #     algorithm. Valid values for jpeg encoding are:
                             #     [bgr8, rgb8]. Valid values for png encoding are:
                             #     [bgr8, rgb8, bgr16, rgb16].
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as bgr8 or mono8
                             #   jpeg image (depending on the number of channels).
                             # - compressed_depth_image_transport:
                             #     ORIG_PIXFMT; compressedDepth CODEC
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #     It is usually one of [16UC1, 32FC1].
                             #   - CODEC is one of [png, rvl]
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as png image.
                             # - Other image transports can store whatever values they
                             #   need for successful decoding of the image. Refer to
                             #   documentation of the other transports for details.

uint8[] data                 # Compressed image buffer
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/FluidPressure": `# Single pressure reading.  This message is appropriate for measuring the
# pressure inside of a fluid (air, water, etc).  This also includes
# atmospheric or barometric pressure.
#
# This message is not appropriate for force/pressure contact sensors.

std_msgs/Header header # timestamp of the measurement
                             # frame_id is the location of the pressure sensor

float64 fluid_pressure       # Absolute pressure reading in Pascals.

float64 variance             # 0 is interpreted as variance unknown
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/Illuminance": `# Single photometric illuminance measurement.  Light should be assumed to be
# measured along the sensor's x-axis (the area of detection is the y-z plane).
# The illuminance should have a 0 or positive value and be received with
# the sensor's +X axis pointing toward the light source.
#
# Photometric illuminance is the measure of the human eye's sensitivity of the
# intensity of light encountering or passing through a surface.
#
# All other Photometric and Radiometric measurements should not use this message.
# This message cannot represent:
#  - Luminous intensity (candela/light source output)
#  - Luminance (nits/light output per area)
#  - Irradiance (watt/area), etc.

std_msgs/Header header # timestamp is the time the illuminance was measured
                             # frame_id is the location and direction of the reading

float64 illuminance          # Measurement of the Photometric Illuminance in Lux.

float64 variance             # 0 is interpreted as variance unknown
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/Image": `# This message contains an uncompressed image
# (0, 0) is at top-left corner of image

std_msgs/Header header # Header timestamp should be acquisition time of image
                             # Header frame_id should be optical frame of camera
                             # origin of frame should be optical center of cameara
                             # +x should point to the right in the image
                             # +y should point down in the image
                             # +z should point into to plane of the image
                             # If the frame_id here and the frame_id of the CameraInfo
                             # message associated with the image conflict
                             # the behavior is undefined

uint32 height                # image height, that is, number of rows
uint32 width                 # image width, that is, number of columns

# The legal values for encoding are in file include/sensor_msgs/image_encodings.hpp
# If you want to standardize a new string format, join
# ros-users@lists.ros.org and send an email proposing a new encoding.

string encoding       # Encoding of pixels -- channel meaning, ordering, size
                      # taken from the list of strings in include/sensor_msgs/image_encodings.hpp

uint8 is_bigendian    # is this data bigendian?
uint32 step           # Full row length in bytes
uint8[] data          # actual matrix data, size is (step * rows)
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/Imu": `# This is a message to hold data from an IMU (Inertial Measurement Unit)
#
# Accelerations should be in m/s^2 (not in g's), and rotational velocity should be in rad/sec
#
# If the covariance of the measurement is known, it should be filled in (if all you know is the
# variance of each measurement, e.g. from the datasheet, just put those along the diagonal)
# A covariance matrix of all zeros will be interpreted as "covariance unknown", and to use the
# data a covariance will have to be assumed or gotten from some other source
#
# If you have no estimate for one of the data elements (e.g. your IMU doesn't produce an
# orientation estimate), please set element 0 of the associated covariance matrix to -1
# If you are interpreting this message, please check for a value of -1 in the first element of each
# covariance matrix, and disregard the associated estimate.

std_msgs/Header header

geometry_msgs/Quaternion orientation
float64[9] orientation_covariance # Row major about x, y, z axes

geometry_msgs/Vector3 angular_velocity
float64[9] angular_velocity_covariance # Row major about x, y, z axes

geometry_msgs/Vector3 linear_acceleration
float64[9] linear_acceleration_covariance # Row major x, y z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/JointState": `# This is a message that holds data to describe the state of a set of torque controlled joints.
#
# The state of each joint (revolute or prismatic) is defined by:
#  * the position of the joint (rad or m),
#  * the velocity of the joint (rad/s or m/s) and
#  * the effort that is applied in the joint (Nm or N).
#
# Each joint is uniquely identified by its name
# The header specifies the time at which the joint states were recorded. All the joint states
# in one message have to be recorded at the same time.
#
# This message consists of a multiple arrays, one for each part of the joint state.
# The goal is to make each of the fields optional. When e.g. your joints have no
# effort associated with them, you can leave the effort array empty.
#
# All arrays in this message should have the same size, or be empty.
# This is the only way to uniquely associate the joint name with the correct
# states.

std_msgs/Header header

string[] name
float64[] position
float64[] velocity
float64[] effort
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/Joy": `# Reports the state of a joystick's axes and buttons.

# The timestamp is the time at which data is received from the joystick.
std_msgs/Header header

# The axes measurements from a joystick.
float32[] axes

# The buttons measurements from a joystick.
int32[] buttons
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/JoyFeedback": `# Declare of the type of feedback
uint8 TYPE_LED    = 0
uint8 TYPE_RUMBLE = 1
uint8 TYPE_BUZZER = 2

uint8 type

# This will hold an id number for each type of each feedback.
# Example, the first led would be id=0, the second would be id=1
uint8 id

# Intensity of the feedback, from 0.0 to 1.0, inclusive.  If device is
# actually binary, driver should treat 0<=x<0.5 as off, 0.5<=x<=1 as on.
float32 intensity`,
  "sensor_msgs/msg/JoyFeedbackArray": `# This message publishes values for multiple feedback at once.
JoyFeedback[] array
================================================================================
MSG: sensor_msgs/JoyFeedback
# Declare of the type of feedback
uint8 TYPE_LED    = 0
uint8 TYPE_RUMBLE = 1
uint8 TYPE_BUZZER = 2

uint8 type

# This will hold an id number for each type of each feedback.
# Example, the first led would be id=0, the second would be id=1
uint8 id

# Intensity of the feedback, from 0.0 to 1.0, inclusive.  If device is
# actually binary, driver should treat 0<=x<0.5 as off, 0.5<=x<=1 as on.
float32 intensity`,
  "sensor_msgs/msg/LaserEcho": `# This message is a submessage of MultiEchoLaserScan and is not intended
# to be used separately.

float32[] echoes  # Multiple values of ranges or intensities.
                  # Each array represents data from the same angle increment.`,
  "sensor_msgs/msg/LaserScan": `# Single scan from a planar laser range-finder
#
# If you have another ranging device with different behavior (e.g. a sonar
# array), please find or create a different message, since applications
# will make fairly laser-specific assumptions about this data

std_msgs/Header header # timestamp in the header is the acquisition time of
                             # the first ray in the scan.
                             #
                             # in frame frame_id, angles are measured around
                             # the positive Z axis (counterclockwise, if Z is up)
                             # with zero angle being forward along the x axis

float32 angle_min            # start angle of the scan [rad]
float32 angle_max            # end angle of the scan [rad]
float32 angle_increment      # angular distance between measurements [rad]

float32 time_increment       # time between measurements [seconds] - if your scanner
                             # is moving, this will be used in interpolating position
                             # of 3d points
float32 scan_time            # time between scans [seconds]

float32 range_min            # minimum range value [m]
float32 range_max            # maximum range value [m]

float32[] ranges             # range data [m]
                             # (Note: values < range_min or > range_max should be discarded)
float32[] intensities        # intensity data [device-specific units].  If your
                             # device does not provide intensities, please leave
                             # the array empty.
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/MagneticField": `# Measurement of the Magnetic Field vector at a specific location.
#
# If the covariance of the measurement is known, it should be filled in.
# If all you know is the variance of each measurement, e.g. from the datasheet,
# just put those along the diagonal.
# A covariance matrix of all zeros will be interpreted as "covariance unknown",
# and to use the data a covariance will have to be assumed or gotten from some
# other source.

std_msgs/Header header               # timestamp is the time the
                                           # field was measured
                                           # frame_id is the location and orientation
                                           # of the field measurement

geometry_msgs/Vector3 magnetic_field # x, y, and z components of the
                                           # field vector in Tesla
                                           # If your sensor does not output 3 axes,
                                           # put NaNs in the components not reported.

float64[9] magnetic_field_covariance       # Row major about x, y, z axes
                                           # 0 is interpreted as variance unknown
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/MultiDOFJointState": `# Representation of state for joints with multiple degrees of freedom,
# following the structure of JointState which can only represent a single degree of freedom.
#
# It is assumed that a joint in a system corresponds to a transform that gets applied
# along the kinematic chain. For example, a planar joint (as in URDF) is 3DOF (x, y, yaw)
# and those 3DOF can be expressed as a transformation matrix, and that transformation
# matrix can be converted back to (x, y, yaw)
#
# Each joint is uniquely identified by its name
# The header specifies the time at which the joint states were recorded. All the joint states
# in one message have to be recorded at the same time.
#
# This message consists of a multiple arrays, one for each part of the joint state.
# The goal is to make each of the fields optional. When e.g. your joints have no
# wrench associated with them, you can leave the wrench array empty.
#
# All arrays in this message should have the same size, or be empty.
# This is the only way to uniquely associate the joint name with the correct
# states.

std_msgs/Header header

string[] joint_names
geometry_msgs/Transform[] transforms
geometry_msgs/Twist[] twist
geometry_msgs/Wrench[] wrench
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Transform
# This represents the transform between two coordinate frames in free space.

Vector3 translation
Quaternion rotation
================================================================================
MSG: geometry_msgs/Twist
# This expresses velocity in free space broken into its linear and angular parts.

Vector3  linear
Vector3  angular
================================================================================
MSG: geometry_msgs/Wrench
# This represents force in free space, separated into its linear and angular parts.

Vector3  force
Vector3  torque
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/MultiEchoLaserScan": `# Single scan from a multi-echo planar laser range-finder
#
# If you have another ranging device with different behavior (e.g. a sonar
# array), please find or create a different message, since applications
# will make fairly laser-specific assumptions about this data

std_msgs/Header header # timestamp in the header is the acquisition time of
                             # the first ray in the scan.
                             #
                             # in frame frame_id, angles are measured around
                             # the positive Z axis (counterclockwise, if Z is up)
                             # with zero angle being forward along the x axis

float32 angle_min            # start angle of the scan [rad]
float32 angle_max            # end angle of the scan [rad]
float32 angle_increment      # angular distance between measurements [rad]

float32 time_increment       # time between measurements [seconds] - if your scanner
                             # is moving, this will be used in interpolating position
                             # of 3d points
float32 scan_time            # time between scans [seconds]

float32 range_min            # minimum range value [m]
float32 range_max            # maximum range value [m]

LaserEcho[] ranges           # range data [m]
                             # (Note: NaNs, values < range_min or > range_max should be discarded)
                             # +Inf measurements are out of range
                             # -Inf measurements are too close to determine exact distance.
LaserEcho[] intensities      # intensity data [device-specific units].  If your
                             # device does not provide intensities, please leave
                             # the array empty.
================================================================================
MSG: sensor_msgs/LaserEcho
# This message is a submessage of MultiEchoLaserScan and is not intended
# to be used separately.

float32[] echoes  # Multiple values of ranges or intensities.
                  # Each array represents data from the same angle increment.
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/NavSatFix": `# Navigation Satellite fix for any Global Navigation Satellite System
#
# Specified using the WGS 84 reference ellipsoid

# header.stamp specifies the ROS time for this measurement (the
#        corresponding satellite time may be reported using the
#        sensor_msgs/TimeReference message).
#
# header.frame_id is the frame of reference reported by the satellite
#        receiver, usually the location of the antenna.  This is a
#        Euclidean frame relative to the vehicle, not a reference
#        ellipsoid.
std_msgs/Header header

# Satellite fix status information.
NavSatStatus status

# Latitude [degrees]. Positive is north of equator; negative is south.
float64 latitude

# Longitude [degrees]. Positive is east of prime meridian; negative is west.
float64 longitude

# Altitude [m]. Positive is above the WGS 84 ellipsoid
# (quiet NaN if no altitude is available).
float64 altitude

# Position covariance [m^2] defined relative to a tangential plane
# through the reported position. The components are East, North, and
# Up (ENU), in row-major order.
#
# Beware: this coordinate system exhibits singularities at the poles.
float64[9] position_covariance

# If the covariance of the fix is known, fill it in completely. If the
# GPS receiver provides the variance of each measurement, put them
# along the diagonal. If only Dilution of Precision is available,
# estimate an approximate covariance from that.

uint8 COVARIANCE_TYPE_UNKNOWN = 0
uint8 COVARIANCE_TYPE_APPROXIMATED = 1
uint8 COVARIANCE_TYPE_DIAGONAL_KNOWN = 2
uint8 COVARIANCE_TYPE_KNOWN = 3

uint8 position_covariance_type
================================================================================
MSG: sensor_msgs/NavSatStatus
# Navigation Satellite fix status for any Global Navigation Satellite System.
#
# Whether to output an augmented fix is determined by both the fix
# type and the last time differential corrections were received.  A
# fix is valid when status >= STATUS_FIX.

int8 STATUS_UNKNOWN = -2        # status is not yet set
int8 STATUS_NO_FIX =  -1        # unable to fix position
int8 STATUS_FIX =      0        # unaugmented fix
int8 STATUS_SBAS_FIX = 1        # with satellite-based augmentation
int8 STATUS_GBAS_FIX = 2        # with ground-based augmentation

int8 status -2 # STATUS_UNKNOWN

# Bits defining which Global Navigation Satellite System signals were
# used by the receiver.

uint16 SERVICE_UNKNOWN = 0  # Remember service is a bitfield, so checking (service & SERVICE_UNKNOWN) will not work. Use == instead.
uint16 SERVICE_GPS =     1
uint16 SERVICE_GLONASS = 2
uint16 SERVICE_COMPASS = 4      # includes BeiDou.
uint16 SERVICE_GALILEO = 8

uint16 service
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/NavSatStatus": `# Navigation Satellite fix status for any Global Navigation Satellite System.
#
# Whether to output an augmented fix is determined by both the fix
# type and the last time differential corrections were received.  A
# fix is valid when status >= STATUS_FIX.

int8 STATUS_UNKNOWN = -2        # status is not yet set
int8 STATUS_NO_FIX =  -1        # unable to fix position
int8 STATUS_FIX =      0        # unaugmented fix
int8 STATUS_SBAS_FIX = 1        # with satellite-based augmentation
int8 STATUS_GBAS_FIX = 2        # with ground-based augmentation

int8 status -2 # STATUS_UNKNOWN

# Bits defining which Global Navigation Satellite System signals were
# used by the receiver.

uint16 SERVICE_UNKNOWN = 0  # Remember service is a bitfield, so checking (service & SERVICE_UNKNOWN) will not work. Use == instead.
uint16 SERVICE_GPS =     1
uint16 SERVICE_GLONASS = 2
uint16 SERVICE_COMPASS = 4      # includes BeiDou.
uint16 SERVICE_GALILEO = 8

uint16 service`,
  "sensor_msgs/msg/PointCloud": `## THIS MESSAGE IS DEPRECATED AS OF FOXY
## Please use sensor_msgs/PointCloud2

# This message holds a collection of 3d points, plus optional additional
# information about each point.

# Time of sensor data acquisition, coordinate frame ID.
std_msgs/Header header

# Array of 3d points. Each Point32 should be interpreted as a 3d point
# in the frame given in the header.
geometry_msgs/Point32[] points

# Each channel should have the same number of elements as points array,
# and the data in each channel should correspond 1:1 with each point.
# Channel names in common practice are listed in ChannelFloat32.msg.
ChannelFloat32[] channels
================================================================================
MSG: geometry_msgs/Point32
# This contains the position of a point in free space(with 32 bits of precision).
# It is recommended to use Point wherever possible instead of Point32.
#
# This recommendation is to promote interoperability.
#
# This message is designed to take up less space when sending
# lots of points at once, as in the case of a PointCloud.

float32 x
float32 y
float32 z
================================================================================
MSG: sensor_msgs/ChannelFloat32
# This message is used by the PointCloud message to hold optional data
# associated with each point in the cloud. The length of the values
# array should be the same as the length of the points array in the
# PointCloud, and each value should be associated with the corresponding
# point.
#
# Channel names in existing practice include:
#   "u", "v" - row and column (respectively) in the left stereo image.
#              This is opposite to usual conventions but remains for
#              historical reasons. The newer PointCloud2 message has no
#              such problem.
#   "rgb" - For point clouds produced by color stereo cameras. uint8
#           (R,G,B) values packed into the least significant 24 bits,
#           in order.
#   "intensity" - laser or pixel intensity.
#   "distance"

# The channel name should give semantics of the channel (e.g.
# "intensity" instead of "value").
string name

# The values array should be 1-1 with the elements of the associated
# PointCloud.
float32[] values
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/PointCloud2": `# This message holds a collection of N-dimensional points, which may
# contain additional information such as normals, intensity, etc. The
# point data is stored as a binary blob, its layout described by the
# contents of the "fields" array.
#
# The point cloud data may be organized 2d (image-like) or 1d (unordered).
# Point clouds organized as 2d images may be produced by camera depth sensors
# such as stereo or time-of-flight.

# Time of sensor data acquisition, and the coordinate frame ID (for 3d points).
std_msgs/Header header

# 2D structure of the point cloud. If the cloud is unordered, height is
# 1 and width is the length of the point cloud.
uint32 height
uint32 width

# Describes the channels and their layout in the binary data blob.
PointField[] fields

bool    is_bigendian # Is this data bigendian?
uint32  point_step   # Length of a point in bytes
uint32  row_step     # Length of a row in bytes
uint8[] data         # Actual point data, size is (row_step*height)

bool is_dense        # True if there are no invalid points
================================================================================
MSG: sensor_msgs/PointField
# This message holds the description of one point entry in the
# PointCloud2 message format.
uint8 INT8    = 1
uint8 UINT8   = 2
uint8 INT16   = 3
uint8 UINT16  = 4
uint8 INT32   = 5
uint8 UINT32  = 6
uint8 FLOAT32 = 7
uint8 FLOAT64 = 8

# Common PointField names are x, y, z, intensity, rgb, rgba
string name      # Name of field
uint32 offset    # Offset from start of point struct
uint8  datatype  # Datatype enumeration, see above
uint32 count     # How many elements in the field
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/PointField": `# This message holds the description of one point entry in the
# PointCloud2 message format.
uint8 INT8    = 1
uint8 UINT8   = 2
uint8 INT16   = 3
uint8 UINT16  = 4
uint8 INT32   = 5
uint8 UINT32  = 6
uint8 FLOAT32 = 7
uint8 FLOAT64 = 8

# Common PointField names are x, y, z, intensity, rgb, rgba
string name      # Name of field
uint32 offset    # Offset from start of point struct
uint8  datatype  # Datatype enumeration, see above
uint32 count     # How many elements in the field`,
  "sensor_msgs/msg/Range": `# Single range reading from an active ranger that emits energy and reports
# one range reading that is valid along an arc at the distance measured.
# This message is  not appropriate for laser scanners. See the LaserScan
# message if you are working with a laser scanner.
#
# This message also can represent a fixed-distance (binary) ranger.  This
# sensor will have min_range===max_range===distance of detection.
# These sensors follow REP 117 and will output -Inf if the object is detected
# and +Inf if the object is outside of the detection range.

std_msgs/Header header # timestamp in the header is the time the ranger
                             # returned the distance reading

# Radiation type enums
# If you want a value added to this list, send an email to the ros-users list
uint8 ULTRASOUND=0
uint8 INFRARED=1

uint8 radiation_type    # the type of radiation used by the sensor
                        # (sound, IR, etc) [enum]

float32 field_of_view   # the size of the arc that the distance reading is
                        # valid for [rad]
                        # the object causing the range reading may have
                        # been anywhere within -field_of_view/2 and
                        # field_of_view/2 at the measured range.
                        # 0 angle corresponds to the x-axis of the sensor.

float32 min_range       # minimum range value [m]
float32 max_range       # maximum range value [m]
                        # Fixed distance rangers require min_range==max_range

float32 range           # range data [m]
                        # (Note: values < range_min or > range_max should be discarded)
                        # Fixed distance rangers only output -Inf or +Inf.
                        # -Inf represents a detection within fixed distance.
                        # (Detection too close to the sensor to quantify)
                        # +Inf represents no detection within the fixed distance.
                        # (Object out of range)

float32 variance        # variance of the range sensor
                        # 0 is interpreted as variance unknown
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/RegionOfInterest": `# This message is used to specify a region of interest within an image.
#
# When used to specify the ROI setting of the camera when the image was
# taken, the height and width fields should either match the height and
# width fields for the associated image; or height = width = 0
# indicates that the full resolution image was captured.

uint32 x_offset  # Leftmost pixel of the ROI
                 # (0 if the ROI includes the left edge of the image)
uint32 y_offset  # Topmost pixel of the ROI
                 # (0 if the ROI includes the top edge of the image)
uint32 height    # Height of ROI
uint32 width     # Width of ROI

# True if a distinct rectified ROI should be calculated from the "raw"
# ROI in this message. Typically this should be False if the full image
# is captured (ROI not used), and True if a subwindow is captured (ROI
# used).
bool do_rectify`,
  "sensor_msgs/msg/RelativeHumidity": `# Single reading from a relative humidity sensor.
# Defines the ratio of partial pressure of water vapor to the saturated vapor
# pressure at a temperature.

std_msgs/Header header # timestamp of the measurement
                             # frame_id is the location of the humidity sensor

float64 relative_humidity    # Expression of the relative humidity
                             # from 0.0 to 1.0.
                             # 0.0 is no partial pressure of water vapor
                             # 1.0 represents partial pressure of saturation

float64 variance             # 0 is interpreted as variance unknown
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/Temperature": `# Single temperature reading.

std_msgs/Header header # timestamp is the time the temperature was measured
                             # frame_id is the location of the temperature reading

float64 temperature          # Measurement of the Temperature in Degrees Celsius.

float64 variance             # 0 is interpreted as variance unknown.
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "sensor_msgs/msg/TimeReference": `# Measurement from an external time source not actively synchronized with the system clock.

std_msgs/Header header      # stamp is system time for which measurement was valid
                                  # frame_id is not used

builtin_interfaces/Time time_ref  # corresponding time from this external source
string source                     # (optional) name of time source
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "service_msgs/msg/ServiceEventInfo": `uint8 REQUEST_SENT = 0
uint8 REQUEST_RECEIVED = 1
uint8 RESPONSE_SENT = 2
uint8 RESPONSE_RECEIVED = 3

# The type of event this message represents
uint8 event_type

# Timestamp for when the event occurred (sent or received time)
builtin_interfaces/Time stamp

# Unique identifier for the client that sent the service request
# Note, this is only unique for the current session.
# The size here has to match the size of rmw_dds_common/msg/Gid,
# but unfortunately we cannot use that message directly due to a
# circular dependency.
char[16] client_gid

# Sequence number for the request
# Combined with the client ID, this creates a unique ID for the service transaction
int64 sequence_number
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "shape_msgs/msg/Mesh": `# Definition of a mesh.

# List of triangles; the index values refer to positions in vertices[].
MeshTriangle[] triangles

# The actual vertices that make up the mesh.
geometry_msgs/Point[] vertices
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: shape_msgs/MeshTriangle
# Definition of a triangle's vertices.

uint32[3] vertex_indices`,
  "shape_msgs/msg/MeshTriangle": `# Definition of a triangle's vertices.

uint32[3] vertex_indices`,
  "shape_msgs/msg/Plane": `# Representation of a plane, using the plane equation ax + by + cz + d = 0.
#
# a := coef[0]
# b := coef[1]
# c := coef[2]
# d := coef[3]
float64[4] coef`,
  "shape_msgs/msg/SolidPrimitive": `# Defines box, sphere, cylinder, cone and prism.
# All shapes are defined to have their bounding boxes centered around 0,0,0.

uint8 BOX=1
uint8 SPHERE=2
uint8 CYLINDER=3
uint8 CONE=4
uint8 PRISM=5

# The type of the shape
uint8 type

# The dimensions of the shape
float64[<=3] dimensions  # At no point will dimensions have a length > 3.

# The meaning of the shape dimensions: each constant defines the index in the 'dimensions' array.

# For type BOX, the X, Y, and Z dimensions are the length of the corresponding sides of the box.
uint8 BOX_X=0
uint8 BOX_Y=1
uint8 BOX_Z=2

# For the SPHERE type, only one component is used, and it gives the radius of the sphere.
uint8 SPHERE_RADIUS=0

# For the CYLINDER and CONE types, the center line is oriented along the Z axis.
# Therefore the CYLINDER_HEIGHT (CONE_HEIGHT) component of dimensions gives the
# height of the cylinder (cone).
# The CYLINDER_RADIUS (CONE_RADIUS) component of dimensions gives the radius of
# the base of the cylinder (cone).
# Cone and cylinder primitives are defined to be circular. The tip of the cone
# is pointing up, along +Z axis.

uint8 CYLINDER_HEIGHT=0
uint8 CYLINDER_RADIUS=1

uint8 CONE_HEIGHT=0
uint8 CONE_RADIUS=1

# For the type PRISM, the center line is oriented along Z axis.
# The PRISM_HEIGHT component of dimensions gives the
# height of the prism.
# The polygon defines the Z axis centered base of the prism.
# The prism is constructed by extruding the base in +Z and -Z
# directions by half of the PRISM_HEIGHT
# Only x and y fields of the points are used in the polygon.
# Points of the polygon are ordered counter-clockwise.

uint8 PRISM_HEIGHT=0
geometry_msgs/Polygon polygon
================================================================================
MSG: geometry_msgs/Point32
# This contains the position of a point in free space(with 32 bits of precision).
# It is recommended to use Point wherever possible instead of Point32.
#
# This recommendation is to promote interoperability.
#
# This message is designed to take up less space when sending
# lots of points at once, as in the case of a PointCloud.

float32 x
float32 y
float32 z
================================================================================
MSG: geometry_msgs/Polygon
# A specification of a polygon where the first and last points are assumed to be connected

Point32[] points`,
  "statistics_msgs/msg/MetricsMessage": `#############################################
# A generic metrics message providing statistics for measurements from different sources. For example,
# measure a system's CPU % for a given window yields the following data points over a window of time:
#
#   - average cpu %
#   - std deviation
#   - min
#   - max
#   - sample count
#
# These are all represented as different 'StatisticDataPoint's.
#############################################

# Name metric measurement source, e.g., node, topic, or process name
string measurement_source_name

# Name of the metric being measured, e.g. cpu_percentage, free_memory_mb, message_age, etc.
string metrics_source

# Unit of measure of the metric, e.g. percent, mb, seconds, etc.
string unit

# Measurement window start time
builtin_interfaces/Time window_start

# Measurement window end time
builtin_interfaces/Time window_stop

# A list of statistics data point, defined in StatisticDataPoint.msg
StatisticDataPoint[] statistics
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: statistics_msgs/StatisticDataPoint
#############################################
# This holds the structure of a single data point of a StatisticDataType.
#
# This message is used in MetricsStatisticsMessage, defined in MetricsStatisticsMessage.msg.
#
# Examples of the value of data point are
# - average size of messages received
# - standard deviation of the period of messages published
# - maximum age of messages published
#
# A value of nan represents no data is available.
# One example is that standard deviation is only available when there are two or more data points but there is only one,
# and in this case the value would be nan.
# +inf and -inf are not allowed.
#
#############################################

# The statistic type of this data point, defined in StatisticDataType.msg
# Default value should be StatisticDataType.STATISTICS_DATA_TYPE_UNINITIALIZED (0).
uint8 data_type

# The value of the data point
float64 data`,
  "statistics_msgs/msg/StatisticDataPoint": `#############################################
# This holds the structure of a single data point of a StatisticDataType.
#
# This message is used in MetricsStatisticsMessage, defined in MetricsStatisticsMessage.msg.
#
# Examples of the value of data point are
# - average size of messages received
# - standard deviation of the period of messages published
# - maximum age of messages published
#
# A value of nan represents no data is available.
# One example is that standard deviation is only available when there are two or more data points but there is only one,
# and in this case the value would be nan.
# +inf and -inf are not allowed.
#
#############################################

# The statistic type of this data point, defined in StatisticDataType.msg
# Default value should be StatisticDataType.STATISTICS_DATA_TYPE_UNINITIALIZED (0).
uint8 data_type

# The value of the data point
float64 data`,
  "statistics_msgs/msg/StatisticDataType": `#############################################
# This file contains the commonly used constants for the statistics data type.
#
# The value 0 is reserved for unitialized statistic message data type.
# Range of values: [0, 255].
# Unallowed values: any value that is not specified in this file.
#
#############################################

# Constant for uninitialized
uint8 STATISTICS_DATA_TYPE_UNINITIALIZED = 0

# Allowed values
uint8 STATISTICS_DATA_TYPE_AVERAGE = 1
uint8 STATISTICS_DATA_TYPE_MINIMUM = 2
uint8 STATISTICS_DATA_TYPE_MAXIMUM = 3
uint8 STATISTICS_DATA_TYPE_STDDEV = 4
uint8 STATISTICS_DATA_TYPE_SAMPLE_COUNT = 5`,
  "std_msgs/msg/Bool": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

bool data`,
  "std_msgs/msg/Byte": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

byte data`,
  "std_msgs/msg/ByteMultiArray": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# Please look at the MultiArrayLayout message definition for
# documentation on all multiarrays.

MultiArrayLayout  layout        # specification of data layout
byte[]            data          # array of data
================================================================================
MSG: std_msgs/MultiArrayDimension
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string label   # label of given dimension
uint32 size    # size of given dimension (in type units)
uint32 stride  # stride of given dimension
================================================================================
MSG: std_msgs/MultiArrayLayout
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# The multiarray declares a generic multi-dimensional array of a
# particular data type.  Dimensions are ordered from outer most
# to inner most.
#
# Accessors should ALWAYS be written in terms of dimension stride
# and specified outer-most dimension first.
#
# multiarray(i,j,k) = data[data_offset + dim_stride[1]*i + dim_stride[2]*j + k]
#
# A standard, 3-channel 640x480 image with interleaved color channels
# would be specified as:
#
# dim[0].label  = "height"
# dim[0].size   = 480
# dim[0].stride = 3*640*480 = 921600  (note dim[0] stride is just size of image)
# dim[1].label  = "width"
# dim[1].size   = 640
# dim[1].stride = 3*640 = 1920
# dim[2].label  = "channel"
# dim[2].size   = 3
# dim[2].stride = 3
#
# multiarray(i,j,k) refers to the ith row, jth column, and kth channel.

MultiArrayDimension[] dim # Array of dimension properties
uint32 data_offset        # padding bytes at front of data`,
  "std_msgs/msg/Char": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

char data`,
  "std_msgs/msg/ColorRGBA": `float32 r
float32 g
float32 b
float32 a`,
  "std_msgs/msg/Empty": ``,
  "std_msgs/msg/Float32": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

float32 data`,
  "std_msgs/msg/Float32MultiArray": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# Please look at the MultiArrayLayout message definition for
# documentation on all multiarrays.

MultiArrayLayout  layout        # specification of data layout
float32[]         data          # array of data
================================================================================
MSG: std_msgs/MultiArrayDimension
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string label   # label of given dimension
uint32 size    # size of given dimension (in type units)
uint32 stride  # stride of given dimension
================================================================================
MSG: std_msgs/MultiArrayLayout
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# The multiarray declares a generic multi-dimensional array of a
# particular data type.  Dimensions are ordered from outer most
# to inner most.
#
# Accessors should ALWAYS be written in terms of dimension stride
# and specified outer-most dimension first.
#
# multiarray(i,j,k) = data[data_offset + dim_stride[1]*i + dim_stride[2]*j + k]
#
# A standard, 3-channel 640x480 image with interleaved color channels
# would be specified as:
#
# dim[0].label  = "height"
# dim[0].size   = 480
# dim[0].stride = 3*640*480 = 921600  (note dim[0] stride is just size of image)
# dim[1].label  = "width"
# dim[1].size   = 640
# dim[1].stride = 3*640 = 1920
# dim[2].label  = "channel"
# dim[2].size   = 3
# dim[2].stride = 3
#
# multiarray(i,j,k) refers to the ith row, jth column, and kth channel.

MultiArrayDimension[] dim # Array of dimension properties
uint32 data_offset        # padding bytes at front of data`,
  "std_msgs/msg/Float64": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

float64 data`,
  "std_msgs/msg/Float64MultiArray": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# Please look at the MultiArrayLayout message definition for
# documentation on all multiarrays.

MultiArrayLayout  layout        # specification of data layout
float64[]         data          # array of data
================================================================================
MSG: std_msgs/MultiArrayDimension
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string label   # label of given dimension
uint32 size    # size of given dimension (in type units)
uint32 stride  # stride of given dimension
================================================================================
MSG: std_msgs/MultiArrayLayout
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# The multiarray declares a generic multi-dimensional array of a
# particular data type.  Dimensions are ordered from outer most
# to inner most.
#
# Accessors should ALWAYS be written in terms of dimension stride
# and specified outer-most dimension first.
#
# multiarray(i,j,k) = data[data_offset + dim_stride[1]*i + dim_stride[2]*j + k]
#
# A standard, 3-channel 640x480 image with interleaved color channels
# would be specified as:
#
# dim[0].label  = "height"
# dim[0].size   = 480
# dim[0].stride = 3*640*480 = 921600  (note dim[0] stride is just size of image)
# dim[1].label  = "width"
# dim[1].size   = 640
# dim[1].stride = 3*640 = 1920
# dim[2].label  = "channel"
# dim[2].size   = 3
# dim[2].stride = 3
#
# multiarray(i,j,k) refers to the ith row, jth column, and kth channel.

MultiArrayDimension[] dim # Array of dimension properties
uint32 data_offset        # padding bytes at front of data`,
  "std_msgs/msg/Header": `# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "std_msgs/msg/Int16": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

int16 data`,
  "std_msgs/msg/Int16MultiArray": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# Please look at the MultiArrayLayout message definition for
# documentation on all multiarrays.

MultiArrayLayout  layout        # specification of data layout
int16[]           data          # array of data
================================================================================
MSG: std_msgs/MultiArrayDimension
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string label   # label of given dimension
uint32 size    # size of given dimension (in type units)
uint32 stride  # stride of given dimension
================================================================================
MSG: std_msgs/MultiArrayLayout
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# The multiarray declares a generic multi-dimensional array of a
# particular data type.  Dimensions are ordered from outer most
# to inner most.
#
# Accessors should ALWAYS be written in terms of dimension stride
# and specified outer-most dimension first.
#
# multiarray(i,j,k) = data[data_offset + dim_stride[1]*i + dim_stride[2]*j + k]
#
# A standard, 3-channel 640x480 image with interleaved color channels
# would be specified as:
#
# dim[0].label  = "height"
# dim[0].size   = 480
# dim[0].stride = 3*640*480 = 921600  (note dim[0] stride is just size of image)
# dim[1].label  = "width"
# dim[1].size   = 640
# dim[1].stride = 3*640 = 1920
# dim[2].label  = "channel"
# dim[2].size   = 3
# dim[2].stride = 3
#
# multiarray(i,j,k) refers to the ith row, jth column, and kth channel.

MultiArrayDimension[] dim # Array of dimension properties
uint32 data_offset        # padding bytes at front of data`,
  "std_msgs/msg/Int32": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

int32 data`,
  "std_msgs/msg/Int32MultiArray": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# Please look at the MultiArrayLayout message definition for
# documentation on all multiarrays.

MultiArrayLayout  layout        # specification of data layout
int32[]           data          # array of data
================================================================================
MSG: std_msgs/MultiArrayDimension
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string label   # label of given dimension
uint32 size    # size of given dimension (in type units)
uint32 stride  # stride of given dimension
================================================================================
MSG: std_msgs/MultiArrayLayout
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# The multiarray declares a generic multi-dimensional array of a
# particular data type.  Dimensions are ordered from outer most
# to inner most.
#
# Accessors should ALWAYS be written in terms of dimension stride
# and specified outer-most dimension first.
#
# multiarray(i,j,k) = data[data_offset + dim_stride[1]*i + dim_stride[2]*j + k]
#
# A standard, 3-channel 640x480 image with interleaved color channels
# would be specified as:
#
# dim[0].label  = "height"
# dim[0].size   = 480
# dim[0].stride = 3*640*480 = 921600  (note dim[0] stride is just size of image)
# dim[1].label  = "width"
# dim[1].size   = 640
# dim[1].stride = 3*640 = 1920
# dim[2].label  = "channel"
# dim[2].size   = 3
# dim[2].stride = 3
#
# multiarray(i,j,k) refers to the ith row, jth column, and kth channel.

MultiArrayDimension[] dim # Array of dimension properties
uint32 data_offset        # padding bytes at front of data`,
  "std_msgs/msg/Int64": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

int64 data`,
  "std_msgs/msg/Int64MultiArray": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# Please look at the MultiArrayLayout message definition for
# documentation on all multiarrays.

MultiArrayLayout  layout        # specification of data layout
int64[]           data          # array of data
================================================================================
MSG: std_msgs/MultiArrayDimension
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string label   # label of given dimension
uint32 size    # size of given dimension (in type units)
uint32 stride  # stride of given dimension
================================================================================
MSG: std_msgs/MultiArrayLayout
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# The multiarray declares a generic multi-dimensional array of a
# particular data type.  Dimensions are ordered from outer most
# to inner most.
#
# Accessors should ALWAYS be written in terms of dimension stride
# and specified outer-most dimension first.
#
# multiarray(i,j,k) = data[data_offset + dim_stride[1]*i + dim_stride[2]*j + k]
#
# A standard, 3-channel 640x480 image with interleaved color channels
# would be specified as:
#
# dim[0].label  = "height"
# dim[0].size   = 480
# dim[0].stride = 3*640*480 = 921600  (note dim[0] stride is just size of image)
# dim[1].label  = "width"
# dim[1].size   = 640
# dim[1].stride = 3*640 = 1920
# dim[2].label  = "channel"
# dim[2].size   = 3
# dim[2].stride = 3
#
# multiarray(i,j,k) refers to the ith row, jth column, and kth channel.

MultiArrayDimension[] dim # Array of dimension properties
uint32 data_offset        # padding bytes at front of data`,
  "std_msgs/msg/Int8": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

int8 data`,
  "std_msgs/msg/Int8MultiArray": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# Please look at the MultiArrayLayout message definition for
# documentation on all multiarrays.

MultiArrayLayout  layout        # specification of data layout
int8[]            data          # array of data
================================================================================
MSG: std_msgs/MultiArrayDimension
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string label   # label of given dimension
uint32 size    # size of given dimension (in type units)
uint32 stride  # stride of given dimension
================================================================================
MSG: std_msgs/MultiArrayLayout
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# The multiarray declares a generic multi-dimensional array of a
# particular data type.  Dimensions are ordered from outer most
# to inner most.
#
# Accessors should ALWAYS be written in terms of dimension stride
# and specified outer-most dimension first.
#
# multiarray(i,j,k) = data[data_offset + dim_stride[1]*i + dim_stride[2]*j + k]
#
# A standard, 3-channel 640x480 image with interleaved color channels
# would be specified as:
#
# dim[0].label  = "height"
# dim[0].size   = 480
# dim[0].stride = 3*640*480 = 921600  (note dim[0] stride is just size of image)
# dim[1].label  = "width"
# dim[1].size   = 640
# dim[1].stride = 3*640 = 1920
# dim[2].label  = "channel"
# dim[2].size   = 3
# dim[2].stride = 3
#
# multiarray(i,j,k) refers to the ith row, jth column, and kth channel.

MultiArrayDimension[] dim # Array of dimension properties
uint32 data_offset        # padding bytes at front of data`,
  "std_msgs/msg/MultiArrayDimension": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string label   # label of given dimension
uint32 size    # size of given dimension (in type units)
uint32 stride  # stride of given dimension`,
  "std_msgs/msg/MultiArrayLayout": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# The multiarray declares a generic multi-dimensional array of a
# particular data type.  Dimensions are ordered from outer most
# to inner most.
#
# Accessors should ALWAYS be written in terms of dimension stride
# and specified outer-most dimension first.
#
# multiarray(i,j,k) = data[data_offset + dim_stride[1]*i + dim_stride[2]*j + k]
#
# A standard, 3-channel 640x480 image with interleaved color channels
# would be specified as:
#
# dim[0].label  = "height"
# dim[0].size   = 480
# dim[0].stride = 3*640*480 = 921600  (note dim[0] stride is just size of image)
# dim[1].label  = "width"
# dim[1].size   = 640
# dim[1].stride = 3*640 = 1920
# dim[2].label  = "channel"
# dim[2].size   = 3
# dim[2].stride = 3
#
# multiarray(i,j,k) refers to the ith row, jth column, and kth channel.

MultiArrayDimension[] dim # Array of dimension properties
uint32 data_offset        # padding bytes at front of data
================================================================================
MSG: std_msgs/MultiArrayDimension
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string label   # label of given dimension
uint32 size    # size of given dimension (in type units)
uint32 stride  # stride of given dimension`,
  "std_msgs/msg/String": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string data`,
  "std_msgs/msg/UInt16": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

uint16 data`,
  "std_msgs/msg/UInt16MultiArray": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# Please look at the MultiArrayLayout message definition for
# documentation on all multiarrays.

MultiArrayLayout  layout        # specification of data layout
uint16[]            data        # array of data
================================================================================
MSG: std_msgs/MultiArrayDimension
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string label   # label of given dimension
uint32 size    # size of given dimension (in type units)
uint32 stride  # stride of given dimension
================================================================================
MSG: std_msgs/MultiArrayLayout
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# The multiarray declares a generic multi-dimensional array of a
# particular data type.  Dimensions are ordered from outer most
# to inner most.
#
# Accessors should ALWAYS be written in terms of dimension stride
# and specified outer-most dimension first.
#
# multiarray(i,j,k) = data[data_offset + dim_stride[1]*i + dim_stride[2]*j + k]
#
# A standard, 3-channel 640x480 image with interleaved color channels
# would be specified as:
#
# dim[0].label  = "height"
# dim[0].size   = 480
# dim[0].stride = 3*640*480 = 921600  (note dim[0] stride is just size of image)
# dim[1].label  = "width"
# dim[1].size   = 640
# dim[1].stride = 3*640 = 1920
# dim[2].label  = "channel"
# dim[2].size   = 3
# dim[2].stride = 3
#
# multiarray(i,j,k) refers to the ith row, jth column, and kth channel.

MultiArrayDimension[] dim # Array of dimension properties
uint32 data_offset        # padding bytes at front of data`,
  "std_msgs/msg/UInt32": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

uint32 data`,
  "std_msgs/msg/UInt32MultiArray": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# Please look at the MultiArrayLayout message definition for
# documentation on all multiarrays.

MultiArrayLayout  layout        # specification of data layout
uint32[]          data          # array of data
================================================================================
MSG: std_msgs/MultiArrayDimension
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string label   # label of given dimension
uint32 size    # size of given dimension (in type units)
uint32 stride  # stride of given dimension
================================================================================
MSG: std_msgs/MultiArrayLayout
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# The multiarray declares a generic multi-dimensional array of a
# particular data type.  Dimensions are ordered from outer most
# to inner most.
#
# Accessors should ALWAYS be written in terms of dimension stride
# and specified outer-most dimension first.
#
# multiarray(i,j,k) = data[data_offset + dim_stride[1]*i + dim_stride[2]*j + k]
#
# A standard, 3-channel 640x480 image with interleaved color channels
# would be specified as:
#
# dim[0].label  = "height"
# dim[0].size   = 480
# dim[0].stride = 3*640*480 = 921600  (note dim[0] stride is just size of image)
# dim[1].label  = "width"
# dim[1].size   = 640
# dim[1].stride = 3*640 = 1920
# dim[2].label  = "channel"
# dim[2].size   = 3
# dim[2].stride = 3
#
# multiarray(i,j,k) refers to the ith row, jth column, and kth channel.

MultiArrayDimension[] dim # Array of dimension properties
uint32 data_offset        # padding bytes at front of data`,
  "std_msgs/msg/UInt64": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

uint64 data`,
  "std_msgs/msg/UInt64MultiArray": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# Please look at the MultiArrayLayout message definition for
# documentation on all multiarrays.

MultiArrayLayout  layout        # specification of data layout
uint64[]          data          # array of data
================================================================================
MSG: std_msgs/MultiArrayDimension
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string label   # label of given dimension
uint32 size    # size of given dimension (in type units)
uint32 stride  # stride of given dimension
================================================================================
MSG: std_msgs/MultiArrayLayout
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# The multiarray declares a generic multi-dimensional array of a
# particular data type.  Dimensions are ordered from outer most
# to inner most.
#
# Accessors should ALWAYS be written in terms of dimension stride
# and specified outer-most dimension first.
#
# multiarray(i,j,k) = data[data_offset + dim_stride[1]*i + dim_stride[2]*j + k]
#
# A standard, 3-channel 640x480 image with interleaved color channels
# would be specified as:
#
# dim[0].label  = "height"
# dim[0].size   = 480
# dim[0].stride = 3*640*480 = 921600  (note dim[0] stride is just size of image)
# dim[1].label  = "width"
# dim[1].size   = 640
# dim[1].stride = 3*640 = 1920
# dim[2].label  = "channel"
# dim[2].size   = 3
# dim[2].stride = 3
#
# multiarray(i,j,k) refers to the ith row, jth column, and kth channel.

MultiArrayDimension[] dim # Array of dimension properties
uint32 data_offset        # padding bytes at front of data`,
  "std_msgs/msg/UInt8": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

uint8 data`,
  "std_msgs/msg/UInt8MultiArray": `# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# Please look at the MultiArrayLayout message definition for
# documentation on all multiarrays.

MultiArrayLayout  layout        # specification of data layout
uint8[]           data          # array of data
================================================================================
MSG: std_msgs/MultiArrayDimension
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

string label   # label of given dimension
uint32 size    # size of given dimension (in type units)
uint32 stride  # stride of given dimension
================================================================================
MSG: std_msgs/MultiArrayLayout
# This was originally provided as an example message.
# It is deprecated as of Foxy
# It is recommended to create your own semantically meaningful message.
# However if you would like to continue using this please use the equivalent in example_msgs.

# The multiarray declares a generic multi-dimensional array of a
# particular data type.  Dimensions are ordered from outer most
# to inner most.
#
# Accessors should ALWAYS be written in terms of dimension stride
# and specified outer-most dimension first.
#
# multiarray(i,j,k) = data[data_offset + dim_stride[1]*i + dim_stride[2]*j + k]
#
# A standard, 3-channel 640x480 image with interleaved color channels
# would be specified as:
#
# dim[0].label  = "height"
# dim[0].size   = 480
# dim[0].stride = 3*640*480 = 921600  (note dim[0] stride is just size of image)
# dim[1].label  = "width"
# dim[1].size   = 640
# dim[1].stride = 3*640 = 1920
# dim[2].label  = "channel"
# dim[2].size   = 3
# dim[2].stride = 3
#
# multiarray(i,j,k) refers to the ith row, jth column, and kth channel.

MultiArrayDimension[] dim # Array of dimension properties
uint32 data_offset        # padding bytes at front of data`,
  "stereo_msgs/msg/DisparityImage": `# Separate header for compatibility with current TimeSynchronizer.
# Likely to be removed in a later release, use image.header instead.
std_msgs/Header header

# Floating point disparity image. The disparities are pre-adjusted for any
# x-offset between the principal points of the two cameras (in the case
# that they are verged). That is: d = x_l - x_r - (cx_l - cx_r)
sensor_msgs/Image image

# Stereo geometry. For disparity d, the depth from the camera is Z = fT/d.
float32 f # Focal length, pixels
float32 t # Baseline, world units

# Subwindow of (potentially) valid disparity values.
sensor_msgs/RegionOfInterest valid_window

# The range of disparities searched.
# In the disparity image, any disparity less than min_disparity is invalid.
# The disparity search range defines the horopter, or 3D volume that the
# stereo algorithm can "see". Points with Z outside of:
#     Z_min = fT / max_disparity
#     Z_max = fT / min_disparity
# could not be found.
float32 min_disparity
float32 max_disparity

# Smallest allowed disparity increment. The smallest achievable depth range
# resolution is delta_Z = (Z^2/fT)*delta_d.
float32 delta_d
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: sensor_msgs/Image
# This message contains an uncompressed image
# (0, 0) is at top-left corner of image

std_msgs/Header header # Header timestamp should be acquisition time of image
                             # Header frame_id should be optical frame of camera
                             # origin of frame should be optical center of cameara
                             # +x should point to the right in the image
                             # +y should point down in the image
                             # +z should point into to plane of the image
                             # If the frame_id here and the frame_id of the CameraInfo
                             # message associated with the image conflict
                             # the behavior is undefined

uint32 height                # image height, that is, number of rows
uint32 width                 # image width, that is, number of columns

# The legal values for encoding are in file include/sensor_msgs/image_encodings.hpp
# If you want to standardize a new string format, join
# ros-users@lists.ros.org and send an email proposing a new encoding.

string encoding       # Encoding of pixels -- channel meaning, ordering, size
                      # taken from the list of strings in include/sensor_msgs/image_encodings.hpp

uint8 is_bigendian    # is this data bigendian?
uint32 step           # Full row length in bytes
uint8[] data          # actual matrix data, size is (step * rows)
================================================================================
MSG: sensor_msgs/RegionOfInterest
# This message is used to specify a region of interest within an image.
#
# When used to specify the ROI setting of the camera when the image was
# taken, the height and width fields should either match the height and
# width fields for the associated image; or height = width = 0
# indicates that the full resolution image was captured.

uint32 x_offset  # Leftmost pixel of the ROI
                 # (0 if the ROI includes the left edge of the image)
uint32 y_offset  # Topmost pixel of the ROI
                 # (0 if the ROI includes the top edge of the image)
uint32 height    # Height of ROI
uint32 width     # Width of ROI

# True if a distinct rectified ROI should be calculated from the "raw"
# ROI in this message. Typically this should be False if the full image
# is captured (ROI not used), and True if a subwindow is captured (ROI
# used).
bool do_rectify`,
  "tf2_msgs/msg/TF2Error": `uint8 NO_ERROR = 0
uint8 LOOKUP_ERROR = 1
uint8 CONNECTIVITY_ERROR = 2
uint8 EXTRAPOLATION_ERROR = 3
uint8 INVALID_ARGUMENT_ERROR = 4
uint8 TIMEOUT_ERROR = 5
uint8 TRANSFORM_ERROR = 6

uint8 error
string error_string`,
  "tf2_msgs/msg/TFMessage": `geometry_msgs/TransformStamped[] transforms
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Transform
# This represents the transform between two coordinate frames in free space.

Vector3 translation
Quaternion rotation
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: geometry_msgs/TransformStamped
# This expresses a transform from coordinate frame header.frame_id
# to the coordinate frame child_frame_id at the time of header.stamp
#
# This message is mostly used by the
# <a href="https://docs.ros.org/en/rolling/p/tf2/">tf2</a> package.
# See its documentation for more information.
#
# The child_frame_id is necessary in addition to the frame_id
# in the Header to communicate the full reference for the transform
# in a self contained message.

# The frame id in the header is used as the reference frame of this transform.
std_msgs/Header header

# The frame id of the child frame to which this transform points.
string child_frame_id

# Translation and rotation in 3-dimensions of child_frame_id from header.frame_id.
Transform transform`,
  "trajectory_msgs/msg/JointTrajectory": `# The header is used to specify the coordinate frame and the reference time for
# the trajectory durations
std_msgs/Header header

# The names of the active joints in each trajectory point. These names are
# ordered and must correspond to the values in each trajectory point.
string[] joint_names

# Array of trajectory points, which describe the positions, velocities,
# accelerations and/or efforts of the joints at each time point.
JointTrajectoryPoint[] points
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: trajectory_msgs/JointTrajectoryPoint
# Each trajectory point specifies either positions[, velocities[, accelerations]]
# or positions[, effort] for the trajectory to be executed.
# All specified values are in the same order as the joint names in JointTrajectory.msg.

# Single DOF joint positions for each joint relative to their "0" position.
# The units depend on the specific joint type: radians for revolute or
# continuous joints, and meters for prismatic joints.
float64[] positions

# The rate of change in position of each joint. Units are joint type dependent.
# Radians/second for revolute or continuous joints, and meters/second for
# prismatic joints.
float64[] velocities

# Rate of change in velocity of each joint. Units are joint type dependent.
# Radians/second^2 for revolute or continuous joints, and meters/second^2 for
# prismatic joints.
float64[] accelerations

# The torque or the force to be applied at each joint. For revolute/continuous
# joints effort denotes a torque in newton-meters. For prismatic joints, effort
# denotes a force in newtons.
float64[] effort

# Desired time from the trajectory start to arrive at this trajectory point.
builtin_interfaces/Duration time_from_start`,
  "trajectory_msgs/msg/JointTrajectoryPoint": `# Each trajectory point specifies either positions[, velocities[, accelerations]]
# or positions[, effort] for the trajectory to be executed.
# All specified values are in the same order as the joint names in JointTrajectory.msg.

# Single DOF joint positions for each joint relative to their "0" position.
# The units depend on the specific joint type: radians for revolute or
# continuous joints, and meters for prismatic joints.
float64[] positions

# The rate of change in position of each joint. Units are joint type dependent.
# Radians/second for revolute or continuous joints, and meters/second for
# prismatic joints.
float64[] velocities

# Rate of change in velocity of each joint. Units are joint type dependent.
# Radians/second^2 for revolute or continuous joints, and meters/second^2 for
# prismatic joints.
float64[] accelerations

# The torque or the force to be applied at each joint. For revolute/continuous
# joints effort denotes a torque in newton-meters. For prismatic joints, effort
# denotes a force in newtons.
float64[] effort

# Desired time from the trajectory start to arrive at this trajectory point.
builtin_interfaces/Duration time_from_start
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec`,
  "trajectory_msgs/msg/MultiDOFJointTrajectory": `# The header is used to specify the coordinate frame and the reference time for the trajectory durations
std_msgs/Header header

# A representation of a multi-dof joint trajectory (each point is a transformation)
# Each point along the trajectory will include an array of positions/velocities/accelerations
# that has the same length as the array of joint names, and has the same order of joints as 
# the joint names array.

string[] joint_names
MultiDOFJointTrajectoryPoint[] points
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Transform
# This represents the transform between two coordinate frames in free space.

Vector3 translation
Quaternion rotation
================================================================================
MSG: geometry_msgs/Twist
# This expresses velocity in free space broken into its linear and angular parts.

Vector3  linear
Vector3  angular
================================================================================
MSG: trajectory_msgs/MultiDOFJointTrajectoryPoint
# Each multi-dof joint can specify a transform (up to 6 DOF).
geometry_msgs/Transform[] transforms

# There can be a velocity specified for the origin of the joint.
geometry_msgs/Twist[] velocities

# There can be an acceleration specified for the origin of the joint.
geometry_msgs/Twist[] accelerations

# Desired time from the trajectory start to arrive at this trajectory point.
builtin_interfaces/Duration time_from_start`,
  "trajectory_msgs/msg/MultiDOFJointTrajectoryPoint": `# Each multi-dof joint can specify a transform (up to 6 DOF).
geometry_msgs/Transform[] transforms

# There can be a velocity specified for the origin of the joint.
geometry_msgs/Twist[] velocities

# There can be an acceleration specified for the origin of the joint.
geometry_msgs/Twist[] accelerations

# Desired time from the trajectory start to arrive at this trajectory point.
builtin_interfaces/Duration time_from_start
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Transform
# This represents the transform between two coordinate frames in free space.

Vector3 translation
Quaternion rotation
================================================================================
MSG: geometry_msgs/Twist
# This expresses velocity in free space broken into its linear and angular parts.

Vector3  linear
Vector3  angular`,
  "type_description_interfaces/msg/Field": `# Represents a single field in a type.

# Name of the field.
string name
# Type of the field, including details about the type like length, nested name, etc.
FieldType type
# Literal default value of the field as a string, as it appeared in the original
# message description file, whether that be .msg/.srv/.action or .idl.
string default_value
================================================================================
MSG: type_description_interfaces/FieldType
# Represents the type of a field and related meta-data.

# A constant for each type supported according to:
#   http://design.ros2.org/articles/legacy_interface_definition.html
# and:
#   http://design.ros2.org/articles/idl_interface_definition.html
# Order is loosely coupled to the order of appearance in the IDL 4.2 spec:
#  https://www.omg.org/spec/IDL/4.2

# Layout of constants across the 0-255 decimal values in the uint8:
#
# - 000    : Reserved for "not set"
# - 001-048: Primitive types, strings, and reserved space for future primitive types
# - 049-096: Fixed sized array of primitive and string types
# - 097-144: Bounded Sequences of primitive and string types
# - 145-192: Unbounded Sequences of primitive and string types
# - 193-255: Reserved space for future array/sequence-like types

uint8 FIELD_TYPE_NOT_SET = 0

# Nested type defined in other .msg/.idl files.
uint8 FIELD_TYPE_NESTED_TYPE = 1

# Integer Types
uint8 FIELD_TYPE_INT8 = 2
uint8 FIELD_TYPE_UINT8 = 3
uint8 FIELD_TYPE_INT16 = 4
uint8 FIELD_TYPE_UINT16 = 5
uint8 FIELD_TYPE_INT32 = 6
uint8 FIELD_TYPE_UINT32 = 7
uint8 FIELD_TYPE_INT64 = 8
uint8 FIELD_TYPE_UINT64 = 9

# Floating-Point Types
uint8 FIELD_TYPE_FLOAT = 10
uint8 FIELD_TYPE_DOUBLE = 11
uint8 FIELD_TYPE_LONG_DOUBLE = 12

# Char and WChar Types
uint8 FIELD_TYPE_CHAR = 13
uint8 FIELD_TYPE_WCHAR = 14

# Boolean Type
uint8 FIELD_TYPE_BOOLEAN = 15

# Byte/Octet Type
uint8 FIELD_TYPE_BYTE = 16

# String Types
uint8 FIELD_TYPE_STRING = 17
uint8 FIELD_TYPE_WSTRING = 18

# Fixed String Types
uint8 FIELD_TYPE_FIXED_STRING = 19
uint8 FIELD_TYPE_FIXED_WSTRING = 20

# Bounded String Types
uint8 FIELD_TYPE_BOUNDED_STRING = 21
uint8 FIELD_TYPE_BOUNDED_WSTRING = 22

# Fixed Sized Array Types
uint8 FIELD_TYPE_NESTED_TYPE_ARRAY = 49
uint8 FIELD_TYPE_INT8_ARRAY = 50
uint8 FIELD_TYPE_UINT8_ARRAY = 51
uint8 FIELD_TYPE_INT16_ARRAY = 52
uint8 FIELD_TYPE_UINT16_ARRAY = 53
uint8 FIELD_TYPE_INT32_ARRAY = 54
uint8 FIELD_TYPE_UINT32_ARRAY = 55
uint8 FIELD_TYPE_INT64_ARRAY = 56
uint8 FIELD_TYPE_UINT64_ARRAY = 57
uint8 FIELD_TYPE_FLOAT_ARRAY = 58
uint8 FIELD_TYPE_DOUBLE_ARRAY = 59
uint8 FIELD_TYPE_LONG_DOUBLE_ARRAY = 60
uint8 FIELD_TYPE_CHAR_ARRAY = 61
uint8 FIELD_TYPE_WCHAR_ARRAY = 62
uint8 FIELD_TYPE_BOOLEAN_ARRAY = 63
uint8 FIELD_TYPE_BYTE_ARRAY = 64
uint8 FIELD_TYPE_STRING_ARRAY = 65
uint8 FIELD_TYPE_WSTRING_ARRAY = 66
uint8 FIELD_TYPE_FIXED_STRING_ARRAY = 67
uint8 FIELD_TYPE_FIXED_WSTRING_ARRAY = 68
uint8 FIELD_TYPE_BOUNDED_STRING_ARRAY = 69
uint8 FIELD_TYPE_BOUNDED_WSTRING_ARRAY = 70

# Bounded Sequence Types
uint8 FIELD_TYPE_NESTED_TYPE_BOUNDED_SEQUENCE = 97
uint8 FIELD_TYPE_INT8_BOUNDED_SEQUENCE = 98
uint8 FIELD_TYPE_UINT8_BOUNDED_SEQUENCE = 99
uint8 FIELD_TYPE_INT16_BOUNDED_SEQUENCE = 100
uint8 FIELD_TYPE_UINT16_BOUNDED_SEQUENCE = 101
uint8 FIELD_TYPE_INT32_BOUNDED_SEQUENCE = 102
uint8 FIELD_TYPE_UINT32_BOUNDED_SEQUENCE = 103
uint8 FIELD_TYPE_INT64_BOUNDED_SEQUENCE = 104
uint8 FIELD_TYPE_UINT64_BOUNDED_SEQUENCE = 105
uint8 FIELD_TYPE_FLOAT_BOUNDED_SEQUENCE = 106
uint8 FIELD_TYPE_DOUBLE_BOUNDED_SEQUENCE = 107
uint8 FIELD_TYPE_LONG_DOUBLE_BOUNDED_SEQUENCE = 108
uint8 FIELD_TYPE_CHAR_BOUNDED_SEQUENCE = 109
uint8 FIELD_TYPE_WCHAR_BOUNDED_SEQUENCE = 110
uint8 FIELD_TYPE_BOOLEAN_BOUNDED_SEQUENCE = 111
uint8 FIELD_TYPE_BYTE_BOUNDED_SEQUENCE = 112
uint8 FIELD_TYPE_STRING_BOUNDED_SEQUENCE = 113
uint8 FIELD_TYPE_WSTRING_BOUNDED_SEQUENCE = 114
uint8 FIELD_TYPE_FIXED_STRING_BOUNDED_SEQUENCE = 115
uint8 FIELD_TYPE_FIXED_WSTRING_BOUNDED_SEQUENCE = 116
uint8 FIELD_TYPE_BOUNDED_STRING_BOUNDED_SEQUENCE = 117
uint8 FIELD_TYPE_BOUNDED_WSTRING_BOUNDED_SEQUENCE = 118

# Unbounded Sequence Types
uint8 FIELD_TYPE_NESTED_TYPE_UNBOUNDED_SEQUENCE = 145
uint8 FIELD_TYPE_INT8_UNBOUNDED_SEQUENCE = 146
uint8 FIELD_TYPE_UINT8_UNBOUNDED_SEQUENCE = 147
uint8 FIELD_TYPE_INT16_UNBOUNDED_SEQUENCE = 148
uint8 FIELD_TYPE_UINT16_UNBOUNDED_SEQUENCE = 149
uint8 FIELD_TYPE_INT32_UNBOUNDED_SEQUENCE = 150
uint8 FIELD_TYPE_UINT32_UNBOUNDED_SEQUENCE = 151
uint8 FIELD_TYPE_INT64_UNBOUNDED_SEQUENCE = 152
uint8 FIELD_TYPE_UINT64_UNBOUNDED_SEQUENCE = 153
uint8 FIELD_TYPE_FLOAT_UNBOUNDED_SEQUENCE = 154
uint8 FIELD_TYPE_DOUBLE_UNBOUNDED_SEQUENCE = 155
uint8 FIELD_TYPE_LONG_DOUBLE_UNBOUNDED_SEQUENCE = 156
uint8 FIELD_TYPE_CHAR_UNBOUNDED_SEQUENCE = 157
uint8 FIELD_TYPE_WCHAR_UNBOUNDED_SEQUENCE = 158
uint8 FIELD_TYPE_BOOLEAN_UNBOUNDED_SEQUENCE = 159
uint8 FIELD_TYPE_BYTE_UNBOUNDED_SEQUENCE = 160
uint8 FIELD_TYPE_STRING_UNBOUNDED_SEQUENCE = 161
uint8 FIELD_TYPE_WSTRING_UNBOUNDED_SEQUENCE = 162
uint8 FIELD_TYPE_FIXED_STRING_UNBOUNDED_SEQUENCE = 163
uint8 FIELD_TYPE_FIXED_WSTRING_UNBOUNDED_SEQUENCE = 164
uint8 FIELD_TYPE_BOUNDED_STRING_UNBOUNDED_SEQUENCE = 165
uint8 FIELD_TYPE_BOUNDED_WSTRING_UNBOUNDED_SEQUENCE = 166

# Identifying number for the type of the field, using one of the above constants.
uint8 type_id 0

# Only used when the type is an array or a bounded sequence.
# In the case of an array, this is the fixed capacity of the array.
# In the case of a bounded sequence, this is the maximum capacity of the sequence.
# In all other cases this field is unused.
uint64 capacity

# Only used when the type is a fixed or bounded string/wstring, or a array/sequence of those.
# In the case of a fixed string/wstring, it is the fixed length of the string.
# In the case of a bounded string/wstring, it is the maximum capacity of the string.
# In the case of an array/sequence of fixed string/wstring, it is the fixed length of the strings.
# In the case of an array/sequence of bounded string/wstring, it is the maximum capacity of the strings.
# It is not currently possible to have different string capacities per element in the array/sequence.
uint64 string_capacity

# Only used when the type is a nested type or array/sequence of nested types.
# This is limited to 255 characters.
# TODO(wjwwood): this 255 character limit was chosen due to this being the limit
#   for DDSI-RTPS based middlewares, which is the most commonly used right now.
#   We lack a ROS 2 specific limit in our design documents, but we should update
#   this and/or link to the design doc when that is available.
string<=255 nested_type_name`,
  "type_description_interfaces/msg/FieldType": `# Represents the type of a field and related meta-data.

# A constant for each type supported according to:
#   http://design.ros2.org/articles/legacy_interface_definition.html
# and:
#   http://design.ros2.org/articles/idl_interface_definition.html
# Order is loosely coupled to the order of appearance in the IDL 4.2 spec:
#  https://www.omg.org/spec/IDL/4.2

# Layout of constants across the 0-255 decimal values in the uint8:
#
# - 000    : Reserved for "not set"
# - 001-048: Primitive types, strings, and reserved space for future primitive types
# - 049-096: Fixed sized array of primitive and string types
# - 097-144: Bounded Sequences of primitive and string types
# - 145-192: Unbounded Sequences of primitive and string types
# - 193-255: Reserved space for future array/sequence-like types

uint8 FIELD_TYPE_NOT_SET = 0

# Nested type defined in other .msg/.idl files.
uint8 FIELD_TYPE_NESTED_TYPE = 1

# Integer Types
uint8 FIELD_TYPE_INT8 = 2
uint8 FIELD_TYPE_UINT8 = 3
uint8 FIELD_TYPE_INT16 = 4
uint8 FIELD_TYPE_UINT16 = 5
uint8 FIELD_TYPE_INT32 = 6
uint8 FIELD_TYPE_UINT32 = 7
uint8 FIELD_TYPE_INT64 = 8
uint8 FIELD_TYPE_UINT64 = 9

# Floating-Point Types
uint8 FIELD_TYPE_FLOAT = 10
uint8 FIELD_TYPE_DOUBLE = 11
uint8 FIELD_TYPE_LONG_DOUBLE = 12

# Char and WChar Types
uint8 FIELD_TYPE_CHAR = 13
uint8 FIELD_TYPE_WCHAR = 14

# Boolean Type
uint8 FIELD_TYPE_BOOLEAN = 15

# Byte/Octet Type
uint8 FIELD_TYPE_BYTE = 16

# String Types
uint8 FIELD_TYPE_STRING = 17
uint8 FIELD_TYPE_WSTRING = 18

# Fixed String Types
uint8 FIELD_TYPE_FIXED_STRING = 19
uint8 FIELD_TYPE_FIXED_WSTRING = 20

# Bounded String Types
uint8 FIELD_TYPE_BOUNDED_STRING = 21
uint8 FIELD_TYPE_BOUNDED_WSTRING = 22

# Fixed Sized Array Types
uint8 FIELD_TYPE_NESTED_TYPE_ARRAY = 49
uint8 FIELD_TYPE_INT8_ARRAY = 50
uint8 FIELD_TYPE_UINT8_ARRAY = 51
uint8 FIELD_TYPE_INT16_ARRAY = 52
uint8 FIELD_TYPE_UINT16_ARRAY = 53
uint8 FIELD_TYPE_INT32_ARRAY = 54
uint8 FIELD_TYPE_UINT32_ARRAY = 55
uint8 FIELD_TYPE_INT64_ARRAY = 56
uint8 FIELD_TYPE_UINT64_ARRAY = 57
uint8 FIELD_TYPE_FLOAT_ARRAY = 58
uint8 FIELD_TYPE_DOUBLE_ARRAY = 59
uint8 FIELD_TYPE_LONG_DOUBLE_ARRAY = 60
uint8 FIELD_TYPE_CHAR_ARRAY = 61
uint8 FIELD_TYPE_WCHAR_ARRAY = 62
uint8 FIELD_TYPE_BOOLEAN_ARRAY = 63
uint8 FIELD_TYPE_BYTE_ARRAY = 64
uint8 FIELD_TYPE_STRING_ARRAY = 65
uint8 FIELD_TYPE_WSTRING_ARRAY = 66
uint8 FIELD_TYPE_FIXED_STRING_ARRAY = 67
uint8 FIELD_TYPE_FIXED_WSTRING_ARRAY = 68
uint8 FIELD_TYPE_BOUNDED_STRING_ARRAY = 69
uint8 FIELD_TYPE_BOUNDED_WSTRING_ARRAY = 70

# Bounded Sequence Types
uint8 FIELD_TYPE_NESTED_TYPE_BOUNDED_SEQUENCE = 97
uint8 FIELD_TYPE_INT8_BOUNDED_SEQUENCE = 98
uint8 FIELD_TYPE_UINT8_BOUNDED_SEQUENCE = 99
uint8 FIELD_TYPE_INT16_BOUNDED_SEQUENCE = 100
uint8 FIELD_TYPE_UINT16_BOUNDED_SEQUENCE = 101
uint8 FIELD_TYPE_INT32_BOUNDED_SEQUENCE = 102
uint8 FIELD_TYPE_UINT32_BOUNDED_SEQUENCE = 103
uint8 FIELD_TYPE_INT64_BOUNDED_SEQUENCE = 104
uint8 FIELD_TYPE_UINT64_BOUNDED_SEQUENCE = 105
uint8 FIELD_TYPE_FLOAT_BOUNDED_SEQUENCE = 106
uint8 FIELD_TYPE_DOUBLE_BOUNDED_SEQUENCE = 107
uint8 FIELD_TYPE_LONG_DOUBLE_BOUNDED_SEQUENCE = 108
uint8 FIELD_TYPE_CHAR_BOUNDED_SEQUENCE = 109
uint8 FIELD_TYPE_WCHAR_BOUNDED_SEQUENCE = 110
uint8 FIELD_TYPE_BOOLEAN_BOUNDED_SEQUENCE = 111
uint8 FIELD_TYPE_BYTE_BOUNDED_SEQUENCE = 112
uint8 FIELD_TYPE_STRING_BOUNDED_SEQUENCE = 113
uint8 FIELD_TYPE_WSTRING_BOUNDED_SEQUENCE = 114
uint8 FIELD_TYPE_FIXED_STRING_BOUNDED_SEQUENCE = 115
uint8 FIELD_TYPE_FIXED_WSTRING_BOUNDED_SEQUENCE = 116
uint8 FIELD_TYPE_BOUNDED_STRING_BOUNDED_SEQUENCE = 117
uint8 FIELD_TYPE_BOUNDED_WSTRING_BOUNDED_SEQUENCE = 118

# Unbounded Sequence Types
uint8 FIELD_TYPE_NESTED_TYPE_UNBOUNDED_SEQUENCE = 145
uint8 FIELD_TYPE_INT8_UNBOUNDED_SEQUENCE = 146
uint8 FIELD_TYPE_UINT8_UNBOUNDED_SEQUENCE = 147
uint8 FIELD_TYPE_INT16_UNBOUNDED_SEQUENCE = 148
uint8 FIELD_TYPE_UINT16_UNBOUNDED_SEQUENCE = 149
uint8 FIELD_TYPE_INT32_UNBOUNDED_SEQUENCE = 150
uint8 FIELD_TYPE_UINT32_UNBOUNDED_SEQUENCE = 151
uint8 FIELD_TYPE_INT64_UNBOUNDED_SEQUENCE = 152
uint8 FIELD_TYPE_UINT64_UNBOUNDED_SEQUENCE = 153
uint8 FIELD_TYPE_FLOAT_UNBOUNDED_SEQUENCE = 154
uint8 FIELD_TYPE_DOUBLE_UNBOUNDED_SEQUENCE = 155
uint8 FIELD_TYPE_LONG_DOUBLE_UNBOUNDED_SEQUENCE = 156
uint8 FIELD_TYPE_CHAR_UNBOUNDED_SEQUENCE = 157
uint8 FIELD_TYPE_WCHAR_UNBOUNDED_SEQUENCE = 158
uint8 FIELD_TYPE_BOOLEAN_UNBOUNDED_SEQUENCE = 159
uint8 FIELD_TYPE_BYTE_UNBOUNDED_SEQUENCE = 160
uint8 FIELD_TYPE_STRING_UNBOUNDED_SEQUENCE = 161
uint8 FIELD_TYPE_WSTRING_UNBOUNDED_SEQUENCE = 162
uint8 FIELD_TYPE_FIXED_STRING_UNBOUNDED_SEQUENCE = 163
uint8 FIELD_TYPE_FIXED_WSTRING_UNBOUNDED_SEQUENCE = 164
uint8 FIELD_TYPE_BOUNDED_STRING_UNBOUNDED_SEQUENCE = 165
uint8 FIELD_TYPE_BOUNDED_WSTRING_UNBOUNDED_SEQUENCE = 166

# Identifying number for the type of the field, using one of the above constants.
uint8 type_id 0

# Only used when the type is an array or a bounded sequence.
# In the case of an array, this is the fixed capacity of the array.
# In the case of a bounded sequence, this is the maximum capacity of the sequence.
# In all other cases this field is unused.
uint64 capacity

# Only used when the type is a fixed or bounded string/wstring, or a array/sequence of those.
# In the case of a fixed string/wstring, it is the fixed length of the string.
# In the case of a bounded string/wstring, it is the maximum capacity of the string.
# In the case of an array/sequence of fixed string/wstring, it is the fixed length of the strings.
# In the case of an array/sequence of bounded string/wstring, it is the maximum capacity of the strings.
# It is not currently possible to have different string capacities per element in the array/sequence.
uint64 string_capacity

# Only used when the type is a nested type or array/sequence of nested types.
# This is limited to 255 characters.
# TODO(wjwwood): this 255 character limit was chosen due to this being the limit
#   for DDSI-RTPS based middlewares, which is the most commonly used right now.
#   We lack a ROS 2 specific limit in our design documents, but we should update
#   this and/or link to the design doc when that is available.
string<=255 nested_type_name`,
  "type_description_interfaces/msg/IndividualTypeDescription": `# Represents a single type, without the types it references, if any.

# Name of the type.
# This is limited to 255 characters.
# TODO(wjwwood): this 255 character limit was chosen due to this being the limit
#   for DDSI-RTPS based middlewares, which is the most commonly used right now.
#   We lack a ROS 2 specific limit in our design documents, but we should update
#   this and/or link to the design doc when that is available.
string<=255 type_name
# Fields of the type.
Field[] fields
================================================================================
MSG: type_description_interfaces/FieldType
# Represents the type of a field and related meta-data.

# A constant for each type supported according to:
#   http://design.ros2.org/articles/legacy_interface_definition.html
# and:
#   http://design.ros2.org/articles/idl_interface_definition.html
# Order is loosely coupled to the order of appearance in the IDL 4.2 spec:
#  https://www.omg.org/spec/IDL/4.2

# Layout of constants across the 0-255 decimal values in the uint8:
#
# - 000    : Reserved for "not set"
# - 001-048: Primitive types, strings, and reserved space for future primitive types
# - 049-096: Fixed sized array of primitive and string types
# - 097-144: Bounded Sequences of primitive and string types
# - 145-192: Unbounded Sequences of primitive and string types
# - 193-255: Reserved space for future array/sequence-like types

uint8 FIELD_TYPE_NOT_SET = 0

# Nested type defined in other .msg/.idl files.
uint8 FIELD_TYPE_NESTED_TYPE = 1

# Integer Types
uint8 FIELD_TYPE_INT8 = 2
uint8 FIELD_TYPE_UINT8 = 3
uint8 FIELD_TYPE_INT16 = 4
uint8 FIELD_TYPE_UINT16 = 5
uint8 FIELD_TYPE_INT32 = 6
uint8 FIELD_TYPE_UINT32 = 7
uint8 FIELD_TYPE_INT64 = 8
uint8 FIELD_TYPE_UINT64 = 9

# Floating-Point Types
uint8 FIELD_TYPE_FLOAT = 10
uint8 FIELD_TYPE_DOUBLE = 11
uint8 FIELD_TYPE_LONG_DOUBLE = 12

# Char and WChar Types
uint8 FIELD_TYPE_CHAR = 13
uint8 FIELD_TYPE_WCHAR = 14

# Boolean Type
uint8 FIELD_TYPE_BOOLEAN = 15

# Byte/Octet Type
uint8 FIELD_TYPE_BYTE = 16

# String Types
uint8 FIELD_TYPE_STRING = 17
uint8 FIELD_TYPE_WSTRING = 18

# Fixed String Types
uint8 FIELD_TYPE_FIXED_STRING = 19
uint8 FIELD_TYPE_FIXED_WSTRING = 20

# Bounded String Types
uint8 FIELD_TYPE_BOUNDED_STRING = 21
uint8 FIELD_TYPE_BOUNDED_WSTRING = 22

# Fixed Sized Array Types
uint8 FIELD_TYPE_NESTED_TYPE_ARRAY = 49
uint8 FIELD_TYPE_INT8_ARRAY = 50
uint8 FIELD_TYPE_UINT8_ARRAY = 51
uint8 FIELD_TYPE_INT16_ARRAY = 52
uint8 FIELD_TYPE_UINT16_ARRAY = 53
uint8 FIELD_TYPE_INT32_ARRAY = 54
uint8 FIELD_TYPE_UINT32_ARRAY = 55
uint8 FIELD_TYPE_INT64_ARRAY = 56
uint8 FIELD_TYPE_UINT64_ARRAY = 57
uint8 FIELD_TYPE_FLOAT_ARRAY = 58
uint8 FIELD_TYPE_DOUBLE_ARRAY = 59
uint8 FIELD_TYPE_LONG_DOUBLE_ARRAY = 60
uint8 FIELD_TYPE_CHAR_ARRAY = 61
uint8 FIELD_TYPE_WCHAR_ARRAY = 62
uint8 FIELD_TYPE_BOOLEAN_ARRAY = 63
uint8 FIELD_TYPE_BYTE_ARRAY = 64
uint8 FIELD_TYPE_STRING_ARRAY = 65
uint8 FIELD_TYPE_WSTRING_ARRAY = 66
uint8 FIELD_TYPE_FIXED_STRING_ARRAY = 67
uint8 FIELD_TYPE_FIXED_WSTRING_ARRAY = 68
uint8 FIELD_TYPE_BOUNDED_STRING_ARRAY = 69
uint8 FIELD_TYPE_BOUNDED_WSTRING_ARRAY = 70

# Bounded Sequence Types
uint8 FIELD_TYPE_NESTED_TYPE_BOUNDED_SEQUENCE = 97
uint8 FIELD_TYPE_INT8_BOUNDED_SEQUENCE = 98
uint8 FIELD_TYPE_UINT8_BOUNDED_SEQUENCE = 99
uint8 FIELD_TYPE_INT16_BOUNDED_SEQUENCE = 100
uint8 FIELD_TYPE_UINT16_BOUNDED_SEQUENCE = 101
uint8 FIELD_TYPE_INT32_BOUNDED_SEQUENCE = 102
uint8 FIELD_TYPE_UINT32_BOUNDED_SEQUENCE = 103
uint8 FIELD_TYPE_INT64_BOUNDED_SEQUENCE = 104
uint8 FIELD_TYPE_UINT64_BOUNDED_SEQUENCE = 105
uint8 FIELD_TYPE_FLOAT_BOUNDED_SEQUENCE = 106
uint8 FIELD_TYPE_DOUBLE_BOUNDED_SEQUENCE = 107
uint8 FIELD_TYPE_LONG_DOUBLE_BOUNDED_SEQUENCE = 108
uint8 FIELD_TYPE_CHAR_BOUNDED_SEQUENCE = 109
uint8 FIELD_TYPE_WCHAR_BOUNDED_SEQUENCE = 110
uint8 FIELD_TYPE_BOOLEAN_BOUNDED_SEQUENCE = 111
uint8 FIELD_TYPE_BYTE_BOUNDED_SEQUENCE = 112
uint8 FIELD_TYPE_STRING_BOUNDED_SEQUENCE = 113
uint8 FIELD_TYPE_WSTRING_BOUNDED_SEQUENCE = 114
uint8 FIELD_TYPE_FIXED_STRING_BOUNDED_SEQUENCE = 115
uint8 FIELD_TYPE_FIXED_WSTRING_BOUNDED_SEQUENCE = 116
uint8 FIELD_TYPE_BOUNDED_STRING_BOUNDED_SEQUENCE = 117
uint8 FIELD_TYPE_BOUNDED_WSTRING_BOUNDED_SEQUENCE = 118

# Unbounded Sequence Types
uint8 FIELD_TYPE_NESTED_TYPE_UNBOUNDED_SEQUENCE = 145
uint8 FIELD_TYPE_INT8_UNBOUNDED_SEQUENCE = 146
uint8 FIELD_TYPE_UINT8_UNBOUNDED_SEQUENCE = 147
uint8 FIELD_TYPE_INT16_UNBOUNDED_SEQUENCE = 148
uint8 FIELD_TYPE_UINT16_UNBOUNDED_SEQUENCE = 149
uint8 FIELD_TYPE_INT32_UNBOUNDED_SEQUENCE = 150
uint8 FIELD_TYPE_UINT32_UNBOUNDED_SEQUENCE = 151
uint8 FIELD_TYPE_INT64_UNBOUNDED_SEQUENCE = 152
uint8 FIELD_TYPE_UINT64_UNBOUNDED_SEQUENCE = 153
uint8 FIELD_TYPE_FLOAT_UNBOUNDED_SEQUENCE = 154
uint8 FIELD_TYPE_DOUBLE_UNBOUNDED_SEQUENCE = 155
uint8 FIELD_TYPE_LONG_DOUBLE_UNBOUNDED_SEQUENCE = 156
uint8 FIELD_TYPE_CHAR_UNBOUNDED_SEQUENCE = 157
uint8 FIELD_TYPE_WCHAR_UNBOUNDED_SEQUENCE = 158
uint8 FIELD_TYPE_BOOLEAN_UNBOUNDED_SEQUENCE = 159
uint8 FIELD_TYPE_BYTE_UNBOUNDED_SEQUENCE = 160
uint8 FIELD_TYPE_STRING_UNBOUNDED_SEQUENCE = 161
uint8 FIELD_TYPE_WSTRING_UNBOUNDED_SEQUENCE = 162
uint8 FIELD_TYPE_FIXED_STRING_UNBOUNDED_SEQUENCE = 163
uint8 FIELD_TYPE_FIXED_WSTRING_UNBOUNDED_SEQUENCE = 164
uint8 FIELD_TYPE_BOUNDED_STRING_UNBOUNDED_SEQUENCE = 165
uint8 FIELD_TYPE_BOUNDED_WSTRING_UNBOUNDED_SEQUENCE = 166

# Identifying number for the type of the field, using one of the above constants.
uint8 type_id 0

# Only used when the type is an array or a bounded sequence.
# In the case of an array, this is the fixed capacity of the array.
# In the case of a bounded sequence, this is the maximum capacity of the sequence.
# In all other cases this field is unused.
uint64 capacity

# Only used when the type is a fixed or bounded string/wstring, or a array/sequence of those.
# In the case of a fixed string/wstring, it is the fixed length of the string.
# In the case of a bounded string/wstring, it is the maximum capacity of the string.
# In the case of an array/sequence of fixed string/wstring, it is the fixed length of the strings.
# In the case of an array/sequence of bounded string/wstring, it is the maximum capacity of the strings.
# It is not currently possible to have different string capacities per element in the array/sequence.
uint64 string_capacity

# Only used when the type is a nested type or array/sequence of nested types.
# This is limited to 255 characters.
# TODO(wjwwood): this 255 character limit was chosen due to this being the limit
#   for DDSI-RTPS based middlewares, which is the most commonly used right now.
#   We lack a ROS 2 specific limit in our design documents, but we should update
#   this and/or link to the design doc when that is available.
string<=255 nested_type_name
================================================================================
MSG: type_description_interfaces/Field
# Represents a single field in a type.

# Name of the field.
string name
# Type of the field, including details about the type like length, nested name, etc.
FieldType type
# Literal default value of the field as a string, as it appeared in the original
# message description file, whether that be .msg/.srv/.action or .idl.
string default_value`,
  "type_description_interfaces/msg/KeyValue": `# Represents an arbitrary key-value pair for application-specific information.

string key
string value`,
  "type_description_interfaces/msg/TypeDescription": `# Represents a complete type description, including the type itself as well as the types it references.

# Description of the type.
IndividualTypeDescription type_description
# Descriptions of all referenced types, recursively.
IndividualTypeDescription[] referenced_type_descriptions
================================================================================
MSG: type_description_interfaces/FieldType
# Represents the type of a field and related meta-data.

# A constant for each type supported according to:
#   http://design.ros2.org/articles/legacy_interface_definition.html
# and:
#   http://design.ros2.org/articles/idl_interface_definition.html
# Order is loosely coupled to the order of appearance in the IDL 4.2 spec:
#  https://www.omg.org/spec/IDL/4.2

# Layout of constants across the 0-255 decimal values in the uint8:
#
# - 000    : Reserved for "not set"
# - 001-048: Primitive types, strings, and reserved space for future primitive types
# - 049-096: Fixed sized array of primitive and string types
# - 097-144: Bounded Sequences of primitive and string types
# - 145-192: Unbounded Sequences of primitive and string types
# - 193-255: Reserved space for future array/sequence-like types

uint8 FIELD_TYPE_NOT_SET = 0

# Nested type defined in other .msg/.idl files.
uint8 FIELD_TYPE_NESTED_TYPE = 1

# Integer Types
uint8 FIELD_TYPE_INT8 = 2
uint8 FIELD_TYPE_UINT8 = 3
uint8 FIELD_TYPE_INT16 = 4
uint8 FIELD_TYPE_UINT16 = 5
uint8 FIELD_TYPE_INT32 = 6
uint8 FIELD_TYPE_UINT32 = 7
uint8 FIELD_TYPE_INT64 = 8
uint8 FIELD_TYPE_UINT64 = 9

# Floating-Point Types
uint8 FIELD_TYPE_FLOAT = 10
uint8 FIELD_TYPE_DOUBLE = 11
uint8 FIELD_TYPE_LONG_DOUBLE = 12

# Char and WChar Types
uint8 FIELD_TYPE_CHAR = 13
uint8 FIELD_TYPE_WCHAR = 14

# Boolean Type
uint8 FIELD_TYPE_BOOLEAN = 15

# Byte/Octet Type
uint8 FIELD_TYPE_BYTE = 16

# String Types
uint8 FIELD_TYPE_STRING = 17
uint8 FIELD_TYPE_WSTRING = 18

# Fixed String Types
uint8 FIELD_TYPE_FIXED_STRING = 19
uint8 FIELD_TYPE_FIXED_WSTRING = 20

# Bounded String Types
uint8 FIELD_TYPE_BOUNDED_STRING = 21
uint8 FIELD_TYPE_BOUNDED_WSTRING = 22

# Fixed Sized Array Types
uint8 FIELD_TYPE_NESTED_TYPE_ARRAY = 49
uint8 FIELD_TYPE_INT8_ARRAY = 50
uint8 FIELD_TYPE_UINT8_ARRAY = 51
uint8 FIELD_TYPE_INT16_ARRAY = 52
uint8 FIELD_TYPE_UINT16_ARRAY = 53
uint8 FIELD_TYPE_INT32_ARRAY = 54
uint8 FIELD_TYPE_UINT32_ARRAY = 55
uint8 FIELD_TYPE_INT64_ARRAY = 56
uint8 FIELD_TYPE_UINT64_ARRAY = 57
uint8 FIELD_TYPE_FLOAT_ARRAY = 58
uint8 FIELD_TYPE_DOUBLE_ARRAY = 59
uint8 FIELD_TYPE_LONG_DOUBLE_ARRAY = 60
uint8 FIELD_TYPE_CHAR_ARRAY = 61
uint8 FIELD_TYPE_WCHAR_ARRAY = 62
uint8 FIELD_TYPE_BOOLEAN_ARRAY = 63
uint8 FIELD_TYPE_BYTE_ARRAY = 64
uint8 FIELD_TYPE_STRING_ARRAY = 65
uint8 FIELD_TYPE_WSTRING_ARRAY = 66
uint8 FIELD_TYPE_FIXED_STRING_ARRAY = 67
uint8 FIELD_TYPE_FIXED_WSTRING_ARRAY = 68
uint8 FIELD_TYPE_BOUNDED_STRING_ARRAY = 69
uint8 FIELD_TYPE_BOUNDED_WSTRING_ARRAY = 70

# Bounded Sequence Types
uint8 FIELD_TYPE_NESTED_TYPE_BOUNDED_SEQUENCE = 97
uint8 FIELD_TYPE_INT8_BOUNDED_SEQUENCE = 98
uint8 FIELD_TYPE_UINT8_BOUNDED_SEQUENCE = 99
uint8 FIELD_TYPE_INT16_BOUNDED_SEQUENCE = 100
uint8 FIELD_TYPE_UINT16_BOUNDED_SEQUENCE = 101
uint8 FIELD_TYPE_INT32_BOUNDED_SEQUENCE = 102
uint8 FIELD_TYPE_UINT32_BOUNDED_SEQUENCE = 103
uint8 FIELD_TYPE_INT64_BOUNDED_SEQUENCE = 104
uint8 FIELD_TYPE_UINT64_BOUNDED_SEQUENCE = 105
uint8 FIELD_TYPE_FLOAT_BOUNDED_SEQUENCE = 106
uint8 FIELD_TYPE_DOUBLE_BOUNDED_SEQUENCE = 107
uint8 FIELD_TYPE_LONG_DOUBLE_BOUNDED_SEQUENCE = 108
uint8 FIELD_TYPE_CHAR_BOUNDED_SEQUENCE = 109
uint8 FIELD_TYPE_WCHAR_BOUNDED_SEQUENCE = 110
uint8 FIELD_TYPE_BOOLEAN_BOUNDED_SEQUENCE = 111
uint8 FIELD_TYPE_BYTE_BOUNDED_SEQUENCE = 112
uint8 FIELD_TYPE_STRING_BOUNDED_SEQUENCE = 113
uint8 FIELD_TYPE_WSTRING_BOUNDED_SEQUENCE = 114
uint8 FIELD_TYPE_FIXED_STRING_BOUNDED_SEQUENCE = 115
uint8 FIELD_TYPE_FIXED_WSTRING_BOUNDED_SEQUENCE = 116
uint8 FIELD_TYPE_BOUNDED_STRING_BOUNDED_SEQUENCE = 117
uint8 FIELD_TYPE_BOUNDED_WSTRING_BOUNDED_SEQUENCE = 118

# Unbounded Sequence Types
uint8 FIELD_TYPE_NESTED_TYPE_UNBOUNDED_SEQUENCE = 145
uint8 FIELD_TYPE_INT8_UNBOUNDED_SEQUENCE = 146
uint8 FIELD_TYPE_UINT8_UNBOUNDED_SEQUENCE = 147
uint8 FIELD_TYPE_INT16_UNBOUNDED_SEQUENCE = 148
uint8 FIELD_TYPE_UINT16_UNBOUNDED_SEQUENCE = 149
uint8 FIELD_TYPE_INT32_UNBOUNDED_SEQUENCE = 150
uint8 FIELD_TYPE_UINT32_UNBOUNDED_SEQUENCE = 151
uint8 FIELD_TYPE_INT64_UNBOUNDED_SEQUENCE = 152
uint8 FIELD_TYPE_UINT64_UNBOUNDED_SEQUENCE = 153
uint8 FIELD_TYPE_FLOAT_UNBOUNDED_SEQUENCE = 154
uint8 FIELD_TYPE_DOUBLE_UNBOUNDED_SEQUENCE = 155
uint8 FIELD_TYPE_LONG_DOUBLE_UNBOUNDED_SEQUENCE = 156
uint8 FIELD_TYPE_CHAR_UNBOUNDED_SEQUENCE = 157
uint8 FIELD_TYPE_WCHAR_UNBOUNDED_SEQUENCE = 158
uint8 FIELD_TYPE_BOOLEAN_UNBOUNDED_SEQUENCE = 159
uint8 FIELD_TYPE_BYTE_UNBOUNDED_SEQUENCE = 160
uint8 FIELD_TYPE_STRING_UNBOUNDED_SEQUENCE = 161
uint8 FIELD_TYPE_WSTRING_UNBOUNDED_SEQUENCE = 162
uint8 FIELD_TYPE_FIXED_STRING_UNBOUNDED_SEQUENCE = 163
uint8 FIELD_TYPE_FIXED_WSTRING_UNBOUNDED_SEQUENCE = 164
uint8 FIELD_TYPE_BOUNDED_STRING_UNBOUNDED_SEQUENCE = 165
uint8 FIELD_TYPE_BOUNDED_WSTRING_UNBOUNDED_SEQUENCE = 166

# Identifying number for the type of the field, using one of the above constants.
uint8 type_id 0

# Only used when the type is an array or a bounded sequence.
# In the case of an array, this is the fixed capacity of the array.
# In the case of a bounded sequence, this is the maximum capacity of the sequence.
# In all other cases this field is unused.
uint64 capacity

# Only used when the type is a fixed or bounded string/wstring, or a array/sequence of those.
# In the case of a fixed string/wstring, it is the fixed length of the string.
# In the case of a bounded string/wstring, it is the maximum capacity of the string.
# In the case of an array/sequence of fixed string/wstring, it is the fixed length of the strings.
# In the case of an array/sequence of bounded string/wstring, it is the maximum capacity of the strings.
# It is not currently possible to have different string capacities per element in the array/sequence.
uint64 string_capacity

# Only used when the type is a nested type or array/sequence of nested types.
# This is limited to 255 characters.
# TODO(wjwwood): this 255 character limit was chosen due to this being the limit
#   for DDSI-RTPS based middlewares, which is the most commonly used right now.
#   We lack a ROS 2 specific limit in our design documents, but we should update
#   this and/or link to the design doc when that is available.
string<=255 nested_type_name
================================================================================
MSG: type_description_interfaces/Field
# Represents a single field in a type.

# Name of the field.
string name
# Type of the field, including details about the type like length, nested name, etc.
FieldType type
# Literal default value of the field as a string, as it appeared in the original
# message description file, whether that be .msg/.srv/.action or .idl.
string default_value
================================================================================
MSG: type_description_interfaces/IndividualTypeDescription
# Represents a single type, without the types it references, if any.

# Name of the type.
# This is limited to 255 characters.
# TODO(wjwwood): this 255 character limit was chosen due to this being the limit
#   for DDSI-RTPS based middlewares, which is the most commonly used right now.
#   We lack a ROS 2 specific limit in our design documents, but we should update
#   this and/or link to the design doc when that is available.
string<=255 type_name
# Fields of the type.
Field[] fields`,
  "type_description_interfaces/msg/TypeSource": `# Represents the original source of a ROS 2 interface definition.

# ROS interface type name, in PACKAGE/NAMESPACE/TYPENAME format.
string type_name

# The type of the original source file, typically matching the file extension.
# Well-known encodings: "idl", "msg", "srv", "action", "dynamic", "implicit".
# "dynamic" specifies a type created programmatically by a user, thus having no source.
# "implicit" specifies a type created automatically as a subtype of a
# complex type (service or action) - such as the request message for a service.
# Implicit types will have no contents, the full source will be available on the parent srv/action.
string encoding

# Dumped contents of the interface definition source file.
# If \`encoding\` is "dynamic" or "implicit", this field will be empty.
string raw_file_contents`,
  "unique_identifier_msgs/msg/UUID": `# A universally unique identifier (UUID).
#
#  http://en.wikipedia.org/wiki/Universally_unique_identifier
#  http://tools.ietf.org/html/rfc4122.html

uint8[16] uuid`,
  "visualization_msgs/msg/ImageMarker": `int32 CIRCLE=0
int32 LINE_STRIP=1
int32 LINE_LIST=2
int32 POLYGON=3
int32 POINTS=4

int32 ADD=0
int32 REMOVE=1

std_msgs/Header header
# Namespace which is used with the id to form a unique id.
string ns
# Unique id within the namespace.
int32 id
# One of the above types, e.g. CIRCLE, LINE_STRIP, etc.
int32 type
# Either ADD or REMOVE.
int32 action
# Two-dimensional coordinate position, in pixel-coordinates.
geometry_msgs/Point position
# The scale of the object, e.g. the diameter for a CIRCLE.
float32 scale
# The outline color of the marker.
std_msgs/ColorRGBA outline_color
# Whether or not to fill in the shape with color.
uint8 filled
# Fill color; in the range: [0.0-1.0]
std_msgs/ColorRGBA fill_color
# How long the object should last before being automatically deleted.
# 0 indicates forever.
builtin_interfaces/Duration lifetime

# Coordinates in 2D in pixel coords. Used for LINE_STRIP, LINE_LIST, POINTS, etc.
geometry_msgs/Point[] points
# The color for each line, point, etc. in the points field.
std_msgs/ColorRGBA[] outline_colors
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: std_msgs/ColorRGBA
float32 r
float32 g
float32 b
float32 a
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "visualization_msgs/msg/InteractiveMarker": `# Time/frame info.
# If header.time is set to 0, the marker will be retransformed into
# its frame on each timestep. You will receive the pose feedback
# in the same frame.
# Otherwise, you might receive feedback in a different frame.
# For rviz, this will be the current 'fixed frame' set by the user.
std_msgs/Header header

# Initial pose. Also, defines the pivot point for rotations.
geometry_msgs/Pose pose

# Identifying string. Must be globally unique in
# the topic that this message is sent through.
string name

# Short description (< 40 characters).
string description

# Scale to be used for default controls (default=1).
float32 scale

# All menu and submenu entries associated with this marker.
MenuEntry[] menu_entries

# List of controls displayed for this marker.
InteractiveMarkerControl[] controls
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: sensor_msgs/CompressedImage
# This message contains a compressed image.

std_msgs/Header header # Header timestamp should be acquisition time of image
                             # Header frame_id should be optical frame of camera
                             # origin of frame should be optical center of cameara
                             # +x should point to the right in the image
                             # +y should point down in the image
                             # +z should point into to plane of the image

string format                # Specifies the format of the data
                             # Acceptable values differ by the image transport used:
                             # - compressed_image_transport:
                             #     ORIG_PIXFMT; CODEC compressed [COMPRESSED_PIXFMT]
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #   - CODEC is one of [jpeg, png, tiff]
                             #   - COMPRESSED_PIXFMT is only appended for color images
                             #     and is the pixel format used by the compression
                             #     algorithm. Valid values for jpeg encoding are:
                             #     [bgr8, rgb8]. Valid values for png encoding are:
                             #     [bgr8, rgb8, bgr16, rgb16].
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as bgr8 or mono8
                             #   jpeg image (depending on the number of channels).
                             # - compressed_depth_image_transport:
                             #     ORIG_PIXFMT; compressedDepth CODEC
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #     It is usually one of [16UC1, 32FC1].
                             #   - CODEC is one of [png, rvl]
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as png image.
                             # - Other image transports can store whatever values they
                             #   need for successful decoding of the image. Refer to
                             #   documentation of the other transports for details.

uint8[] data                 # Compressed image buffer
================================================================================
MSG: std_msgs/ColorRGBA
float32 r
float32 g
float32 b
float32 a
================================================================================
MSG: visualization_msgs/MeshFile
# Used to send raw mesh files.

# The filename is used for both debug purposes and to provide a file extension
# for whatever parser is used.
string filename

# This stores the raw text of the mesh file.
uint8[] data
================================================================================
MSG: visualization_msgs/UVCoordinate
# Location of the pixel as a ratio of the width of a 2D texture.
# Values should be in range: [0.0-1.0].
float32 u
float32 v
================================================================================
MSG: visualization_msgs/Marker
# See:
#  - http://www.ros.org/wiki/rviz/DisplayTypes/Marker
#  - http://www.ros.org/wiki/rviz/Tutorials/Markers%3A%20Basic%20Shapes
#
# for more information on using this message with rviz.

int32 ARROW=0
int32 CUBE=1
int32 SPHERE=2
int32 CYLINDER=3
int32 LINE_STRIP=4
int32 LINE_LIST=5
int32 CUBE_LIST=6
int32 SPHERE_LIST=7
int32 POINTS=8
int32 TEXT_VIEW_FACING=9
int32 MESH_RESOURCE=10
int32 TRIANGLE_LIST=11
int32 ARROW_STRIP=12

int32 ADD=0
int32 MODIFY=0
int32 DELETE=2
int32 DELETEALL=3

# Header for timestamp and frame id.
std_msgs/Header header
# Namespace in which to place the object.
# Used in conjunction with id to create a unique name for the object.
string ns
# Object ID used in conjunction with the namespace for manipulating and deleting the object later.
int32 id
# Type of object.
int32 type
# Action to take; one of:
#  - 0 add/modify an object
#  - 1 (deprecated)
#  - 2 deletes an object (with the given ns and id)
#  - 3 deletes all objects (or those with the given ns if any)
int32 action
# Pose of the object with respect the frame_id specified in the header.
geometry_msgs/Pose pose
# Scale of the object; 1,1,1 means default (usually 1 meter square).
geometry_msgs/Vector3 scale
# Color of the object; in the range: [0.0-1.0]
std_msgs/ColorRGBA color
# How long the object should last before being automatically deleted.
# 0 indicates forever.
builtin_interfaces/Duration lifetime
# If this marker should be frame-locked, i.e. retransformed into its frame every timestep.
bool frame_locked

# Only used if the type specified has some use for them (eg. POINTS, LINE_STRIP, ARROW_STRIP, etc.)
geometry_msgs/Point[] points
# Only used if the type specified has some use for them (eg. POINTS, LINE_STRIP, etc.)
# The number of colors provided must either be 0 or equal to the number of points provided.
# NOTE: alpha is not yet used
std_msgs/ColorRGBA[] colors

# Texture resource is a special URI that can either reference a texture file in
# a format acceptable to (resource retriever)[https://docs.ros.org/en/rolling/p/resource_retriever/]
# or an embedded texture via a string matching the format:
#   "embedded://texture_name"
string texture_resource
# An image to be loaded into the rendering engine as the texture for this marker.
# This will be used iff texture_resource is set to embedded.
sensor_msgs/CompressedImage texture
# Location of each vertex within the texture; in the range: [0.0-1.0]
UVCoordinate[] uv_coordinates

# Only used for text markers
string text

# Only used for MESH_RESOURCE markers.
# Similar to texture_resource, mesh_resource uses resource retriever to load a mesh.
# Optionally, a mesh file can be sent in-message via the mesh_file field. If doing so,
# use the following format for mesh_resource:
#   "embedded://mesh_name"
string mesh_resource
MeshFile mesh_file
bool mesh_use_embedded_materials
================================================================================
MSG: visualization_msgs/InteractiveMarkerControl
# Represents a control that is to be displayed together with an interactive marker

# Identifying string for this control.
# You need to assign a unique value to this to receive feedback from the GUI
# on what actions the user performs on this control (e.g. a button click).
string name


# Defines the local coordinate frame (relative to the pose of the parent
# interactive marker) in which is being rotated and translated.
# Default: Identity
geometry_msgs/Quaternion orientation


# Orientation mode: controls how orientation changes.
# INHERIT: Follow orientation of interactive marker
# FIXED: Keep orientation fixed at initial state
# VIEW_FACING: Align y-z plane with screen (x: forward, y:left, z:up).
uint8 INHERIT = 0
uint8 FIXED = 1
uint8 VIEW_FACING = 2

uint8 orientation_mode

# Interaction mode for this control
#
# NONE: This control is only meant for visualization; no context menu.
# MENU: Like NONE, but right-click menu is active.
# BUTTON: Element can be left-clicked.
# MOVE_AXIS: Translate along local x-axis.
# MOVE_PLANE: Translate in local y-z plane.
# ROTATE_AXIS: Rotate around local x-axis.
# MOVE_ROTATE: Combines MOVE_PLANE and ROTATE_AXIS.
uint8 NONE = 0
uint8 MENU = 1
uint8 BUTTON = 2
uint8 MOVE_AXIS = 3
uint8 MOVE_PLANE = 4
uint8 ROTATE_AXIS = 5
uint8 MOVE_ROTATE = 6
# "3D" interaction modes work with the mouse+SHIFT+CTRL or with 3D cursors.
# MOVE_3D: Translate freely in 3D space.
# ROTATE_3D: Rotate freely in 3D space about the origin of parent frame.
# MOVE_ROTATE_3D: Full 6-DOF freedom of translation and rotation about the cursor origin.
uint8 MOVE_3D = 7
uint8 ROTATE_3D = 8
uint8 MOVE_ROTATE_3D = 9

uint8 interaction_mode


# If true, the contained markers will also be visible
# when the gui is not in interactive mode.
bool always_visible


# Markers to be displayed as custom visual representation.
# Leave this empty to use the default control handles.
#
# Note:
# - The markers can be defined in an arbitrary coordinate frame,
#   but will be transformed into the local frame of the interactive marker.
# - If the header of a marker is empty, its pose will be interpreted as
#   relative to the pose of the parent interactive marker.
Marker[] markers


# In VIEW_FACING mode, set this to true if you don't want the markers
# to be aligned with the camera view point. The markers will show up
# as in INHERIT mode.
bool independent_marker_orientation


# Short description (< 40 characters) of what this control does,
# e.g. "Move the robot".
# Default: A generic description based on the interaction mode
string description
================================================================================
MSG: visualization_msgs/MenuEntry
# MenuEntry message.
#
# Each InteractiveMarker message has an array of MenuEntry messages.
# A collection of MenuEntries together describe a
# menu/submenu/subsubmenu/etc tree, though they are stored in a flat
# array.  The tree structure is represented by giving each menu entry
# an ID number and a "parent_id" field.  Top-level entries are the
# ones with parent_id = 0.  Menu entries are ordered within their
# level the same way they are ordered in the containing array.  Parent
# entries must appear before their children.
#
# Example:
# - id = 3
#   parent_id = 0
#   title = "fun"
# - id = 2
#   parent_id = 0
#   title = "robot"
# - id = 4
#   parent_id = 2
#   title = "pr2"
# - id = 5
#   parent_id = 2
#   title = "turtle"
#
# Gives a menu tree like this:
#  - fun
#  - robot
#    - pr2
#    - turtle

# ID is a number for each menu entry.  Must be unique within the
# control, and should never be 0.
uint32 id

# ID of the parent of this menu entry, if it is a submenu.  If this
# menu entry is a top-level entry, set parent_id to 0.
uint32 parent_id

# menu / entry title
string title

# Arguments to command indicated by command_type (below)
string command

# Command_type stores the type of response desired when this menu
# entry is clicked.
# FEEDBACK: send an InteractiveMarkerFeedback message with menu_entry_id set to this entry's id.
# ROSRUN: execute "rosrun" with arguments given in the command field (above).
# ROSLAUNCH: execute "roslaunch" with arguments given in the command field (above).
uint8 FEEDBACK=0
uint8 ROSRUN=1
uint8 ROSLAUNCH=2
uint8 command_type`,
  "visualization_msgs/msg/InteractiveMarkerControl": `# Represents a control that is to be displayed together with an interactive marker

# Identifying string for this control.
# You need to assign a unique value to this to receive feedback from the GUI
# on what actions the user performs on this control (e.g. a button click).
string name


# Defines the local coordinate frame (relative to the pose of the parent
# interactive marker) in which is being rotated and translated.
# Default: Identity
geometry_msgs/Quaternion orientation


# Orientation mode: controls how orientation changes.
# INHERIT: Follow orientation of interactive marker
# FIXED: Keep orientation fixed at initial state
# VIEW_FACING: Align y-z plane with screen (x: forward, y:left, z:up).
uint8 INHERIT = 0
uint8 FIXED = 1
uint8 VIEW_FACING = 2

uint8 orientation_mode

# Interaction mode for this control
#
# NONE: This control is only meant for visualization; no context menu.
# MENU: Like NONE, but right-click menu is active.
# BUTTON: Element can be left-clicked.
# MOVE_AXIS: Translate along local x-axis.
# MOVE_PLANE: Translate in local y-z plane.
# ROTATE_AXIS: Rotate around local x-axis.
# MOVE_ROTATE: Combines MOVE_PLANE and ROTATE_AXIS.
uint8 NONE = 0
uint8 MENU = 1
uint8 BUTTON = 2
uint8 MOVE_AXIS = 3
uint8 MOVE_PLANE = 4
uint8 ROTATE_AXIS = 5
uint8 MOVE_ROTATE = 6
# "3D" interaction modes work with the mouse+SHIFT+CTRL or with 3D cursors.
# MOVE_3D: Translate freely in 3D space.
# ROTATE_3D: Rotate freely in 3D space about the origin of parent frame.
# MOVE_ROTATE_3D: Full 6-DOF freedom of translation and rotation about the cursor origin.
uint8 MOVE_3D = 7
uint8 ROTATE_3D = 8
uint8 MOVE_ROTATE_3D = 9

uint8 interaction_mode


# If true, the contained markers will also be visible
# when the gui is not in interactive mode.
bool always_visible


# Markers to be displayed as custom visual representation.
# Leave this empty to use the default control handles.
#
# Note:
# - The markers can be defined in an arbitrary coordinate frame,
#   but will be transformed into the local frame of the interactive marker.
# - If the header of a marker is empty, its pose will be interpreted as
#   relative to the pose of the parent interactive marker.
Marker[] markers


# In VIEW_FACING mode, set this to true if you don't want the markers
# to be aligned with the camera view point. The markers will show up
# as in INHERIT mode.
bool independent_marker_orientation


# Short description (< 40 characters) of what this control does,
# e.g. "Move the robot".
# Default: A generic description based on the interaction mode
string description
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: sensor_msgs/CompressedImage
# This message contains a compressed image.

std_msgs/Header header # Header timestamp should be acquisition time of image
                             # Header frame_id should be optical frame of camera
                             # origin of frame should be optical center of cameara
                             # +x should point to the right in the image
                             # +y should point down in the image
                             # +z should point into to plane of the image

string format                # Specifies the format of the data
                             # Acceptable values differ by the image transport used:
                             # - compressed_image_transport:
                             #     ORIG_PIXFMT; CODEC compressed [COMPRESSED_PIXFMT]
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #   - CODEC is one of [jpeg, png, tiff]
                             #   - COMPRESSED_PIXFMT is only appended for color images
                             #     and is the pixel format used by the compression
                             #     algorithm. Valid values for jpeg encoding are:
                             #     [bgr8, rgb8]. Valid values for png encoding are:
                             #     [bgr8, rgb8, bgr16, rgb16].
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as bgr8 or mono8
                             #   jpeg image (depending on the number of channels).
                             # - compressed_depth_image_transport:
                             #     ORIG_PIXFMT; compressedDepth CODEC
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #     It is usually one of [16UC1, 32FC1].
                             #   - CODEC is one of [png, rvl]
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as png image.
                             # - Other image transports can store whatever values they
                             #   need for successful decoding of the image. Refer to
                             #   documentation of the other transports for details.

uint8[] data                 # Compressed image buffer
================================================================================
MSG: std_msgs/ColorRGBA
float32 r
float32 g
float32 b
float32 a
================================================================================
MSG: visualization_msgs/MeshFile
# Used to send raw mesh files.

# The filename is used for both debug purposes and to provide a file extension
# for whatever parser is used.
string filename

# This stores the raw text of the mesh file.
uint8[] data
================================================================================
MSG: visualization_msgs/UVCoordinate
# Location of the pixel as a ratio of the width of a 2D texture.
# Values should be in range: [0.0-1.0].
float32 u
float32 v
================================================================================
MSG: visualization_msgs/Marker
# See:
#  - http://www.ros.org/wiki/rviz/DisplayTypes/Marker
#  - http://www.ros.org/wiki/rviz/Tutorials/Markers%3A%20Basic%20Shapes
#
# for more information on using this message with rviz.

int32 ARROW=0
int32 CUBE=1
int32 SPHERE=2
int32 CYLINDER=3
int32 LINE_STRIP=4
int32 LINE_LIST=5
int32 CUBE_LIST=6
int32 SPHERE_LIST=7
int32 POINTS=8
int32 TEXT_VIEW_FACING=9
int32 MESH_RESOURCE=10
int32 TRIANGLE_LIST=11
int32 ARROW_STRIP=12

int32 ADD=0
int32 MODIFY=0
int32 DELETE=2
int32 DELETEALL=3

# Header for timestamp and frame id.
std_msgs/Header header
# Namespace in which to place the object.
# Used in conjunction with id to create a unique name for the object.
string ns
# Object ID used in conjunction with the namespace for manipulating and deleting the object later.
int32 id
# Type of object.
int32 type
# Action to take; one of:
#  - 0 add/modify an object
#  - 1 (deprecated)
#  - 2 deletes an object (with the given ns and id)
#  - 3 deletes all objects (or those with the given ns if any)
int32 action
# Pose of the object with respect the frame_id specified in the header.
geometry_msgs/Pose pose
# Scale of the object; 1,1,1 means default (usually 1 meter square).
geometry_msgs/Vector3 scale
# Color of the object; in the range: [0.0-1.0]
std_msgs/ColorRGBA color
# How long the object should last before being automatically deleted.
# 0 indicates forever.
builtin_interfaces/Duration lifetime
# If this marker should be frame-locked, i.e. retransformed into its frame every timestep.
bool frame_locked

# Only used if the type specified has some use for them (eg. POINTS, LINE_STRIP, ARROW_STRIP, etc.)
geometry_msgs/Point[] points
# Only used if the type specified has some use for them (eg. POINTS, LINE_STRIP, etc.)
# The number of colors provided must either be 0 or equal to the number of points provided.
# NOTE: alpha is not yet used
std_msgs/ColorRGBA[] colors

# Texture resource is a special URI that can either reference a texture file in
# a format acceptable to (resource retriever)[https://docs.ros.org/en/rolling/p/resource_retriever/]
# or an embedded texture via a string matching the format:
#   "embedded://texture_name"
string texture_resource
# An image to be loaded into the rendering engine as the texture for this marker.
# This will be used iff texture_resource is set to embedded.
sensor_msgs/CompressedImage texture
# Location of each vertex within the texture; in the range: [0.0-1.0]
UVCoordinate[] uv_coordinates

# Only used for text markers
string text

# Only used for MESH_RESOURCE markers.
# Similar to texture_resource, mesh_resource uses resource retriever to load a mesh.
# Optionally, a mesh file can be sent in-message via the mesh_file field. If doing so,
# use the following format for mesh_resource:
#   "embedded://mesh_name"
string mesh_resource
MeshFile mesh_file
bool mesh_use_embedded_materials`,
  "visualization_msgs/msg/InteractiveMarkerFeedback": `# Time/frame info.
std_msgs/Header header

# Identifying string. Must be unique in the topic namespace.
string client_id

# Feedback message sent back from the GUI, e.g.
# when the status of an interactive marker was modified by the user.

# Specifies which interactive marker and control this message refers to
string marker_name
string control_name

# Type of the event
# KEEP_ALIVE: sent while dragging to keep up control of the marker
# MENU_SELECT: a menu entry has been selected
# BUTTON_CLICK: a button control has been clicked
# POSE_UPDATE: the pose has been changed using one of the controls
uint8 KEEP_ALIVE = 0
uint8 POSE_UPDATE = 1
uint8 MENU_SELECT = 2
uint8 BUTTON_CLICK = 3

uint8 MOUSE_DOWN = 4
uint8 MOUSE_UP = 5

uint8 event_type

# Current pose of the marker
# Note: Has to be valid for all feedback types.
geometry_msgs/Pose pose

# Contains the ID of the selected menu entry
# Only valid for MENU_SELECT events.
uint32 menu_entry_id

# If event_type is BUTTON_CLICK, MOUSE_DOWN, or MOUSE_UP, mouse_point
# may contain the 3 dimensional position of the event on the
# control.  If it does, mouse_point_valid will be true.  mouse_point
# will be relative to the frame listed in the header.
geometry_msgs/Point mouse_point
bool mouse_point_valid
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "visualization_msgs/msg/InteractiveMarkerInit": `# Identifying string. Must be unique in the topic namespace
# that this server works on.
string server_id

# Sequence number.
# The client will use this to detect if it has missed a subsequent
# update.  Every update message will have the same sequence number as
# an init message.  Clients will likely want to unsubscribe from the
# init topic after a successful initialization to avoid receiving
# duplicate data.
uint64 seq_num

# All markers.
InteractiveMarker[] markers
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: sensor_msgs/CompressedImage
# This message contains a compressed image.

std_msgs/Header header # Header timestamp should be acquisition time of image
                             # Header frame_id should be optical frame of camera
                             # origin of frame should be optical center of cameara
                             # +x should point to the right in the image
                             # +y should point down in the image
                             # +z should point into to plane of the image

string format                # Specifies the format of the data
                             # Acceptable values differ by the image transport used:
                             # - compressed_image_transport:
                             #     ORIG_PIXFMT; CODEC compressed [COMPRESSED_PIXFMT]
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #   - CODEC is one of [jpeg, png, tiff]
                             #   - COMPRESSED_PIXFMT is only appended for color images
                             #     and is the pixel format used by the compression
                             #     algorithm. Valid values for jpeg encoding are:
                             #     [bgr8, rgb8]. Valid values for png encoding are:
                             #     [bgr8, rgb8, bgr16, rgb16].
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as bgr8 or mono8
                             #   jpeg image (depending on the number of channels).
                             # - compressed_depth_image_transport:
                             #     ORIG_PIXFMT; compressedDepth CODEC
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #     It is usually one of [16UC1, 32FC1].
                             #   - CODEC is one of [png, rvl]
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as png image.
                             # - Other image transports can store whatever values they
                             #   need for successful decoding of the image. Refer to
                             #   documentation of the other transports for details.

uint8[] data                 # Compressed image buffer
================================================================================
MSG: std_msgs/ColorRGBA
float32 r
float32 g
float32 b
float32 a
================================================================================
MSG: visualization_msgs/MeshFile
# Used to send raw mesh files.

# The filename is used for both debug purposes and to provide a file extension
# for whatever parser is used.
string filename

# This stores the raw text of the mesh file.
uint8[] data
================================================================================
MSG: visualization_msgs/UVCoordinate
# Location of the pixel as a ratio of the width of a 2D texture.
# Values should be in range: [0.0-1.0].
float32 u
float32 v
================================================================================
MSG: visualization_msgs/Marker
# See:
#  - http://www.ros.org/wiki/rviz/DisplayTypes/Marker
#  - http://www.ros.org/wiki/rviz/Tutorials/Markers%3A%20Basic%20Shapes
#
# for more information on using this message with rviz.

int32 ARROW=0
int32 CUBE=1
int32 SPHERE=2
int32 CYLINDER=3
int32 LINE_STRIP=4
int32 LINE_LIST=5
int32 CUBE_LIST=6
int32 SPHERE_LIST=7
int32 POINTS=8
int32 TEXT_VIEW_FACING=9
int32 MESH_RESOURCE=10
int32 TRIANGLE_LIST=11
int32 ARROW_STRIP=12

int32 ADD=0
int32 MODIFY=0
int32 DELETE=2
int32 DELETEALL=3

# Header for timestamp and frame id.
std_msgs/Header header
# Namespace in which to place the object.
# Used in conjunction with id to create a unique name for the object.
string ns
# Object ID used in conjunction with the namespace for manipulating and deleting the object later.
int32 id
# Type of object.
int32 type
# Action to take; one of:
#  - 0 add/modify an object
#  - 1 (deprecated)
#  - 2 deletes an object (with the given ns and id)
#  - 3 deletes all objects (or those with the given ns if any)
int32 action
# Pose of the object with respect the frame_id specified in the header.
geometry_msgs/Pose pose
# Scale of the object; 1,1,1 means default (usually 1 meter square).
geometry_msgs/Vector3 scale
# Color of the object; in the range: [0.0-1.0]
std_msgs/ColorRGBA color
# How long the object should last before being automatically deleted.
# 0 indicates forever.
builtin_interfaces/Duration lifetime
# If this marker should be frame-locked, i.e. retransformed into its frame every timestep.
bool frame_locked

# Only used if the type specified has some use for them (eg. POINTS, LINE_STRIP, ARROW_STRIP, etc.)
geometry_msgs/Point[] points
# Only used if the type specified has some use for them (eg. POINTS, LINE_STRIP, etc.)
# The number of colors provided must either be 0 or equal to the number of points provided.
# NOTE: alpha is not yet used
std_msgs/ColorRGBA[] colors

# Texture resource is a special URI that can either reference a texture file in
# a format acceptable to (resource retriever)[https://docs.ros.org/en/rolling/p/resource_retriever/]
# or an embedded texture via a string matching the format:
#   "embedded://texture_name"
string texture_resource
# An image to be loaded into the rendering engine as the texture for this marker.
# This will be used iff texture_resource is set to embedded.
sensor_msgs/CompressedImage texture
# Location of each vertex within the texture; in the range: [0.0-1.0]
UVCoordinate[] uv_coordinates

# Only used for text markers
string text

# Only used for MESH_RESOURCE markers.
# Similar to texture_resource, mesh_resource uses resource retriever to load a mesh.
# Optionally, a mesh file can be sent in-message via the mesh_file field. If doing so,
# use the following format for mesh_resource:
#   "embedded://mesh_name"
string mesh_resource
MeshFile mesh_file
bool mesh_use_embedded_materials
================================================================================
MSG: visualization_msgs/InteractiveMarkerControl
# Represents a control that is to be displayed together with an interactive marker

# Identifying string for this control.
# You need to assign a unique value to this to receive feedback from the GUI
# on what actions the user performs on this control (e.g. a button click).
string name


# Defines the local coordinate frame (relative to the pose of the parent
# interactive marker) in which is being rotated and translated.
# Default: Identity
geometry_msgs/Quaternion orientation


# Orientation mode: controls how orientation changes.
# INHERIT: Follow orientation of interactive marker
# FIXED: Keep orientation fixed at initial state
# VIEW_FACING: Align y-z plane with screen (x: forward, y:left, z:up).
uint8 INHERIT = 0
uint8 FIXED = 1
uint8 VIEW_FACING = 2

uint8 orientation_mode

# Interaction mode for this control
#
# NONE: This control is only meant for visualization; no context menu.
# MENU: Like NONE, but right-click menu is active.
# BUTTON: Element can be left-clicked.
# MOVE_AXIS: Translate along local x-axis.
# MOVE_PLANE: Translate in local y-z plane.
# ROTATE_AXIS: Rotate around local x-axis.
# MOVE_ROTATE: Combines MOVE_PLANE and ROTATE_AXIS.
uint8 NONE = 0
uint8 MENU = 1
uint8 BUTTON = 2
uint8 MOVE_AXIS = 3
uint8 MOVE_PLANE = 4
uint8 ROTATE_AXIS = 5
uint8 MOVE_ROTATE = 6
# "3D" interaction modes work with the mouse+SHIFT+CTRL or with 3D cursors.
# MOVE_3D: Translate freely in 3D space.
# ROTATE_3D: Rotate freely in 3D space about the origin of parent frame.
# MOVE_ROTATE_3D: Full 6-DOF freedom of translation and rotation about the cursor origin.
uint8 MOVE_3D = 7
uint8 ROTATE_3D = 8
uint8 MOVE_ROTATE_3D = 9

uint8 interaction_mode


# If true, the contained markers will also be visible
# when the gui is not in interactive mode.
bool always_visible


# Markers to be displayed as custom visual representation.
# Leave this empty to use the default control handles.
#
# Note:
# - The markers can be defined in an arbitrary coordinate frame,
#   but will be transformed into the local frame of the interactive marker.
# - If the header of a marker is empty, its pose will be interpreted as
#   relative to the pose of the parent interactive marker.
Marker[] markers


# In VIEW_FACING mode, set this to true if you don't want the markers
# to be aligned with the camera view point. The markers will show up
# as in INHERIT mode.
bool independent_marker_orientation


# Short description (< 40 characters) of what this control does,
# e.g. "Move the robot".
# Default: A generic description based on the interaction mode
string description
================================================================================
MSG: visualization_msgs/MenuEntry
# MenuEntry message.
#
# Each InteractiveMarker message has an array of MenuEntry messages.
# A collection of MenuEntries together describe a
# menu/submenu/subsubmenu/etc tree, though they are stored in a flat
# array.  The tree structure is represented by giving each menu entry
# an ID number and a "parent_id" field.  Top-level entries are the
# ones with parent_id = 0.  Menu entries are ordered within their
# level the same way they are ordered in the containing array.  Parent
# entries must appear before their children.
#
# Example:
# - id = 3
#   parent_id = 0
#   title = "fun"
# - id = 2
#   parent_id = 0
#   title = "robot"
# - id = 4
#   parent_id = 2
#   title = "pr2"
# - id = 5
#   parent_id = 2
#   title = "turtle"
#
# Gives a menu tree like this:
#  - fun
#  - robot
#    - pr2
#    - turtle

# ID is a number for each menu entry.  Must be unique within the
# control, and should never be 0.
uint32 id

# ID of the parent of this menu entry, if it is a submenu.  If this
# menu entry is a top-level entry, set parent_id to 0.
uint32 parent_id

# menu / entry title
string title

# Arguments to command indicated by command_type (below)
string command

# Command_type stores the type of response desired when this menu
# entry is clicked.
# FEEDBACK: send an InteractiveMarkerFeedback message with menu_entry_id set to this entry's id.
# ROSRUN: execute "rosrun" with arguments given in the command field (above).
# ROSLAUNCH: execute "roslaunch" with arguments given in the command field (above).
uint8 FEEDBACK=0
uint8 ROSRUN=1
uint8 ROSLAUNCH=2
uint8 command_type
================================================================================
MSG: visualization_msgs/InteractiveMarker
# Time/frame info.
# If header.time is set to 0, the marker will be retransformed into
# its frame on each timestep. You will receive the pose feedback
# in the same frame.
# Otherwise, you might receive feedback in a different frame.
# For rviz, this will be the current 'fixed frame' set by the user.
std_msgs/Header header

# Initial pose. Also, defines the pivot point for rotations.
geometry_msgs/Pose pose

# Identifying string. Must be globally unique in
# the topic that this message is sent through.
string name

# Short description (< 40 characters).
string description

# Scale to be used for default controls (default=1).
float32 scale

# All menu and submenu entries associated with this marker.
MenuEntry[] menu_entries

# List of controls displayed for this marker.
InteractiveMarkerControl[] controls`,
  "visualization_msgs/msg/InteractiveMarkerPose": `
# Time/frame info.
std_msgs/Header header

# Initial pose. Also, defines the pivot point for rotations.
geometry_msgs/Pose pose

# Identifying string. Must be globally unique in
# the topic that this message is sent through.
string name
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id`,
  "visualization_msgs/msg/InteractiveMarkerUpdate": `
# Identifying string. Must be unique in the topic namespace
# that this server works on.
string server_id

# Sequence number.
# The client will use this to detect if it has missed an update.
uint64 seq_num

# Type holds the purpose of this message.  It must be one of UPDATE or KEEP_ALIVE.
# UPDATE: Incremental update to previous state.
#         The sequence number must be 1 higher than for
#         the previous update.
# KEEP_ALIVE: Indicates the that the server is still living.
#             The sequence number does not increase.
#             No payload data should be filled out (markers, poses, or erases).
uint8 KEEP_ALIVE = 0
uint8 UPDATE = 1

uint8 type

# Note: No guarantees on the order of processing.
#       Contents must be kept consistent by sender.

# Markers to be added or updated
InteractiveMarker[] markers

# Poses of markers that should be moved
InteractiveMarkerPose[] poses

# Names of markers to be erased
string[] erases
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: sensor_msgs/CompressedImage
# This message contains a compressed image.

std_msgs/Header header # Header timestamp should be acquisition time of image
                             # Header frame_id should be optical frame of camera
                             # origin of frame should be optical center of cameara
                             # +x should point to the right in the image
                             # +y should point down in the image
                             # +z should point into to plane of the image

string format                # Specifies the format of the data
                             # Acceptable values differ by the image transport used:
                             # - compressed_image_transport:
                             #     ORIG_PIXFMT; CODEC compressed [COMPRESSED_PIXFMT]
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #   - CODEC is one of [jpeg, png, tiff]
                             #   - COMPRESSED_PIXFMT is only appended for color images
                             #     and is the pixel format used by the compression
                             #     algorithm. Valid values for jpeg encoding are:
                             #     [bgr8, rgb8]. Valid values for png encoding are:
                             #     [bgr8, rgb8, bgr16, rgb16].
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as bgr8 or mono8
                             #   jpeg image (depending on the number of channels).
                             # - compressed_depth_image_transport:
                             #     ORIG_PIXFMT; compressedDepth CODEC
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #     It is usually one of [16UC1, 32FC1].
                             #   - CODEC is one of [png, rvl]
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as png image.
                             # - Other image transports can store whatever values they
                             #   need for successful decoding of the image. Refer to
                             #   documentation of the other transports for details.

uint8[] data                 # Compressed image buffer
================================================================================
MSG: std_msgs/ColorRGBA
float32 r
float32 g
float32 b
float32 a
================================================================================
MSG: visualization_msgs/MeshFile
# Used to send raw mesh files.

# The filename is used for both debug purposes and to provide a file extension
# for whatever parser is used.
string filename

# This stores the raw text of the mesh file.
uint8[] data
================================================================================
MSG: visualization_msgs/UVCoordinate
# Location of the pixel as a ratio of the width of a 2D texture.
# Values should be in range: [0.0-1.0].
float32 u
float32 v
================================================================================
MSG: visualization_msgs/Marker
# See:
#  - http://www.ros.org/wiki/rviz/DisplayTypes/Marker
#  - http://www.ros.org/wiki/rviz/Tutorials/Markers%3A%20Basic%20Shapes
#
# for more information on using this message with rviz.

int32 ARROW=0
int32 CUBE=1
int32 SPHERE=2
int32 CYLINDER=3
int32 LINE_STRIP=4
int32 LINE_LIST=5
int32 CUBE_LIST=6
int32 SPHERE_LIST=7
int32 POINTS=8
int32 TEXT_VIEW_FACING=9
int32 MESH_RESOURCE=10
int32 TRIANGLE_LIST=11
int32 ARROW_STRIP=12

int32 ADD=0
int32 MODIFY=0
int32 DELETE=2
int32 DELETEALL=3

# Header for timestamp and frame id.
std_msgs/Header header
# Namespace in which to place the object.
# Used in conjunction with id to create a unique name for the object.
string ns
# Object ID used in conjunction with the namespace for manipulating and deleting the object later.
int32 id
# Type of object.
int32 type
# Action to take; one of:
#  - 0 add/modify an object
#  - 1 (deprecated)
#  - 2 deletes an object (with the given ns and id)
#  - 3 deletes all objects (or those with the given ns if any)
int32 action
# Pose of the object with respect the frame_id specified in the header.
geometry_msgs/Pose pose
# Scale of the object; 1,1,1 means default (usually 1 meter square).
geometry_msgs/Vector3 scale
# Color of the object; in the range: [0.0-1.0]
std_msgs/ColorRGBA color
# How long the object should last before being automatically deleted.
# 0 indicates forever.
builtin_interfaces/Duration lifetime
# If this marker should be frame-locked, i.e. retransformed into its frame every timestep.
bool frame_locked

# Only used if the type specified has some use for them (eg. POINTS, LINE_STRIP, ARROW_STRIP, etc.)
geometry_msgs/Point[] points
# Only used if the type specified has some use for them (eg. POINTS, LINE_STRIP, etc.)
# The number of colors provided must either be 0 or equal to the number of points provided.
# NOTE: alpha is not yet used
std_msgs/ColorRGBA[] colors

# Texture resource is a special URI that can either reference a texture file in
# a format acceptable to (resource retriever)[https://docs.ros.org/en/rolling/p/resource_retriever/]
# or an embedded texture via a string matching the format:
#   "embedded://texture_name"
string texture_resource
# An image to be loaded into the rendering engine as the texture for this marker.
# This will be used iff texture_resource is set to embedded.
sensor_msgs/CompressedImage texture
# Location of each vertex within the texture; in the range: [0.0-1.0]
UVCoordinate[] uv_coordinates

# Only used for text markers
string text

# Only used for MESH_RESOURCE markers.
# Similar to texture_resource, mesh_resource uses resource retriever to load a mesh.
# Optionally, a mesh file can be sent in-message via the mesh_file field. If doing so,
# use the following format for mesh_resource:
#   "embedded://mesh_name"
string mesh_resource
MeshFile mesh_file
bool mesh_use_embedded_materials
================================================================================
MSG: visualization_msgs/InteractiveMarkerControl
# Represents a control that is to be displayed together with an interactive marker

# Identifying string for this control.
# You need to assign a unique value to this to receive feedback from the GUI
# on what actions the user performs on this control (e.g. a button click).
string name


# Defines the local coordinate frame (relative to the pose of the parent
# interactive marker) in which is being rotated and translated.
# Default: Identity
geometry_msgs/Quaternion orientation


# Orientation mode: controls how orientation changes.
# INHERIT: Follow orientation of interactive marker
# FIXED: Keep orientation fixed at initial state
# VIEW_FACING: Align y-z plane with screen (x: forward, y:left, z:up).
uint8 INHERIT = 0
uint8 FIXED = 1
uint8 VIEW_FACING = 2

uint8 orientation_mode

# Interaction mode for this control
#
# NONE: This control is only meant for visualization; no context menu.
# MENU: Like NONE, but right-click menu is active.
# BUTTON: Element can be left-clicked.
# MOVE_AXIS: Translate along local x-axis.
# MOVE_PLANE: Translate in local y-z plane.
# ROTATE_AXIS: Rotate around local x-axis.
# MOVE_ROTATE: Combines MOVE_PLANE and ROTATE_AXIS.
uint8 NONE = 0
uint8 MENU = 1
uint8 BUTTON = 2
uint8 MOVE_AXIS = 3
uint8 MOVE_PLANE = 4
uint8 ROTATE_AXIS = 5
uint8 MOVE_ROTATE = 6
# "3D" interaction modes work with the mouse+SHIFT+CTRL or with 3D cursors.
# MOVE_3D: Translate freely in 3D space.
# ROTATE_3D: Rotate freely in 3D space about the origin of parent frame.
# MOVE_ROTATE_3D: Full 6-DOF freedom of translation and rotation about the cursor origin.
uint8 MOVE_3D = 7
uint8 ROTATE_3D = 8
uint8 MOVE_ROTATE_3D = 9

uint8 interaction_mode


# If true, the contained markers will also be visible
# when the gui is not in interactive mode.
bool always_visible


# Markers to be displayed as custom visual representation.
# Leave this empty to use the default control handles.
#
# Note:
# - The markers can be defined in an arbitrary coordinate frame,
#   but will be transformed into the local frame of the interactive marker.
# - If the header of a marker is empty, its pose will be interpreted as
#   relative to the pose of the parent interactive marker.
Marker[] markers


# In VIEW_FACING mode, set this to true if you don't want the markers
# to be aligned with the camera view point. The markers will show up
# as in INHERIT mode.
bool independent_marker_orientation


# Short description (< 40 characters) of what this control does,
# e.g. "Move the robot".
# Default: A generic description based on the interaction mode
string description
================================================================================
MSG: visualization_msgs/MenuEntry
# MenuEntry message.
#
# Each InteractiveMarker message has an array of MenuEntry messages.
# A collection of MenuEntries together describe a
# menu/submenu/subsubmenu/etc tree, though they are stored in a flat
# array.  The tree structure is represented by giving each menu entry
# an ID number and a "parent_id" field.  Top-level entries are the
# ones with parent_id = 0.  Menu entries are ordered within their
# level the same way they are ordered in the containing array.  Parent
# entries must appear before their children.
#
# Example:
# - id = 3
#   parent_id = 0
#   title = "fun"
# - id = 2
#   parent_id = 0
#   title = "robot"
# - id = 4
#   parent_id = 2
#   title = "pr2"
# - id = 5
#   parent_id = 2
#   title = "turtle"
#
# Gives a menu tree like this:
#  - fun
#  - robot
#    - pr2
#    - turtle

# ID is a number for each menu entry.  Must be unique within the
# control, and should never be 0.
uint32 id

# ID of the parent of this menu entry, if it is a submenu.  If this
# menu entry is a top-level entry, set parent_id to 0.
uint32 parent_id

# menu / entry title
string title

# Arguments to command indicated by command_type (below)
string command

# Command_type stores the type of response desired when this menu
# entry is clicked.
# FEEDBACK: send an InteractiveMarkerFeedback message with menu_entry_id set to this entry's id.
# ROSRUN: execute "rosrun" with arguments given in the command field (above).
# ROSLAUNCH: execute "roslaunch" with arguments given in the command field (above).
uint8 FEEDBACK=0
uint8 ROSRUN=1
uint8 ROSLAUNCH=2
uint8 command_type
================================================================================
MSG: visualization_msgs/InteractiveMarker
# Time/frame info.
# If header.time is set to 0, the marker will be retransformed into
# its frame on each timestep. You will receive the pose feedback
# in the same frame.
# Otherwise, you might receive feedback in a different frame.
# For rviz, this will be the current 'fixed frame' set by the user.
std_msgs/Header header

# Initial pose. Also, defines the pivot point for rotations.
geometry_msgs/Pose pose

# Identifying string. Must be globally unique in
# the topic that this message is sent through.
string name

# Short description (< 40 characters).
string description

# Scale to be used for default controls (default=1).
float32 scale

# All menu and submenu entries associated with this marker.
MenuEntry[] menu_entries

# List of controls displayed for this marker.
InteractiveMarkerControl[] controls
================================================================================
MSG: visualization_msgs/InteractiveMarkerPose

# Time/frame info.
std_msgs/Header header

# Initial pose. Also, defines the pivot point for rotations.
geometry_msgs/Pose pose

# Identifying string. Must be globally unique in
# the topic that this message is sent through.
string name`,
  "visualization_msgs/msg/Marker": `# See:
#  - http://www.ros.org/wiki/rviz/DisplayTypes/Marker
#  - http://www.ros.org/wiki/rviz/Tutorials/Markers%3A%20Basic%20Shapes
#
# for more information on using this message with rviz.

int32 ARROW=0
int32 CUBE=1
int32 SPHERE=2
int32 CYLINDER=3
int32 LINE_STRIP=4
int32 LINE_LIST=5
int32 CUBE_LIST=6
int32 SPHERE_LIST=7
int32 POINTS=8
int32 TEXT_VIEW_FACING=9
int32 MESH_RESOURCE=10
int32 TRIANGLE_LIST=11
int32 ARROW_STRIP=12

int32 ADD=0
int32 MODIFY=0
int32 DELETE=2
int32 DELETEALL=3

# Header for timestamp and frame id.
std_msgs/Header header
# Namespace in which to place the object.
# Used in conjunction with id to create a unique name for the object.
string ns
# Object ID used in conjunction with the namespace for manipulating and deleting the object later.
int32 id
# Type of object.
int32 type
# Action to take; one of:
#  - 0 add/modify an object
#  - 1 (deprecated)
#  - 2 deletes an object (with the given ns and id)
#  - 3 deletes all objects (or those with the given ns if any)
int32 action
# Pose of the object with respect the frame_id specified in the header.
geometry_msgs/Pose pose
# Scale of the object; 1,1,1 means default (usually 1 meter square).
geometry_msgs/Vector3 scale
# Color of the object; in the range: [0.0-1.0]
std_msgs/ColorRGBA color
# How long the object should last before being automatically deleted.
# 0 indicates forever.
builtin_interfaces/Duration lifetime
# If this marker should be frame-locked, i.e. retransformed into its frame every timestep.
bool frame_locked

# Only used if the type specified has some use for them (eg. POINTS, LINE_STRIP, ARROW_STRIP, etc.)
geometry_msgs/Point[] points
# Only used if the type specified has some use for them (eg. POINTS, LINE_STRIP, etc.)
# The number of colors provided must either be 0 or equal to the number of points provided.
# NOTE: alpha is not yet used
std_msgs/ColorRGBA[] colors

# Texture resource is a special URI that can either reference a texture file in
# a format acceptable to (resource retriever)[https://docs.ros.org/en/rolling/p/resource_retriever/]
# or an embedded texture via a string matching the format:
#   "embedded://texture_name"
string texture_resource
# An image to be loaded into the rendering engine as the texture for this marker.
# This will be used iff texture_resource is set to embedded.
sensor_msgs/CompressedImage texture
# Location of each vertex within the texture; in the range: [0.0-1.0]
UVCoordinate[] uv_coordinates

# Only used for text markers
string text

# Only used for MESH_RESOURCE markers.
# Similar to texture_resource, mesh_resource uses resource retriever to load a mesh.
# Optionally, a mesh file can be sent in-message via the mesh_file field. If doing so,
# use the following format for mesh_resource:
#   "embedded://mesh_name"
string mesh_resource
MeshFile mesh_file
bool mesh_use_embedded_materials
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: sensor_msgs/CompressedImage
# This message contains a compressed image.

std_msgs/Header header # Header timestamp should be acquisition time of image
                             # Header frame_id should be optical frame of camera
                             # origin of frame should be optical center of cameara
                             # +x should point to the right in the image
                             # +y should point down in the image
                             # +z should point into to plane of the image

string format                # Specifies the format of the data
                             # Acceptable values differ by the image transport used:
                             # - compressed_image_transport:
                             #     ORIG_PIXFMT; CODEC compressed [COMPRESSED_PIXFMT]
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #   - CODEC is one of [jpeg, png, tiff]
                             #   - COMPRESSED_PIXFMT is only appended for color images
                             #     and is the pixel format used by the compression
                             #     algorithm. Valid values for jpeg encoding are:
                             #     [bgr8, rgb8]. Valid values for png encoding are:
                             #     [bgr8, rgb8, bgr16, rgb16].
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as bgr8 or mono8
                             #   jpeg image (depending on the number of channels).
                             # - compressed_depth_image_transport:
                             #     ORIG_PIXFMT; compressedDepth CODEC
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #     It is usually one of [16UC1, 32FC1].
                             #   - CODEC is one of [png, rvl]
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as png image.
                             # - Other image transports can store whatever values they
                             #   need for successful decoding of the image. Refer to
                             #   documentation of the other transports for details.

uint8[] data                 # Compressed image buffer
================================================================================
MSG: std_msgs/ColorRGBA
float32 r
float32 g
float32 b
float32 a
================================================================================
MSG: visualization_msgs/MeshFile
# Used to send raw mesh files.

# The filename is used for both debug purposes and to provide a file extension
# for whatever parser is used.
string filename

# This stores the raw text of the mesh file.
uint8[] data
================================================================================
MSG: visualization_msgs/UVCoordinate
# Location of the pixel as a ratio of the width of a 2D texture.
# Values should be in range: [0.0-1.0].
float32 u
float32 v`,
  "visualization_msgs/msg/MarkerArray": `Marker[] markers
================================================================================
MSG: builtin_interfaces/Duration
# Duration defines a period between two time points.
# Messages of this datatype are of ROS Time following this design:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The duration -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The duration 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: geometry_msgs/Point
# This contains the position of a point in free space
float64 x
float64 y
float64 z
================================================================================
MSG: geometry_msgs/Quaternion
# This represents an orientation in free space in quaternion form.

float64 x 0
float64 y 0
float64 z 0
float64 w 1
================================================================================
MSG: geometry_msgs/Pose
# A representation of pose in free space, composed of position and orientation.

Point position
Quaternion orientation
================================================================================
MSG: geometry_msgs/Vector3
# This represents a vector in free space.

# This is semantically different than a point.
# A vector is always anchored at the origin.
# When a transform is applied to a vector, only the rotational component is applied.

float64 x
float64 y
float64 z
================================================================================
MSG: builtin_interfaces/Time
# This message communicates ROS Time defined here:
# https://design.ros2.org/articles/clock_and_time.html

# The seconds component, valid over all int32 values.
int32 sec

# The nanoseconds component, valid in the range [0, 1e9), to be added to the seconds component. 
# e.g.
# The time -1.7 seconds is represented as {sec: -2, nanosec: 3e8}
# The time 1.7 seconds is represented as {sec: 1, nanosec: 7e8}
uint32 nanosec
================================================================================
MSG: std_msgs/Header
# Standard metadata for higher-level stamped data types.
# This is generally used to communicate timestamped data
# in a particular coordinate frame.

# Two-integer timestamp that is expressed as seconds and nanoseconds.
builtin_interfaces/Time stamp

# Transform frame with which this data is associated.
string frame_id
================================================================================
MSG: sensor_msgs/CompressedImage
# This message contains a compressed image.

std_msgs/Header header # Header timestamp should be acquisition time of image
                             # Header frame_id should be optical frame of camera
                             # origin of frame should be optical center of cameara
                             # +x should point to the right in the image
                             # +y should point down in the image
                             # +z should point into to plane of the image

string format                # Specifies the format of the data
                             # Acceptable values differ by the image transport used:
                             # - compressed_image_transport:
                             #     ORIG_PIXFMT; CODEC compressed [COMPRESSED_PIXFMT]
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #   - CODEC is one of [jpeg, png, tiff]
                             #   - COMPRESSED_PIXFMT is only appended for color images
                             #     and is the pixel format used by the compression
                             #     algorithm. Valid values for jpeg encoding are:
                             #     [bgr8, rgb8]. Valid values for png encoding are:
                             #     [bgr8, rgb8, bgr16, rgb16].
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as bgr8 or mono8
                             #   jpeg image (depending on the number of channels).
                             # - compressed_depth_image_transport:
                             #     ORIG_PIXFMT; compressedDepth CODEC
                             #   where:
                             #   - ORIG_PIXFMT is pixel format of the raw image, i.e.
                             #     the content of sensor_msgs/Image/encoding with
                             #     values from include/sensor_msgs/image_encodings.h
                             #     It is usually one of [16UC1, 32FC1].
                             #   - CODEC is one of [png, rvl]
                             #   If the field is empty or does not correspond to the
                             #   above pattern, the image is treated as png image.
                             # - Other image transports can store whatever values they
                             #   need for successful decoding of the image. Refer to
                             #   documentation of the other transports for details.

uint8[] data                 # Compressed image buffer
================================================================================
MSG: std_msgs/ColorRGBA
float32 r
float32 g
float32 b
float32 a
================================================================================
MSG: visualization_msgs/MeshFile
# Used to send raw mesh files.

# The filename is used for both debug purposes and to provide a file extension
# for whatever parser is used.
string filename

# This stores the raw text of the mesh file.
uint8[] data
================================================================================
MSG: visualization_msgs/UVCoordinate
# Location of the pixel as a ratio of the width of a 2D texture.
# Values should be in range: [0.0-1.0].
float32 u
float32 v
================================================================================
MSG: visualization_msgs/Marker
# See:
#  - http://www.ros.org/wiki/rviz/DisplayTypes/Marker
#  - http://www.ros.org/wiki/rviz/Tutorials/Markers%3A%20Basic%20Shapes
#
# for more information on using this message with rviz.

int32 ARROW=0
int32 CUBE=1
int32 SPHERE=2
int32 CYLINDER=3
int32 LINE_STRIP=4
int32 LINE_LIST=5
int32 CUBE_LIST=6
int32 SPHERE_LIST=7
int32 POINTS=8
int32 TEXT_VIEW_FACING=9
int32 MESH_RESOURCE=10
int32 TRIANGLE_LIST=11
int32 ARROW_STRIP=12

int32 ADD=0
int32 MODIFY=0
int32 DELETE=2
int32 DELETEALL=3

# Header for timestamp and frame id.
std_msgs/Header header
# Namespace in which to place the object.
# Used in conjunction with id to create a unique name for the object.
string ns
# Object ID used in conjunction with the namespace for manipulating and deleting the object later.
int32 id
# Type of object.
int32 type
# Action to take; one of:
#  - 0 add/modify an object
#  - 1 (deprecated)
#  - 2 deletes an object (with the given ns and id)
#  - 3 deletes all objects (or those with the given ns if any)
int32 action
# Pose of the object with respect the frame_id specified in the header.
geometry_msgs/Pose pose
# Scale of the object; 1,1,1 means default (usually 1 meter square).
geometry_msgs/Vector3 scale
# Color of the object; in the range: [0.0-1.0]
std_msgs/ColorRGBA color
# How long the object should last before being automatically deleted.
# 0 indicates forever.
builtin_interfaces/Duration lifetime
# If this marker should be frame-locked, i.e. retransformed into its frame every timestep.
bool frame_locked

# Only used if the type specified has some use for them (eg. POINTS, LINE_STRIP, ARROW_STRIP, etc.)
geometry_msgs/Point[] points
# Only used if the type specified has some use for them (eg. POINTS, LINE_STRIP, etc.)
# The number of colors provided must either be 0 or equal to the number of points provided.
# NOTE: alpha is not yet used
std_msgs/ColorRGBA[] colors

# Texture resource is a special URI that can either reference a texture file in
# a format acceptable to (resource retriever)[https://docs.ros.org/en/rolling/p/resource_retriever/]
# or an embedded texture via a string matching the format:
#   "embedded://texture_name"
string texture_resource
# An image to be loaded into the rendering engine as the texture for this marker.
# This will be used iff texture_resource is set to embedded.
sensor_msgs/CompressedImage texture
# Location of each vertex within the texture; in the range: [0.0-1.0]
UVCoordinate[] uv_coordinates

# Only used for text markers
string text

# Only used for MESH_RESOURCE markers.
# Similar to texture_resource, mesh_resource uses resource retriever to load a mesh.
# Optionally, a mesh file can be sent in-message via the mesh_file field. If doing so,
# use the following format for mesh_resource:
#   "embedded://mesh_name"
string mesh_resource
MeshFile mesh_file
bool mesh_use_embedded_materials`,
  "visualization_msgs/msg/MenuEntry": `# MenuEntry message.
#
# Each InteractiveMarker message has an array of MenuEntry messages.
# A collection of MenuEntries together describe a
# menu/submenu/subsubmenu/etc tree, though they are stored in a flat
# array.  The tree structure is represented by giving each menu entry
# an ID number and a "parent_id" field.  Top-level entries are the
# ones with parent_id = 0.  Menu entries are ordered within their
# level the same way they are ordered in the containing array.  Parent
# entries must appear before their children.
#
# Example:
# - id = 3
#   parent_id = 0
#   title = "fun"
# - id = 2
#   parent_id = 0
#   title = "robot"
# - id = 4
#   parent_id = 2
#   title = "pr2"
# - id = 5
#   parent_id = 2
#   title = "turtle"
#
# Gives a menu tree like this:
#  - fun
#  - robot
#    - pr2
#    - turtle

# ID is a number for each menu entry.  Must be unique within the
# control, and should never be 0.
uint32 id

# ID of the parent of this menu entry, if it is a submenu.  If this
# menu entry is a top-level entry, set parent_id to 0.
uint32 parent_id

# menu / entry title
string title

# Arguments to command indicated by command_type (below)
string command

# Command_type stores the type of response desired when this menu
# entry is clicked.
# FEEDBACK: send an InteractiveMarkerFeedback message with menu_entry_id set to this entry's id.
# ROSRUN: execute "rosrun" with arguments given in the command field (above).
# ROSLAUNCH: execute "roslaunch" with arguments given in the command field (above).
uint8 FEEDBACK=0
uint8 ROSRUN=1
uint8 ROSLAUNCH=2
uint8 command_type`,
  "visualization_msgs/msg/MeshFile": `# Used to send raw mesh files.

# The filename is used for both debug purposes and to provide a file extension
# for whatever parser is used.
string filename

# This stores the raw text of the mesh file.
uint8[] data`,
  "visualization_msgs/msg/UVCoordinate": `# Location of the pixel as a ratio of the width of a 2D texture.
# Values should be in range: [0.0-1.0].
float32 u
float32 v`,
};
