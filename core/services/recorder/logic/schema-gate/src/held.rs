//! Per-topic sample queue entries (payload by reference).

/// One backbone sample waiting for a ros2dds schema.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeldSample<Payload> {
    /// MCAP log time in nanoseconds.
    pub log_time: u64,
    /// MCAP publish time in nanoseconds.
    pub publish_time: u64,
    /// Sample bytes (held by reference through `Payload`).
    pub payload: Payload,
}
