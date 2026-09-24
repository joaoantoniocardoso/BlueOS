//! Values must stay aligned with `blueos_example_msgs/msg/PumpState` constants (IDL is wired in `app/`).

pub const SELF_TEST_IDLE: u8 = 0;
pub const SELF_TEST_RUNNING: u8 = 1;
pub const SELF_TEST_PASSED: u8 = 2;
pub const SELF_TEST_FAILED: u8 = 3;
pub const SELF_TEST_CANCELLED: u8 = 4;
