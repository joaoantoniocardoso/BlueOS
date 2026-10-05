# blueos_example_msgs/msg/SelfTestCompleted
# Event on blueos/v1/example/event/SelfTestCompleted.

bool passed
string detail
# One result per check, in the order the self-test runs them: motor, seal, level sensor.
bool[3] checks
