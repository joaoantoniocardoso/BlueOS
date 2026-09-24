use alloc::string::String;

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub moving: bool,
    pub ack: Option<String>,
    pub gyro_offset: Option<i32>,
    pub ground_pressure: Option<i32>,
}

#[derive(Clone, Debug)]
pub enum Query {
    GetSnapshot,
}

#[derive(Clone, Debug)]
pub struct View {
    pub snapshot: Snapshot,
}
