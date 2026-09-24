use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::keys::{STATE_ACK, STATE_OFFSETS, ack_wire, offset_wire, sensor_wire};
use crate::model::{Event, IoRequest, Sensor, Snapshot};
use blueos_cqrs::Effect;
use blueos_jobs::{JobGraph, JobId, JobSpec, Jobs};

pub(crate) const ADVANCE_ACK: &str = "advance_ack";
pub(crate) const ACK_FAILED: &str = "FAILED";
pub(crate) const ACK_IN_PROGRESS: &str = "IN_PROGRESS";
pub(crate) const ACK_ACCEPTED: &str = "ACCEPTED";
pub(crate) const FAKE_GYRO_OFFSET: i32 = 1;
pub(crate) const FAKE_GROUND_PRESSURE: i32 = 1013;

pub(crate) fn preflight(
    snapshot: &mut Snapshot,
    jobs: &mut Jobs,
    correlation: u32,
    sensor: Sensor,
) -> (Vec<Event>, Vec<Effect<IoRequest>>) {
    if snapshot.moving {
        snapshot.ack = Some(ACK_FAILED.to_string());
        return (
            vec![Event::Ack {
                sensor,
                ack: ACK_FAILED.to_string(),
            }],
            vec![
                Effect::Publish {
                    key: STATE_ACK.to_string(),
                    payload: ack_wire(sensor, ACK_FAILED),
                },
                Effect::Reply {
                    correlation,
                    payload: ACK_FAILED.as_bytes().to_vec(),
                },
            ],
        );
    }
    snapshot.ack = Some(ACK_IN_PROGRESS.to_string());
    jobs.enqueue(JobGraph::Leaf(JobSpec {
        name: ADVANCE_ACK.to_string(),
        payload: sensor_wire(sensor).to_vec(),
    }));
    (
        vec![Event::Ack {
            sensor,
            ack: ACK_IN_PROGRESS.to_string(),
        }],
        vec![
            Effect::Publish {
                key: STATE_ACK.to_string(),
                payload: ack_wire(sensor, ACK_IN_PROGRESS),
            },
            Effect::Reply {
                correlation,
                payload: ACK_IN_PROGRESS.as_bytes().to_vec(),
            },
        ],
    )
}

pub(crate) fn advance_ack(
    snapshot: &mut Snapshot,
    jobs: &mut Jobs,
    job_id: JobId,
    sensor: Sensor,
) -> (Vec<Event>, Vec<Effect<IoRequest>>) {
    let _ = jobs.complete(job_id, true); // already-terminal complete is a no-op
    snapshot.ack = Some(ACK_ACCEPTED.to_string());
    let value = match sensor {
        Sensor::Gyro => {
            snapshot.gyro_offset = Some(FAKE_GYRO_OFFSET);
            FAKE_GYRO_OFFSET
        }
        Sensor::Baro => {
            snapshot.ground_pressure = Some(FAKE_GROUND_PRESSURE);
            FAKE_GROUND_PRESSURE
        }
    };
    (
        vec![
            Event::Ack {
                sensor,
                ack: ACK_ACCEPTED.to_string(),
            },
            Event::OffsetsWritten { sensor, value },
        ],
        vec![
            Effect::Publish {
                key: STATE_ACK.to_string(),
                payload: ack_wire(sensor, ACK_ACCEPTED),
            },
            Effect::Publish {
                key: STATE_OFFSETS.to_string(),
                payload: offset_wire(sensor, value),
            },
        ],
    )
}
