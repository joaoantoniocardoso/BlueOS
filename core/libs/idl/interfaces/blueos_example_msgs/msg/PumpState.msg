# blueos_example_msgs/msg/PumpState
# Published on blueos/v1/example/state/pump (teaching example, D-20).

uint8 SELF_TEST_IDLE=0
uint8 SELF_TEST_RUNNING=1
uint8 SELF_TEST_PASSED=2
uint8 SELF_TEST_FAILED=3
uint8 SELF_TEST_CANCELLED=4

uint8 level
uint8 max_level
uint8 self_test_phase
bool self_test_active
