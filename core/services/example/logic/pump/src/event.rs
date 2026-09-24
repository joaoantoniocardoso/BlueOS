use alloc::string::String;
use alloc::vec::Vec;
#[derive(Clone, Debug, PartialEq)]
pub enum PumpEvent {
    LevelChanged { level: u8 },
    SelfTestStarted,
    SelfTestCompleted { passed: bool, detail: String },
    RestartRequired { fields: Vec<String> },
}
