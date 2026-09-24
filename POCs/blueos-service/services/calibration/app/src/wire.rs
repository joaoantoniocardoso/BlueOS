use calibration_sensors::{Command, Sensor};

pub(crate) fn parse_ack(payload: &[u8]) -> Option<Command> {
    let text = std::str::from_utf8(payload).ok()?.trim();
    let (sensor, ack) = text.split_once(':')?;
    Some(Command::AckObserved {
        sensor: parse_sensor(sensor)?,
        ack: ack.to_string(),
    })
}

pub(crate) fn parse_offsets(payload: &[u8]) -> Option<Command> {
    let text = std::str::from_utf8(payload).ok()?.trim();
    let (sensor, value) = text.split_once(':')?;
    Some(Command::OffsetsRead {
        sensor: parse_sensor(sensor)?,
        value: value.parse().ok()?,
    })
}

fn parse_sensor(text: &str) -> Option<Sensor> {
    match text {
        "gyro" => Some(Sensor::Gyro),
        "baro" => Some(Sensor::Baro),
        _ => None,
    }
}
