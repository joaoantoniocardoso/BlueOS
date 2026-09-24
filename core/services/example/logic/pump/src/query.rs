#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PumpQuery {
    Level,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PumpQueryView {
    pub level: u8,
    pub max_level: u8,
}
