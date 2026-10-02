//! Reply to a Zenoh command query (`blueos_msgs/msg/CommandAck`, D-10).

pub use blueos_idl::{Message, msg::blueos_msgs::CommandAck};

/// Sentinel [`CommandAck::job_id`] when no root job was started (rejection or command without work).
///
/// Assigned job ids start at **1**; **0** means none.
pub const JOB_ID_NONE: u64 = 0;

#[cfg(test)]
mod tests {
    use super::{CommandAck, JOB_ID_NONE, Message};

    #[test]
    fn job_id_none_is_zero() {
        assert_eq!(JOB_ID_NONE, 0);
    }

    #[test]
    fn command_ack_round_trip_with_assigned_job_id() {
        let message = CommandAck {
            accepted: true,
            job_id: 42,
            reason: "queued".into(),
        };
        let payload = message.encode().expect("encode");
        let decoded = CommandAck::decode(&payload).expect("decode");
        assert_eq!(message, decoded);
        assert_ne!(decoded.job_id, JOB_ID_NONE);
    }

    #[test]
    fn command_ack_no_job_uses_job_id_none() {
        let message = CommandAck {
            accepted: false,
            job_id: JOB_ID_NONE,
            reason: "busy".into(),
        };
        let payload = message.encode().expect("encode");
        let decoded = CommandAck::decode(&payload).expect("decode");
        assert_eq!(decoded.job_id, JOB_ID_NONE);
        assert!(!decoded.accepted);
    }
}
