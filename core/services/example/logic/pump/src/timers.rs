use blueos_cqrs::TimerId;

/// Kernel timer armed while a self-test job graph is in flight. Cancelled on success, failure, or user cancel.
pub const SELF_TEST_TIMEOUT_TIMER: TimerId = TimerId(1);
