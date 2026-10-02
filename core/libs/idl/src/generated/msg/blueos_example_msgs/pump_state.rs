#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use serde::{Deserialize, Serialize};
pub mod constants_pump_state {
    pub const SELF_TEST_IDLE: u8 = 0u8;
    pub const SELF_TEST_RUNNING: u8 = 1u8;
    pub const SELF_TEST_PASSED: u8 = 2u8;
    pub const SELF_TEST_FAILED: u8 = 3u8;
    pub const SELF_TEST_CANCELLED: u8 = 4u8;
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PumpState {
    pub level: u8,
    pub max_level: u8,
    pub self_test_phase: u8,
    pub self_test_active: bool,
}
impl CdrStruct for PumpState {
    fn cdr_decode_fields(reader: &mut cdr::Reader) -> Result<Self, Error> {
        Ok(Self {
            level: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u8()?
            },
            max_level: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u8()?
            },
            self_test_phase: if reader.is_exhausted() {
                Default::default()
            } else {
                reader.read_u8()?
            },
            self_test_active: if reader.is_exhausted() {
                false
            } else {
                reader.read_bool()?
            },
        })
    }
    fn cdr_encode_fields(&self, writer: &mut cdr::Writer) -> Result<(), Error> {
        writer.write_u8(self.level)?;
        writer.write_u8(self.max_level)?;
        writer.write_u8(self.self_test_phase)?;
        writer.write_bool(self.self_test_active)?;
        Ok(())
    }
}
impl Message for PumpState {
    const SCHEMA: &'static str = "# blueos_example_msgs/msg/PumpState\n# Published on blueos/v1/example/state/pump (teaching example, D-20).\n\nuint8 SELF_TEST_IDLE=0\nuint8 SELF_TEST_RUNNING=1\nuint8 SELF_TEST_PASSED=2\nuint8 SELF_TEST_FAILED=3\nuint8 SELF_TEST_CANCELLED=4\n\nuint8 level\nuint8 max_level\nuint8 self_test_phase\nbool self_test_active";
    const SCHEMA_NAME: &'static str = "blueos_example_msgs/msg/PumpState";
    const TYPE_HASH: &'static str =
        "ae282ec7a925f05c8bac321b8a984dd7f8062c853806f21308e5d73eb8200941";
}
impl PumpState {
    pub const KNOWN_FIELD_COUNT: usize = 4usize;
}
