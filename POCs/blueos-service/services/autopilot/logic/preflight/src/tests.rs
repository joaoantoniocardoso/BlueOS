use super::*;
use crate::keys::offset_wire;
use crate::preflight::{
    ACK_ACCEPTED, ACK_FAILED, ACK_IN_PROGRESS, FAKE_GROUND_PRESSURE, FAKE_GYRO_OFFSET,
};
use blueos_cqrs::App;

fn tick_job(effects: &[Effect<IoRequest>]) -> JobId {
    effects
        .iter()
        .find_map(|effect| match effect {
            Effect::Io(IoRequest::Tick { job_id }) => Some(*job_id),
            _ => None,
        })
        .expect("tick")
}

#[test]
fn query_reads_snapshot_without_mutation() {
    let app = App::<Autopilot>::new(Snapshot {
        moving: true,
        ack: Some(ACK_ACCEPTED.to_string()),
        gyro_offset: Some(9),
        ground_pressure: Some(8),
    });
    let view = app.query(Query::GetSnapshot);
    assert!(view.snapshot.moving);
    assert_eq!(view.snapshot.gyro_offset, Some(9));
    assert_eq!(app.snapshot.gyro_offset, Some(9));
    assert!(app.jobs.snapshot().jobs.is_empty());
}

#[test]
fn moving_preflight_replies_failed_without_job() {
    let mut app = App::<Autopilot>::new(Snapshot::default());
    let _ = app.handle(Command::SetMoving(true)); // SetMoving emits no events
    let (events, effects) = app.handle(Command::Preflight {
        correlation: 3,
        sensor: Sensor::Gyro,
    });
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Ack { sensor: Sensor::Gyro, ack } if ack == ACK_FAILED
    )));
    assert!(effects.iter().any(|effect| matches!(
        effect,
        Effect::Reply { correlation: 3, payload } if payload.as_slice() == ACK_FAILED.as_bytes()
    )));
    assert!(effects.iter().any(|effect| matches!(
        effect,
        Effect::Publish { key, .. } if key == STATE_ACK
    )));
    assert!(!effects.iter().any(|effect| matches!(effect, Effect::Io(_))));
    assert_eq!(app.snapshot.ack.as_deref(), Some(ACK_FAILED));
    assert_eq!(app.snapshot.gyro_offset, None);
}

#[test]
fn still_preflight_then_advance_ack_writes_gyro_offset() {
    let mut app = App::<Autopilot>::new(Snapshot::default());
    let (events, effects) = app.handle(Command::Preflight {
        correlation: 7,
        sensor: Sensor::Gyro,
    });
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Ack { ack, .. } if ack == ACK_IN_PROGRESS
    )));
    assert!(effects.iter().any(|effect| matches!(
        effect,
        Effect::Reply { correlation: 7, payload } if payload.as_slice() == ACK_IN_PROGRESS.as_bytes()
    )));
    assert_eq!(app.snapshot.ack.as_deref(), Some(ACK_IN_PROGRESS));
    let job_id = tick_job(&effects);
    let (events, effects) = app.handle(Command::AdvanceAck {
        job_id,
        sensor: Sensor::Gyro,
    });
    assert!(events.iter().any(|event| matches!(
        event,
        Event::Ack { ack, .. } if ack == ACK_ACCEPTED
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        Event::OffsetsWritten {
            sensor: Sensor::Gyro,
            value: FAKE_GYRO_OFFSET
        }
    )));
    assert!(effects.iter().any(|effect| matches!(
        effect,
        Effect::Publish { key, .. } if key == STATE_ACK
    )));
    assert!(effects.iter().any(|effect| match effect {
        Effect::Publish { key, payload } => {
            key == STATE_OFFSETS && *payload == offset_wire(Sensor::Gyro, FAKE_GYRO_OFFSET)
        }
        _ => false,
    }));
    assert_eq!(app.snapshot.ack.as_deref(), Some(ACK_ACCEPTED));
    assert_eq!(app.snapshot.gyro_offset, Some(FAKE_GYRO_OFFSET));
    assert_eq!(app.snapshot.ground_pressure, None);
}

#[test]
fn advance_ack_writes_baro_ground_pressure() {
    let mut app = App::<Autopilot>::new(Snapshot::default());
    let (_, effects) = app.handle(Command::Preflight {
        correlation: 0,
        sensor: Sensor::Baro,
    });
    let job_id = tick_job(&effects);
    let _ = app.handle(Command::AdvanceAck {
        job_id,
        sensor: Sensor::Baro,
    }); // events unused; snapshot is the assertion
    assert_eq!(app.snapshot.ground_pressure, Some(FAKE_GROUND_PRESSURE));
    assert_eq!(app.snapshot.gyro_offset, None);
}
