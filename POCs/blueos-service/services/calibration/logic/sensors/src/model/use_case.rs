#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UseCase {
    CalibrateGyroscope,
    CalibrateBarometer,
    CalibrateStationarySensors,
    CancelCalibration,
}
