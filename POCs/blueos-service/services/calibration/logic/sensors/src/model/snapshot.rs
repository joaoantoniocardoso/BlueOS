use alloc::string::String;

use crate::model::Sensor;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CalibrationStatus {
    #[default]
    Idle,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub gyro: CalibrationStatus,
    pub baro: CalibrationStatus,
    pub last_ack: Option<String>,
    pub gyro_offset: Option<i32>,
    pub ground_pressure: Option<i32>,
}

impl Snapshot {
    pub(crate) fn status_of(&self, sensor: Sensor) -> CalibrationStatus {
        match sensor {
            Sensor::Gyro => self.gyro,
            Sensor::Baro => self.baro,
        }
    }

    pub(crate) fn set_status(&mut self, sensor: Sensor, status: CalibrationStatus) {
        match sensor {
            Sensor::Gyro => self.gyro = status,
            Sensor::Baro => self.baro = status,
        }
    }

    pub(crate) fn set_offset(&mut self, sensor: Sensor, value: i32) {
        match sensor {
            Sensor::Gyro => self.gyro_offset = Some(value),
            Sensor::Baro => self.ground_pressure = Some(value),
        }
    }

    pub(crate) fn offset_of(&self, sensor: Sensor) -> Option<i32> {
        match sensor {
            Sensor::Gyro => self.gyro_offset,
            Sensor::Baro => self.ground_pressure,
        }
    }

    pub(crate) fn clear_offset(&mut self, sensor: Sensor) {
        match sensor {
            Sensor::Gyro => self.gyro_offset = None,
            Sensor::Baro => self.ground_pressure = None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Query {
    GetSnapshot,
}

#[derive(Clone, Debug)]
pub struct View {
    pub snapshot: Snapshot,
    pub jobs: blueos_jobs::JobsSnapshot,
}
