#![allow(missing_docs, reason = "generated from ROS .msg sources")]
use crate::{
    cdr,
    error::Error,
    message::{CdrStruct, Message},
};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PumpStateSelfTestPhase {
    #[default]
    Idle,
    Running,
    Passed,
    Failed,
    Cancelled,
    Unknown(u8),
}
impl PumpStateSelfTestPhase {
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            0u8 => Self::Idle,
            1u8 => Self::Running,
            2u8 => Self::Passed,
            3u8 => Self::Failed,
            4u8 => Self::Cancelled,
            raw => Self::Unknown(raw),
        }
    }
    pub fn as_raw(self) -> u8 {
        match self {
            Self::Idle => 0u8,
            Self::Running => 1u8,
            Self::Passed => 2u8,
            Self::Failed => 3u8,
            Self::Cancelled => 4u8,
            Self::Unknown(raw) => raw,
        }
    }
}
impl serde::Serialize for PumpStateSelfTestPhase {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        <u8>::serialize(&self.as_raw(), serializer)
    }
}
impl<'de> serde::Deserialize<'de> for PumpStateSelfTestPhase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self::from_raw(<u8>::deserialize(deserializer)?))
    }
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PumpState {
    pub level: u8,
    pub max_level: u8,
    pub self_test_phase: PumpStateSelfTestPhase,
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
                <PumpStateSelfTestPhase>::default()
            } else {
                PumpStateSelfTestPhase::from_raw(reader.read_u8()?)
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
        writer.write_u8(self.self_test_phase.as_raw())?;
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
